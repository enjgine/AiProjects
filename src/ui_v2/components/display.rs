// src/ui_v2/components/display.rs
//! Display-only components for presenting information

use super::base_component::{UIComponent, BaseComponent, ComponentState};
use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, Layout};
use macroquad::prelude::*;

/// Simple text label component
pub struct Label {
    base: BaseComponent,
    text: String,
    alignment: TextAlignment,
    word_wrap: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Label {
    pub fn new(text: String) -> Self {
        Self {
            base: BaseComponent::new(),
            text,
            alignment: TextAlignment::Left,
            word_wrap: false,
        }
    }

    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}

impl UIComponent<()> for Label {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        let rect = self.base.state.layout.get_rect();
        let text_color = if self.base.state.enabled {
            context.theme.text_color
        } else {
            context.theme.dimmed(context.theme.text_color)
        };

        // Simple text rendering (could be enhanced with word wrap)
        let text_size = measure_text(&self.text, None, context.font_size as u16, context.scale_factor);
        
        let text_x = match self.alignment {
            TextAlignment::Left => rect.x,
            TextAlignment::Center => rect.x + (rect.w - text_size.width) / 2.0,
            TextAlignment::Right => rect.x + rect.w - text_size.width,
        };

        let text_y = rect.y + (rect.h + text_size.height) / 2.0;

        draw_text(&self.text, text_x, text_y, context.font_size, text_color);

        Ok(None)
    }

    fn handle_input(&mut self, _input: &InputEvent) -> ComponentResult {
        Ok(None) // Labels don't handle input
    }

    fn get_bounds(&self) -> Rect {
        self.base.state.layout.get_rect()
    }

    fn set_position(&mut self, position: Vec2) {
        self.base.state.layout.position = position;
    }

    fn set_size(&mut self, size: Vec2) {
        self.base.state.layout.size = size;
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

/// Progress bar component
pub struct ProgressBar {
    base: BaseComponent,
    value: f32,
    max_value: f32,
    show_text: bool,
    format_string: String,
}

impl ProgressBar {
    pub fn new(max_value: f32) -> Self {
        Self {
            base: BaseComponent::new(),
            value: 0.0,
            max_value,
            show_text: true,
            format_string: "{:.1}%".to_string(),
        }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, self.max_value);
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, self.max_value);
    }

    pub fn get_progress(&self) -> f32 {
        if self.max_value > 0.0 {
            self.value / self.max_value
        } else {
            0.0
        }
    }
}

impl UIComponent<()> for ProgressBar {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        let rect = self.base.state.layout.get_rect();
        let progress = self.get_progress();

        // Background
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, context.theme.dimmed(context.theme.background_color));
        
        // Progress fill
        let fill_width = rect.w * progress;
        if fill_width > 0.0 {
            draw_rectangle(rect.x, rect.y, fill_width, rect.h, context.theme.primary_color);
        }

        // Border
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, context.theme.border_width, context.theme.border_color);

        // Text
        if self.show_text {
            let percentage = progress * 100.0;
            let text = format!("{:.1}%", percentage);
            let text_size = measure_text(&text, None, context.font_size as u16, context.scale_factor);
            let text_x = rect.x + (rect.w - text_size.width) / 2.0;
            let text_y = rect.y + (rect.h + text_size.height) / 2.0;

            draw_text(&text, text_x, text_y, context.font_size * 0.9, context.theme.text_color);
        }

        Ok(None)
    }

    fn handle_input(&mut self, _input: &InputEvent) -> ComponentResult {
        Ok(None) // Progress bars don't handle input
    }

    fn get_bounds(&self) -> Rect {
        self.base.state.layout.get_rect()
    }

    fn set_position(&mut self, position: Vec2) {
        self.base.state.layout.position = position;
    }

    fn set_size(&mut self, size: Vec2) {
        self.base.state.layout.size = size;
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

// Placeholder structs for other display components
pub struct DataTable {
    base: BaseComponent,
}

impl DataTable {
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(),
        }
    }
}

pub struct ItemList {
    base: BaseComponent,
}

impl ItemList {
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(),
        }
    }
}

// TODO: Implement full UIComponent traits for DataTable and ItemList