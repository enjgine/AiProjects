// src/ui/ui_state.rs
//! UI state management for the Stellar Dominion game.
//!
//! This module handles UI-specific state that doesn't affect game simulation:
//! - Panel visibility and states
//! - User input modes and selections
//! - Temporary UI data (not persisted in save files)
//! - UI preferences and settings
//!
//! All state transitions maintain consistency invariants and provide validation
//! for proper UI behavior within the game's EventBus architecture.

use crate::core::types::*;

/// Represents all panels that can be opened/closed in the UI
#[derive(Debug, Clone, Copy, Default)]
pub struct PanelStates {
    pub planet_panel_open: bool,
    pub ship_panel_open: bool,
    pub resource_panel_open: bool,
    pub build_menu_open: bool,
    pub settings_panel_open: bool,
}

impl PanelStates {
    /// Close all panels except the resource panel
    pub fn close_all_except_resource(&mut self) {
        self.planet_panel_open = false;
        self.ship_panel_open = false;
        self.build_menu_open = false;
        self.settings_panel_open = false;
        // Keep resource_panel_open as is
    }
    
    /// Check if any editing panel is open (planet, ship, build menu)
    pub fn has_editing_panel_open(&self) -> bool {
        self.planet_panel_open || self.ship_panel_open || self.build_menu_open
    }
}

/// Represents visualization display options
#[derive(Debug, Clone, Copy)]
pub struct DisplayOptions {
    pub show_orbits: bool,
    pub show_ship_paths: bool,
    pub show_resource_flows: bool,
    pub show_detailed_tooltips: bool,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            show_orbits: true,
            show_ship_paths: true,
            show_resource_flows: false,
            show_detailed_tooltips: true,
        }
    }
}

/// UI scale constraints
const MIN_UI_SCALE: f32 = 0.5;
const MAX_UI_SCALE: f32 = 2.0;
const DEFAULT_UI_SCALE: f32 = 1.0;

/// Main UI state container managing all UI-specific data
#[derive(Debug, Clone)]
pub struct UIState {
    // Entity selection state
    pub selected_planet: Option<PlanetId>,
    pub selected_ship: Option<ShipId>,
    pub selected_building_type: Option<BuildingType>,
    
    // Panel management
    pub panels: PanelStates,
    
    // Game flow control
    pub paused: bool,
    
    // Display preferences
    pub display: DisplayOptions,
    
    // Temporary editing state (not persisted)
    pub worker_allocation_temp: WorkerAllocation,
    pub resource_transfer_temp: ResourceBundle,
    pub transfer_target: Option<PlanetId>,
    
    // UI configuration
    ui_scale: f32, // Private to enforce validation
    pub auto_pause_on_events: bool,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            selected_planet: None,
            selected_ship: None,
            selected_building_type: None,
            panels: PanelStates {
                resource_panel_open: true, // Resource panel starts open
                ..Default::default()
            },
            paused: false,
            display: DisplayOptions::default(),
            worker_allocation_temp: WorkerAllocation::default(),
            resource_transfer_temp: ResourceBundle::default(),
            transfer_target: None,
            ui_scale: DEFAULT_UI_SCALE,
            auto_pause_on_events: false,
        }
    }
}

impl UIState {
    /// Create a new UIState with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Select a planet and update UI state accordingly
    /// Clears any ship selection and closes irrelevant panels
    pub fn select_planet(&mut self, planet_id: Option<PlanetId>) {
        self.selected_planet = planet_id;
        self.selected_ship = None;  // Clear ship selection
        self.selected_building_type = None; // Clear building selection
        
        // Update panel visibility based on selection
        self.panels.planet_panel_open = planet_id.is_some();
        self.panels.ship_panel_open = false;
        self.panels.build_menu_open = false;
        
        // Clear temporary editing state when switching selections
        self.clear_temp_state();
    }
    
    /// Select a ship and update UI state accordingly
    /// Clears any planet selection and closes irrelevant panels
    pub fn select_ship(&mut self, ship_id: Option<ShipId>) {
        self.selected_ship = ship_id;
        self.selected_planet = None;  // Clear planet selection
        self.selected_building_type = None; // Clear building selection
        
        // Update panel visibility based on selection
        self.panels.ship_panel_open = ship_id.is_some();
        self.panels.planet_panel_open = false;
        self.panels.build_menu_open = false;
        
        // Clear temporary editing state when switching selections
        self.clear_temp_state();
    }
    
    /// Clear all entity selections and close related panels
    pub fn clear_selection(&mut self) {
        self.selected_planet = None;
        self.selected_ship = None;
        self.selected_building_type = None;
        
        // Close entity-specific panels
        self.panels.close_all_except_resource();
        
        // Clear any temporary editing state
        self.clear_temp_state();
    }
    
    /// Toggle planet panel visibility if a planet is selected
    pub fn toggle_planet_panel(&mut self) {
        if self.selected_planet.is_some() {
            self.panels.planet_panel_open = !self.panels.planet_panel_open;
        }
    }
    
    /// Toggle ship panel visibility if a ship is selected
    pub fn toggle_ship_panel(&mut self) {
        if self.selected_ship.is_some() {
            self.panels.ship_panel_open = !self.panels.ship_panel_open;
        }
    }
    
    /// Toggle build menu visibility if a planet is selected
    pub fn toggle_build_menu(&mut self) {
        if self.selected_planet.is_some() {
            self.panels.build_menu_open = !self.panels.build_menu_open;
            if self.panels.build_menu_open {
                self.selected_building_type = None;  // Clear selection when opening
            } else {
                // Clear temp state when closing build menu
                self.selected_building_type = None;
            }
        }
    }
    
    /// Set game pause state
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
    
    /// Toggle game pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    /// Toggle orbital path display
    pub fn toggle_orbits(&mut self) {
        self.display.show_orbits = !self.display.show_orbits;
    }
    
    /// Toggle ship trajectory display
    pub fn toggle_ship_paths(&mut self) {
        self.display.show_ship_paths = !self.display.show_ship_paths;
    }
    
    /// Toggle resource flow visualization
    pub fn toggle_resource_flows(&mut self) {
        self.display.show_resource_flows = !self.display.show_resource_flows;
    }
    
    /// Toggle detailed tooltip display
    pub fn toggle_detailed_tooltips(&mut self) {
        self.display.show_detailed_tooltips = !self.display.show_detailed_tooltips;
    }
    
    /// Toggle settings panel visibility
    pub fn toggle_settings_panel(&mut self) {
        self.panels.settings_panel_open = !self.panels.settings_panel_open;
    }
    
    /// Toggle resource panel visibility
    pub fn toggle_resource_panel(&mut self) {
        self.panels.resource_panel_open = !self.panels.resource_panel_open;
    }
    
    /// Initialize worker allocation editing with current values
    pub fn start_worker_allocation_edit(&mut self, current: WorkerAllocation) {
        self.worker_allocation_temp = current;
    }
    
    /// Initialize resource transfer editing with specified target
    pub fn start_resource_transfer_edit(&mut self, target: Option<PlanetId>) {
        self.resource_transfer_temp = ResourceBundle::default();
        self.transfer_target = target;
    }
    
    /// Set the building type selection for construction
    pub fn set_selected_building(&mut self, building_type: Option<BuildingType>) {
        self.selected_building_type = building_type;
    }
    
    /// Clear temporary edit state
    pub fn clear_temp_state(&mut self) {
        self.worker_allocation_temp = WorkerAllocation::default();
        self.resource_transfer_temp = ResourceBundle::default();
        self.transfer_target = None;
        self.selected_building_type = None;
    }
    
    /// Check if any editing operation is in progress
    pub fn is_editing(&self) -> bool {
        self.transfer_target.is_some() || 
        self.panels.build_menu_open ||
        self.selected_building_type.is_some()
    }
    
    /// Check if any entity is currently selected
    pub fn has_selection(&self) -> bool {
        self.selected_planet.is_some() || self.selected_ship.is_some()
    }
    
    /// Validate that UI state is consistent
    pub fn validate_state(&self) -> bool {
        // Planet panel should only be open if planet is selected
        if self.panels.planet_panel_open && self.selected_planet.is_none() {
            return false;
        }
        
        // Ship panel should only be open if ship is selected
        if self.panels.ship_panel_open && self.selected_ship.is_none() {
            return false;
        }
        
        // Build menu should only be open if planet is selected
        if self.panels.build_menu_open && self.selected_planet.is_none() {
            return false;
        }
        
        // UI scale should be within bounds
        if self.ui_scale < MIN_UI_SCALE || self.ui_scale > MAX_UI_SCALE {
            return false;
        }
        
        true
    }
    
    /// Get current UI scale factor (always within valid bounds)
    pub fn get_ui_scale(&self) -> f32 {
        self.ui_scale.clamp(MIN_UI_SCALE, MAX_UI_SCALE)
    }
    
    /// Set UI scale with strict bounds checking
    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale = scale.clamp(MIN_UI_SCALE, MAX_UI_SCALE);
    }
    
    /// Check if detailed information should be shown
    pub fn should_show_details(&self) -> bool {
        self.display.show_detailed_tooltips
    }
    
    /// Get current display options
    pub fn get_display_options(&self) -> &DisplayOptions {
        &self.display
    }
    
    /// Get current panel states
    pub fn get_panel_states(&self) -> &PanelStates {
        &self.panels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ui_state_new() {
        let state = UIState::new();
        assert_eq!(state.selected_planet, None);
        assert_eq!(state.selected_ship, None);
        assert!(!state.paused);
        assert!(state.display.show_orbits);
        assert!(state.panels.resource_panel_open);
        assert!(!state.panels.planet_panel_open);
        assert!(state.validate_state());
    }
    
    #[test]
    fn test_planet_selection() {
        let mut state = UIState::new();
        let planet_id = 42;
        
        state.select_planet(Some(planet_id));
        assert_eq!(state.selected_planet, Some(planet_id));
        assert_eq!(state.selected_ship, None);
        assert!(state.panels.planet_panel_open);
        assert!(!state.panels.ship_panel_open);
        assert!(state.validate_state());
    }
    
    #[test]
    fn test_ship_selection() {
        let mut state = UIState::new();
        let ship_id = 123;
        
        state.select_ship(Some(ship_id));
        assert_eq!(state.selected_ship, Some(ship_id));
        assert_eq!(state.selected_planet, None);
        assert!(state.panels.ship_panel_open);
        assert!(!state.panels.planet_panel_open);
        assert!(state.validate_state());
    }
    
    #[test]
    fn test_clear_selection() {
        let mut state = UIState::new();
        state.select_planet(Some(42));
        
        state.clear_selection();
        assert_eq!(state.selected_planet, None);
        assert_eq!(state.selected_ship, None);
        assert_eq!(state.selected_building_type, None);
        assert!(!state.panels.planet_panel_open);
        assert!(!state.panels.ship_panel_open);
        assert!(state.validate_state());
    }
    
    #[test]
    fn test_ui_scale_bounds() {
        let mut state = UIState::new();
        
        state.set_ui_scale(0.1);  // Below minimum
        assert_eq!(state.get_ui_scale(), 0.5);
        
        state.set_ui_scale(5.0);  // Above maximum
        assert_eq!(state.get_ui_scale(), 2.0);
        
        state.set_ui_scale(1.5);  // Valid value
        assert_eq!(state.get_ui_scale(), 1.5);
    }
    
    #[test]
    fn test_toggle_functions() {
        let mut state = UIState::new();
        
        let initial_pause = state.paused;
        state.toggle_pause();
        assert_ne!(state.paused, initial_pause);
        
        let initial_orbits = state.display.show_orbits;
        state.toggle_orbits();
        assert_ne!(state.display.show_orbits, initial_orbits);
        
        let initial_paths = state.display.show_ship_paths;
        state.toggle_ship_paths();
        assert_ne!(state.display.show_ship_paths, initial_paths);
        
        assert!(state.validate_state());
    }
    
    #[test]
    fn test_editing_state() {
        let mut state = UIState::new();
        
        assert!(!state.is_editing());
        
        state.start_resource_transfer_edit(Some(42));
        assert!(state.is_editing());
        
        state.clear_temp_state();
        assert!(!state.is_editing());
        assert!(state.validate_state());
    }
    
    #[test]
    fn test_state_validation() {
        let mut state = UIState::new();
        
        // Valid initial state
        assert!(state.validate_state());
        
        // Valid planet selection
        state.select_planet(Some(42));
        assert!(state.validate_state());
        
        // Invalid state: panel open without selection
        state.selected_planet = None;
        assert!(!state.validate_state());
        
        // Fix state
        state.panels.planet_panel_open = false;
        assert!(state.validate_state());
    }
    
    #[test]
    fn test_panel_state_consistency() {
        let mut state = UIState::new();
        
        // Select planet should open planet panel and close others
        state.select_planet(Some(42));
        assert!(state.panels.planet_panel_open);
        assert!(!state.panels.ship_panel_open);
        assert!(!state.panels.build_menu_open);
        
        // Select ship should open ship panel and close others
        state.select_ship(Some(123));
        assert!(!state.panels.planet_panel_open);
        assert!(state.panels.ship_panel_open);
        assert!(!state.panels.build_menu_open);
        
        // Clear selection should close entity panels
        state.clear_selection();
        assert!(!state.panels.planet_panel_open);
        assert!(!state.panels.ship_panel_open);
        assert!(!state.panels.build_menu_open);
        // But resource panel should remain open
        assert!(state.panels.resource_panel_open);
    }
    
    #[test]
    fn test_ui_scale_bounds() {
        let mut state = UIState::new();
        
        // Test bounds enforcement
        state.set_ui_scale(0.1);  // Below minimum
        assert_eq!(state.get_ui_scale(), 0.5);
        
        state.set_ui_scale(5.0);  // Above maximum  
        assert_eq!(state.get_ui_scale(), 2.0);
        
        state.set_ui_scale(1.5);  // Valid value
        assert_eq!(state.get_ui_scale(), 1.5);
        
        assert!(state.validate_state());
    }
}