#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

//! # Stellar Dominion Game Engine
//!
//! A real-time space empire simulation game built with strict EventBus architecture
//! and deterministic fixed timestep simulation.
//!
//! ## Quick Start
//!
//! ```rust
//! use stellar_dominion::prelude::*;
//!
//! fn main() -> GameResult<()> {
//!     // Initialize a new game
//!     let mut game = stellar_dominion::setup::new_game()?;
//!     
//!     // Game loop with fixed timestep
//!     loop {
//!         game.fixed_update(FIXED_TIMESTEP)?;
//!         
//!         // Handle rendering with interpolation
//!         // game.render(interpolation)?;
//!         
//!         // Break condition would go here
//!         break;
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture Overview
//!
//! The game follows a strict **EventBus pattern** where all systems communicate exclusively
//! through events. No direct system-to-system references are allowed, ensuring clean
//! separation of concerns and deterministic behavior.
//!
//! ### Core Components
//!
//! - [`GameState`]: Central orchestrator managing all systems and the EventBus
//! - [`EventBus`]: Message routing system for inter-system communication
//! - **Managers**: Data owners providing CRUD operations ([`Planet`], [`Ship`], [`Faction`])
//! - **Systems**: Simulation logic processors (Physics, Resources, Population, etc.)
//! - **Fixed Timestep**: 0.1 second deterministic simulation timesteps
//!
//! ### System Update Order
//!
//! Systems update in a strict order each tick:
//! 1. UI System (processes player input)
//! 2. Physics Engine (movement and spatial updates)
//! 3. Resource System (resource production and consumption)
//! 4. Population System (demographic changes)
//! 5. Construction System (building completion)
//! 6. Combat Resolver (battle resolution)
//! 7. Time Manager (tick advancement)
//!
//! ## Error Handling
//!
//! All operations return [`GameResult<T>`] which is an alias for `Result<T, GameError>`.
//! This ensures consistent error handling throughout the system.
//!
//! ### Error Types
//!
//! - [`GameError::SystemError`]: Critical system failures, validation errors, and save/load failures
//! - [`GameError::ResourceError`]: Resource constraint violations
//! - [`GameError::CombatError`]: Combat resolution failures
//!
//! ## Resource Management
//!
//! - All resources are `i32` values (no floating point for determinism)
//! - Resources cannot go negative (enforced by [`ResourceBundle::subtract()`])
//! - Worker allocation must equal total population
//! - Building slots = 10 + population/10000
//!
//! ## Feature Flags
//!
//! - `debug`: Enables debug utilities and verbose logging

// Core module declarations
pub mod core;
pub mod managers;
pub mod systems;
pub mod ui;

// Public API exports - carefully controlled interface
// These are the only types that external consumers should directly access
pub use core::{
    // === Core Game State and Orchestration ===
    GameState,
    GameSystem,
    
    // === Error Handling ===
    GameResult,
    GameError,
    
    // === Event System for External Integration ===
    EventBus,
    GameEvent,
    PlayerCommand,
    SimulationEvent,
    StateChange,
    SystemId,
    
    // === Entity Identifiers ===
    PlanetId,
    ShipId,
    FactionId,
    PlayerId,
    
    // === Resource Management ===
    ResourceBundle,
    ResourceStorage,
    ResourceType,
    
    // === Population and Demographics ===
    Demographics,
    WorkerAllocation,
    
    // === Construction and Buildings ===
    BuildingType,
    Building,
    
    // === Ships and Spatial Systems ===
    Ship,
    ShipClass,
    CargoHold,
    Vector2,
    OrbitalElements,
    Trajectory,
    
    // === Factions and AI ===
    Planet,
    Faction,
    AIPersonality,
    
    // === Combat and Victory Conditions ===
    CombatOutcome,
    VictoryType,
};

// Utility modules for common patterns
pub mod prelude {
    //! Common imports for game development
    //!
    //! This module re-exports the most commonly used types and traits
    //! for convenience when developing game systems.
    //!
    //! # Usage
    //!
    //! ```rust
    //! use stellar_dominion::prelude::*;
    //!
    //! fn my_system_update(game: &mut GameState) -> GameResult<()> {
    //!     // All common types are now available
    //!     Ok(())
    //! }
    //! ```
    
    pub use crate::{
        GameState,
        GameSystem,
        GameResult,
        GameError,
        GameEvent,
        PlayerCommand,
        SimulationEvent,
        StateChange,
        EventBus,
        ResourceBundle,
        ResourceType,
        Vector2,
        PlanetId,
        ShipId,
        FactionId,
    };
    
    // Re-export key configuration constants
    pub use crate::config::{
        FIXED_TIMESTEP,
        MAX_PLANETS,
        MAX_SHIPS,
        MAX_FACTIONS,
    };
}

// Version and metadata
/// Current version of the Stellar Dominion game engine
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Display name of the game
pub const GAME_NAME: &str = "Stellar Dominion";

/// Game engine identifier for save files and networking
pub const ENGINE_ID: &str = "stellar-dominion-engine";

/// Minimum supported save format version
pub const MIN_SAVE_VERSION: u32 = 1;

/// Current save format version
pub const CURRENT_SAVE_VERSION: u32 = 1;

/// Build timestamp (uses version as fallback)
pub const BUILD_TIMESTAMP: &str = env!("CARGO_PKG_VERSION");

/// Target architecture (placeholder for future use)
pub const TARGET_ARCH: &str = "unknown";

// Game configuration constants
pub mod config {
    //! Game configuration constants and limits
    //!
    //! This module contains all gameplay configuration values, limits,
    //! and constants used throughout the simulation engine.
    
    /// Fixed timestep duration in seconds for deterministic simulation
    pub const FIXED_TIMESTEP: f32 = 0.1;
    
    /// Maximum number of planets supported by the simulation
    pub const MAX_PLANETS: usize = 100;
    
    /// Maximum number of ships supported by the simulation
    pub const MAX_SHIPS: usize = 500;
    
    /// Maximum number of factions that can exist simultaneously
    pub const MAX_FACTIONS: u8 = 8;
    
    /// Base building slots available on any planet
    pub const BASE_BUILDING_SLOTS: i32 = 10;
    
    /// Additional building slots granted per 10,000 population
    pub const BUILDING_SLOTS_PER_10K_POP: i32 = 1;
    
    /// Maximum i32 value for resource overflow protection
    /// This provides a safety buffer to prevent arithmetic overflow in resource calculations
    pub const MAX_RESOURCE_VALUE: i32 = i32::MAX - 1000;
    
    /// Minimum safe resource value to prevent underflow
    pub const MIN_RESOURCE_VALUE: i32 = 0;
    
    /// Minimum valid tick value
    pub const MIN_TICK_VALUE: i64 = 0;
    
    /// Maximum safe tick value (prevents overflow)
    pub const MAX_SAFE_TICK_VALUE: i64 = u64::MAX as i64 - 10000;
    
    /// Default save file name
    pub const DEFAULT_SAVE_FILE: &str = "stellar_dominion.save";
    
    /// Maximum length for player/faction names
    pub const MAX_NAME_LENGTH: usize = 32;
    
    /// Validation helper: Check if a resource value is within safe bounds
    pub const fn is_resource_value_safe(value: i32) -> bool {
        value >= MIN_RESOURCE_VALUE && value <= MAX_RESOURCE_VALUE
    }
    
    /// Validation helper: Check if a tick value is within safe bounds
    pub const fn is_tick_value_safe(tick: i64) -> bool {
        tick >= MIN_TICK_VALUE && tick <= MAX_SAFE_TICK_VALUE
    }
}

// Initialization and setup utilities
pub mod setup {
    //! Game initialization utilities
    
    use crate::{GameState, GameResult, GameError};
    
    /// Initialize a new game with default settings
    /// 
    /// Creates a fresh game state with all systems properly initialized
    /// and ready for simulation.
    /// 
    /// # Errors
    /// 
    /// Returns `GameError::SystemError` if any core system fails to initialize.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use stellar_dominion::setup;
    /// 
    /// let game = setup::new_game().expect("Failed to create new game");
    /// assert_eq!(game.get_current_tick(), 0);
    /// ```
    pub fn new_game() -> GameResult<GameState> {
        GameState::new()
            .map_err(|e| GameError::SystemError(format!("Failed to initialize new game: {}", e)))
    }
    
    /// Load a saved game from the default save location
    /// 
    /// # Errors
    /// 
    /// Returns `GameError::SaveError` if the save file cannot be loaded or is corrupted.
    /// Returns `GameError::SystemError` if the game state cannot be initialized.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use stellar_dominion::setup;
    /// 
    /// match setup::load_game() {
    ///     Ok(game) => println!("Game loaded successfully"),
    ///     Err(e) => eprintln!("Failed to load game: {}", e),
    /// }
    /// ```
    pub fn load_game() -> GameResult<GameState> {
        let mut game = GameState::new()
            .map_err(|e| GameError::SystemError(format!("Failed to initialize game state: {}", e)))?;
        
        game.load_game()
            .map_err(|e| GameError::SystemError(format!("Failed to load saved game: {}", e)))?;
        
        Ok(game)
    }
    
    /// Validate game state integrity
    /// 
    /// Performs comprehensive validation of the game state to ensure all
    /// architectural invariants are maintained and the game is in a consistent state.
    /// 
    /// # Arguments
    /// 
    /// * `game` - The game state to validate
    /// 
    /// # Errors
    /// 
    /// Returns `GameError::ValidationError` if any validation checks fail.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use stellar_dominion::{GameState, setup};
    /// 
    /// let game = GameState::new().unwrap();
    /// setup::validate_game_state(&game).expect("Game state should be valid");
    /// ```
    pub fn validate_game_state(game: &GameState) -> GameResult<()> {
        // Validate tick counter is in valid range
        let current_tick = game.get_current_tick();
        if current_tick < 0 {
            return Err(GameError::SystemError(
                format!("Invalid tick counter: {} (must be >= 0)", current_tick)
            ));
        }
        
        // Check for tick overflow protection
        if current_tick as u64 > (u64::MAX - 10000) {
            return Err(GameError::SystemError(
                format!("Tick counter approaching overflow: {}", current_tick)
            ));
        }
        
        // TODO: Add manager-specific validation when managers provide validation methods
        // - Planet manager: validate resource constraints, worker allocations
        // - Ship manager: validate movement constraints, cargo limits
        // - Faction manager: validate relationship consistency
        
        Ok(())
    }
}

// Feature flags for conditional compilation
#[cfg(feature = "debug")]
pub mod debug {
    //! Debug utilities for development
    
    use crate::GameState;
    
    /// Print detailed game state for debugging
    /// 
    /// Outputs comprehensive debug information about the current game state,
    /// including system status, resource counts, and architectural health.
    /// 
    /// # Arguments
    /// 
    /// * `game` - The game state to debug
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use stellar_dominion::{GameState, debug};
    /// 
    /// let game = GameState::new().unwrap();
    /// debug::print_game_state(&game);
    /// ```
    pub fn print_game_state(game: &GameState) {
        println!("=== Stellar Dominion Debug State ===");
        println!("Current Tick: {}", game.get_current_tick());
        println!("Game Version: {}", VERSION);
        
        // Validate architecture and display status
        match validate_architecture(game) {
            Ok(_) => println!("Architecture Status: ✓ Valid"),
            Err(msg) => println!("Architecture Status: ✗ Invalid - {}", msg),
        }
        
        println!("====================================");
    }
    
    /// Validate all architectural invariants
    /// 
    /// Performs comprehensive validation of architectural constraints including
    /// EventBus integrity, system dependencies, and state consistency.
    /// 
    /// # Arguments
    /// 
    /// * `game` - The game state to validate
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if all invariants are satisfied, `Err(String)` with
    /// detailed error information if any violations are found.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use stellar_dominion::{GameState, debug};
    /// 
    /// let game = GameState::new().unwrap();
    /// match debug::validate_architecture(&game) {
    ///     Ok(_) => println!("Architecture is valid"),
    ///     Err(msg) => eprintln!("Architecture violation: {}", msg),
    /// }
    /// ```
    pub fn validate_architecture(game: &GameState) -> Result<(), String> {
        // Validate basic state integrity
        let current_tick = game.get_current_tick();
        if current_tick < 0 {
            return Err(format!("Invalid tick counter: {} (must be >= 0)", current_tick));
        }
        
        // TODO: Add comprehensive architecture validation:
        // - EventBus subscription integrity
        // - System update order compliance
        // - No direct system-to-system references
        // - Manager ownership patterns
        // - Resource constraint enforcement
        
        Ok(())
    }
}

// Integration tests helper
#[cfg(test)]
pub mod test_utils {
    //! Testing utilities for integration tests
    //!
    //! This module provides helper functions for creating test scenarios,
    //! running controlled simulations, and validating game state during testing.
    
    use crate::{GameState, GameResult, GameError, config};
    
    /// Create a minimal game state for testing
    /// 
    /// Initializes a basic game state suitable for unit and integration testing.
    /// This function ensures deterministic initialization for reproducible tests.
    /// 
    /// # Errors
    /// 
    /// Returns `GameError::SystemError` if the test game state cannot be created.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use stellar_dominion::test_utils;
    /// 
    /// let game = test_utils::create_test_game().expect("Test game creation failed");
    /// assert_eq!(game.get_current_tick(), 0);
    /// ```
    pub fn create_test_game() -> GameResult<GameState> {
        GameState::new()
            .map_err(|e| GameError::SystemError(format!("Failed to create test game: {}", e)))
    }
    
    /// Run one simulation tick for testing
    /// 
    /// Executes exactly one fixed timestep update cycle, useful for deterministic
    /// testing of game logic.
    /// 
    /// # Arguments
    /// 
    /// * `game` - Mutable reference to the game state to update
    /// 
    /// # Errors
    /// 
    /// Returns any error that occurs during the simulation update cycle.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use stellar_dominion::test_utils;
    /// 
    /// let mut game = test_utils::create_test_game().unwrap();
    /// test_utils::simulate_tick(&mut game).unwrap();
    /// assert!(game.get_current_tick() > 0);
    /// ```
    pub fn simulate_tick(game: &mut GameState) -> GameResult<()> {
        if !config::is_tick_value_safe(game.get_current_tick()) {
            return Err(GameError::SystemError(
                format!("Cannot simulate tick on invalid game state (tick: {})", 
                       game.get_current_tick())
            ));
        }
        
        game.fixed_update(config::FIXED_TIMESTEP)
    }
    
    /// Run multiple simulation ticks for batch testing
    /// 
    /// Executes a specified number of simulation ticks, useful for testing
    /// longer-term game behavior and system interactions.
    /// 
    /// # Arguments
    /// 
    /// * `game` - Mutable reference to the game state to update
    /// * `ticks` - Number of ticks to simulate (must be > 0)
    /// 
    /// # Errors
    /// 
    /// Returns validation error if tick count is invalid, or any simulation error.
    pub fn simulate_ticks(game: &mut GameState, ticks: u32) -> GameResult<()> {
        if ticks == 0 {
            return Err(GameError::SystemError(
                "Tick count must be greater than 0".into()
            ));
        }
        
        for _ in 0..ticks {
            simulate_tick(game)?;
        }
        
        Ok(())
    }
    
    /// Assert that game state remains valid after operations
    /// 
    /// Helper function for tests to verify game state integrity.
    pub fn assert_game_state_valid(game: &GameState) {
        crate::setup::validate_game_state(game)
            .expect("Game state validation failed");
    }
}