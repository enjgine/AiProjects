// src/managers/faction_manager.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use std::collections::HashMap;

pub struct FactionManager {
    factions: Vec<Faction>,
    faction_index: HashMap<FactionId, usize>,
}

impl FactionManager {
    pub fn new() -> Self {
        Self {
            factions: Vec::new(),
            faction_index: HashMap::new(),
        }
    }
    
    pub fn create_faction(&mut self, name: String, is_player: bool, ai_type: AIPersonality) -> GameResult<FactionId> {
        let id = self.factions.len() as FactionId;
        
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
        
        self.factions[*index].score = score;
        Ok(())
    }
    
    pub fn get_all_factions(&self) -> &Vec<Faction> {
        &self.factions
    }
    
    pub fn get_all_factions_cloned(&self) -> GameResult<Vec<Faction>> {
        Ok(self.factions.clone())
    }
    
    pub fn load_factions(&mut self, factions: Vec<Faction>) -> GameResult<()> {
        // Replace all factions with loaded data
        self.factions = factions;
        
        // Rebuild the index
        self.faction_index.clear();
        for (index, faction) in self.factions.iter().enumerate() {
            self.faction_index.insert(faction.id, index);
        }
        
        Ok(())
    }
    
    pub fn handle_event(&mut self, _event: &GameEvent) -> GameResult<()> {
        // FactionManager doesn't need to handle events currently
        Ok(())
    }
}