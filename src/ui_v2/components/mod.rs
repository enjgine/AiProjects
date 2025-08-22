// src/ui_v2/components/mod.rs
//! Reusable UI components
//! 
//! This module provides generic, composable UI components that can be used
//! to build any user interface. Components follow consistent patterns:
//! - Generic over data types where applicable
//! - Stateless where possible (state managed by views)
//! - Event-driven interactions
//! - Themeable appearance

pub mod base_component;
pub mod interactive;
pub mod container;
pub mod display;
pub mod layout;

// Re-export main component types
pub use base_component::{UIComponent, ComponentState};
pub use interactive::{Button, Dropdown, Slider, TextInput};
pub use container::{Panel, ListView};
pub use display::{Label, ProgressBar, DataTable, ItemList};
pub use layout::{Container, TabContainer, Splitter};

// Convenience type aliases
pub use base_component::UIComponent as Component;

/// Common component builder pattern
pub trait ComponentBuilder<T> {
    fn build(self) -> T;
    fn with_layout(self, layout: crate::ui_v2::core::Layout) -> Self;
    fn with_theme_override(self, theme: crate::ui_v2::core::Theme) -> Self;
}

/// Component validation helpers
pub mod validation {
    use super::*;
    use crate::ui_v2::core::{ComponentResult, ComponentError};

    pub fn validate_layout(layout: &crate::ui_v2::core::Layout) -> ComponentResult {
        if layout.size.x <= 0.0 || layout.size.y <= 0.0 {
            return Err(ComponentError::LayoutError("Component size must be positive".into()));
        }
        Ok(None)
    }

    pub fn validate_position(layout: &crate::ui_v2::core::Layout, screen_size: (f32, f32)) -> ComponentResult {
        if layout.position.x < 0.0 || layout.position.y < 0.0 ||
           layout.position.x >= screen_size.0 || layout.position.y >= screen_size.1 {
            return Err(ComponentError::LayoutError("Component position outside screen bounds".into()));
        }
        Ok(None)
    }
}