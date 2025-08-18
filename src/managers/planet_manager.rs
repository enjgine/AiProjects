// src/managers/planet_manager.rs
use crate::core::{GameResult, GameEvent, EventBus, GameSystem};
use crate::core::types::*;
use crate::core::events::*;
use std::collections::HashMap;

pub struct PlanetManager {
    planets: Vec<Planet>,
    next_id: PlanetId,
    planet_index: HashMap<PlanetId, usize>,
}

impl PlanetManager {
    pub fn new() -> Self {
        Self {
            planets: Vec::with_capacity(100), // Pre-allocate for performance
            next_id: 0,
            planet_index: HashMap::with_capacity(100),
        }
    }
    
    // CRUD Operations
    // Helper method to get planet index with consistent error handling
    fn get_planet_index(&self, id: PlanetId) -> GameResult<usize> {
        self.planet_index.get(&id)
            .copied()
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))
    }
    
    // Helper method for consistent building slot calculation
    fn calculate_building_slots(&self, population: i32) -> usize {
        (10 + (population / 10000)) as usize
    }
    
    pub fn create_planet(&mut self, position: OrbitalElements, controller: Option<FactionId>) -> GameResult<PlanetId> {
        let id = self.next_id;
        
        // Check for ID overflow
        if self.next_id == u32::MAX {
            return Err(GameError::SystemError("Maximum number of planets reached".into()));
        }
        self.next_id += 1;
        
        let planet = Planet {
            id,
            position,
            resources: ResourceStorage {
                current: ResourceBundle::default(),
                capacity: ResourceBundle {
                    minerals: 10000,
                    food: 5000,
                    energy: 1000,
                    alloys: 1000,
                    components: 500,
                    fuel: 2000,
                },
            },
            population: Demographics::default(),
            developments: Vec::new(),
            controller,
        };
        
        let index = self.planets.len();
        self.planets.push(planet);
        self.planet_index.insert(id, index);
        
        Ok(id)
    }
    
    pub fn get_planet(&self, id: PlanetId) -> GameResult<&Planet> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        Ok(&self.planets[*index])
    }
    
    // REMOVED: get_planet_mut violates manager pattern
    // Use specific modification methods instead
    
    pub fn get_all_planets(&self) -> &Vec<Planet> {
        &self.planets
    }
    
    // Get planet count for efficiency (avoid cloning when just need count)
    pub fn get_planet_count(&self) -> usize {
        self.planets.len()
    }
    
    // More efficient method when you only need planet IDs
    pub fn get_all_planet_ids(&self) -> Vec<PlanetId> {
        self.planets.iter().map(|p| p.id).collect()
    }
    
    pub fn get_all_planets_cloned(&self) -> GameResult<Vec<Planet>> {
        Ok(self.planets.clone())
    }
    
    // Safe planet modification without exposing mutable references
    pub fn modify_planet<F>(&mut self, id: PlanetId, modifier: F) -> GameResult<()>
    where
        F: FnOnce(&mut Planet) -> GameResult<()>,
    {
        let index = self.get_planet_index(id)?;
        let planet = &mut self.planets[index];
        modifier(planet)?;
        
        // Validate planet state after modification
        planet.resources.validate()?;
        planet.population.allocation.validate(planet.population.total)?;
        
        Ok(())
    }
    
    // Validate all planets for consistency
    pub fn validate_all_planets(&self) -> GameResult<()> {
        for planet in &self.planets {
            planet.resources.validate()?;
            planet.population.allocation.validate(planet.population.total)?;
            
            // Check building slot constraints
            let max_slots = self.calculate_building_slots(planet.population.total);
            if planet.developments.len() > max_slots {
                return Err(GameError::InvalidOperation(
                    format!("Planet {} has {} buildings but only {} slots available", 
                           planet.id, planet.developments.len(), max_slots)
                ));
            }
        }
        Ok(())
    }
    
    pub fn get_planets_by_faction(&self, faction: FactionId) -> Vec<&Planet> {
        let mut result = Vec::new();
        for planet in &self.planets {
            if planet.controller == Some(faction) {
                result.push(planet);
            }
        }
        result
    }
    
    // Resource Management
    pub fn add_resources(&mut self, id: PlanetId, resources: ResourceBundle) -> GameResult<()> {
        // Validate input resources are non-negative
        resources.validate_non_negative()?;
        
        let index = self.get_planet_index(id)?;
        let planet = &mut self.planets[index];
        
        // Check for potential overflow before addition
        if planet.resources.current.minerals.saturating_add(resources.minerals) == i32::MAX ||
           planet.resources.current.food.saturating_add(resources.food) == i32::MAX ||
           planet.resources.current.energy.saturating_add(resources.energy) == i32::MAX ||
           planet.resources.current.alloys.saturating_add(resources.alloys) == i32::MAX ||
           planet.resources.current.components.saturating_add(resources.components) == i32::MAX ||
           planet.resources.current.fuel.saturating_add(resources.fuel) == i32::MAX {
            return Err(GameError::InvalidOperation("Resource addition would cause overflow".into()));
        }
        
        // Use helper method to check if we can store the additional resources
        if !planet.resources.can_store(&resources) {
            return Err(GameError::InvalidOperation("Storage capacity exceeded".into()));
        }
        
        // Use ResourceBundle's built-in add method for safer arithmetic
        planet.resources.current.add(&resources)?;
        
        Ok(())
    }
    
    pub fn remove_resources(&mut self, id: PlanetId, resources: ResourceBundle) -> GameResult<()> {
        // Validate input resources are non-negative
        resources.validate_non_negative()?;
        
        let index = self.get_planet_index(id)?;
        let planet = &mut self.planets[index];
        
        // Use ResourceBundle's built-in subtract method which includes affordability check
        planet.resources.current.subtract(&resources)?;
        Ok(())
    }
    
    // Population Management
    pub fn update_population(&mut self, id: PlanetId, amount: i32) -> GameResult<()> {
        let index = self.get_planet_index(id)?;
        let planet = &mut self.planets[index];
        
        // Check for overflow before addition
        let new_total = planet.population.total.saturating_add(amount);
        
        if new_total < 0 {
            return Err(GameError::InvalidOperation("Population cannot be negative".into()));
        }
        
        if new_total == i32::MAX && amount > 0 {
            return Err(GameError::InvalidOperation("Population update would cause overflow".into()));
        }
        
        planet.population.total = new_total;
        
        // If population changed, we may need to adjust worker allocation to stay valid
        if planet.population.allocation.validate(new_total).is_err() {
            // Reset allocation to all unassigned workers
            planet.population.allocation = WorkerAllocation {
                agriculture: 0,
                mining: 0,
                industry: 0,
                research: 0,
                military: 0,
                unassigned: new_total,
            };
        }
        
        Ok(())
    }
    
    pub fn set_worker_allocation(&mut self, id: PlanetId, allocation: WorkerAllocation) -> GameResult<()> {
        let index = self.get_planet_index(id)?;
        let planet = &mut self.planets[index];
        
        // Validate allocation matches total population
        allocation.validate(planet.population.total)?;
        
        // Ensure minimum 10% unassigned workers using integer arithmetic
        let min_unassigned = planet.population.total / 10; // Integer division gives floor
        if allocation.unassigned < min_unassigned {
            return Err(GameError::InvalidOperation(
                format!("Must maintain at least {} unassigned workers (10% of {})", 
                       min_unassigned, planet.population.total)
            ));
        }
        
        planet.population.allocation = allocation;
        Ok(())
    }
    
    // Building Management
    pub fn add_building(&mut self, id: PlanetId, building_type: BuildingType) -> GameResult<()> {
        let index = self.get_planet_index(id)?;
        
        // Calculate slots before getting mutable reference
        let population = self.planets[index].population.total;
        let available_slots = self.calculate_building_slots(population);
        let current_buildings = self.planets[index].developments.len();
        
        if current_buildings >= available_slots {
            return Err(GameError::InvalidOperation(
                format!("No available building slots. Current: {}, Max: {}", 
                       current_buildings, available_slots)
            ));
        }
        
        let building = Building {
            building_type,
            tier: 1,
            operational: true,
        };
        
        self.planets[index].developments.push(building);
        Ok(())
    }
    
    pub fn get_building_count(&self, id: PlanetId, building_type: BuildingType) -> GameResult<usize> {
        let planet = self.get_planet(id)?;
        let mut count = 0;
        for building in &planet.developments {
            if building.building_type == building_type {
                count += 1;
            }
        }
        Ok(count)
    }
    
    pub fn get_available_building_slots(&self, id: PlanetId) -> GameResult<usize> {
        let planet = self.get_planet(id)?;
        let max_slots = self.calculate_building_slots(planet.population.total);
        Ok(max_slots.saturating_sub(planet.developments.len()))
    }
    
    // Planet Control
    pub fn change_controller(&mut self, id: PlanetId, new_controller: Option<FactionId>) -> GameResult<()> {
        let index = self.get_planet_index(id)?;
        self.planets[index].controller = new_controller;
        Ok(())
    }
    
    // Storage capacity management
    pub fn upgrade_storage(&mut self, id: PlanetId, additional_capacity: ResourceBundle) -> GameResult<()> {
        // Validate input is non-negative
        additional_capacity.validate_non_negative()?;
        
        let index = self.get_planet_index(id)?;
        let planet = &mut self.planets[index];
        
        // Use ResourceBundle's add method for safer arithmetic with overflow protection
        planet.resources.capacity.add(&additional_capacity)?;
        
        Ok(())
    }
    
    pub fn load_planets(&mut self, planets: Vec<Planet>) -> GameResult<()> {
        // Replace all planets with loaded data
        self.planets = planets;
        
        // Rebuild the index
        self.planet_index.clear();
        for (index, planet) in self.planets.iter().enumerate() {
            self.planet_index.insert(planet.id, index);
        }
        
        // Update next_id to be higher than any existing ID
        self.next_id = self.planets.iter()
            .map(|p| p.id)
            .max()
            .unwrap_or(0) + 1;
            
        Ok(())
    }
}

impl GameSystem for PlanetManager {
    fn update(&mut self, _delta: f32, _events: &mut EventBus) -> GameResult<()> {
        // PlanetManager is primarily reactive, no periodic updates needed
        Ok(())
    }
    
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    PlayerCommand::BuildStructure { planet, building_type } => {
                        self.add_building(*planet, *building_type)?;
                    }
                    PlayerCommand::AllocateWorkers { planet, allocation } => {
                        self.set_worker_allocation(*planet, allocation.clone())?;
                    }
                    _ => {} // Ignore other commands
                }
            }
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    SimulationEvent::ConstructionCompleted { planet, building } => {
                        // Building is already added by construction system,
                        // just validate it was successful
                        let _current_count = self.get_building_count(*planet, *building)?;
                    }
                    SimulationEvent::PlanetConquered { planet, new_owner } => {
                        self.change_controller(*planet, Some(*new_owner))?;
                    }
                    _ => {} // Ignore other simulation events
                }
            }
            GameEvent::StateChanged(state_change) => {
                match state_change {
                    StateChange::PlanetUpdated(planet_id) => {
                        // Validate planet still exists and is in valid state
                        let planet = self.get_planet(*planet_id)?;
                        planet.resources.validate()?;
                        planet.population.allocation.validate(planet.population.total)?;
                        
                        // Validate building constraints
                        let max_slots = self.calculate_building_slots(planet.population.total);
                        if planet.developments.len() > max_slots {
                            return Err(GameError::InvalidOperation(
                                format!("Planet {} exceeds building slot limit", planet_id)
                            ));
                        }
                    }
                    _ => {} // Ignore other state changes
                }
            }
        }
        Ok(())
    }
}