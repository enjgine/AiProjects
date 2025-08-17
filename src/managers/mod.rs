// src/managers/mod.rs
pub mod planet_manager;
pub mod ship_manager;
pub mod faction_manager;

pub use planet_manager::PlanetManager;
pub use ship_manager::ShipManager;
pub use faction_manager::FactionManager;