// src/ui_v2/core/view_controller.rs
//! View lifecycle and coordination management

use super::{ViewId, ViewEvent, ViewType, ViewData, ComponentResult};
use crate::ui_v2::views::View;
// PlayerCommand import removed - handled by individual views
use std::collections::HashMap;

/// Manages the lifecycle of all UI views
pub struct ViewController {
    active_views: HashMap<ViewId, Box<dyn View>>,
    view_stack: Vec<ViewId>, // Z-order for rendering and input
    next_view_id: ViewId,
    view_type_registry: HashMap<ViewType, ViewId>, // Track singleton views
}

impl ViewController {
    pub fn new() -> Self {
        Self {
            active_views: HashMap::new(),
            view_stack: Vec::new(),
            next_view_id: 0,
            view_type_registry: HashMap::new(),
        }
    }

    /// Create a new view and add it to the controller
    pub fn create_view(&mut self, view: Box<dyn View>, view_type: ViewType) -> ViewId {
        let view_id = self.next_view_id;
        self.next_view_id += 1;

        // For singleton views, close existing instance
        if let Some(existing_id) = self.view_type_registry.get(&view_type) {
            self.close_view(*existing_id);
        }

        self.active_views.insert(view_id, view);
        self.view_stack.push(view_id);
        self.view_type_registry.insert(view_type, view_id);

        view_id
    }

    /// Close and remove a view
    pub fn close_view(&mut self, view_id: ViewId) {
        if let Some(view) = self.active_views.remove(&view_id) {
            // Remove from stack
            self.view_stack.retain(|&id| id != view_id);
            
            // Remove from type registry
            self.view_type_registry.retain(|_, &mut id| id != view_id);
            
            // Let view clean up
            drop(view);
        }
    }

    /// Handle view events
    pub fn handle_view_event(&mut self, event: ViewEvent) -> ComponentResult {
        match event {
            ViewEvent::ShowEntity { entity_type, id } => {
                // Create or update entity view
                self.show_entity_view(entity_type, id)
            }
            ViewEvent::UpdateData { view_type, data } => {
                self.update_view_data_by_type(view_type, data)
            }
            ViewEvent::CloseView { view_id } => {
                self.close_view(view_id);
                Ok(None)
            }
            ViewEvent::ToggleView { view_type } => {
                self.toggle_view_type(view_type)
            }
            ViewEvent::RefreshView { view_id } => {
                self.refresh_view(view_id)
            }
            ViewEvent::ShowView { view_type } => {
                // Show view by type - would create/show view
                Ok(None)
            }
            ViewEvent::HideView { view_type } => {
                // Hide view by type - would hide view
                Ok(None)
            }
        }
    }

    /// Show an entity in the appropriate view
    fn show_entity_view(&mut self, entity_type: super::EntityType, _id: u32) -> ComponentResult {
        // For now, just track that we would create the appropriate view
        // Full implementation would create EntityView<T> with appropriate adapter
        match entity_type {
            super::EntityType::Planet => {
                // Would create EntityView<Planet> with PlanetAdapter
            }
            super::EntityType::Ship => {
                // Would create EntityView<Ship> with ShipAdapter
            }
            super::EntityType::Faction => {
                // Would create EntityView<Faction> with FactionAdapter
            }
            super::EntityType::Resource => {
                // Would create DataView with ResourceAdapter
            }
        }
        Ok(None)
    }

    /// Update data for a specific view by type
    fn update_view_data_by_type(&mut self, view_type: String, data: ViewData) -> ComponentResult {
        // For now, just find the first view matching the type
        for view in self.active_views.values_mut() {
            if view.get_view_type() == view_type {
                return view.update_data(data);
            }
        }
        Ok(None)
    }

    /// Toggle visibility of a view type
    fn toggle_view_type(&mut self, view_type: ViewType) -> ComponentResult {
        if let Some(&view_id) = self.view_type_registry.get(&view_type) {
            if let Some(view) = self.active_views.get_mut(&view_id) {
                let currently_visible = view.is_visible();
                view.set_visible(!currently_visible);
            }
        }
        Ok(None)
    }

    /// Refresh a view's content
    fn refresh_view(&mut self, view_id: ViewId) -> ComponentResult {
        if let Some(view) = self.active_views.get_mut(&view_id) {
            view.refresh()
        } else {
            Ok(None)
        }
    }

    /// Render all active views in stack order
    pub fn render_all(&mut self, context: &super::RenderContext) -> ComponentResult {
        for &view_id in &self.view_stack {
            if let Some(view) = self.active_views.get_mut(&view_id) {
                if view.is_visible() {
                    view.render(context)?;
                }
            }
        }
        Ok(None)
    }

    /// Handle input for all views (reverse stack order for top-most first)
    pub fn handle_input(&mut self, input: &super::InputEvent) -> ComponentResult {
        // Process input from top-most view down until one handles it
        for &view_id in self.view_stack.iter().rev() {
            if let Some(view) = self.active_views.get_mut(&view_id) {
                if view.is_visible() {
                    if let Ok(Some(command)) = view.handle_input(input) {
                        return Ok(Some(command));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Update all views
    pub fn update_all(&mut self, delta_time: f32) -> ComponentResult {
        for view in self.active_views.values_mut() {
            view.update(delta_time)?;
        }
        Ok(None)
    }

    /// Get all active view IDs
    pub fn get_active_views(&self) -> Vec<ViewId> {
        self.view_stack.clone()
    }

    /// Check if a view type is currently active
    pub fn is_view_type_active(&self, view_type: &ViewType) -> bool {
        self.view_type_registry.contains_key(view_type)
    }

    /// Bring a view to the front of the stack
    pub fn bring_to_front(&mut self, view_id: ViewId) {
        if let Some(pos) = self.view_stack.iter().position(|&id| id == view_id) {
            let view_id = self.view_stack.remove(pos);
            self.view_stack.push(view_id);
        }
    }

    /// Close all views
    pub fn close_all(&mut self) {
        self.active_views.clear();
        self.view_stack.clear();
        self.view_type_registry.clear();
    }
}

impl Default for ViewController {
    fn default() -> Self {
        Self::new()
    }
}