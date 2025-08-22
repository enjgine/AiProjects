// src/systems/save_system.rs
//! Simplified save system for Stellar Dominion
//! 
//! Features:
//! - JSON serialization for simplicity and debugging
//! - Named save files
//! - Save list management
//! - Deterministic state preservation

use crate::core::{GameResult, GameEvent, EventBus, GameState, GameSystem};
use crate::core::types::*;
use crate::core::events::PlayerCommand;
use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Simple save data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub save_name: String,
    pub timestamp: u64,
    pub tick: u64,
    pub planets: Vec<Planet>,
    pub ships: Vec<Ship>,
    pub factions: Vec<Faction>,
    pub game_configuration: GameConfiguration,
}

/// Save file metadata for the save browser
#[derive(Debug, Clone)]
pub struct SaveInfo {
    pub name: String,
    pub timestamp: u64,
    pub tick: u64,
    pub planets: usize,
    pub ships: usize,
    pub factions: usize,
}

impl SaveInfo {
    pub fn from_save_data(data: &SaveData) -> Self {
        Self {
            name: data.save_name.clone(),
            timestamp: data.timestamp,
            tick: data.tick,
            planets: data.planets.len(),
            ships: data.ships.len(),
            factions: data.factions.len(),
        }
    }
}

/// Simplified save system
pub struct SaveSystem {
    save_directory: PathBuf,
    current_save_name: Option<String>,
}

impl SaveSystem {
    pub fn new() -> Self {
        let save_dir = PathBuf::from("saves");
        if !save_dir.exists() {
            std::fs::create_dir_all(&save_dir).unwrap_or_else(|e| {
                eprintln!("Warning: Could not create saves directory: {}", e);
            });
        }
        
        Self {
            save_directory: save_dir,
            current_save_name: None,
        }
    }
    
    /// Save game with current save name or default
    pub fn save_game(&mut self, state: &GameState) -> GameResult<()> {
        let save_name = self.current_save_name
            .clone()
            .unwrap_or_else(|| "quicksave".to_string());
        self.save_game_to_slot(state, &save_name)
    }
    
    /// Save game to specific named slot
    pub fn save_game_to_slot(&mut self, state: &GameState, slot_name: &str) -> GameResult<()> {
        let save_data = SaveData {
            version: 1,
            save_name: slot_name.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            tick: state.get_current_tick(),
            planets: state.planet_manager.get_all_planets().clone(),
            ships: state.ship_manager.get_all_ships().clone(),
            factions: state.faction_manager.get_all_factions().to_vec(),
            game_configuration: state.game_initializer.get_configuration().clone(),
        };
        
        let file_path = self.get_save_path(slot_name);
        let json = serde_json::to_string_pretty(&save_data)
            .map_err(|e| GameError::SaveError(format!("JSON serialization failed: {}", e)))?;
        
        let mut file = File::create(&file_path)
            .map_err(|e| GameError::SaveError(format!("Could not create save file: {}", e)))?;
        
        file.write_all(json.as_bytes())
            .map_err(|e| GameError::SaveError(format!("Could not write save file: {}", e)))?;
        
        self.current_save_name = Some(slot_name.to_string());
        Ok(())
    }
    
    /// Load game from default save
    pub fn load_game(&self) -> GameResult<SaveData> {
        let save_name = "quicksave";
        self.load_game_from_slot(save_name)
    }
    
    /// Load game from specific named slot
    pub fn load_game_from_slot(&self, slot_name: &str) -> GameResult<SaveData> {
        let file_path = self.get_save_path(slot_name);
        
        if !file_path.exists() {
            return Err(GameError::SaveError(format!("Save file '{}' not found", slot_name)));
        }
        
        let mut file = File::open(&file_path)
            .map_err(|e| GameError::SaveError(format!("Could not open save file: {}", e)))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| GameError::SaveError(format!("Could not read save file: {}", e)))?;
        
        let save_data: SaveData = serde_json::from_str(&contents)
            .map_err(|e| GameError::SaveError(format!("JSON deserialization failed: {}", e)))?;
        
        self.validate_save_integrity(&save_data)?;
        Ok(save_data)
    }
    
    /// List all available save files
    pub fn list_saves(&self) -> GameResult<Vec<SaveInfo>> {
        let mut saves = Vec::new();
        
        if !self.save_directory.exists() {
            return Ok(saves);
        }
        
        let entries = fs::read_dir(&self.save_directory)
            .map_err(|e| GameError::SaveError(format!("Could not read saves directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| GameError::SaveError(format!("Error reading directory entry: {}", e)))?;
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sav") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.load_game_from_slot(stem) {
                        Ok(save_data) => saves.push(SaveInfo::from_save_data(&save_data)),
                        Err(_) => continue, // Skip corrupted saves
                    }
                }
            }
        }
        
        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(saves)
    }
    
    /// Check if a save file exists
    pub fn save_exists(&self, slot_name: &str) -> bool {
        self.get_save_path(slot_name).exists()
    }
    
    /// Delete a save file
    pub fn delete_save(&self, slot_name: &str) -> GameResult<()> {
        let file_path = self.get_save_path(slot_name);
        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| GameError::SaveError(format!("Could not delete save file: {}", e)))?;
        }
        Ok(())
    }
    
    /// Validate save data integrity
    pub fn validate_save_integrity(&self, save_data: &SaveData) -> GameResult<()> {
        // Basic validation
        if save_data.version != 1 {
            return Err(GameError::SaveError(format!("Unsupported save version: {}", save_data.version)));
        }
        
        if save_data.planets.is_empty() {
            return Err(GameError::SaveError("Save file contains no planets".to_string()));
        }
        
        if save_data.factions.is_empty() {
            return Err(GameError::SaveError("Save file contains no factions".to_string()));
        }
        
        // Validate resource constraints
        for planet in &save_data.planets {
            planet.population.allocation.validate(planet.population.total)?;
            planet.resources.validate()?;
        }
        
        Ok(())
    }
    
    /// Get the file path for a save slot
    fn get_save_path(&self, slot_name: &str) -> PathBuf {
        self.save_directory.join(format!("{}.sav", slot_name))
    }
}

impl GameSystem for SaveSystem {
    fn update(&mut self, _delta: f32, _events: &mut EventBus) -> GameResult<()> {
        // Save system doesn't need regular updates
        Ok(())
    }
    
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        // Save system responds to save/load commands but doesn't handle them directly
        // The GameState handles these commands and calls the save system methods
        match event {
            GameEvent::PlayerCommand(PlayerCommand::SaveGame) |
            GameEvent::PlayerCommand(PlayerCommand::LoadGame) => {
                // These are handled by GameState, not here
                Ok(())
            }
            _ => Ok(())
        }
    }
}

impl Default for SaveSystem {
    fn default() -> Self {
        Self::new()
    }
}