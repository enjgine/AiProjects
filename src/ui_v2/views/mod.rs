// src/ui_v2/views/mod.rs
//! View implementations for different UI patterns

pub mod base_view;
pub mod entity_view;
pub mod data_view;
pub mod dialog_view;

pub use base_view::*;
pub use entity_view::*;
pub use data_view::*;
pub use dialog_view::*;

use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, ViewData};
use crate::core::events::PlayerCommand;

/// Core trait that all views must implement
pub trait View {
    /// Render the view with the given context
    fn render(&mut self, context: &RenderContext) -> ComponentResult;
    
    /// Handle input events
    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult;
    
    /// Update view state
    fn update(&mut self, delta_time: f32) -> ComponentResult;
    
    /// Update view data
    fn update_data(&mut self, data: ViewData) -> ComponentResult;
    
    /// Check if view is visible
    fn is_visible(&self) -> bool;
    
    /// Set view visibility
    fn set_visible(&mut self, visible: bool);
    
    /// Refresh view content
    fn refresh(&mut self) -> ComponentResult;
    
    /// Get view's unique type identifier
    fn get_view_type(&self) -> &'static str;
}