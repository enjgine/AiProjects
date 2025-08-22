// src/ui_v2/panels/mod.rs
//! Production-ready migrated panels for ui_v2 system
//! 
//! These panels replace the corresponding panels in src/ui/panels/ with
//! component-based implementations that maintain full compatibility.

pub mod planet_panel_migrated;
pub mod ship_panel_migrated;
pub mod resource_panel_migrated;

pub use planet_panel_migrated::PlanetPanelMigrated;
pub use ship_panel_migrated::ShipPanelMigrated;
pub use resource_panel_migrated::ResourcePanelMigrated;