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
    
    /// Process resource production for all planets in the game state
    /// This method should be called by GameState during tick processing  
    pub fn process_production(&mut self, planets: &[Planet], event_bus: &mut EventBus) -> GameResult<()> {
        for planet in planets {
            // Calculate production for this planet
            let production = self.calculate_planet_production(planet)?;
            
            // Track consumption
            self.consumption_tracking.insert(planet.id, production);
            
            // Check for resource shortages using functional approach
            let current_resources = planet.resources.current;
            let shortage_resources = self.detect_resource_shortages(&current_resources, &production);
            
            // Emit shortage events before applying production
            for resource_type in shortage_resources {
                event_bus.queue_event(GameEvent::SimulationEvent(
                    SimulationEvent::ResourceShortage {
                        planet: planet.id,
                        resource: resource_type,
                    }
                ));
            }
            
            // Calculate actual production considering storage constraints
            let effective_production = self.calculate_effective_production(
                &current_resources, &planet.resources.capacity, &production
            )?;
            
            // This will be handled by GameState calling planet_manager.add_resources
            // Emit production event with effective resources
            event_bus.queue_event(GameEvent::SimulationEvent(
                SimulationEvent::ResourcesProduced {
                    planet: planet.id,
                    resources: effective_production,
                }
            ));
            
            // Emit planet updated state change
            event_bus.queue_event(GameEvent::StateChanged(StateChange::PlanetUpdated(planet.id)));
        }
        
        Ok(())
    }
    
    /// Validate a resource transfer between planets
    /// Returns the validated transfer amount (may be less than requested)
    pub fn validate_transfer(&self, source: &Planet, destination: &Planet, requested: ResourceBundle) -> GameResult<ResourceBundle> {
        // Validate source has enough resources
        if !source.resources.current.can_afford(&requested) {
            return Err(GameError::InsufficientResources {
                required: requested,
                available: source.resources.current,
            });
        }
        
        // Validate destination capacity using existing storage methods
        if !destination.resources.can_store(&requested) {
            // Calculate maximum transferable amount
            let available_space = destination.resources.available_space();
            let max_transfer = ResourceBundle {
                minerals: requested.minerals.min(available_space.minerals),
                food: requested.food.min(available_space.food),
                energy: requested.energy.min(available_space.energy),
                alloys: requested.alloys.min(available_space.alloys),
                components: requested.components.min(available_space.components),
                fuel: requested.fuel.min(available_space.fuel),
            };
            
            if max_transfer.total() == 0 {
                return Err(GameError::InvalidOperation("Destination has no storage capacity".into()));
            }
            
            return Ok(max_transfer);
        }
        
        Ok(requested)
    }
    
    pub fn get_consumption_for_planet(&self, planet_id: PlanetId) -> Option<&ResourceBundle> {
        self.consumption_tracking.get(&planet_id)
    }
    
    /// Validate ship cargo loading operation
    /// Returns the actual loadable amount considering ship capacity and planet resources
    pub fn validate_cargo_loading(&self, ship: &Ship, planet: &Planet, requested: ResourceBundle, current_tick: u64) -> GameResult<ResourceBundle> {
        // Check if ship is at planet using proper orbital calculation
        let planet_position = self.calculate_planet_position(&planet.position, current_tick);
        let distance = ship.position.distance_to(&planet_position);
        
        if distance > 0.5 { // Realistic docking range
            return Err(GameError::InvalidOperation(
                format!("Ship {} not in docking range of planet {} (distance: {:.2})", 
                       ship.id, planet.id, distance)
            ));
        }
        
        // Check planet resource availability
        if !planet.resources.current.can_afford(&requested) {
            return Err(GameError::InsufficientResources {
                required: requested,
                available: planet.resources.current,
            });
        }
        
        // Check ship cargo capacity
        if !ship.cargo.can_load(&requested, 0) {
            let available_space = ship.cargo.available_space();
            let max_loadable = ResourceBundle {
                minerals: requested.minerals.min(available_space),
                food: requested.food.min(available_space),
                energy: requested.energy.min(available_space),
                alloys: requested.alloys.min(available_space),
                components: requested.components.min(available_space),
                fuel: requested.fuel.min(available_space),
            };
            
            if max_loadable.total() == 0 {
                return Err(GameError::InvalidOperation("Ship cargo hold is full".into()));
            }
            
            return Ok(max_loadable);
        }
        
        Ok(requested)
    }
    
    /// Validate ship cargo unloading operation
    /// Returns the actual unloadable amount considering planet storage capacity
    pub fn validate_cargo_unloading(&self, ship: &Ship, planet: &Planet, current_tick: u64) -> GameResult<ResourceBundle> {
        // Check if ship is at planet using proper orbital calculation
        let planet_position = self.calculate_planet_position(&planet.position, current_tick);
        let distance = ship.position.distance_to(&planet_position);
        
        if distance > 0.5 { // Realistic docking range
            return Err(GameError::InvalidOperation(
                format!("Ship {} not in docking range of planet {} (distance: {:.2})", 
                       ship.id, planet.id, distance)
            ));
        }
        
        let cargo_resources = ship.cargo.resources;
        
        // Check planet storage capacity using existing methods
        if !planet.resources.can_store(&cargo_resources) {
            let available_space = planet.resources.available_space();
            let max_unloadable = ResourceBundle {
                minerals: cargo_resources.minerals.min(available_space.minerals),
                food: cargo_resources.food.min(available_space.food),
                energy: cargo_resources.energy.min(available_space.energy),
                alloys: cargo_resources.alloys.min(available_space.alloys),
                components: cargo_resources.components.min(available_space.components),
                fuel: cargo_resources.fuel.min(available_space.fuel),
            };
            
            if max_unloadable.total() == 0 {
                return Err(GameError::InvalidOperation("Planet has no storage capacity".into()));
            }
            
            return Ok(max_unloadable);
        }
        
        Ok(cargo_resources)
    }
    
    /// Helper method to detect resource shortages
    fn detect_resource_shortages(&self, current: &ResourceBundle, production: &ResourceBundle) -> Vec<ResourceType> {
        let mut shortages = Vec::new();
        
        if current.minerals + production.minerals < 0 {
            shortages.push(ResourceType::Minerals);
        }
        if current.food + production.food < 0 {
            shortages.push(ResourceType::Food);
        }
        if current.energy + production.energy < 0 {
            shortages.push(ResourceType::Energy);
        }
        if current.alloys + production.alloys < 0 {
            shortages.push(ResourceType::Alloys);
        }
        if current.components + production.components < 0 {
            shortages.push(ResourceType::Components);
        }
        if current.fuel + production.fuel < 0 {
            shortages.push(ResourceType::Fuel);
        }
        
        shortages
    }
    
    /// Calculate effective production considering storage constraints
    fn calculate_effective_production(&self, current: &ResourceBundle, capacity: &ResourceBundle, production: &ResourceBundle) -> GameResult<ResourceBundle> {
        // Handle positive production (capped by storage) and negative production (consumption)
        let effective_production = ResourceBundle {
            minerals: if production.minerals > 0 {
                production.minerals.min(capacity.minerals - current.minerals)
            } else {
                production.minerals // Allow negative (consumption)
            },
            food: if production.food > 0 {
                production.food.min(capacity.food - current.food)
            } else {
                production.food
            },
            energy: if production.energy > 0 {
                production.energy.min(capacity.energy - current.energy)
            } else {
                production.energy
            },
            alloys: if production.alloys > 0 {
                production.alloys.min(capacity.alloys - current.alloys)
            } else {
                production.alloys
            },
            components: if production.components > 0 {
                production.components.min(capacity.components - current.components)
            } else {
                production.components
            },
            fuel: if production.fuel > 0 {
                production.fuel.min(capacity.fuel - current.fuel)
            } else {
                production.fuel
            },
        };
        
        Ok(effective_production)
    }
    
    /// Calculate planet position at given tick for proper orbital mechanics
    fn calculate_planet_position(&self, orbital_elements: &OrbitalElements, current_tick: u64) -> Vector2 {
        let time_in_orbit = (current_tick as f32) / orbital_elements.period;
        let angle = orbital_elements.phase + (time_in_orbit * 2.0 * std::f32::consts::PI);
        
        Vector2 {
            x: orbital_elements.semi_major_axis * angle.cos(),
            y: orbital_elements.semi_major_axis * angle.sin(),
        }
    }
}

impl GameSystem for ResourceSystem {
    fn update(&mut self, _delta: f32, _event_bus: &mut EventBus) -> GameResult<()> {
        // ResourceSystem processes production during tick events, not on continuous update
        // All processing is coordinated through GameState to maintain architecture boundaries
        Ok(())
    }
    
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        // ResourceSystem validates events but does not perform direct state mutations
        // This maintains the EventBus architecture where only managers modify state
        match event {
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    SimulationEvent::TickCompleted(_tick) => {
                        // Tick processing coordinated by GameState
                    }
                    SimulationEvent::ResourcesProduced { planet, resources } => {
                        // Update consumption tracking for this planet
                        self.consumption_tracking.insert(*planet, *resources);
                    }
                    _ => {}
                }
            }
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    PlayerCommand::TransferResources { .. } |
                    PlayerCommand::LoadShipCargo { .. } |
                    PlayerCommand::UnloadShipCargo { .. } => {
                        // Commands validated and processed by GameState coordination
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}