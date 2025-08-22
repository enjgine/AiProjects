// src/ui_v2/views/entity_view.rs
//! Generic entity view for displaying game objects

use super::{View, BaseView};
use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, ViewData, Layout};
use crate::ui_v2::components::{UIComponent, Button};
use crate::ui_v2::adapters::EntityAdapter;
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;

/// Generic view for displaying any entity type
pub struct EntityView<T> {
    base: BaseView,
    entity_data: Option<T>,
    adapter: Box<dyn EntityAdapter<T>>,
    scroll_offset: f32,
    content_height: f32,
}

impl<T> EntityView<T> {
    pub fn new(title: String, adapter: Box<dyn EntityAdapter<T>>) -> Self {
        Self {
            base: BaseView::new(title),
            entity_data: None,
            adapter,
            scroll_offset: 0.0,
            content_height: 0.0,
        }
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn get_layout(&self) -> &Layout {
        &self.base.layout
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.base.layout = layout;
    }

    pub fn set_entity(&mut self, entity: T) {
        self.entity_data = Some(entity);
        self.refresh_content();
    }

    pub fn get_entity(&self) -> Option<&T> {
        self.entity_data.as_ref()
    }

    fn refresh_content(&mut self) {
        // Clear existing components
        self.base.components.clear();

        if let Some(entity) = &self.entity_data {
            let content_area = self.base.get_content_area();
            let mut y_offset = 0.0;
            let line_height = 25.0;

            // Get entity fields from adapter
            let fields = self.adapter.get_display_fields(entity);
            
            for (label, value) in fields {
                // Create label-value pairs (could be enhanced with actual label components)
                y_offset += line_height;
            }

            // Get entity actions from adapter
            let actions = self.adapter.get_actions(entity);
            
            for (action_name, command) in actions {
                let button = Button::new(action_name)
                    .with_click_command(command)
                    .with_layout(Layout::new(
                        content_area.x + 10.0,
                        content_area.y + y_offset,
                        120.0,
                        30.0
                    ));
                
                self.base.add_component(Box::new(button));
                y_offset += 35.0;
            }

            self.content_height = y_offset;
        }
    }

    fn render_entity_content(&self, context: &RenderContext) -> ComponentResult {
        if let Some(entity) = &self.entity_data {
            let content_area = self.base.get_content_area();
            let mut y_pos = content_area.y - self.scroll_offset;
            let line_height = 25.0;

            // Render entity fields
            let fields = self.adapter.get_display_fields(entity);
            
            for (label, value) in fields {
                if y_pos >= content_area.y - line_height && 
                   y_pos <= content_area.y + content_area.h {
                    
                    // Render label
                    draw_text(
                        &format!("{}:", label),
                        content_area.x + 10.0,
                        y_pos + line_height - 5.0,
                        context.font_size * 0.9,
                        context.theme.text_color
                    );
                    
                    // Render value
                    draw_text(
                        &value,
                        content_area.x + 120.0,
                        y_pos + line_height - 5.0,
                        context.font_size * 0.9,
                        context.theme.secondary_text_color
                    );
                }
                
                y_pos += line_height;
            }
        }

        Ok(None)
    }

    fn handle_scroll(&mut self, delta: f32) {
        let content_area = self.base.get_content_area();
        let max_scroll = (self.content_height - content_area.h).max(0.0);
        
        self.scroll_offset = (self.scroll_offset - delta * 30.0).clamp(0.0, max_scroll);
    }
}

impl<T> View for EntityView<T> 
where 
    T: Clone + 'static 
{
    fn render(&mut self, context: &RenderContext) -> ComponentResult {
        // Render base view (background, title, border)
        self.base.render_base(context)?;
        
        // Render entity-specific content
        self.render_entity_content(context)?;
        
        // Render components (buttons, etc.)
        self.base.render_components(context)?;

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        // Handle scrolling
        if let InputEvent::Scroll { x, y, delta } = input {
            let mouse_pos = Vec2::new(*x, *y);
            if self.base.contains_point(mouse_pos) {
                self.handle_scroll(*delta);
                return Ok(None);
            }
        }

        // Handle component input
        self.base.handle_input_base(input)
    }

    fn update(&mut self, delta_time: f32) -> ComponentResult {
        self.base.update_components(delta_time)
    }

    fn update_data(&mut self, data: ViewData) -> ComponentResult {
        // Extract entity data based on type
        // This would need to be implemented per entity type
        // For now, just refresh if we have data
        if self.entity_data.is_some() {
            self.refresh_content();
        }
        Ok(None)
    }

    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.base.visible = visible;
    }

    fn refresh(&mut self) -> ComponentResult {
        self.refresh_content();
        Ok(None)
    }

    fn get_view_type(&self) -> &'static str {
        "EntityView"
    }
}