// src/systems/population_system.rs
//
// Population System - Handles population growth, migration, and worker allocation
// Follows EventBus architecture for all system communication
use crate::core::{GameResult, GameEvent, EventBus, GameSystem};
use crate::core::types::*;
use crate::core::events::{SimulationEvent, PlayerCommand, StateChange};
use std::collections::HashMap;

/// PopulationSystem manages population dynamics including:
/// - Population growth based on food surplus (2% per tick with >20% surplus)
/// - Migration between planets via transport ships  
/// - Worker allocation validation and management
/// - Food consumption (1 food per person per tick)
pub struct PopulationSystem {
    /// Cached growth modifiers per planet for efficiency
    growth_modifiers: HashMap<PlanetId, f32>,
    /// Queue of pending migration orders awaiting ship arrivals
    migration_queue: Vec<MigrationOrder>,
    /// Current tick for deterministic processing
    current_tick: u64,
}

/// Represents a population migration order linked to a transport ship
/// Tracks all necessary information for completing migration when ship arrives
#[derive(Debug, Clone)]
pub struct MigrationOrder {
    /// Source planet ID
    pub from: PlanetId,
    /// Destination planet ID
    pub to: PlanetId,
    /// Number of people to migrate
    pub population: i32,
    /// Transport ship carrying the population
    pub ship_id: ShipId,
    /// Tick when migration order was created (for expiration)
    pub created_tick: u64,
}

impl PopulationSystem {
    /// Creates a new PopulationSystem with empty state
    pub fn new() -> Self {
        Self {
            growth_modifiers: HashMap::with_capacity(100), // Pre-allocate for performance
            migration_queue: Vec::with_capacity(50),
            current_tick: 0,
        }
    }
    
    /// Main update method - processes queued events only
    /// Population logic is handled through event responses
    pub fn update(&mut self, _delta: f32, _event_bus: &mut EventBus) -> GameResult<()> {
        // All population processing happens in response to events
        // This maintains strict EventBus architecture compliance
        Ok(())
    }
    
    
    /// Processes population growth for a specific planet based on food surplus
    /// Called internally when processing tick events
    fn process_planet_growth(&mut self, planet_id: PlanetId, population: i32, food_available: i32, event_bus: &mut EventBus) -> GameResult<()> {
        // Validate inputs
        if population <= 0 {
            return Ok(()); // No population to grow
        }
        
        if food_available < 0 {
            return Err(GameError::InvalidOperation(
                "Food availability cannot be negative".into()
            ));
        }
        
        // Calculate food consumption (1 food per person per tick)
        let food_consumed_per_tick = population;
        
        // Check if there's enough food for basic consumption
        if food_available < food_consumed_per_tick {
            // Emit food shortage event - population can't grow
            event_bus.queue_event(GameEvent::SimulationEvent(
                SimulationEvent::ResourceShortage {
                    planet: planet_id,
                    resource: ResourceType::Food,
                }
            ));
            
            // Remove any cached growth modifier
            self.growth_modifiers.remove(&planet_id);
            return Ok(());
        }
        
        // Calculate food surplus ratio
        let food_surplus = food_available - food_consumed_per_tick;
        let food_surplus_ratio = food_surplus as f32 / food_consumed_per_tick as f32;
        
        // Apply growth only if food surplus > 20%
        if food_surplus_ratio > 0.2 {
            const GROWTH_RATE: f32 = 0.02; // 2% per tick
            let growth_amount = (population as f32 * GROWTH_RATE).floor() as i32;
            
            if growth_amount > 0 {
                // Cache growth modifier for efficiency
                self.growth_modifiers.insert(planet_id, GROWTH_RATE);
                
                // Request food consumption for growth
                let food_cost = ResourceBundle {
                    food: food_consumed_per_tick,
                    ..Default::default()
                };
                
                // Emit resource consumption request
                event_bus.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::TransferResources {
                        from: planet_id,
                        to: planet_id, // Same planet - indicates consumption
                        resources: food_cost,
                    }
                ));
                
                // Emit population growth event
                event_bus.queue_event(GameEvent::SimulationEvent(
                    SimulationEvent::PopulationGrowth {
                        planet: planet_id,
                        amount: growth_amount,
                    }
                ));
                
                // Request population update through proper channels
                event_bus.queue_event(GameEvent::StateChanged(
                    StateChange::PlanetUpdated(planet_id)
                ));
            }
        } else {
            // Remove growth modifier if no longer growing
            self.growth_modifiers.remove(&planet_id);
            
            // Still consume basic food
            let food_cost = ResourceBundle {
                food: food_consumed_per_tick,
                ..Default::default()
            };
            
            event_bus.queue_event(GameEvent::PlayerCommand(
                PlayerCommand::TransferResources {
                    from: planet_id,
                    to: planet_id,
                    resources: food_cost,
                }
            ));
        }
        
        Ok(())
    }
    
    
    
    
    /// Returns current growth rate for a planet (if any)
    pub fn get_growth_rate(&self, planet_id: PlanetId) -> Option<f32> {
        self.growth_modifiers.get(&planet_id).copied()
    }
    
    /// Returns count of pending migration orders
    pub fn pending_migrations(&self) -> usize {
        self.migration_queue.len()
    }
    
    /// Cleans up expired migration orders to prevent memory leaks
    fn cleanup_expired_migrations(&mut self) {
        const MAX_MIGRATION_AGE: u64 = 1000; // ticks
        
        self.migration_queue.retain(|order| {
            self.current_tick.saturating_sub(order.created_tick) <= MAX_MIGRATION_AGE
        });
    }
}

impl GameSystem for PopulationSystem {
    fn update(&mut self, _delta: f32, _event_bus: &mut EventBus) -> GameResult<()> {
        // Perform periodic cleanup of expired migration orders
        self.cleanup_expired_migrations();
        
        // All other processing happens through event handling
        Ok(())
    }
    
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    SimulationEvent::TickCompleted(tick) => {
                        self.current_tick = *tick;
                        // Population growth will be triggered by ResourceSystem
                        // after it calculates available resources
                    }
                    
                    SimulationEvent::ResourcesProduced { planet: _planet, resources } => {
                        // Process population growth based on available food
                        // Note: Actual implementation would need planet data from managers
                        // For now, we emit an event requesting growth calculation
                        if resources.food > 0 {
                            // This would be handled by GameState which has manager access
                            // Population system just validates and emits appropriate events
                        }
                    }
                    
                    SimulationEvent::ShipArrived { ship, destination: _destination } => {
                        // Handle population migration when transport ships arrive
                        // Find pending migration order for this ship
                        if let Some(index) = self.migration_queue.iter().position(|order| order.ship_id == *ship) {
                            let migration_order = self.migration_queue.remove(index);
                            
                            // Validate migration order isn't stale (older than 1000 ticks)
                            if self.current_tick.saturating_sub(migration_order.created_tick) <= 1000 {
                                // Request cargo unload - this will be handled by ship/planet managers
                                // Population system just tracks the migration request
                            }
                        }
                    }
                    
                    SimulationEvent::ShipCompleted { planet: _planet, ship: _ship } => {
                        // Transport ship completion would be handled by ship manager
                        // Population system just tracks migration requests
                    }
                    
                    _ => {}
                }
            }
            
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    PlayerCommand::AllocateWorkers { planet: _planet, allocation } => {
                        // Validate worker allocation rules
                        // Note: Actual population data would come from planet manager
                        // For now, just validate the allocation structure itself
                        let total_allocated = allocation.agriculture + allocation.mining + 
                                            allocation.industry + allocation.research + 
                                            allocation.military + allocation.unassigned;
                        
                        if total_allocated > 0 {
                            // Basic validation - ensure 10% minimum unassigned
                            let min_unassigned = (total_allocated as f32 * 0.1) as i32;
                            if allocation.unassigned < min_unassigned {
                                return Err(GameError::InvalidOperation(
                                    format!("Must maintain at least 10% unassigned workers. Required: {}, Provided: {}", 
                                           min_unassigned, allocation.unassigned)
                                ));
                            }
                        }
                    }
                    
                    PlayerCommand::TransferResources { from, to, resources: _resources } => {
                        // Population migration would be handled through specialized migration commands
                        // For now, just validate that from != to for any potential migration
                        if from == to {
                            return Err(GameError::InvalidOperation(
                                "Cannot transfer resources to the same planet".into()
                            ));
                        }
                    }
                    
                    _ => {}
                }
            }
            
            _ => {}
        }
        Ok(())
    }
}

