// src/ui_v2/components/layout.rs
//! Layout and container components for organizing UI elements

use super::base_component::{UIComponent, BaseComponent, ComponentState};
use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, Layout};
use macroquad::prelude::*;

/// Generic container for layout management
pub struct Container {
    base: BaseComponent,
    children: Vec<Box<dyn UIComponent<()>>>,
    layout_type: LayoutType,
    spacing: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutType {
    Vertical,
    Horizontal,
    Grid(usize), // columns
    Free,        // absolute positioning
}

impl Container {
    pub fn new(layout_type: LayoutType) -> Self {
        Self {
            base: BaseComponent::new(),
            children: Vec::new(),
            layout_type,
            spacing: 5.0,
        }
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn add_child(&mut self, child: Box<dyn UIComponent<()>>) {
        self.children.push(child);
        self.update_child_layouts();
    }

    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    fn update_child_layouts(&mut self) {
        let content_area = self.get_content_area();
        
        match self.layout_type {
            LayoutType::Vertical => {
                let child_height = if !self.children.is_empty() {
                    (content_area.h - (self.children.len() as f32 - 1.0) * self.spacing) / self.children.len() as f32
                } else {
                    0.0
                };

                for (i, child) in self.children.iter_mut().enumerate() {
                    let y = content_area.y + i as f32 * (child_height + self.spacing);
                    child.set_position(Vec2::new(content_area.x, y));
                    child.set_size(Vec2::new(content_area.w, child_height));
                }
            }
            LayoutType::Horizontal => {
                let child_width = if !self.children.is_empty() {
                    (content_area.w - (self.children.len() as f32 - 1.0) * self.spacing) / self.children.len() as f32
                } else {
                    0.0
                };

                for (i, child) in self.children.iter_mut().enumerate() {
                    let x = content_area.x + i as f32 * (child_width + self.spacing);
                    child.set_position(Vec2::new(x, content_area.y));
                    child.set_size(Vec2::new(child_width, content_area.h));
                }
            }
            LayoutType::Grid(columns) => {
                if columns > 0 && !self.children.is_empty() {
                    let rows = (self.children.len() + columns - 1) / columns;
                    let cell_width = (content_area.w - (columns as f32 - 1.0) * self.spacing) / columns as f32;
                    let cell_height = (content_area.h - (rows as f32 - 1.0) * self.spacing) / rows as f32;

                    for (i, child) in self.children.iter_mut().enumerate() {
                        let col = i % columns;
                        let row = i / columns;
                        let x = content_area.x + col as f32 * (cell_width + self.spacing);
                        let y = content_area.y + row as f32 * (cell_height + self.spacing);
                        
                        child.set_position(Vec2::new(x, y));
                        child.set_size(Vec2::new(cell_width, cell_height));
                    }
                }
            }
            LayoutType::Free => {
                // Children maintain their own positioning
            }
        }
    }

    fn get_content_area(&self) -> Rect {
        let rect = self.base.state.layout.get_rect();
        Rect::new(
            rect.x + self.base.state.layout.padding,
            rect.y + self.base.state.layout.padding,
            rect.w - self.base.state.layout.padding * 2.0,
            rect.h - self.base.state.layout.padding * 2.0
        )
    }
}

impl UIComponent<()> for Container {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        // Render background if needed
        self.base.render_background(context);

        // Render children
        let unit_data = ();
        for child in &mut self.children {
            child.render(&unit_data, context)?;
        }

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        if !self.base.state.enabled || !self.base.state.visible {
            return Ok(None);
        }

        // Handle child input (reverse order for top-most first)
        for child in self.children.iter_mut().rev() {
            if let Ok(Some(command)) = child.handle_input(input) {
                return Ok(Some(command));
            }
        }

        Ok(None)
    }

    fn update(&mut self, delta_time: f32) -> ComponentResult {
        for child in &mut self.children {
            child.update(delta_time)?;
        }
        Ok(None)
    }

    fn get_bounds(&self) -> Rect {
        self.base.state.layout.get_rect()
    }

    fn set_position(&mut self, position: Vec2) {
        self.base.state.layout.position = position;
        self.update_child_layouts();
    }

    fn set_size(&mut self, size: Vec2) {
        self.base.state.layout.size = size;
        self.update_child_layouts();
    }

    fn is_visible(&self) -> bool {
        self.base.state.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.base.state.visible = visible;
    }

    fn get_state(&self) -> ComponentState {
        self.base.state.clone()
    }
}

// Placeholder structs for other layout components
pub struct TabContainer {
    base: BaseComponent,
}

impl TabContainer {
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(),
        }
    }
}

pub struct Splitter {
    base: BaseComponent,
}

impl Splitter {
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(),
        }
    }
}

// TODO: Implement full UIComponent traits for TabContainer and Splitter