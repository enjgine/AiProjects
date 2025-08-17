// src/managers/planet_manager.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use std::collections::HashMap;

pub struct PlanetManager {
    planets: Vec<Planet>,
    next_id: PlanetId,
    planet_index: HashMap<PlanetId, usize>,
}

impl PlanetManager {
    pub fn new() -> Self {
        Self {
            planets: Vec::new(),
            next_id: 0,
            planet_index: HashMap::new(),
        }
    }
    
    pub fn add_resources(&mut self, id: PlanetId, resources: ResourceBundle) -> GameResult<()> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        
        let planet = &mut self.planets[*index];
        
        // Check storage capacity
        let new_total = ResourceBundle {
            minerals: planet.resources.current.minerals + resources.minerals,
            food: planet.resources.current.food + resources.food,
            energy: planet.resources.current.energy + resources.energy,
            alloys: planet.resources.current.alloys + resources.alloys,
            components: planet.resources.current.components + resources.components,
            fuel: planet.resources.current.fuel + resources.fuel,
        };
        
        if !planet.resources.capacity.can_afford(&new_total) {
            return Err(GameError::InvalidOperation("Storage capacity exceeded".into()));
        }
        
        planet.resources.current = new_total;
        Ok(())
    }
    
    pub fn get_planet(&self, id: PlanetId) -> GameResult<&Planet> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        Ok(&self.planets[*index])
    }
    
    pub fn update_population(&mut self, id: PlanetId, amount: i32) -> GameResult<()> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        
        let planet = &mut self.planets[*index];
        let new_total = planet.population.total + amount;
        
        if new_total < 0 {
            return Err(GameError::InvalidOperation("Population cannot be negative".into()));
        }
        
        planet.population.total = new_total;
        Ok(())
    }
}