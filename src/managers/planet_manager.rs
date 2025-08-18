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
            planets: Vec::new(),
            next_id: 0,
            planet_index: HashMap::new(),
        }
    }
    
    // CRUD Operations
    pub fn create_planet(&mut self, position: OrbitalElements, controller: Option<FactionId>) -> GameResult<PlanetId> {
        let id = self.next_id;
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
    
    pub fn get_planet_mut(&mut self, id: PlanetId) -> GameResult<&mut Planet> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        Ok(&mut self.planets[*index])
    }
    
    pub fn get_all_planets(&self) -> &Vec<Planet> {
        &self.planets
    }
    
    pub fn get_all_planets_cloned(&self) -> GameResult<Vec<Planet>> {
        Ok(self.planets.clone())
    }
    
    pub fn get_planets_by_faction(&self, faction: FactionId) -> Vec<&Planet> {
        self.planets.iter()
            .filter(|p| p.controller == Some(faction))
            .collect()
    }
    
    // Resource Management
    pub fn add_resources(&mut self, id: PlanetId, resources: ResourceBundle) -> GameResult<()> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        
        let planet = &mut self.planets[*index];
        
        // Check storage capacity for each resource
        let new_minerals = planet.resources.current.minerals + resources.minerals;
        let new_food = planet.resources.current.food + resources.food;
        let new_energy = planet.resources.current.energy + resources.energy;
        let new_alloys = planet.resources.current.alloys + resources.alloys;
        let new_components = planet.resources.current.components + resources.components;
        let new_fuel = planet.resources.current.fuel + resources.fuel;
        
        if new_minerals > planet.resources.capacity.minerals ||
           new_food > planet.resources.capacity.food ||
           new_energy > planet.resources.capacity.energy ||
           new_alloys > planet.resources.capacity.alloys ||
           new_components > planet.resources.capacity.components ||
           new_fuel > planet.resources.capacity.fuel {
            return Err(GameError::InvalidOperation("Storage capacity exceeded".into()));
        }
        
        planet.resources.current.minerals = new_minerals;
        planet.resources.current.food = new_food;
        planet.resources.current.energy = new_energy;
        planet.resources.current.alloys = new_alloys;
        planet.resources.current.components = new_components;
        planet.resources.current.fuel = new_fuel;
        
        Ok(())
    }
    
    pub fn remove_resources(&mut self, id: PlanetId, resources: ResourceBundle) -> GameResult<()> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        
        let planet = &mut self.planets[*index];
        
        // Validate we can afford the cost
        if !planet.resources.current.can_afford(&resources) {
            return Err(GameError::InsufficientResources {
                required: resources,
                available: planet.resources.current,
            });
        }
        
        planet.resources.current.subtract(&resources)?;
        Ok(())
    }
    
    // Population Management
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
    
    pub fn set_worker_allocation(&mut self, id: PlanetId, allocation: WorkerAllocation) -> GameResult<()> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        
        let planet = &mut self.planets[*index];
        
        // Validate allocation matches total population
        allocation.validate(planet.population.total)?;
        
        // Ensure minimum 10% unassigned workers
        let min_unassigned = (planet.population.total as f32 * 0.1) as i32;
        if allocation.unassigned < min_unassigned {
            return Err(GameError::InvalidOperation(
                format!("Must maintain at least {}% unassigned workers", 10)
            ));
        }
        
        planet.population.allocation = allocation;
        Ok(())
    }
    
    // Building Management
    pub fn add_building(&mut self, id: PlanetId, building_type: BuildingType) -> GameResult<()> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        
        let planet = &mut self.planets[*index];
        
        // Calculate available building slots: 10 + population/10000
        let available_slots = 10 + (planet.population.total / 10000);
        if planet.developments.len() >= available_slots as usize {
            return Err(GameError::InvalidOperation(
                format!("No available building slots. Current: {}, Max: {}", 
                       planet.developments.len(), available_slots)
            ));
        }
        
        let building = Building {
            building_type,
            tier: 1,
            operational: true,
        };
        
        planet.developments.push(building);
        Ok(())
    }
    
    pub fn get_building_count(&self, id: PlanetId, building_type: BuildingType) -> GameResult<usize> {
        let planet = self.get_planet(id)?;
        Ok(planet.developments.iter()
            .filter(|b| b.building_type == building_type)
            .count())
    }
    
    pub fn get_available_building_slots(&self, id: PlanetId) -> GameResult<i32> {
        let planet = self.get_planet(id)?;
        let max_slots = 10 + (planet.population.total / 10000);
        Ok(max_slots - planet.developments.len() as i32)
    }
    
    // Planet Control
    pub fn change_controller(&mut self, id: PlanetId, new_controller: Option<FactionId>) -> GameResult<()> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        
        self.planets[*index].controller = new_controller;
        Ok(())
    }
    
    // Storage capacity management
    pub fn upgrade_storage(&mut self, id: PlanetId, additional_capacity: ResourceBundle) -> GameResult<()> {
        let index = self.planet_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))?;
        
        let planet = &mut self.planets[*index];
        planet.resources.capacity.minerals += additional_capacity.minerals;
        planet.resources.capacity.food += additional_capacity.food;
        planet.resources.capacity.energy += additional_capacity.energy;
        planet.resources.capacity.alloys += additional_capacity.alloys;
        planet.resources.capacity.components += additional_capacity.components;
        planet.resources.capacity.fuel += additional_capacity.fuel;
        
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
                        planet.resources.current.validate_non_negative()?;
                        planet.population.allocation.validate(planet.population.total)?;
                    }
                    _ => {} // Ignore other state changes
                }
            }
        }
        Ok(())
    }
}