// src/ui/panels/mod.rs
//! UI panel modules for the Stellar Dominion game.
//! 
//! This module provides specialized panels for displaying game information:
//! - PlanetPanel: Displays detailed planet information including resources, population, and buildings
//! - ShipPanel: Shows ship status, position, fuel, and cargo information  
//! - ResourcePanel: Empire-wide resource summary and game status information
//!
//! All panels follow the EventBus architecture and only emit PlayerCommand events.

pub mod planet_panel;
pub mod ship_panel; 
pub mod resource_panel;

pub use planet_panel::PlanetPanel;
pub use ship_panel::ShipPanel;
pub use resource_panel::ResourcePanel;

// Core types are imported by individual panel modules as needed

/// Common trait for all UI panels to ensure consistent interface
pub trait Panel {
    /// Initialize a new panel instance
    fn new() -> Self;
    
    /// Show the panel (make it visible)
    fn show(&mut self);
    
    /// Hide the panel (make it invisible)
    fn hide(&mut self);
    
    /// Check if the panel is currently visible
    fn is_visible(&self) -> bool;
    
    /// Toggle panel visibility
    fn toggle(&mut self) {
        if self.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }
}