// src/systems/resource_system.rs
use crate::core::{GameResult, GameEvent, EventBus, GameSystem};
use crate::core::types::*;
use crate::core::events::*;
use std::collections::HashMap;

pub struct ResourceSystem {
    production_rates: HashMap<BuildingType, ResourceBundle>,
    consumption_tracking: HashMap<PlanetId, ResourceBundle>,
}

impl ResourceSystem {
    pub fn new() -> Self {
        let mut production_rates = HashMap::new();
        
        // Define production rates per worker per tick (0.1 seconds)
        production_rates.insert(BuildingType::Mine, ResourceBundle {
            minerals: 10,
            food: 0,
            energy: -2,
            alloys: 0,
            components: 0,
            fuel: 0,
        });
        
        production_rates.insert(BuildingType::Farm, ResourceBundle {
            minerals: 0,
            food: 8,
            energy: -1,
            alloys: 0,
            components: 0,
            fuel: 0,
        });
        
        production_rates.insert(BuildingType::PowerPlant, ResourceBundle {
            minerals: 0,
            food: 0,
            energy: 15,
            alloys: 0,
            components: 0,
            fuel: 0,
        });
        
        production_rates.insert(BuildingType::Factory, ResourceBundle {
            minerals: -5,
            food: 0,
            energy: -3,
            alloys: 5,
            components: 0,
            fuel: 0,
        });
        
        production_rates.insert(BuildingType::ResearchLab, ResourceBundle {
            minerals: 0,
            food: -1,
            energy: -2,
            alloys: 0,
            components: 3,
            fuel: 0,
        });
        
        Self {
            production_rates,
            consumption_tracking: HashMap::new(),
        }
    }
    
    pub fn calculate_planet_production(&self, planet: &Planet) -> GameResult<ResourceBundle> {
        let mut total_production = ResourceBundle::default();
        
        // Calculate base production from workers
        let allocation = &planet.population.allocation;
        total_production.minerals += allocation.mining * 2; // Base mining rate
        total_production.food += allocation.agriculture * 3; // Base farming rate
        total_production.energy += allocation.industry; // Base energy from workers
        
        // Calculate building bonuses
        for building in &planet.developments {
            if !building.operational {
                continue;
            }
            
            if let Some(production_rate) = self.production_rates.get(&building.building_type) {
                // Building efficiency scales with tier
                let efficiency_multiplier = building.tier as i32;
                
                total_production.minerals += production_rate.minerals * efficiency_multiplier;
                total_production.food += production_rate.food * efficiency_multiplier;
                total_production.energy += production_rate.energy * efficiency_multiplier;
                total_production.alloys += production_rate.alloys * efficiency_multiplier;
                total_production.components += production_rate.components * efficiency_multiplier;
                total_production.fuel += production_rate.fuel * efficiency_multiplier;
            }
        }
        
        Ok(total_production)
    }
    
    pub fn process_production(&mut self, planet_manager: &mut crate::managers::PlanetManager, event_bus: &mut EventBus) -> GameResult<()> {
        let all_planets = planet_manager.get_all_planets().clone();
        
        for planet in &all_planets {
            // Calculate production for this planet
            let production = self.calculate_planet_production(planet)?;
            
            // Track consumption
            self.consumption_tracking.insert(planet.id, production);
            
            // Check for negative resources (consumption exceeding production)
            let current_resources = planet.resources.current;
            let mut shortage_resources = Vec::new();
            
            if current_resources.minerals + production.minerals < 0 {
                shortage_resources.push(ResourceType::Minerals);
            }
            if current_resources.food + production.food < 0 {
                shortage_resources.push(ResourceType::Food);
            }
            if current_resources.energy + production.energy < 0 {
                shortage_resources.push(ResourceType::Energy);
            }
            if current_resources.alloys + production.alloys < 0 {
                shortage_resources.push(ResourceType::Alloys);
            }
            if current_resources.components + production.components < 0 {
                shortage_resources.push(ResourceType::Components);
            }
            if current_resources.fuel + production.fuel < 0 {
                shortage_resources.push(ResourceType::Fuel);
            }
            
            // Emit shortage events before applying production
            for resource_type in shortage_resources {
                event_bus.queue_event(GameEvent::SimulationEvent(
                    SimulationEvent::ResourceShortage {
                        planet: planet.id,
                        resource: resource_type,
                    }
                ));
            }
            
            // Apply production (can result in negative resources if consumption exceeds production)
            // This validates against storage capacity
            match planet_manager.add_resources(planet.id, production) {
                Ok(()) => {
                    // Emit successful production event
                    event_bus.queue_event(GameEvent::SimulationEvent(
                        SimulationEvent::ResourcesProduced {
                            planet: planet.id,
                            resources: production,
                        }
                    ));
                }
                Err(_) => {
                    // Handle overflow by capping to maximum capacity
                    let capacity = planet.resources.capacity;
                    let current = planet.resources.current;
                    
                    let capped_production = ResourceBundle {
                        minerals: (capacity.minerals - current.minerals).min(production.minerals),
                        food: (capacity.food - current.food).min(production.food),
                        energy: (capacity.energy - current.energy).min(production.energy),
                        alloys: (capacity.alloys - current.alloys).min(production.alloys),
                        components: (capacity.components - current.components).min(production.components),
                        fuel: (capacity.fuel - current.fuel).min(production.fuel),
                    };
                    
                    planet_manager.add_resources(planet.id, capped_production)?;
                    
                    event_bus.queue_event(GameEvent::SimulationEvent(
                        SimulationEvent::ResourcesProduced {
                            planet: planet.id,
                            resources: capped_production,
                        }
                    ));
                }
            }
            
            // Emit planet updated state change
            event_bus.queue_event(GameEvent::StateChanged(StateChange::PlanetUpdated(planet.id)));
        }
        
        Ok(())
    }
    
    pub fn process_transfer(&mut self, from: PlanetId, to: PlanetId, resources: ResourceBundle, planet_manager: &mut crate::managers::PlanetManager, event_bus: &mut EventBus) -> GameResult<()> {
        // Validate source planet has enough resources
        let source_planet = planet_manager.get_planet(from)?;
        if !source_planet.resources.current.can_afford(&resources) {
            return Err(GameError::InsufficientResources {
                required: resources,
                available: source_planet.resources.current,
            });
        }
        
        // Validate destination exists and has capacity
        let dest_planet = planet_manager.get_planet(to)?;
        let dest_capacity = dest_planet.resources.capacity;
        let dest_current = dest_planet.resources.current;
        
        // Check if destination can store the resources
        if dest_current.minerals + resources.minerals > dest_capacity.minerals ||
           dest_current.food + resources.food > dest_capacity.food ||
           dest_current.energy + resources.energy > dest_capacity.energy ||
           dest_current.alloys + resources.alloys > dest_capacity.alloys ||
           dest_current.components + resources.components > dest_capacity.components ||
           dest_current.fuel + resources.fuel > dest_capacity.fuel {
            return Err(GameError::InvalidOperation("Destination storage capacity exceeded".into()));
        }
        
        // Perform the transfer
        planet_manager.remove_resources(from, resources)?;
        planet_manager.add_resources(to, resources)?;
        
        // Emit state change events
        event_bus.queue_event(GameEvent::StateChanged(StateChange::PlanetUpdated(from)));
        event_bus.queue_event(GameEvent::StateChanged(StateChange::PlanetUpdated(to)));
        
        Ok(())
    }
    
    pub fn get_consumption_for_planet(&self, planet_id: PlanetId) -> Option<&ResourceBundle> {
        self.consumption_tracking.get(&planet_id)
    }
    
    pub fn process_ship_loading(&mut self, ship_id: ShipId, planet_id: PlanetId, resources: ResourceBundle, 
                               planet_manager: &mut crate::managers::PlanetManager, 
                               ship_manager: &mut crate::managers::ShipManager, 
                               event_bus: &mut EventBus) -> GameResult<()> {
        // Validate ship exists and is at the planet
        let ship = ship_manager.get_ship(ship_id)?;
        let planet = planet_manager.get_planet(planet_id)?;
        
        // Check if ship is at the planet (within loading range)
        let distance = ((ship.position.x - planet.position.semi_major_axis).powi(2) + 
                       (ship.position.y).powi(2)).sqrt(); // Simplified planet position
        if distance > 1.0 { // Loading range of 1 unit
            return Err(GameError::InvalidOperation("Ship not at planet for loading".into()));
        }
        
        // Check if planet has enough resources
        if !planet.resources.current.can_afford(&resources) {
            return Err(GameError::InsufficientResources {
                required: resources,
                available: planet.resources.current,
            });
        }
        
        // Load cargo onto ship
        ship_manager.load_cargo(ship_id, resources)?;
        
        // Remove resources from planet
        planet_manager.remove_resources(planet_id, resources)?;
        
        // Emit events
        event_bus.queue_event(GameEvent::StateChanged(StateChange::PlanetUpdated(planet_id)));
        event_bus.queue_event(GameEvent::StateChanged(StateChange::ShipUpdated(ship_id)));
        
        Ok(())
    }
    
    pub fn process_ship_unloading(&mut self, ship_id: ShipId, planet_id: PlanetId,
                                 planet_manager: &mut crate::managers::PlanetManager,
                                 ship_manager: &mut crate::managers::ShipManager,
                                 event_bus: &mut EventBus) -> GameResult<()> {
        // Validate ship exists and is at the planet
        let ship = ship_manager.get_ship(ship_id)?;
        let planet = planet_manager.get_planet(planet_id)?;
        
        // Check if ship is at the planet (within unloading range)
        let distance = ((ship.position.x - planet.position.semi_major_axis).powi(2) + 
                       (ship.position.y).powi(2)).sqrt(); // Simplified planet position
        if distance > 1.0 { // Unloading range of 1 unit
            return Err(GameError::InvalidOperation("Ship not at planet for unloading".into()));
        }
        
        // Get cargo contents
        let cargo_resources = *ship_manager.get_cargo_contents(ship_id)?;
        
        // Check if planet can store the resources
        let planet_capacity = planet.resources.capacity;
        let planet_current = planet.resources.current;
        
        if planet_current.minerals + cargo_resources.minerals > planet_capacity.minerals ||
           planet_current.food + cargo_resources.food > planet_capacity.food ||
           planet_current.energy + cargo_resources.energy > planet_capacity.energy ||
           planet_current.alloys + cargo_resources.alloys > planet_capacity.alloys ||
           planet_current.components + cargo_resources.components > planet_capacity.components ||
           planet_current.fuel + cargo_resources.fuel > planet_capacity.fuel {
            return Err(GameError::InvalidOperation("Planet storage capacity would be exceeded".into()));
        }
        
        // Unload cargo from ship
        let unloaded_resources = ship_manager.unload_cargo(ship_id)?;
        
        // Add resources to planet
        planet_manager.add_resources(planet_id, unloaded_resources)?;
        
        // Emit events
        event_bus.queue_event(GameEvent::StateChanged(StateChange::PlanetUpdated(planet_id)));
        event_bus.queue_event(GameEvent::StateChanged(StateChange::ShipUpdated(ship_id)));
        
        Ok(())
    }
}

impl GameSystem for ResourceSystem {
    fn update(&mut self, _delta: f32, _event_bus: &mut EventBus) -> GameResult<()> {
        // ResourceSystem processes production during tick events, not on update
        Ok(())
    }
    
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        // The ResourceSystem only tracks events for validation purposes
        // Actual processing is done through the public methods called by GameState
        match event {
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    SimulationEvent::TickCompleted(_tick) => {
                        // Production processing happens in GameState::process_tick_production
                    }
                    _ => {}
                }
            }
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    PlayerCommand::TransferResources { from: _, to: _, resources: _ } => {
                        // Transfer processing happens in GameState::process_resource_transfer
                    }
                    PlayerCommand::LoadShipCargo { ship: _, planet: _, resources: _ } => {
                        // Cargo loading processing happens in GameState::process_ship_cargo_loading
                    }
                    PlayerCommand::UnloadShipCargo { ship: _, planet: _ } => {
                        // Cargo unloading processing happens in GameState::process_ship_cargo_unloading
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}