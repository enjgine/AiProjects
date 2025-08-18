// src/managers/ship_manager.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
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
}