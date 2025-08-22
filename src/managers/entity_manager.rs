// src/managers/entity_manager.rs
//! Consolidated manager for planets and factions to reduce code duplication.
//! Combines PlanetManager and FactionManager functionality with shared patterns.

use crate::core::{GameResult, GameEvent, EventBus, GameSystem};
use crate::core::types::*;
use crate::core::events::*;
use std::collections::HashMap;

/// Manages both planets and factions with similar CRUD operations
pub struct EntityManager {
    // Planet management
    planets: Vec<Planet>,
    next_planet_id: PlanetId,
    planet_index: HashMap<PlanetId, usize>,

    // Faction management  
    factions: Vec<Faction>,
    next_faction_id: FactionId,
    faction_index: HashMap<FactionId, usize>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            planets: Vec::with_capacity(100),
            next_planet_id: 0,
            planet_index: HashMap::with_capacity(100),
            
            factions: Vec::new(),
            next_faction_id: 0,
            faction_index: HashMap::new(),
        }
    }

    // === PLANET MANAGEMENT ===
    
    fn get_planet_index(&self, id: PlanetId) -> GameResult<usize> {
        self.planet_index.get(&id)
            .copied()
            .ok_or_else(|| GameError::InvalidTarget(format!("Planet {} not found", id)))
    }

    pub fn create_planet(&mut self, position: OrbitalElements, controller: Option<FactionId>) -> GameResult<PlanetId> {
        if let Some(faction_id) = controller {
            self.get_faction_index(faction_id)?; // Validate faction exists
        }

        let id = self.next_planet_id;
        self.next_planet_id += 1;

        let planet = Planet {
            id,
            position,
            resources: ResourceStorage::default(),
            population: Demographics::default(),
            developments: Vec::new(),
            controller,
        };

        self.planets.push(planet);
        self.planet_index.insert(id, self.planets.len() - 1);
        Ok(id)
    }

    pub fn get_planet(&self, id: PlanetId) -> Option<&Planet> {
        self.planet_index.get(&id).map(|&index| &self.planets[index])
    }

    pub fn get_planet_mut(&mut self, id: PlanetId) -> Option<&mut Planet> {
        self.planet_index.get(&id).map(|&index| &mut self.planets[index])
    }

    pub fn get_all_planets(&self) -> &[Planet] {
        &self.planets
    }

    pub fn planet_count(&self) -> usize {
        self.planets.len()
    }

    pub fn update_planet_resources(&mut self, id: PlanetId, resources: ResourceStorage) -> GameResult<()> {
        let index = self.get_planet_index(id)?;
        self.planets[index].resources = resources;
        Ok(())
    }

    pub fn update_planet_population(&mut self, id: PlanetId, population: Demographics) -> GameResult<()> {
        let index = self.get_planet_index(id)?;
        self.planets[index].population = population;
        Ok(())
    }

    pub fn add_building(&mut self, id: PlanetId, building: Building) -> GameResult<()> {
        let index = self.get_planet_index(id)?;
        let planet = &mut self.planets[index];
        
        let building_slots = self.calculate_building_slots(planet);
        if planet.developments.len() >= building_slots {
            return Err(GameError::InvalidOperation(
                format!("Planet {} has no available building slots", id)
            ));
        }

        planet.developments.push(building);
        Ok(())
    }

    fn calculate_building_slots(&self, planet: &Planet) -> usize {
        10 + (planet.population.total as usize / 10000)
    }

    pub fn load_planets(&mut self, planets: Vec<Planet>) -> GameResult<()> {
        for planet in &planets {
            if let Some(faction_id) = planet.controller {
                if self.get_faction_index(faction_id).is_err() {
                    return Err(GameError::InvalidOperation(
                        format!("Planet {} references non-existent faction {}", planet.id, faction_id)
                    ));
                }
            }
            planet.population.allocation.validate(planet.population.total)?;
        }

        self.planets = planets;
        self.planet_index.clear();
        self.planet_index.reserve(self.planets.len());
        
        for (index, planet) in self.planets.iter().enumerate() {
            self.planet_index.insert(planet.id, index);
        }

        self.next_planet_id = self.planets.iter()
            .map(|p| p.id)
            .max()
            .map(|max_id| max_id + 1)
            .unwrap_or(0);

        Ok(())
    }

    // === FACTION MANAGEMENT ===
    
    fn get_faction_index(&self, id: FactionId) -> GameResult<usize> {
        self.faction_index.get(&id)
            .copied()
            .ok_or_else(|| GameError::InvalidTarget(format!("Faction {} not found", id)))
    }

    pub fn create_faction(&mut self, name: String, is_player: bool, ai_type: AIPersonality) -> GameResult<FactionId> {
        if name.trim().is_empty() {
            return Err(GameError::InvalidOperation("Faction name cannot be empty".into()));
        }

        if self.factions.iter().any(|f| f.name == name) {
            return Err(GameError::InvalidOperation(format!("Faction with name '{}' already exists", name)));
        }

        let id = self.next_faction_id;
        self.next_faction_id += 1;

        let faction = Faction {
            id,
            name,
            is_player,
            ai_type,
            score: 0,
        };

        self.factions.push(faction);
        self.faction_index.insert(id, self.factions.len() - 1);
        Ok(id)
    }

    pub fn get_faction(&self, id: FactionId) -> GameResult<&Faction> {
        let index = self.get_faction_index(id)?;
        Ok(&self.factions[index])
    }

    pub fn update_score(&mut self, id: FactionId, score: i32) -> GameResult<()> {
        if score < 0 {
            return Err(GameError::InvalidOperation("Faction score cannot be negative".into()));
        }

        let index = self.get_faction_index(id)?;
        self.factions[index].score = score;
        Ok(())
    }

    pub fn add_score(&mut self, id: FactionId, points: i32) -> GameResult<()> {
        let index = self.get_faction_index(id)?;
        let new_score = self.factions[index].score.saturating_add(points);
        self.factions[index].score = new_score;
        Ok(())
    }

    pub fn get_all_factions(&self) -> &[Faction] {
        &self.factions
    }

    pub fn faction_count(&self) -> usize {
        self.factions.len()
    }

    pub fn find_faction_by_name(&self, name: &str) -> Option<&Faction> {
        self.factions.iter().find(|f| f.name == name)
    }

    pub fn get_player_faction(&self) -> Option<&Faction> {
        self.factions.iter().find(|f| f.is_player)
    }

    pub fn load_factions(&mut self, factions: Vec<Faction>) -> GameResult<()> {
        // Validation
        for faction in &factions {
            if faction.name.trim().is_empty() {
                return Err(GameError::InvalidOperation("Loaded faction has empty name".into()));
            }
            if faction.score < 0 {
                return Err(GameError::InvalidOperation("Loaded faction has negative score".into()));
            }
        }

        // Check for duplicates
        let mut seen_ids = std::collections::HashSet::new();
        let mut seen_names = std::collections::HashSet::new();
        for faction in &factions {
            if !seen_ids.insert(faction.id) {
                return Err(GameError::InvalidOperation(format!("Duplicate faction ID: {}", faction.id)));
            }
            if !seen_names.insert(&faction.name) {
                return Err(GameError::InvalidOperation(format!("Duplicate faction name: {}", faction.name)));
            }
        }

        self.factions = factions;
        self.faction_index.clear();
        self.faction_index.reserve(self.factions.len());
        
        for (index, faction) in self.factions.iter().enumerate() {
            self.faction_index.insert(faction.id, index);
        }

        self.next_faction_id = self.factions.iter()
            .map(|f| f.id)
            .max()
            .map(|max_id| max_id + 1)
            .unwrap_or(0);

        Ok(())
    }
}

impl GameSystem for EntityManager {
    fn update(&mut self, _delta: f32, _events: &mut EventBus) -> GameResult<()> {
        // No regular updates needed
        Ok(())
    }

    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::StateChanged(state_change) => {
                match state_change {
                    StateChange::PlanetUpdated(planet_id) => {
                        self.get_planet_index(*planet_id)?;
                        Ok(())
                    }
                    StateChange::FactionUpdated(faction_id) => {
                        self.get_faction_index(*faction_id)?;
                        Ok(())
                    }
                    StateChange::GameOver(winner_id) => {
                        self.get_faction_index(*winner_id)?;
                        Ok(())
                    }
                    _ => Ok(())
                }
            }
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    SimulationEvent::PlanetConquered { planet: planet_id, new_owner } => {
                        self.get_planet_index(*planet_id)?;
                        self.get_faction_index(*new_owner)?;
                        Ok(())
                    }
                    _ => Ok(())
                }
            }
            _ => Ok(())
        }
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}