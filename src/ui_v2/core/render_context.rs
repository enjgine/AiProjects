// src/ui_v2/core/render_context.rs
//! Rendering context and utilities for UI components

// Minimal imports for render context
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;

/// Result type for component operations
pub type ComponentResult = Result<Option<PlayerCommand>, ComponentError>;

/// Errors that can occur in component operations
#[derive(Debug, Clone)]
pub enum ComponentError {
    InvalidState(String),
    RenderError(String),
    InputError(String),
    LayoutError(String),
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentError::InvalidState(msg) => write!(f, "Invalid component state: {}", msg),
            ComponentError::RenderError(msg) => write!(f, "Render error: {}", msg),
            ComponentError::InputError(msg) => write!(f, "Input error: {}", msg),
            ComponentError::LayoutError(msg) => write!(f, "Layout error: {}", msg),
        }
    }
}

impl std::error::Error for ComponentError {}

/// Context provided to components during rendering
pub struct RenderContext {
    pub screen_width: f32,
    pub screen_height: f32,
    pub delta_time: f32,
    pub mouse_position: Vec2,
    pub theme: Theme,
    pub font_size: f32,
    pub scale_factor: f32,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            screen_width: screen_width(),
            screen_height: screen_height(),
            delta_time: 0.016, // ~60 FPS default
            mouse_position: Vec2::from(mouse_position()),
            theme: Theme::default(),
            font_size: 14.0,
            scale_factor: 1.0,
        }
    }

    pub fn update(&mut self) {
        self.screen_width = screen_width();
        self.screen_height = screen_height();
        self.mouse_position = Vec2::from(mouse_position());
    }

    /// Check if a point is within screen bounds
    pub fn is_on_screen(&self, point: Vec2) -> bool {
        point.x >= 0.0 && point.x <= self.screen_width &&
        point.y >= 0.0 && point.y <= self.screen_height
    }

    /// Get screen center point
    pub fn screen_center(&self) -> Vec2 {
        Vec2::new(self.screen_width / 2.0, self.screen_height / 2.0)
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Theme configuration for consistent UI appearance
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub background_color: Color,
    pub panel_background: Color,
    pub text_color: Color,
    pub secondary_text_color: Color,
    pub highlighted_text_color: Color,
    pub accent_color: Color,
    pub border_color: Color,
    pub success_color: Color,
    pub warning_color: Color,
    pub error_color: Color,
    pub panel_alpha: f32,
    pub border_width: f32,
    pub corner_radius: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_color: Color::new(0.2, 0.4, 0.8, 1.0),
            secondary_color: Color::new(0.3, 0.3, 0.4, 1.0),
            background_color: Color::new(0.1, 0.1, 0.2, 0.9),
            panel_background: Color::new(0.15, 0.15, 0.25, 0.95),
            text_color: WHITE,
            secondary_text_color: Color::new(0.8, 0.8, 0.8, 1.0),
            highlighted_text_color: Color::new(1.0, 1.0, 0.8, 1.0),
            accent_color: Color::new(0.8, 0.6, 0.2, 1.0),
            border_color: WHITE,
            success_color: GREEN,
            warning_color: YELLOW,
            error_color: RED,
            panel_alpha: 0.9,
            border_width: 2.0,
            corner_radius: 4.0,
        }
    }
}

impl Theme {
    /// Get a dimmed version of a color
    pub fn dimmed(&self, color: Color) -> Color {
        Color::new(color.r * 0.7, color.g * 0.7, color.b * 0.7, color.a)
    }

    /// Get a highlighted version of a color  
    pub fn highlighted(&self, color: Color) -> Color {
        Color::new(
            (color.r * 1.3).min(1.0),
            (color.g * 1.3).min(1.0), 
            (color.b * 1.3).min(1.0),
            color.a
        )
    }
}