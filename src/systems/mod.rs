// src/systems/mod.rs
//
// System implementations for Stellar Dominion
//
// All systems follow the EventBus architecture:
// - Communicate exclusively through events (no direct references)
// - Implement GameSystem trait (update + handle_event methods)
// - Process in fixed order: Physics → Resources → Population → Construction → Combat → Time
// - Emit events for state changes, never mutate directly
// - Return GameResult<T> from all operations
//
// Systems subscribe to events in GameState::new() and are called in
// GameState::fixed_update() following the strict architectural order.

pub mod time_manager;
pub mod physics_engine;
pub mod resource_system;
pub mod population_system;
pub mod construction;
pub mod combat_resolver;
pub mod save_system;
pub mod game_initializer;

// Re-export all systems for use in GameState
pub use time_manager::TimeManager;
pub use physics_engine::PhysicsEngine;
pub use resource_system::ResourceSystem;
pub use population_system::PopulationSystem;
pub use construction::ConstructionSystem;
pub use combat_resolver::CombatResolver;
pub use save_system::SaveSystem;
pub use game_initializer::GameInitializer;

// Ensure all systems implement the required GameSystem trait
// This is enforced at compile time when systems are instantiated in GameState