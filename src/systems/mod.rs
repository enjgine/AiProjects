// src/systems/mod.rs
pub mod time_manager;
pub mod physics_engine;
pub mod resource_system;
pub mod population_system;
pub mod construction;
pub mod combat_resolver;
pub mod save_system;

pub use time_manager::TimeManager;
pub use physics_engine::PhysicsEngine;
pub use resource_system::ResourceSystem;
pub use population_system::PopulationSystem;
pub use construction::ConstructionSystem;
pub use combat_resolver::CombatResolver;
pub use save_system::SaveSystem;