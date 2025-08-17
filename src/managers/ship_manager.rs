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
}