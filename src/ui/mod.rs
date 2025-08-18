// src/ui/mod.rs
//! UI module for the Stellar Dominion game.
//!
//! This module provides a complete UI system that follows the EventBus architecture:
//! - UIRenderer: Main UI system that only generates PlayerCommand events
//! - InputHandler: Processes raw input and converts to game commands
//! - Panels: Specialized UI components for different game information
//!
//! All UI components follow strict EventBus compliance:
//! - No direct access to game state (read-only GameState reference only)
//! - All state changes via PlayerCommand events
//! - Pure rendering and input processing only

// Core UI modules
pub mod renderer;
pub mod input_handler;
pub mod panels;

// Future modules (planned architecture)
// These will be implemented when needed:
// - camera: View transformation and viewport management
// - ui_state: UI-specific state management separate from game state

// Re-export main components for easy access
pub use renderer::UIRenderer;
pub use input_handler::InputHandler;
pub use panels::{PlanetPanel, ShipPanel, ResourcePanel, Panel};

// Re-export utility modules for external use
pub use ui_config::validate_ui_config;
pub use ui_utils::{point_in_rect, center_text_in_rect, draw_button, validate_rect_bounds};

// UI-specific types and constants
use crate::core::types::{GameResult, GameError};

/// UI configuration constants and validation
pub mod ui_config {
    use super::{GameResult, GameError};
    
    pub const PANEL_BACKGROUND_ALPHA: f32 = 0.8;
    pub const BUTTON_HEIGHT: f32 = 25.0;
    pub const BUTTON_PADDING: f32 = 10.0;
    pub const PANEL_MARGIN: f32 = 10.0;
    pub const MIN_ZOOM: f32 = 0.1;
    pub const MAX_ZOOM: f32 = 5.0;
    pub const CAMERA_SPEED_BASE: f32 = 5.0;
    pub const FONT_SIZE_DEFAULT: f32 = 14.0;
    pub const FONT_SIZE_MIN: f32 = 8.0;
    pub const FONT_SIZE_MAX: f32 = 72.0;
    
    /// Validates UI configuration values at runtime
    pub fn validate_ui_config() -> GameResult<()> {
        if PANEL_BACKGROUND_ALPHA < 0.0 || PANEL_BACKGROUND_ALPHA > 1.0 {
            return Err(GameError::InvalidOperation(
                "Panel background alpha must be between 0.0 and 1.0".to_string()
            ));
        }
        
        if BUTTON_HEIGHT <= 0.0 || BUTTON_PADDING < 0.0 || PANEL_MARGIN < 0.0 {
            return Err(GameError::InvalidOperation(
                "UI dimensions must be positive".to_string()
            ));
        }
        
        if MIN_ZOOM <= 0.0 || MAX_ZOOM <= MIN_ZOOM {
            return Err(GameError::InvalidOperation(
                "Invalid zoom configuration".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Common UI utility functions with proper error handling
pub mod ui_utils {
    use super::*;
    use super::ui_config::*;
    use macroquad::prelude::*;
    
    /// Check if a point is within a rectangle
    /// 
    /// # Arguments
    /// * `x`, `y` - Point coordinates to test
    /// * `rect_x`, `rect_y` - Rectangle top-left corner
    /// * `width`, `height` - Rectangle dimensions (must be non-negative)
    /// 
    /// # Returns
    /// `GameResult<bool>` indicating if point is within rectangle
    pub fn point_in_rect(x: f32, y: f32, rect_x: f32, rect_y: f32, width: f32, height: f32) -> GameResult<bool> {
        if width < 0.0 || height < 0.0 {
            return Err(GameError::InvalidOperation(
                "Rectangle dimensions cannot be negative".to_string()
            ));
        }
        
        Ok(x >= rect_x && x <= rect_x + width && y >= rect_y && y <= rect_y + height)
    }
    
    /// Calculate text position for centering within a rectangle
    /// 
    /// # Arguments
    /// * `text` - Text to center
    /// * `rect_x`, `rect_y`, `rect_w`, `rect_h` - Rectangle bounds
    /// * `font_size` - Font size (must be within valid range)
    /// 
    /// # Returns
    /// `GameResult<(f32, f32)>` with text position coordinates
    pub fn center_text_in_rect(
        text: &str, 
        rect_x: f32, 
        rect_y: f32, 
        rect_w: f32, 
        rect_h: f32, 
        font_size: f32
    ) -> GameResult<(f32, f32)> {
        if text.is_empty() {
            return Err(GameError::InvalidOperation("Cannot center empty text".to_string()));
        }
        
        if font_size < FONT_SIZE_MIN || font_size > FONT_SIZE_MAX {
            return Err(GameError::InvalidOperation(
                format!("Font size {} outside valid range ({}-{})", font_size, FONT_SIZE_MIN, FONT_SIZE_MAX)
            ));
        }
        
        if rect_w <= 0.0 || rect_h <= 0.0 {
            return Err(GameError::InvalidOperation(
                "Rectangle dimensions must be positive for text centering".to_string()
            ));
        }
        
        let text_dims = measure_text(text, None, font_size as u16, 1.0);
        Ok((
            rect_x + (rect_w - text_dims.width) / 2.0,
            rect_y + (rect_h + text_dims.height) / 2.0
        ))
    }
    
    /// Draw a styled button and return if clicked
    /// 
    /// # Arguments
    /// * `x`, `y`, `w`, `h` - Button bounds
    /// * `text` - Button label text
    /// * `enabled` - Whether button is interactive
    /// 
    /// # Returns
    /// `GameResult<bool>` indicating if button was clicked
    pub fn draw_button(x: f32, y: f32, w: f32, h: f32, text: &str, enabled: bool) -> GameResult<bool> {
        if w <= 0.0 || h <= 0.0 {
            return Err(GameError::InvalidOperation(
                "Button dimensions must be positive".to_string()
            ));
        }
        
        if text.is_empty() {
            return Err(GameError::InvalidOperation("Button text cannot be empty".to_string()));
        }
        
        let (mouse_x, mouse_y) = mouse_position();
        let hovered = point_in_rect(mouse_x, mouse_y, x, y, w, h)? && enabled;
        let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);
        
        // Use constants instead of magic numbers
        let color = if !enabled {
            Color::new(0.1, 0.1, 0.1, 1.0)
        } else if hovered {
            Color::new(0.3, 0.3, 0.4, 1.0)
        } else {
            Color::new(0.2, 0.2, 0.3, 1.0)
        };
        
        draw_rectangle(x, y, w, h, color);
        draw_rectangle_lines(x, y, w, h, 1.0, if enabled { WHITE } else { GRAY });
        
        let text_color = if enabled { WHITE } else { GRAY };
        let (text_x, text_y) = center_text_in_rect(text, x, y, w, h, FONT_SIZE_DEFAULT)?;
        draw_text(text, text_x, text_y, FONT_SIZE_DEFAULT, text_color);
        
        Ok(clicked)
    }
    
    /// Validate rectangle bounds for UI operations
    /// 
    /// # Arguments
    /// * `x`, `y` - Position coordinates
    /// * `w`, `h` - Dimensions
    /// 
    /// # Returns
    /// `GameResult<()>` if bounds are valid
    pub fn validate_rect_bounds(x: f32, y: f32, w: f32, h: f32) -> GameResult<()> {
        if w <= 0.0 || h <= 0.0 {
            return Err(GameError::InvalidOperation(
                "Rectangle dimensions must be positive".to_string()
            ));
        }
        
        if x.is_nan() || y.is_nan() || w.is_nan() || h.is_nan() {
            return Err(GameError::InvalidOperation(
                "Rectangle coordinates cannot be NaN".to_string()
            ));
        }
        
        if x.is_infinite() || y.is_infinite() || w.is_infinite() || h.is_infinite() {
            return Err(GameError::InvalidOperation(
                "Rectangle coordinates cannot be infinite".to_string()
            ));
        }
        
        Ok(())
    }
}