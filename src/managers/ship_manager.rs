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
        // Input validation
        if !position.x.is_finite() || !position.y.is_finite() {
            return Err(GameError::InvalidOperation("Ship position must have finite coordinates".into()));
        }
        
        // Prevent integer overflow on ship IDs
        if self.next_id == ShipId::MAX {
            return Err(GameError::SystemError("Maximum number of ships reached".into()));
        }
        
        let id = self.next_id;
        self.next_id += 1;
        
        // Initialize cargo capacity based on ship class
        let cargo_capacity = match ship_class {
            ShipClass::Scout => 0,      // No cargo capacity
            ShipClass::Transport => 1000, // High cargo capacity
            ShipClass::Warship => 100,   // Minimal cargo capacity
            ShipClass::Colony => 500,    // Medium cargo capacity for colonists
        };
        
        let ship = Ship {
            id,
            ship_class,
            position,
            trajectory: None,
            cargo: CargoHold {
                resources: ResourceBundle::default(),
                population: 0,
                capacity: cargo_capacity,
            },
            fuel: 100.0, // Default fuel - TODO: Make this i32 for consistency
            owner,
        };
        
        // Validate ship before adding
        ship.validate()?;
        
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
        // Input validation
        if !position.x.is_finite() || !position.y.is_finite() {
            return Err(GameError::InvalidOperation("Position must have finite coordinates".into()));
        }
        
        let index = self.ship_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Ship {} not found", id)))?;
        
        self.ships[*index].position = position;
        
        // Clear trajectory when position is manually updated
        self.ships[*index].trajectory = None;
        
        Ok(())
    }
    
    pub fn destroy_ship(&mut self, id: ShipId) -> GameResult<()> {
        let index = self.ship_index.remove(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Ship {} not found", id)))?;
        
        // More efficient: swap_remove to avoid shifting all elements
        self.ships.swap_remove(index);
        
        // Only need to update the index for the swapped element (if any)
        if index < self.ships.len() {
            let swapped_ship = &self.ships[index];
            self.ship_index.insert(swapped_ship.id, index);
        }
        
        Ok(())
    }
    
    // Private helper for internal use only - violates architecture if exposed
    fn get_ship_mut(&mut self, id: ShipId) -> GameResult<&mut Ship> {
        let index = self.ship_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Ship {} not found", id)))?;
        Ok(&mut self.ships[*index])
    }
    
    pub fn load_cargo(&mut self, ship_id: ShipId, resources: ResourceBundle) -> GameResult<()> {
        // Input validation
        resources.validate_non_negative()?;
        
        let ship = self.get_ship_mut(ship_id)?;
        
        // Check if ship can carry resources (only transport ships for now)
        if ship.ship_class != ShipClass::Transport {
            return Err(GameError::InvalidOperation("Only transport ships can carry cargo".into()));
        }
        
        // Use proper CargoHold validation methods
        if !ship.cargo.can_load(&resources, 0) {
            return Err(GameError::InsufficientResources {
                required: ResourceBundle {
                    minerals: ship.cargo.available_space().max(resources.total() as i32),
                    ..Default::default()
                },
                available: ResourceBundle {
                    minerals: ship.cargo.available_space(),
                    ..Default::default()
                },
            });
        }
        
        // Load the resources using safer addition
        ship.cargo.resources.add(&resources)?;
        
        // Validate cargo state after loading
        ship.cargo.validate()?;
        
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
        // Input validation
        if !trajectory.origin.x.is_finite() || !trajectory.origin.y.is_finite() ||
           !trajectory.destination.x.is_finite() || !trajectory.destination.y.is_finite() {
            return Err(GameError::InvalidOperation("Trajectory coordinates must be finite".into()));
        }
        if trajectory.fuel_cost < 0.0 || !trajectory.fuel_cost.is_finite() {
            return Err(GameError::InvalidOperation("Fuel cost must be positive and finite".into()));
        }
        if trajectory.departure_time > trajectory.arrival_time {
            return Err(GameError::InvalidOperation("Departure time cannot be after arrival time".into()));
        }
        
        let ship = self.get_ship_mut(ship_id)?;
        
        // Validate ship has sufficient fuel for trajectory
        if ship.fuel < trajectory.fuel_cost {
            return Err(GameError::InsufficientResources {
                required: ResourceBundle { fuel: trajectory.fuel_cost as i32, ..Default::default() },
                available: ResourceBundle { fuel: ship.fuel as i32, ..Default::default() },
            });
        }
        
        ship.trajectory = Some(trajectory);
        Ok(())
    }
    
    pub fn consume_fuel(&mut self, ship_id: ShipId, amount: f32) -> GameResult<()> {
        // Input validation
        if amount < 0.0 || !amount.is_finite() {
            return Err(GameError::InvalidOperation("Fuel amount must be positive and finite".into()));
        }
        
        let ship = self.get_ship_mut(ship_id)?;
        
        if ship.fuel < amount {
            return Err(GameError::InsufficientResources {
                required: ResourceBundle { fuel: amount as i32, ..Default::default() },
                available: ResourceBundle { fuel: ship.fuel as i32, ..Default::default() },
            });
        }
        
        ship.fuel -= amount;
        
        // Validate ship state after fuel consumption
        ship.validate()?;
        
        Ok(())
    }
    
    pub fn get_ships_at_planet(&self, planet_position: Vector2, radius: f32) -> GameResult<Vec<ShipId>> {
        // Input validation
        if !planet_position.x.is_finite() || !planet_position.y.is_finite() {
            return Err(GameError::InvalidOperation("Planet position must have finite coordinates".into()));
        }
        if radius < 0.0 || !radius.is_finite() {
            return Err(GameError::InvalidOperation("Radius must be positive and finite".into()));
        }
        
        // Use more efficient distance calculation (avoid sqrt when possible)
        let radius_squared = radius * radius;
        let ships = self.ships.iter()
            .filter(|ship| {
                let dx = ship.position.x - planet_position.x;
                let dy = ship.position.y - planet_position.y;
                dx * dx + dy * dy <= radius_squared
            })
            .map(|ship| ship.id)
            .collect();
        
        Ok(ships)
    }
    
    // Returns immutable reference to ships - read-only access
    pub fn get_all_ships(&self) -> &Vec<Ship> {
        &self.ships
    }
    
    pub fn get_all_ships_cloned(&self) -> GameResult<Vec<Ship>> {
        // Validate all ships before returning cloned data
        for ship in &self.ships {
            ship.validate()?;
        }
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
                    SimulationEvent::ShipCompleted { planet: _, ship: _ } => {
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
        // Input validation
        if !target.x.is_finite() || !target.y.is_finite() {
            return Err(GameError::InvalidOperation("Target position must have finite coordinates".into()));
        }
        
        // Get ship data without holding mutable reference
        let (ship_position, ship_class, ship_fuel) = {
            let ship = self.get_ship_mut(ship_id)?;
            (ship.position, ship.ship_class, ship.fuel)
        };
        
        // Use Vector2 distance method for consistency
        let distance = ship_position.distance_to(&target);
        
        // Prevent movement to same position
        if distance < 0.1 {
            return Ok(()); // Already at destination
        }
        
        // Calculate fuel cost based on ship class efficiency
        let fuel_cost = self.calculate_fuel_cost_for_class(ship_class, distance);
        
        // Validate fuel availability
        if ship_fuel < fuel_cost {
            return Err(GameError::InsufficientResources {
                required: ResourceBundle { fuel: fuel_cost as i32, ..Default::default() },
                available: ResourceBundle { fuel: ship_fuel as i32, ..Default::default() },
            });
        }
        
        // Create trajectory with proper validation
        let trajectory = Trajectory {
            origin: ship_position,
            destination: target,
            departure_time: 0, // Will be set by PhysicsEngine
            arrival_time: 0,   // Will be calculated by PhysicsEngine
            fuel_cost,
        };
        
        // Now get mutable reference to set trajectory
        let ship = self.get_ship_mut(ship_id)?;
        ship.trajectory = Some(trajectory);
        Ok(())
    }
    
    // Helper method to centralize fuel cost calculation
    fn calculate_fuel_cost_for_class(&self, ship_class: ShipClass, distance: f32) -> f32 {
        match ship_class {
            ShipClass::Scout => distance / 200.0,      // More efficient
            ShipClass::Transport => distance / 100.0,  // Standard
            ShipClass::Warship => distance / 80.0,     // Less efficient
            ShipClass::Colony => distance / 60.0,      // Least efficient
        }
    }
    
    fn handle_combat_resolved(&mut self, outcome: &CombatOutcome) -> GameResult<()> {
        let mut errors = Vec::new();
        
        // Remove destroyed ships and track errors for proper reporting
        for &ship_id in &outcome.attacker_losses {
            if let Err(e) = self.destroy_ship(ship_id) {
                errors.push(format!("Failed to destroy attacker ship {}: {}", ship_id, e));
            }
        }
        
        for &ship_id in &outcome.defender_losses {
            if let Err(e) = self.destroy_ship(ship_id) {
                errors.push(format!("Failed to destroy defender ship {}: {}", ship_id, e));
            }
        }
        
        // Report errors if any occurred during combat resolution
        if !errors.is_empty() {
            return Err(GameError::SystemError(
                format!("Combat resolution errors: {}", errors.join("; "))
            ));
        }
        
        Ok(())
    }
    
    pub fn calculate_fuel_cost(&self, ship_id: ShipId, distance: f32) -> GameResult<f32> {
        // Input validation
        if distance < 0.0 || !distance.is_finite() {
            return Err(GameError::InvalidOperation("Distance must be positive and finite".into()));
        }
        
        let ship = self.get_ship(ship_id)?;
        
        Ok(self.calculate_fuel_cost_for_class(ship.ship_class, distance))
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
        // Validate all ships before loading
        for ship in &ships {
            ship.validate()?;
            
            // Validate position coordinates
            if !ship.position.x.is_finite() || !ship.position.y.is_finite() {
                return Err(GameError::InvalidOperation(
                    format!("Ship {} has invalid position coordinates", ship.id)
                ));
            }
        }
        
        // Check for duplicate ship IDs using Vec-based approach
        let mut ship_ids: Vec<ShipId> = ships.iter().map(|s| s.id).collect();
        ship_ids.sort_unstable();
        for window in ship_ids.windows(2) {
            if window[0] == window[1] {
                return Err(GameError::InvalidOperation(
                    format!("Duplicate ship ID {} found", window[0])
                ));
            }
        }
        
        // Replace all ships with validated data
        self.ships = ships;
        
        // Rebuild the index
        self.ship_index.clear();
        for (index, ship) in self.ships.iter().enumerate() {
            self.ship_index.insert(ship.id, index);
        }
        
        // Update next_id to prevent conflicts
        self.next_id = self.ships.iter()
            .map(|s| s.id)
            .max()
            .map(|max_id| max_id.saturating_add(1))
            .unwrap_or(0);
            
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
        
        // Transport already has proper capacity from create_ship
        
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