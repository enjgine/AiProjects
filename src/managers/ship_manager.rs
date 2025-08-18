// src/managers/ship_manager.rs
use crate::core::{GameResult, GameEvent};
use crate::core::types::*;
use crate::core::events::{PlayerCommand, SimulationEvent};
use std::collections::HashMap;

pub struct ShipManager {
    ships: Vec<Ship>,
    next_id: ShipId,
    ship_index: HashMap<ShipId, usize>,
}

impl ShipManager {
    pub fn new() -> Self {
        Self {
            ships: Vec::new(),
            next_id: 0,
            ship_index: HashMap::new(),
        }
    }
    
    pub fn create_ship(&mut self, ship_class: ShipClass, position: Vector2, owner: FactionId) -> GameResult<ShipId> {
        let id = self.next_id;
        self.next_id += 1;
        
        let ship = Ship {
            id,
            ship_class,
            position,
            trajectory: None,
            cargo: CargoHold::default(),
            fuel: 100.0, // Default fuel
            owner,
        };
        
        self.ships.push(ship);
        self.ship_index.insert(id, self.ships.len() - 1);
        
        Ok(id)
    }
    
    pub fn get_ship(&self, id: ShipId) -> GameResult<&Ship> {
        let index = self.ship_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Ship {} not found", id)))?;
        Ok(&self.ships[*index])
    }
    
    pub fn update_position(&mut self, id: ShipId, position: Vector2) -> GameResult<()> {
        let index = self.ship_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Ship {} not found", id)))?;
        
        self.ships[*index].position = position;
        Ok(())
    }
    
    pub fn destroy_ship(&mut self, id: ShipId) -> GameResult<()> {
        let index = self.ship_index.remove(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Ship {} not found", id)))?;
        
        self.ships.remove(index);
        
        // Rebuild index as positions shifted
        self.ship_index.clear();
        for (i, ship) in self.ships.iter().enumerate() {
            self.ship_index.insert(ship.id, i);
        }
        
        Ok(())
    }
    
    pub fn get_ship_mut(&mut self, id: ShipId) -> GameResult<&mut Ship> {
        let index = self.ship_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Ship {} not found", id)))?;
        Ok(&mut self.ships[*index])
    }
    
    pub fn load_cargo(&mut self, ship_id: ShipId, resources: ResourceBundle) -> GameResult<()> {
        let ship = self.get_ship_mut(ship_id)?;
        
        // Check if ship can carry resources (only transport ships for now)
        if ship.ship_class != ShipClass::Transport {
            return Err(GameError::InvalidOperation("Only transport ships can carry cargo".into()));
        }
        
        // Calculate current cargo weight (simplified: assume each resource unit = 1 weight)
        let current_weight = ship.cargo.resources.minerals + ship.cargo.resources.food +
                            ship.cargo.resources.energy + ship.cargo.resources.alloys +
                            ship.cargo.resources.components + ship.cargo.resources.fuel +
                            ship.cargo.population;
        
        let additional_weight = resources.minerals + resources.food + resources.energy +
                               resources.alloys + resources.components + resources.fuel;
        
        if current_weight + additional_weight > ship.cargo.capacity {
            return Err(GameError::InvalidOperation("Cargo capacity exceeded".into()));
        }
        
        // Load the resources
        ship.cargo.resources.minerals += resources.minerals;
        ship.cargo.resources.food += resources.food;
        ship.cargo.resources.energy += resources.energy;
        ship.cargo.resources.alloys += resources.alloys;
        ship.cargo.resources.components += resources.components;
        ship.cargo.resources.fuel += resources.fuel;
        
        Ok(())
    }
    
    pub fn unload_cargo(&mut self, ship_id: ShipId) -> GameResult<ResourceBundle> {
        let ship = self.get_ship_mut(ship_id)?;
        
        let cargo_resources = ship.cargo.resources;
        ship.cargo.resources = ResourceBundle::default();
        
        Ok(cargo_resources)
    }
    
    pub fn get_cargo_capacity(&self, ship_id: ShipId) -> GameResult<i32> {
        let ship = self.get_ship(ship_id)?;
        Ok(ship.cargo.capacity)
    }
    
    pub fn get_cargo_contents(&self, ship_id: ShipId) -> GameResult<&ResourceBundle> {
        let ship = self.get_ship(ship_id)?;
        Ok(&ship.cargo.resources)
    }
    
    pub fn set_trajectory(&mut self, ship_id: ShipId, trajectory: Trajectory) -> GameResult<()> {
        let ship = self.get_ship_mut(ship_id)?;
        ship.trajectory = Some(trajectory);
        Ok(())
    }
    
    pub fn consume_fuel(&mut self, ship_id: ShipId, amount: f32) -> GameResult<()> {
        let ship = self.get_ship_mut(ship_id)?;
        
        if ship.fuel < amount {
            return Err(GameError::InvalidOperation("Insufficient fuel".into()));
        }
        
        ship.fuel -= amount;
        Ok(())
    }
    
    pub fn get_ships_at_planet(&self, planet_position: Vector2, radius: f32) -> Vec<ShipId> {
        self.ships.iter()
            .filter(|ship| {
                let distance = ((ship.position.x - planet_position.x).powi(2) + 
                               (ship.position.y - planet_position.y).powi(2)).sqrt();
                distance <= radius
            })
            .map(|ship| ship.id)
            .collect()
    }
    
    pub fn get_all_ships(&self) -> &Vec<Ship> {
        &self.ships
    }
    
    pub fn get_all_ships_cloned(&self) -> GameResult<Vec<Ship>> {
        Ok(self.ships.clone())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    PlayerCommand::MoveShip { ship, target } => {
                        self.handle_move_ship(*ship, *target)
                    }
                    PlayerCommand::LoadShipCargo { ship, planet: _, resources } => {
                        self.load_cargo(*ship, *resources)
                    }
                    PlayerCommand::UnloadShipCargo { ship, planet: _ } => {
                        self.unload_cargo(*ship)?;
                        Ok(())
                    }
                    _ => Ok(())
                }
            }
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    SimulationEvent::ShipCompleted { planet: _, ship } => {
                        // Ship already created by ConstructionSystem, just mark as completed
                        Ok(())
                    }
                    SimulationEvent::CombatResolved { attacker: _, defender: _, outcome } => {
                        self.handle_combat_resolved(outcome)
                    }
                    _ => Ok(())
                }
            }
            _ => Ok(())
        }
    }
    
    fn handle_move_ship(&mut self, ship_id: ShipId, target: Vector2) -> GameResult<()> {
        let ship = self.get_ship_mut(ship_id)?;
        
        // Calculate fuel cost: Speed × Distance / 100
        let distance = ((target.x - ship.position.x).powi(2) + 
                       (target.y - ship.position.y).powi(2)).sqrt();
        
        // Base fuel consumption (can be modified by ship class)
        let fuel_cost = match ship.ship_class {
            ShipClass::Scout => distance / 200.0,      // More efficient
            ShipClass::Transport => distance / 100.0,  // Standard
            ShipClass::Warship => distance / 80.0,     // Less efficient
            ShipClass::Colony => distance / 60.0,      // Least efficient
        };
        
        // Validate fuel availability
        if ship.fuel < fuel_cost {
            return Err(GameError::InsufficientResources {
                required: ResourceBundle { fuel: fuel_cost as i32, ..Default::default() },
                available: ResourceBundle { fuel: ship.fuel as i32, ..Default::default() },
            });
        }
        
        // Create trajectory
        let trajectory = Trajectory {
            origin: ship.position,
            destination: target,
            departure_time: 0, // Will be set by PhysicsEngine
            arrival_time: 0,   // Will be calculated by PhysicsEngine
            fuel_cost,
        };
        
        ship.trajectory = Some(trajectory);
        Ok(())
    }
    
    fn handle_combat_resolved(&mut self, outcome: &CombatOutcome) -> GameResult<()> {
        // Remove destroyed ships
        for &ship_id in &outcome.attacker_losses {
            if let Err(_) = self.destroy_ship(ship_id) {
                // Ship might already be destroyed, continue
            }
        }
        
        for &ship_id in &outcome.defender_losses {
            if let Err(_) = self.destroy_ship(ship_id) {
                // Ship might already be destroyed, continue
            }
        }
        
        Ok(())
    }
    
    pub fn calculate_fuel_cost(&self, ship_id: ShipId, distance: f32) -> GameResult<f32> {
        let ship = self.get_ship(ship_id)?;
        
        let fuel_cost = match ship.ship_class {
            ShipClass::Scout => distance / 200.0,
            ShipClass::Transport => distance / 100.0,
            ShipClass::Warship => distance / 80.0,
            ShipClass::Colony => distance / 60.0,
        };
        
        Ok(fuel_cost)
    }
    
    pub fn get_ships_by_owner(&self, owner: FactionId) -> Vec<ShipId> {
        self.ships.iter()
            .filter(|ship| ship.owner == owner)
            .map(|ship| ship.id)
            .collect()
    }
    
    pub fn get_ships_by_class(&self, ship_class: ShipClass) -> Vec<ShipId> {
        self.ships.iter()
            .filter(|ship| ship.ship_class == ship_class)
            .map(|ship| ship.id)
            .collect()
    }
    
    pub fn load_ships(&mut self, ships: Vec<Ship>) -> GameResult<()> {
        // Replace all ships with loaded data
        self.ships = ships;
        
        // Rebuild the index
        self.ship_index.clear();
        for (index, ship) in self.ships.iter().enumerate() {
            self.ship_index.insert(ship.id, index);
        }
        
        // Update next_id to be higher than any existing ID
        self.next_id = self.ships.iter()
            .map(|s| s.id)
            .max()
            .unwrap_or(0) + 1;
            
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ship() {
        let mut manager = ShipManager::new();
        
        let ship_id = manager.create_ship(
            ShipClass::Scout,
            Vector2 { x: 0.0, y: 0.0 },
            1
        ).unwrap();
        
        assert_eq!(ship_id, 0);
        
        let ship = manager.get_ship(ship_id).unwrap();
        assert_eq!(ship.ship_class, ShipClass::Scout);
        assert_eq!(ship.owner, 1);
        assert_eq!(ship.fuel, 100.0);
    }

    #[test]
    fn test_fuel_consumption_by_ship_class() {
        let mut manager = ShipManager::new();
        
        let scout_id = manager.create_ship(ShipClass::Scout, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        let transport_id = manager.create_ship(ShipClass::Transport, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        let warship_id = manager.create_ship(ShipClass::Warship, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        let colony_id = manager.create_ship(ShipClass::Colony, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        
        let distance = 100.0;
        
        let scout_cost = manager.calculate_fuel_cost(scout_id, distance).unwrap();
        let transport_cost = manager.calculate_fuel_cost(transport_id, distance).unwrap();
        let warship_cost = manager.calculate_fuel_cost(warship_id, distance).unwrap();
        let colony_cost = manager.calculate_fuel_cost(colony_id, distance).unwrap();
        
        // Scout should be most efficient, colony least efficient
        assert!(scout_cost < transport_cost);
        assert!(transport_cost < warship_cost);
        assert!(warship_cost < colony_cost);
        
        // Specific expected values based on implementation
        assert_eq!(scout_cost, 0.5);      // 100.0 / 200.0
        assert_eq!(transport_cost, 1.0);  // 100.0 / 100.0
        assert_eq!(warship_cost, 1.25);   // 100.0 / 80.0
    }

    #[test]
    fn test_cargo_capacity_validation() {
        let mut manager = ShipManager::new();
        
        // Only transport ships can carry cargo
        let transport_id = manager.create_ship(ShipClass::Transport, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        let scout_id = manager.create_ship(ShipClass::Scout, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        
        // Set transport capacity
        {
            let transport = manager.get_ship_mut(transport_id).unwrap();
            transport.cargo.capacity = 100;
        }
        
        let resources = ResourceBundle {
            minerals: 50,
            food: 30,
            ..Default::default()
        };
        
        // Transport should be able to load cargo
        assert!(manager.load_cargo(transport_id, resources).is_ok());
        
        // Scout should not be able to load cargo
        assert!(manager.load_cargo(scout_id, resources).is_err());
    }

    #[test]
    fn test_destroy_ship() {
        let mut manager = ShipManager::new();
        
        let ship1_id = manager.create_ship(ShipClass::Scout, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        let ship2_id = manager.create_ship(ShipClass::Transport, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        
        assert_eq!(manager.get_all_ships().len(), 2);
        
        // Destroy first ship
        assert!(manager.destroy_ship(ship1_id).is_ok());
        assert_eq!(manager.get_all_ships().len(), 1);
        
        // Should not be able to get destroyed ship
        assert!(manager.get_ship(ship1_id).is_err());
        
        // Second ship should still be accessible
        assert!(manager.get_ship(ship2_id).is_ok());
    }

    #[test]
    fn test_consume_fuel() {
        let mut manager = ShipManager::new();
        let ship_id = manager.create_ship(ShipClass::Scout, Vector2 { x: 0.0, y: 0.0 }, 1).unwrap();
        
        // Should start with 100.0 fuel
        let ship = manager.get_ship(ship_id).unwrap();
        assert_eq!(ship.fuel, 100.0);
        
        // Consume some fuel
        assert!(manager.consume_fuel(ship_id, 25.0).is_ok());
        
        let ship = manager.get_ship(ship_id).unwrap();
        assert_eq!(ship.fuel, 75.0);
        
        // Try to consume more fuel than available
        assert!(manager.consume_fuel(ship_id, 100.0).is_err());
        
        // Fuel should remain unchanged after failed consumption
        let ship = manager.get_ship(ship_id).unwrap();
        assert_eq!(ship.fuel, 75.0);
    }
}