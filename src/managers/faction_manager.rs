// src/managers/faction_manager.rs
use crate::core::{GameResult, GameEvent};
use crate::core::types::*;
use std::collections::HashMap;

pub struct FactionManager {
    factions: Vec<Faction>,
    faction_index: HashMap<FactionId, usize>,
    next_id: FactionId,
}

impl FactionManager {
    pub fn new() -> Self {
        Self {
            factions: Vec::new(),
            faction_index: HashMap::new(),
            next_id: 0,
        }
    }
    
    pub fn create_faction(&mut self, name: String, is_player: bool, ai_type: AIPersonality) -> GameResult<FactionId> {
        // Validate faction name
        if name.trim().is_empty() {
            return Err(GameError::InvalidOperation("Faction name cannot be empty".into()));
        }
        
        // Check for duplicate names
        if self.factions.iter().any(|f| f.name == name) {
            return Err(GameError::InvalidOperation(format!("Faction with name '{}' already exists", name)));
        }
        
        let id = self.next_id;
        self.next_id += 1;
        
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
        let index = self.faction_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Faction {} not found", id)))?;
        Ok(&self.factions[*index])
    }
    
    pub fn update_score(&mut self, id: FactionId, score: i32) -> GameResult<()> {
        let index = self.faction_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Faction {} not found", id)))?;
        
        // Validate score (prevent overflow)
        if score < 0 {
            return Err(GameError::InvalidOperation("Faction score cannot be negative".into()));
        }
        
        self.factions[*index].score = score;
        Ok(())
    }
    
    pub fn add_score(&mut self, id: FactionId, points: i32) -> GameResult<()> {
        let index = self.faction_index.get(&id)
            .ok_or_else(|| GameError::InvalidTarget(format!("Faction {} not found", id)))?;
        
        let current_score = self.factions[*index].score;
        let new_score = current_score.saturating_add(points);
        
        self.factions[*index].score = new_score;
        Ok(())
    }
    
    pub fn get_all_factions(&self) -> &[Faction] {
        &self.factions
    }
    
    pub fn count(&self) -> usize {
        self.factions.len()
    }
    
    pub fn find_by_name(&self, name: &str) -> Option<&Faction> {
        self.factions.iter().find(|f| f.name == name)
    }
    
    pub fn get_player_faction(&self) -> Option<&Faction> {
        self.factions.iter().find(|f| f.is_player)
    }
    
    pub fn load_factions(&mut self, factions: Vec<Faction>) -> GameResult<()> {
        // Validate loaded factions
        for faction in &factions {
            if faction.name.trim().is_empty() {
                return Err(GameError::InvalidOperation("Loaded faction has empty name".into()));
            }
            if faction.score < 0 {
                return Err(GameError::InvalidOperation("Loaded faction has negative score".into()));
            }
        }
        
        // Check for duplicate IDs or names
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
        
        // Replace all factions with loaded data
        self.factions = factions;
        
        // Rebuild the index efficiently
        self.faction_index.clear();
        self.faction_index.reserve(self.factions.len());
        for (index, faction) in self.factions.iter().enumerate() {
            self.faction_index.insert(faction.id, index);
        }
        
        // Update next_id to prevent conflicts
        self.next_id = self.factions.iter()
            .map(|f| f.id)
            .max()
            .map(|max_id| max_id + 1)
            .unwrap_or(0);
        
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::StateChanged(state_change) => {
                match state_change {
                    crate::core::events::StateChange::FactionUpdated(faction_id) => {
                        // Validate that the faction exists after update
                        self.get_faction(*faction_id)?;
                        Ok(())
                    }
                    crate::core::events::StateChange::GameOver(winner_id) => {
                        // Verify the winner exists
                        self.get_faction(*winner_id)?;
                        Ok(())
                    }
                    _ => Ok(())
                }
            }
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    crate::core::events::SimulationEvent::PlanetConquered { new_owner, .. } => {
                        // Verify the conquering faction exists
                        self.get_faction(*new_owner)?;
                        Ok(())
                    }
                    _ => Ok(())
                }
            }
            _ => Ok(())
        }
    }
}