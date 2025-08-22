// src/ui_v2/core/ui_system.rs
//! Main UI system coordinator and entry point

use super::{
    RenderContext, Theme, ComponentResult, 
    ViewEvent, ViewId, ViewType
};
use super::view_controller::ViewController;
use super::input_controller::InputController;
use crate::core::events::PlayerCommand;
use macroquad::prelude::Vec2;
use macroquad::prelude::*;

/// Main UI system that coordinates all UI subsystems
pub struct UISystem {
    view_controller: ViewController,
    input_controller: InputController,
    theme: Theme,
    screen_dimensions: (f32, f32),
    scale_factor: f32,
    font_size: f32,
    enabled: bool,
}

impl UISystem {
    pub fn new() -> Self {
        Self {
            view_controller: ViewController::new(),
            input_controller: InputController::new(),
            theme: Theme::default(),
            screen_dimensions: (1024.0, 768.0),
            scale_factor: 1.0,
            font_size: 16.0,
            enabled: true,
        }
    }

    /// Initialize the UI system with screen dimensions
    pub fn initialize(&mut self, screen_width: f32, screen_height: f32) {
        self.screen_dimensions = (screen_width, screen_height);
        self.update_scale_factor();
    }

    /// Main update loop - processes input and updates views
    pub fn update(&mut self, delta_time: f32) -> Vec<PlayerCommand> {
        if !self.enabled {
            return Vec::new();
        }

        let mut commands = Vec::new();

        // Update screen dimensions if changed
        let current_screen = (screen_width(), screen_height());
        if current_screen != self.screen_dimensions {
            self.screen_dimensions = current_screen;
            self.update_scale_factor();
        }

        // Process input events
        let input_events = self.input_controller.process_input(delta_time);
        
        // Generate UI commands from input patterns
        let ui_commands = self.input_controller.generate_ui_commands(&input_events);
        commands.extend(ui_commands);

        // Handle input events through view controller
        for event in &input_events {
            if let Ok(Some(command)) = self.view_controller.handle_input(event) {
                commands.push(command);
            }
        }

        // Update all views
        if let Err(e) = self.view_controller.update_all(delta_time) {
            eprintln!("UI update error: {:?}", e);
        }

        commands
    }

    /// Render all UI components
    pub fn render(&mut self) {
        if !self.enabled {
            return;
        }

        let context = self.create_render_context();
        
        // Render all active views
        if let Err(e) = self.view_controller.render_all(&context) {
            eprintln!("UI render error: {:?}", e);
        }
    }
    
    /// Send a view event to the system
    pub fn send_view_event(&mut self, event: ViewEvent) {
        if let Err(e) = self.view_controller.handle_view_event(event) {
            eprintln!("View event error: {:?}", e);
        }
    }

    /// Handle view events from external systems
    pub fn handle_view_event(&mut self, event: ViewEvent) -> ComponentResult {
        self.view_controller.handle_view_event(event)
    }

    /// Create a view of the specified type
    pub fn create_view(&mut self, view_type: ViewType) -> ViewId {
        // For now, this is a placeholder - full implementation would
        // create the appropriate view based on the type
        match view_type {
            ViewType::PlanetPanel => {
                // Would create EntityView<Planet> with PlanetAdapter
                0 // Placeholder
            }
            ViewType::ShipPanel => {
                // Would create EntityView<Ship> with ShipAdapter
                0 // Placeholder
            }
            ViewType::MainMenu => {
                // Would create MenuView with appropriate configuration
                0 // Placeholder
            }
            ViewType::GameOptions => {
                // Would create DialogView with options form
                0 // Placeholder
            }
            ViewType::SaveLoad => {
                // Would create DialogView with save/load interface
                0 // Placeholder
            }
            ViewType::ResourcePanel => {
                // Would create DataView with resource display
                0 // Placeholder
            }
        }
    }

    /// Close a specific view
    pub fn close_view(&mut self, view_id: ViewId) {
        self.view_controller.close_view(view_id);
    }

    /// Toggle visibility of a view type
    pub fn toggle_view(&mut self, view_type: ViewType) -> ComponentResult {
        self.view_controller.handle_view_event(ViewEvent::ToggleView { view_type })
    }

    /// Check if a view type is currently active
    pub fn is_view_active(&self, view_type: ViewType) -> bool {
        self.view_controller.is_view_type_active(&view_type)
    }

    /// Set the UI theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Get current theme
    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    /// Enable or disable the entire UI system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if UI system is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get current mouse position
    pub fn get_mouse_position(&self) -> Vec2 {
        self.input_controller.get_mouse_position()
    }

    /// Check if any mouse button is currently pressed
    pub fn is_mouse_active(&self) -> bool {
        self.input_controller.is_any_mouse_button_down()
    }

    /// Close all views
    pub fn close_all_views(&mut self) {
        self.view_controller.close_all();
    }

    /// Get performance metrics for debugging
    pub fn get_metrics(&self) -> UIMetrics {
        UIMetrics {
            active_views: self.view_controller.get_active_views().len(),
            screen_dimensions: self.screen_dimensions,
            scale_factor: self.scale_factor,
            enabled: self.enabled,
        }
    }

    /// Create render context for current frame
    fn create_render_context(&self) -> RenderContext {
        RenderContext::new()
    }

    /// Update scale factor based on screen size
    fn update_scale_factor(&mut self) {
        // Scale UI based on screen height (768p = 1.0 scale)
        self.scale_factor = (self.screen_dimensions.1 / 768.0).max(0.5).min(2.0);
        self.font_size = 16.0 * self.scale_factor;
    }
}

impl Default for UISystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance and debugging metrics for the UI system
#[derive(Debug, Clone)]
pub struct UIMetrics {
    pub active_views: usize,
    pub screen_dimensions: (f32, f32),
    pub scale_factor: f32,
    pub enabled: bool,
}

/// UI System factory for creating common configurations
pub struct UISystemBuilder {
    theme: Option<Theme>,
    scale_factor: Option<f32>,
    font_size: Option<f32>,
}

impl UISystemBuilder {
    pub fn new() -> Self {
        Self {
            theme: None,
            scale_factor: None,
            font_size: None,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn with_scale_factor(mut self, scale: f32) -> Self {
        self.scale_factor = Some(scale);
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn build(self) -> UISystem {
        let mut system = UISystem::new();
        
        if let Some(theme) = self.theme {
            system.set_theme(theme);
        }
        
        if let Some(scale) = self.scale_factor {
            system.scale_factor = scale;
        }
        
        if let Some(font_size) = self.font_size {
            system.font_size = font_size;
        }
        
        system
    }
}

impl Default for UISystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}