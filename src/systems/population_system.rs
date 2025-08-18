// src/systems/population_system.rs
use crate::core::{GameResult, GameEvent, EventBus, GameSystem};
use crate::core::types::*;
use std::collections::HashMap;

pub struct PopulationSystem {
    growth_modifiers: HashMap<PlanetId, f32>,
    migration_queue: Vec<MigrationOrder>,
}

#[derive(Debug, Clone)]
pub struct MigrationOrder {
    pub from: PlanetId,
    pub to: PlanetId,
    pub population: i32,
    pub ship_id: ShipId,
}

impl PopulationSystem {
    pub fn new() -> Self {
        Self {
            growth_modifiers: HashMap::new(),
            migration_queue: Vec::new(),
        }
    }
    
    pub fn update(&mut self, delta: f32, event_bus: &mut EventBus) -> GameResult<()> {
        // PopulationSystem processes growth during tick events, not during update
        // Migration is handled in handle_event when ships arrive
        Ok(())
    }
    
    
    pub fn process_growth(&mut self, tick: u64, planet_manager: &mut crate::managers::PlanetManager, event_bus: &mut EventBus) -> GameResult<()> {
        // Calculate population growth based on food surplus
        // Growth +2%/tick with food surplus >20%
        
        let all_planets = planet_manager.get_all_planets().clone();
        
        for planet in all_planets.iter() {
            // Only process planets with population
            if planet.population.total <= 0 {
                continue;
            }
            
            // Calculate food surplus percentage
            let food_available = planet.resources.current.food;
            let food_consumed_per_tick = planet.population.total; // 1 food per person per tick
            let food_surplus_ratio = if food_consumed_per_tick > 0 {
                (food_available - food_consumed_per_tick) as f32 / food_consumed_per_tick as f32
            } else {
                0.0
            };
            
            // Apply growth if food surplus > 20%
            if food_surplus_ratio > 0.2 {
                let growth_rate = 0.02; // 2% per tick
                let growth_amount = (planet.population.total as f32 * growth_rate) as i32;
                
                if growth_amount > 0 {
                    // Update population through planet manager
                    planet_manager.update_population(planet.id, growth_amount)?;
                    
                    // Store growth modifier for this planet
                    self.growth_modifiers.insert(planet.id, growth_rate);
                    
                    // Emit population growth event
                    event_bus.queue_event(GameEvent::SimulationEvent(
                        crate::core::events::SimulationEvent::PopulationGrowth {
                            planet: planet.id,
                            amount: growth_amount,
                        }
                    ));
                    
                    // Emit planet updated state change
                    event_bus.queue_event(GameEvent::StateChanged(
                        crate::core::events::StateChange::PlanetUpdated(planet.id)
                    ));
                }
            } else {
                // Remove growth modifier if no longer growing
                self.growth_modifiers.remove(&planet.id);
            }
        }
        
        Ok(())
    }
    
    pub fn process_migration(&mut self, ship_id: ShipId, planet_manager: &mut crate::managers::PlanetManager, ship_manager: &mut crate::managers::ShipManager, event_bus: &mut EventBus) -> GameResult<()> {
        // Handle population migration when transport ships arrive
        
        // Check if there's a migration order for this ship
        if let Some(index) = self.migration_queue.iter().position(|order| order.ship_id == ship_id) {
            let migration_order = self.migration_queue.remove(index);
            
            // Get the ship to verify it's a transport ship with population
            let ship = ship_manager.get_ship(ship_id)?;
            if ship.ship_class != ShipClass::Transport {
                return Err(GameError::InvalidOperation(
                    "Only transport ships can carry population".into()
                ));
            }
            
            // Check if ship has population cargo
            if ship.cargo.population <= 0 {
                return Ok(()); // No population to transfer
            }
            
            // Find destination planet based on ship position (simplified - assumes ship arrives at planet)
            let destination_planet = planet_manager.get_all_planets()
                .iter()
                .find(|planet| {
                    // Simple distance check - in a real implementation, this would use physics
                    let distance = ((planet.position.semi_major_axis - ship.position.x).powi(2) + 
                                   (0.0 - ship.position.y).powi(2)).sqrt();
                    distance < 1.0 // Within 1 unit
                })
                .map(|p| p.id);
            
            if let Some(planet_id) = destination_planet {
                // Transfer population from ship to planet
                let population_to_transfer = ship.cargo.population;
                
                // Update planet population
                planet_manager.update_population(planet_id, population_to_transfer)?;
                
                // Clear ship cargo (this should be done through ship manager methods)
                // For now, we'll emit an event to signal the transfer
                event_bus.queue_event(GameEvent::SimulationEvent(
                    crate::core::events::SimulationEvent::PopulationGrowth {
                        planet: planet_id,
                        amount: population_to_transfer,
                    }
                ));
                
                // Emit planet updated state change
                event_bus.queue_event(GameEvent::StateChanged(
                    crate::core::events::StateChange::PlanetUpdated(planet_id)
                ));
                
                // Emit ship updated state change
                event_bus.queue_event(GameEvent::StateChanged(
                    crate::core::events::StateChange::ShipUpdated(ship_id)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn process_allocation(&mut self, planet_id: PlanetId, allocation: WorkerAllocation, planet_manager: &mut crate::managers::PlanetManager, event_bus: &mut EventBus) -> GameResult<()> {
        // Update worker allocation for a planet with validation
        
        // Get current planet to validate allocation
        let planet = planet_manager.get_planet(planet_id)?;
        
        // Validate that allocation matches total population
        allocation.validate(planet.population.total)?;
        
        // Ensure minimum 10% unassigned workers (requirement from specifications)
        let min_unassigned = (planet.population.total as f32 * 0.1) as i32;
        if allocation.unassigned < min_unassigned {
            return Err(GameError::InvalidOperation(
                format!("Must maintain at least 10% unassigned workers. Required: {}, Provided: {}", 
                       min_unassigned, allocation.unassigned)
            ));
        }
        
        // Update allocation through planet manager
        planet_manager.set_worker_allocation(planet_id, allocation)?;
        
        // Emit planet updated state change
        event_bus.queue_event(GameEvent::StateChanged(
            crate::core::events::StateChange::PlanetUpdated(planet_id)
        ));
        
        Ok(())
    }
}

impl GameSystem for PopulationSystem {
    fn update(&mut self, _delta: f32, _event_bus: &mut EventBus) -> GameResult<()> {
        // PopulationSystem processes growth during tick events, not during update
        // Migration is handled in handle_event when ships arrive
        Ok(())
    }
    
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    crate::core::events::SimulationEvent::TickCompleted(_tick) => {
                        // Population growth processing is handled by GameState
                    }
                    crate::core::events::SimulationEvent::ShipArrived { ship: _ship, destination: _ } => {
                        // Migration processing is handled by GameState
                    }
                    _ => {}
                }
            }
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    crate::core::events::PlayerCommand::AllocateWorkers { planet: _planet, allocation: _allocation } => {
                        // Worker allocation processing is handled by GameState
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}