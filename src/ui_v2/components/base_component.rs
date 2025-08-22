// src/ui_v2/components/base_component.rs
//! Base component trait and common functionality

use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, Layout};
use macroquad::prelude::*;
// Vec2 is already available from macroquad::prelude::*

/// Core trait that all UI components must implement
pub trait UIComponent<T> {
    /// Render the component with the given data and context
    fn render(&mut self, data: &T, context: &RenderContext) -> ComponentResult;
    
    /// Handle input events and return any resulting commands
    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult;
    
    /// Get the component's layout bounds
    fn get_bounds(&self) -> Rect;
    
    /// Set the component's position
    fn set_position(&mut self, position: Vec2);
    
    /// Set the component's size
    fn set_size(&mut self, size: Vec2);
    
    /// Check if the component is currently visible
    fn is_visible(&self) -> bool;
    
    /// Set component visibility
    fn set_visible(&mut self, visible: bool);
    
    /// Get component's current state for persistence/debugging
    fn get_state(&self) -> ComponentState;
    
    /// Update component from input events (optional, default implementation)
    fn update(&mut self, _delta_time: f32) -> ComponentResult {
        Ok(None)
    }
}

/// Generic component state for debugging and persistence
#[derive(Debug, Clone)]
pub struct ComponentState {
    pub layout: Layout,
    pub visible: bool,
    pub enabled: bool,
    pub focused: bool,
    pub hovered: bool,
    pub custom_data: std::collections::HashMap<String, String>,
}

impl Default for ComponentState {
    fn default() -> Self {
        Self {
            layout: Layout::default(),
            visible: true,
            enabled: true,
            focused: false,
            hovered: false,
            custom_data: std::collections::HashMap::new(),
        }
    }
}

/// Base implementation for components that provides common functionality
pub struct BaseComponent {
    pub state: ComponentState,
    pub id: Option<String>,
}

impl BaseComponent {
    pub fn new() -> Self {
        Self {
            state: ComponentState::default(),
            id: None,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.state.layout = layout;
        self
    }

    pub fn get_layout(&self) -> &Layout {
        &self.state.layout
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.state.layout = layout;
    }

    /// Check if mouse is over this component
    pub fn is_mouse_over(&self, mouse_pos: Vec2) -> bool {
        if !self.state.visible || !self.state.enabled {
            return false;
        }
        self.state.layout.contains_point(mouse_pos)
    }

    /// Update hover state based on mouse position
    pub fn update_hover_state(&mut self, mouse_pos: Vec2) {
        self.state.hovered = self.is_mouse_over(mouse_pos);
    }

    /// Get the component's visual bounds including padding
    pub fn get_visual_bounds(&self) -> Rect {
        let layout = &self.state.layout;
        let rect = layout.get_rect();
        Rect::new(
            rect.x - layout.padding,
            rect.y - layout.padding,
            rect.w + layout.padding * 2.0,
            rect.h + layout.padding * 2.0,
        )
    }

    /// Render component background with theme
    pub fn render_background(&self, context: &RenderContext) {
        if !self.state.visible {
            return;
        }

        let rect = self.get_visual_bounds();
        let color = if self.state.hovered {
            context.theme.highlighted(context.theme.background_color)
        } else {
            context.theme.background_color
        };

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
        
        if self.state.focused {
            draw_rectangle_lines(
                rect.x, rect.y, rect.w, rect.h,
                context.theme.border_width + 1.0,
                context.theme.accent_color
            );
        } else {
            draw_rectangle_lines(
                rect.x, rect.y, rect.w, rect.h,
                context.theme.border_width,
                context.theme.border_color
            );
        }
    }
}

impl Default for BaseComponent {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for components that can be styled with themes
pub trait Themeable {
    fn apply_theme(&mut self, theme: &crate::ui_v2::core::Theme);
}

/// Trait for components that can be enabled/disabled
pub trait Stateful {
    fn set_enabled(&mut self, enabled: bool);
    fn is_enabled(&self) -> bool;
    fn set_focused(&mut self, focused: bool);
    fn is_focused(&self) -> bool;
}