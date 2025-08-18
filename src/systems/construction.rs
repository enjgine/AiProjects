// src/systems/construction.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use std::collections::HashMap;

/// Represents a building construction order in the queue
#[derive(Debug, Clone)]
pub struct ConstructionOrder {
    pub building_type: BuildingType,
    pub planet_id: PlanetId,
    pub start_tick: u64,
    pub completion_tick: u64,
    pub cost_paid: ResourceBundle,
}

/// Represents a ship construction order in the queue
#[derive(Debug, Clone)]
pub struct ShipOrder {
    pub ship_class: ShipClass,
    pub planet_id: PlanetId,
    pub start_tick: u64,
    pub completion_tick: u64,
    pub cost_paid: ResourceBundle,
}

/// Construction system manages building and ship construction queues
pub struct ConstructionSystem {
    building_queue: HashMap<PlanetId, Vec<ConstructionOrder>>,
    ship_queue: HashMap<PlanetId, Vec<ShipOrder>>,
    building_costs: HashMap<BuildingType, (ResourceBundle, u64)>,
    ship_costs: HashMap<ShipClass, (ResourceBundle, u64)>,
    current_tick: u64,
}

impl ConstructionSystem {
    pub fn new() -> Self {
        let mut building_costs = HashMap::new();
        let mut ship_costs = HashMap::new();
        
        // Define building construction costs and times (in ticks)
        building_costs.insert(BuildingType::Mine, (ResourceBundle {
            minerals: 100,
            food: 0,
            energy: 0,
            alloys: 20,
            components: 10,
            fuel: 0,
        }, 10));
        
        building_costs.insert(BuildingType::Farm, (ResourceBundle {
            minerals: 50,
            food: 0,
            energy: 0,
            alloys: 10,
            components: 5,
            fuel: 0,
        }, 8));
        
        building_costs.insert(BuildingType::PowerPlant, (ResourceBundle {
            minerals: 80,
            food: 0,
            energy: 0,
            alloys: 30,
            components: 15,
            fuel: 0,
        }, 12));
        
        building_costs.insert(BuildingType::Factory, (ResourceBundle {
            minerals: 120,
            food: 0,
            energy: 10,
            alloys: 40,
            components: 25,
            fuel: 0,
        }, 15));
        
        building_costs.insert(BuildingType::ResearchLab, (ResourceBundle {
            minerals: 90,
            food: 0,
            energy: 5,
            alloys: 25,
            components: 30,
            fuel: 0,
        }, 14));
        
        building_costs.insert(BuildingType::Spaceport, (ResourceBundle {
            minerals: 200,
            food: 0,
            energy: 20,
            alloys: 80,
            components: 60,
            fuel: 0,
        }, 25));
        
        building_costs.insert(BuildingType::DefensePlatform, (ResourceBundle {
            minerals: 150,
            food: 0,
            energy: 15,
            alloys: 100,
            components: 50,
            fuel: 0,
        }, 20));
        
        building_costs.insert(BuildingType::StorageFacility, (ResourceBundle {
            minerals: 60,
            food: 0,
            energy: 0,
            alloys: 15,
            components: 8,
            fuel: 0,
        }, 6));
        
        building_costs.insert(BuildingType::Habitat, (ResourceBundle {
            minerals: 80,
            food: 10,
            energy: 5,
            alloys: 20,
            components: 15,
            fuel: 0,
        }, 10));
        
        // Define ship construction costs and times (in ticks)
        ship_costs.insert(ShipClass::Scout, (ResourceBundle {
            minerals: 50,
            food: 0,
            energy: 0,
            alloys: 30,
            components: 25,
            fuel: 10,
        }, 8));
        
        ship_costs.insert(ShipClass::Transport, (ResourceBundle {
            minerals: 80,
            food: 0,
            energy: 0,
            alloys: 40,
            components: 35,
            fuel: 15,
        }, 12));
        
        ship_costs.insert(ShipClass::Colony, (ResourceBundle {
            minerals: 150,
            food: 50,
            energy: 10,
            alloys: 60,
            components: 80,
            fuel: 25,
        }, 20));
        
        ship_costs.insert(ShipClass::Warship, (ResourceBundle {
            minerals: 200,
            food: 0,
            energy: 5,
            alloys: 120,
            components: 100,
            fuel: 30,
        }, 25));
        
        Self {
            building_queue: HashMap::new(),
            ship_queue: HashMap::new(),
            building_costs,
            ship_costs,
            current_tick: 0,
        }
    }
    
    /// Update construction system - processes completion of construction orders
    pub fn update(&mut self, _delta: f32, event_bus: &mut EventBus) -> GameResult<()> {
        // Process building completions
        self.process_building_completions(event_bus)?;
        
        // Process ship completions
        self.process_ship_completions(event_bus)?;
        
        Ok(())
    }
    
    /// Handle events from the EventBus
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    crate::core::events::PlayerCommand::BuildStructure { planet, building_type } => {
                        // Note: Resource validation must be done by ResourceSystem
                        // We queue the order and emit an event requesting resource deduction
                        self.request_building_construction(*planet, *building_type)?;
                    }
                    crate::core::events::PlayerCommand::ConstructShip { planet, ship_class } => {
                        self.request_ship_construction(*planet, *ship_class)?;
                    }
                    _ => {}
                }
            }
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    crate::core::events::SimulationEvent::TickCompleted(tick) => {
                        self.current_tick = *tick;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Request building construction - emits resource requirement event
    fn request_building_construction(&mut self, planet_id: PlanetId, building_type: BuildingType) -> GameResult<()> {
        let (cost, build_time) = self.building_costs.get(&building_type)
            .ok_or_else(|| GameError::SystemError(format!("Unknown building type: {:?}", building_type)))?;
        
        // For now, we'll add to queue immediately
        // In a full implementation, this would wait for resource confirmation
        let order = ConstructionOrder {
            building_type,
            planet_id,
            start_tick: self.current_tick,
            completion_tick: self.current_tick + build_time,
            cost_paid: *cost,
        };
        
        self.building_queue
            .entry(planet_id)
            .or_insert_with(Vec::new)
            .push(order);
            
        Ok(())
    }
    
    /// Request ship construction - emits resource requirement event  
    fn request_ship_construction(&mut self, planet_id: PlanetId, ship_class: ShipClass) -> GameResult<()> {
        let (cost, build_time) = self.ship_costs.get(&ship_class)
            .ok_or_else(|| GameError::SystemError(format!("Unknown ship class: {:?}", ship_class)))?;
        
        // For now, we'll add to queue immediately
        // In a full implementation, this would wait for resource confirmation
        let order = ShipOrder {
            ship_class,
            planet_id,
            start_tick: self.current_tick,
            completion_tick: self.current_tick + build_time,
            cost_paid: *cost,
        };
        
        self.ship_queue
            .entry(planet_id)
            .or_insert_with(Vec::new)
            .push(order);
            
        Ok(())
    }
    
    /// Process completed building constructions
    fn process_building_completions(&mut self, event_bus: &mut EventBus) -> GameResult<()> {
        let mut completed_orders = Vec::new();
        
        // Collect completed orders
        for (planet_id, orders) in self.building_queue.iter_mut() {
            let mut i = 0;
            while i < orders.len() {
                if orders[i].completion_tick <= self.current_tick {
                    let completed = orders.remove(i);
                    completed_orders.push((*planet_id, completed));
                } else {
                    i += 1;
                }
            }
        }
        
        // Emit completion events
        for (planet_id, order) in completed_orders {
            event_bus.queue_event(GameEvent::SimulationEvent(
                crate::core::events::SimulationEvent::ConstructionCompleted {
                    planet: planet_id,
                    building: order.building_type,
                }
            ));
        }
        
        Ok(())
    }
    
    /// Process completed ship constructions
    fn process_ship_completions(&mut self, event_bus: &mut EventBus) -> GameResult<()> {
        let mut completed_orders = Vec::new();
        
        // Collect completed orders
        for (planet_id, orders) in self.ship_queue.iter_mut() {
            let mut i = 0;
            while i < orders.len() {
                if orders[i].completion_tick <= self.current_tick {
                    let completed = orders.remove(i);
                    completed_orders.push((*planet_id, completed));
                } else {
                    i += 1;
                }
            }
        }
        
        // Emit completion events with placeholder ship IDs
        // Real implementation would get ship ID from ShipManager
        for (planet_id, order) in completed_orders {
            event_bus.queue_event(GameEvent::SimulationEvent(
                crate::core::events::SimulationEvent::ShipCompleted {
                    planet: planet_id,
                    ship: 0, // Placeholder - would be real ship ID from ShipManager
                }
            ));
        }
        
        Ok(())
    }
    
    /// Get building construction cost and time
    pub fn get_building_cost(&self, building_type: BuildingType) -> Option<&(ResourceBundle, u64)> {
        self.building_costs.get(&building_type)
    }
    
    /// Get ship construction cost and time
    pub fn get_ship_cost(&self, ship_class: ShipClass) -> Option<&(ResourceBundle, u64)> {
        self.ship_costs.get(&ship_class)
    }
    
    /// Get current building queue for a planet
    pub fn get_building_queue(&self, planet_id: PlanetId) -> Vec<&ConstructionOrder> {
        self.building_queue.get(&planet_id)
            .map(|queue| queue.iter().collect())
            .unwrap_or_default()
    }
    
    /// Get current ship queue for a planet
    pub fn get_ship_queue(&self, planet_id: PlanetId) -> Vec<&ShipOrder> {
        self.ship_queue.get(&planet_id)
            .map(|queue| queue.iter().collect())
            .unwrap_or_default()
    }
    
    /// Cancel building construction (returns resources if implemented)
    pub fn cancel_building(&mut self, planet_id: PlanetId, order_index: usize) -> GameResult<ConstructionOrder> {
        let queue = self.building_queue.get_mut(&planet_id)
            .ok_or_else(|| GameError::InvalidOperation("No construction queue for planet".into()))?;
        
        if order_index >= queue.len() {
            return Err(GameError::InvalidOperation("Invalid construction order index".into()));
        }
        
        Ok(queue.remove(order_index))
    }
    
    /// Cancel ship construction (returns resources if implemented)
    pub fn cancel_ship(&mut self, planet_id: PlanetId, order_index: usize) -> GameResult<ShipOrder> {
        let queue = self.ship_queue.get_mut(&planet_id)
            .ok_or_else(|| GameError::InvalidOperation("No ship construction queue for planet".into()))?;
        
        if order_index >= queue.len() {
            return Err(GameError::InvalidOperation("Invalid ship order index".into()));
        }
        
        Ok(queue.remove(order_index))
    }
    
    /// Get total construction queue length for a planet
    pub fn get_total_queue_length(&self, planet_id: PlanetId) -> usize {
        let building_count = self.building_queue.get(&planet_id)
            .map(|q| q.len()).unwrap_or(0);
        let ship_count = self.ship_queue.get(&planet_id)
            .map(|q| q.len()).unwrap_or(0);
        building_count + ship_count
    }
    
    /// Validate construction system state
    pub fn validate(&self) -> GameResult<()> {
        // Validate all orders have valid completion times
        for (_, orders) in &self.building_queue {
            for order in orders {
                if order.completion_tick < order.start_tick {
                    return Err(GameError::SystemError(
                        "Building order has invalid completion time".into()
                    ));
                }
                order.cost_paid.validate_non_negative()?;
            }
        }
        
        for (_, orders) in &self.ship_queue {
            for order in orders {
                if order.completion_tick < order.start_tick {
                    return Err(GameError::SystemError(
                        "Ship order has invalid completion time".into()
                    ));
                }
                order.cost_paid.validate_non_negative()?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_construction_system_creation() {
        let system = ConstructionSystem::new();
        assert_eq!(system.current_tick, 0);
        assert!(system.building_queue.is_empty());
        assert!(system.ship_queue.is_empty());
        
        // Verify all building types have costs defined
        for building_type in [
            BuildingType::Mine,
            BuildingType::Farm,
            BuildingType::PowerPlant,
            BuildingType::Factory,
            BuildingType::ResearchLab,
            BuildingType::Spaceport,
            BuildingType::DefensePlatform,
            BuildingType::StorageFacility,
            BuildingType::Habitat,
        ] {
            assert!(system.get_building_cost(building_type).is_some());
        }
        
        // Verify all ship classes have costs defined
        for ship_class in [
            ShipClass::Scout,
            ShipClass::Transport,
            ShipClass::Colony,
            ShipClass::Warship,
        ] {
            assert!(system.get_ship_cost(ship_class).is_some());
        }
    }
    
    #[test]
    fn test_building_construction_request() {
        let mut system = ConstructionSystem::new();
        let mut event_bus = EventBus::new();
        
        // Test building construction request
        let result = system.request_building_construction(1, BuildingType::Mine);
        assert!(result.is_ok());
        
        let queue = system.get_building_queue(1);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].building_type, BuildingType::Mine);
        assert_eq!(queue[0].planet_id, 1);
    }
    
    #[test]
    fn test_ship_construction_request() {
        let mut system = ConstructionSystem::new();
        
        // Test ship construction request
        let result = system.request_ship_construction(1, ShipClass::Scout);
        assert!(result.is_ok());
        
        let queue = system.get_ship_queue(1);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].ship_class, ShipClass::Scout);
        assert_eq!(queue[0].planet_id, 1);
    }
    
    #[test]
    fn test_construction_completion() {
        let mut system = ConstructionSystem::new();
        let mut event_bus = EventBus::new();
        
        // Add a building to queue
        system.request_building_construction(1, BuildingType::Mine).unwrap();
        
        // Advance time past completion
        system.current_tick = 15; // Mine takes 10 ticks
        
        // Process completions
        system.process_building_completions(&mut event_bus).unwrap();
        
        // Verify building queue is now empty
        let queue = system.get_building_queue(1);
        assert_eq!(queue.len(), 0);
        
        // Verify completion event was emitted
        assert_eq!(event_bus.queued_events.len(), 1);
    }
    
    #[test]
    fn test_queue_cancellation() {
        let mut system = ConstructionSystem::new();
        
        // Add orders to queues
        system.request_building_construction(1, BuildingType::Mine).unwrap();
        system.request_ship_construction(1, ShipClass::Scout).unwrap();
        
        // Test cancellation
        let cancelled_building = system.cancel_building(1, 0);
        assert!(cancelled_building.is_ok());
        assert_eq!(cancelled_building.unwrap().building_type, BuildingType::Mine);
        
        let cancelled_ship = system.cancel_ship(1, 0);
        assert!(cancelled_ship.is_ok());
        assert_eq!(cancelled_ship.unwrap().ship_class, ShipClass::Scout);
        
        // Verify queues are empty
        assert_eq!(system.get_building_queue(1).len(), 0);
        assert_eq!(system.get_ship_queue(1).len(), 0);
    }
    
    #[test]
    fn test_validation() {
        let system = ConstructionSystem::new();
        let result = system.validate();
        assert!(result.is_ok());
    }
}