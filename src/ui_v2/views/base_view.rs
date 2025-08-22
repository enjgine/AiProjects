// src/ui_v2/views/base_view.rs
//! Base view implementation providing common functionality

use super::View;
use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, ViewData, Layout};
use crate::ui_v2::components::UIComponent;
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;

/// Base view state and common functionality
pub struct BaseView {
    pub layout: Layout,
    pub visible: bool,
    pub title: String,
    pub background_color: Option<Color>,
    pub components: Vec<Box<dyn UIComponent<()>>>,
}

impl BaseView {
    pub fn new(title: String) -> Self {
        Self {
            layout: Layout::default(),
            visible: true,
            title,
            background_color: None,
            components: Vec::new(),
        }
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn add_component(&mut self, component: Box<dyn UIComponent<()>>) {
        self.components.push(component);
    }

    /// Render the base view background and title
    pub fn render_base(&self, context: &RenderContext) -> ComponentResult {
        if !self.visible {
            return Ok(None);
        }

        let rect = self.layout.get_rect();
        
        // Render background
        let bg_color = self.background_color.unwrap_or(context.theme.panel_background);
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);
        
        // Render border
        draw_rectangle_lines(
            rect.x, rect.y, rect.w, rect.h,
            context.theme.border_width,
            context.theme.border_color
        );

        // Render title if present
        if !self.title.is_empty() {
            let title_height = 30.0;
            draw_rectangle(
                rect.x, rect.y, rect.w, title_height,
                context.theme.primary_color
            );
            
            let text_size = measure_text(&self.title, None, context.font_size as u16, context.scale_factor);
            let text_x = rect.x + (rect.w - text_size.width) / 2.0;
            let text_y = rect.y + (title_height + text_size.height) / 2.0;
            
            draw_text(&self.title, text_x, text_y, context.font_size, context.theme.text_color);
        }

        Ok(None)
    }

    /// Handle input for all components
    pub fn handle_input_base(&mut self, input: &InputEvent) -> ComponentResult {
        if !self.visible {
            return Ok(None);
        }

        // Process components in reverse order (top-most first)
        for component in self.components.iter_mut().rev() {
            if let Ok(Some(command)) = component.handle_input(input) {
                return Ok(Some(command));
            }
        }

        Ok(None)
    }

    /// Render all components
    pub fn render_components(&mut self, context: &RenderContext) -> ComponentResult {
        if !self.visible {
            return Ok(None);
        }

        let unit_data = ();
        for component in &mut self.components {
            component.render(&unit_data, context)?;
        }

        Ok(None)
    }

    /// Update all components
    pub fn update_components(&mut self, delta_time: f32) -> ComponentResult {
        for component in &mut self.components {
            component.update(delta_time)?;
        }
        Ok(None)
    }

    /// Get the content area (excluding title bar)
    pub fn get_content_area(&self) -> Rect {
        let rect = self.layout.get_rect();
        let title_height = if self.title.is_empty() { 0.0 } else { 30.0 };
        
        Rect::new(
            rect.x + self.layout.padding,
            rect.y + title_height + self.layout.padding,
            rect.w - self.layout.padding * 2.0,
            rect.h - title_height - self.layout.padding * 2.0
        )
    }

    /// Check if a point is within the view bounds
    pub fn contains_point(&self, point: Vec2) -> bool {
        if !self.visible {
            return false;
        }
        self.layout.contains_point(point)
    }
}

impl Default for BaseView {
    fn default() -> Self {
        Self::new(String::new())
    }
}