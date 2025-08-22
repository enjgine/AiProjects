// src/ui_v2/adapters/mod.rs
//! Entity adapters for converting game data to UI-displayable format

pub mod entity_adapter;
pub mod planet_adapter;
pub mod ship_adapter;
pub mod faction_adapter;

pub use entity_adapter::*;
pub use planet_adapter::*;
pub use ship_adapter::*;
pub use faction_adapter::*;