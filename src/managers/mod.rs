// src/managers/mod.rs
//! Manager implementations for Stellar Dominion
//! 
//! This module contains all data management structures that own game entities.
//! Managers provide controlled access to data through validated CRUD operations
//! that return GameResult<T> for all state mutations.
//!
//! ## Architecture Compliance
//! - All managers implement the Manager pattern with owned data collections
//! - State mutations are validated before application
//! - Managers communicate only through the EventBus
//! - No direct inter-manager dependencies allowed
//!
//! ## Manager Responsibilities:
//! - PlanetManager: Owns planets, handles resource/population/building operations
//! - ShipManager: Owns ships, handles movement, cargo, and combat interactions
//! - FactionManager: Owns factions, tracks scores and player relationships

/// Planet management implementation providing controlled access to planet data
pub mod planet_manager;
/// Ship management implementation handling fleet operations and movement
pub mod ship_manager;
/// Faction management implementation tracking player relationships and scores
pub mod faction_manager;

pub use planet_manager::PlanetManager;
pub use ship_manager::ShipManager;
pub use faction_manager::FactionManager;

use crate::core::{GameResult, GameEvent};

/// Common trait for all managers that need to handle events
/// This ensures consistent event handling patterns across all managers
pub trait ManagerEventHandler {
    /// Handle an event from the EventBus
    /// Returns GameResult to ensure proper error propagation
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>;
}

/// Validation utilities for manager implementations
pub mod validation {
    use crate::core::{GameResult, GameError};
    
    /// Validates that an ID is within reasonable bounds to prevent overflow attacks
    pub fn validate_id_bounds(id: u32, max_entities: u32) -> GameResult<()> {
        if id >= max_entities {
            return Err(GameError::InvalidTarget(
                format!("ID {} exceeds maximum allowed entities ({})", id, max_entities)
            ));
        }
        Ok(())
    }
    
    /// Validates string inputs to prevent empty or malicious data
    pub fn validate_name(name: &str, context: &str) -> GameResult<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(GameError::InvalidOperation(
                format!("{} name cannot be empty", context)
            ));
        }
        if trimmed.len() > 100 {
            return Err(GameError::InvalidOperation(
                format!("{} name too long (max 100 characters)", context)
            ));
        }
        if trimmed.chars().any(|c| c.is_control() && c != '\t' && c != '\n') {
            return Err(GameError::InvalidOperation(
                format!("{} name contains invalid characters", context)
            ));
        }
        Ok(())
    }
    
    /// Validates numeric values to prevent overflow in calculations
    pub fn validate_numeric_bounds(value: i32, min: i32, max: i32, context: &str) -> GameResult<()> {
        if value < min || value > max {
            return Err(GameError::InvalidOperation(
                format!("{} value {} out of bounds ({} to {})", context, value, min, max)
            ));
        }
        Ok(())
    }
}

/// Performance optimization utilities for manager collections
pub mod optimization {
    use std::collections::HashMap;
    
    /// Pre-allocates collections with reasonable capacity to reduce reallocations
    pub fn create_entity_vec<T>() -> Vec<T> {
        Vec::with_capacity(1000) // Expected max entities per game
    }
    
    /// Creates an optimally-sized index HashMap
    pub fn create_index_map<K, V>() -> HashMap<K, V> 
    where 
        K: std::hash::Hash + Eq,
    {
        HashMap::with_capacity(1000) // Match entity vec capacity
    }
    
    /// Validates collection size to prevent memory exhaustion
    pub fn check_collection_size<T>(collection: &Vec<T>, max_size: usize, entity_type: &str) -> crate::core::GameResult<()> {
        if collection.len() >= max_size {
            return Err(crate::core::GameError::SystemError(
                format!("{} collection at maximum capacity ({})", entity_type, max_size)
            ));
        }
        Ok(())
    }
}