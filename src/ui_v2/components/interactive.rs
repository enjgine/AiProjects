// src/ui_v2/components/interactive.rs
//! Interactive UI components (buttons, dropdowns, inputs)

use super::base_component::{UIComponent, BaseComponent, ComponentState, Themeable, Stateful};
use crate::ui_v2::core::{RenderContext, ComponentResult, ComponentError, InputEvent, Layout};
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;
// Vec2 is already available from macroquad::prelude::*

/// Generic button component
pub struct Button {
    base: BaseComponent,
    text: String,
    on_click: Option<PlayerCommand>,
    pressed: bool,
}

impl Button {
    pub fn new(text: String) -> Self {
        Self {
            base: BaseComponent::new(),
            text,
            on_click: None,
            pressed: false,
        }
    }

    pub fn with_click_command(mut self, command: PlayerCommand) -> Self {
        self.on_click = Some(command);
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn get_layout(&self) -> &Layout {
        self.base.get_layout()
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.base.set_layout(layout);
    }

    pub fn set_click_command(&mut self, command: PlayerCommand) {
        self.on_click = Some(command);
    }

    fn is_clicked(&self, input: &InputEvent) -> bool {
        if let InputEvent::MouseClick { x, y, button } = input {
            if *button == MouseButton::Left {
                let mouse_pos = Vec2::new(*x, *y);
                return self.base.is_mouse_over(mouse_pos);
            }
        }
        false
    }
}

impl UIComponent<()> for Button {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        // Update hover state
        self.base.update_hover_state(context.mouse_position);

        // Render background
        let rect = self.base.state.layout.get_rect();
        let bg_color = if self.pressed {
            context.theme.dimmed(context.theme.primary_color)
        } else if self.base.state.hovered {
            context.theme.highlighted(context.theme.primary_color)
        } else {
            context.theme.primary_color
        };

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);
        draw_rectangle_lines(
            rect.x, rect.y, rect.w, rect.h,
            context.theme.border_width,
            context.theme.border_color
        );

        // Render text
        let text_color = if self.base.state.enabled {
            context.theme.text_color
        } else {
            context.theme.dimmed(context.theme.text_color)
        };

        let text_size = measure_text(&self.text, None, context.font_size as u16, context.scale_factor);
        let text_x = rect.x + (rect.w - text_size.width) / 2.0;
        let text_y = rect.y + (rect.h + text_size.height) / 2.0;

        draw_text(&self.text, text_x, text_y, context.font_size, text_color);

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        if !self.base.state.enabled || !self.base.state.visible {
            return Ok(None);
        }

        match input {
            InputEvent::MouseClick { .. } => {
                if self.is_clicked(input) {
                    self.pressed = true;
                    return Ok(self.on_click.clone());
                }
            }
            InputEvent::MouseRelease { .. } => {
                self.pressed = false;
            }
            _ => {}
        }

        Ok(None)
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

impl Stateful for Button {
    fn set_enabled(&mut self, enabled: bool) {
        self.base.state.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.base.state.enabled
    }

    fn set_focused(&mut self, focused: bool) {
        self.base.state.focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.base.state.focused
    }
}

/// Generic dropdown component
pub struct Dropdown<T> {
    base: BaseComponent,
    items: Vec<(T, String)>, // (value, display_text)
    selected_index: Option<usize>,
    expanded: bool,
    on_selection: Option<Box<dyn Fn(&T) -> PlayerCommand>>,
}

impl<T: Clone> Dropdown<T> {
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(),
            items: Vec::new(),
            selected_index: None,
            expanded: false,
            on_selection: None,
        }
    }

    pub fn with_items(mut self, items: Vec<(T, String)>) -> Self {
        self.items = items;
        self
    }

    pub fn with_selection_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&T) -> PlayerCommand + 'static,
    {
        self.on_selection = Some(Box::new(handler));
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        if index.map_or(true, |i| i < self.items.len()) {
            self.selected_index = index;
        }
    }

    pub fn get_layout(&self) -> &Layout {
        self.base.get_layout()
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.base.set_layout(layout);
    }

    pub fn set_items(&mut self, items: Vec<(T, String)>) {
        self.items = items;
    }

    pub fn get_selected(&self) -> Option<&T> {
        self.selected_index
            .and_then(|i| self.items.get(i))
            .map(|(item, _)| item)
    }

    fn get_display_text(&self) -> &str {
        self.selected_index
            .and_then(|i| self.items.get(i))
            .map(|(_, text)| text.as_str())
            .unwrap_or("Select...")
    }
}

impl<T: Clone> UIComponent<()> for Dropdown<T> {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        self.base.update_hover_state(context.mouse_position);

        let rect = self.base.state.layout.get_rect();
        
        // Main dropdown box
        let bg_color = if self.expanded {
            context.theme.highlighted(context.theme.secondary_color)
        } else if self.base.state.hovered {
            context.theme.highlighted(context.theme.background_color)
        } else {
            context.theme.background_color
        };

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);
        draw_rectangle_lines(
            rect.x, rect.y, rect.w, rect.h,
            context.theme.border_width,
            context.theme.border_color
        );

        // Display text
        let text = self.get_display_text();
        let text_color = if self.base.state.enabled {
            context.theme.text_color
        } else {
            context.theme.dimmed(context.theme.text_color)
        };

        draw_text(
            text,
            rect.x + 10.0,
            rect.y + rect.h - 5.0,
            context.font_size,
            text_color
        );

        // Dropdown arrow
        let arrow_text = if self.expanded { "▲" } else { "▼" };
        draw_text(
            arrow_text,
            rect.x + rect.w - 20.0,
            rect.y + rect.h - 5.0,
            context.font_size * 0.8,
            text_color
        );

        // Expanded dropdown items
        if self.expanded {
            let item_height = 25.0;
            let max_visible = 6;
            let visible_items = self.items.len().min(max_visible);
            let dropdown_height = visible_items as f32 * item_height;

            // Background for expanded area
            draw_rectangle(
                rect.x,
                rect.y + rect.h,
                rect.w,
                dropdown_height,
                context.theme.background_color
            );
            draw_rectangle_lines(
                rect.x,
                rect.y + rect.h,
                rect.w,
                dropdown_height,
                context.theme.border_width,
                context.theme.border_color
            );

            // Render items
            for (i, (_, display_text)) in self.items.iter().take(max_visible).enumerate() {
                let item_y = rect.y + rect.h + i as f32 * item_height;
                let is_selected = self.selected_index == Some(i);

                if is_selected {
                    draw_rectangle(
                        rect.x + 1.0,
                        item_y + 1.0,
                        rect.w - 2.0,
                        item_height - 2.0,
                        context.theme.primary_color
                    );
                }

                draw_text(
                    display_text,
                    rect.x + 10.0,
                    item_y + item_height - 8.0,
                    context.font_size * 0.9,
                    text_color
                );
            }
        }

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        if !self.base.state.enabled || !self.base.state.visible {
            return Ok(None);
        }

        if let InputEvent::MouseClick { x, y, button } = input {
            if *button == MouseButton::Left {
                let mouse_pos = Vec2::new(*x, *y);
                let rect = self.base.state.layout.get_rect();

                // Check main dropdown click
                if rect.contains(mouse_pos) {
                    self.expanded = !self.expanded;
                    return Ok(None);
                }

                // Check expanded items click
                if self.expanded {
                    let item_height = 25.0;
                    let dropdown_y = rect.y + rect.h;
                    
                    if mouse_pos.x >= rect.x && mouse_pos.x <= rect.x + rect.w &&
                       mouse_pos.y >= dropdown_y {
                        let item_index = ((mouse_pos.y - dropdown_y) / item_height) as usize;
                        if item_index < self.items.len() {
                            self.selected_index = Some(item_index);
                            self.expanded = false;
                            
                            if let Some(handler) = &self.on_selection {
                                if let Some((item, _)) = self.items.get(item_index) {
                                    return Ok(Some(handler(item)));
                                }
                            }
                        }
                    }
                } else if self.expanded {
                    // Click outside closes dropdown
                    self.expanded = false;
                }
            }
        }

        Ok(None)
    }

    fn get_bounds(&self) -> Rect {
        let mut rect = self.base.state.layout.get_rect();
        if self.expanded {
            let item_height = 25.0;
            let max_visible = 6;
            let visible_items = self.items.len().min(max_visible);
            rect.h += visible_items as f32 * item_height;
        }
        rect
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

impl<T: Clone> Default for Dropdown<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder implementations for other interactive components
pub struct Slider {
    base: BaseComponent,
    min_value: f32,
    max_value: f32,
    current_value: f32,
}

impl Slider {
    pub fn new(min: f32, max: f32, initial: f32) -> Self {
        Self {
            base: BaseComponent::new(),
            min_value: min,
            max_value: max,
            current_value: initial.clamp(min, max),
        }
    }
}

pub struct TextInput {
    base: BaseComponent,
    text: String,
    placeholder: String,
    cursor_position: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(),
            text: String::new(),
            placeholder: String::new(),
            cursor_position: 0,
        }
    }
}

impl UIComponent<()> for Slider {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        self.base.update_hover_state(context.mouse_position);
        let rect = self.base.state.layout.get_rect();

        // Render track
        let track_height = 4.0;
        let track_y = rect.y + (rect.h - track_height) / 2.0;
        draw_rectangle(rect.x, track_y, rect.w, track_height, context.theme.dimmed(context.theme.border_color));

        // Calculate thumb position
        let value_ratio = (self.current_value - self.min_value) / (self.max_value - self.min_value);
        let thumb_x = rect.x + value_ratio * (rect.w - 20.0);
        let thumb_size = 20.0;

        // Render thumb
        let thumb_color = if self.base.state.hovered {
            context.theme.highlighted(context.theme.primary_color)
        } else {
            context.theme.primary_color
        };

        draw_rectangle(thumb_x, rect.y + (rect.h - thumb_size) / 2.0, thumb_size, thumb_size, thumb_color);
        draw_rectangle_lines(
            thumb_x, rect.y + (rect.h - thumb_size) / 2.0, thumb_size, thumb_size,
            context.theme.border_width, context.theme.border_color
        );

        // Render value text
        let value_text = format!("{:.1}", self.current_value);
        draw_text(
            &value_text,
            rect.x + rect.w + 10.0,
            rect.y + rect.h - 5.0,
            context.font_size * 0.9,
            context.theme.text_color
        );

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        if !self.base.state.enabled || !self.base.state.visible {
            return Ok(None);
        }

        if let InputEvent::MouseClick { x, y, button } = input {
            if *button == MouseButton::Left {
                let mouse_pos = Vec2::new(*x, *y);
                if self.base.is_mouse_over(mouse_pos) {
                    let rect = self.base.state.layout.get_rect();
                    let relative_x = (*x - rect.x).clamp(0.0, rect.w);
                    let value_ratio = relative_x / rect.w;
                    self.current_value = self.min_value + value_ratio * (self.max_value - self.min_value);
                    self.current_value = self.current_value.clamp(self.min_value, self.max_value);
                }
            }
        }

        Ok(None)
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

impl Stateful for Slider {
    fn set_enabled(&mut self, enabled: bool) {
        self.base.state.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.base.state.enabled
    }

    fn set_focused(&mut self, focused: bool) {
        self.base.state.focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.base.state.focused
    }
}

impl Slider {
    pub fn get_value(&self) -> f32 {
        self.current_value
    }

    pub fn set_value(&mut self, value: f32) {
        self.current_value = value.clamp(self.min_value, self.max_value);
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn get_layout(&self) -> &Layout {
        self.base.get_layout()
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.base.set_layout(layout);
    }
}

impl UIComponent<()> for TextInput {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        self.base.update_hover_state(context.mouse_position);
        let rect = self.base.state.layout.get_rect();

        // Render background
        let bg_color = if self.base.state.focused {
            context.theme.highlighted(context.theme.background_color)
        } else if self.base.state.hovered {
            context.theme.background_color
        } else {
            context.theme.dimmed(context.theme.background_color)
        };

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);
        
        // Render border
        let border_color = if self.base.state.focused {
            context.theme.accent_color
        } else {
            context.theme.border_color
        };
        let border_width = if self.base.state.focused {
            context.theme.border_width + 1.0
        } else {
            context.theme.border_width
        };

        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, border_width, border_color);

        // Render text or placeholder
        let display_text = if self.text.is_empty() && !self.placeholder.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };

        let text_color = if self.text.is_empty() && !self.placeholder.is_empty() {
            context.theme.dimmed(context.theme.text_color)
        } else {
            context.theme.text_color
        };

        // Simple text rendering (could be enhanced with scrolling for long text)
        draw_text(
            display_text,
            rect.x + 5.0,
            rect.y + rect.h - 8.0,
            context.font_size,
            text_color
        );

        // Render cursor if focused
        if self.base.state.focused {
            let text_width = if self.cursor_position > 0 {
                let cursor_text = &self.text[..self.cursor_position.min(self.text.len())];
                measure_text(cursor_text, None, context.font_size as u16, context.scale_factor).width
            } else {
                0.0
            };

            draw_line(
                rect.x + 5.0 + text_width,
                rect.y + 5.0,
                rect.x + 5.0 + text_width,
                rect.y + rect.h - 5.0,
                2.0,
                context.theme.accent_color
            );
        }

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        if !self.base.state.enabled || !self.base.state.visible {
            return Ok(None);
        }

        match input {
            InputEvent::MouseClick { x, y, button } => {
                if *button == MouseButton::Left {
                    let mouse_pos = Vec2::new(*x, *y);
                    let was_focused = self.base.state.focused;
                    self.base.state.focused = self.base.is_mouse_over(mouse_pos);
                    
                    // Update cursor position based on click location
                    if self.base.state.focused && !was_focused {
                        // For simplicity, just move cursor to end
                        self.cursor_position = self.text.len();
                    }
                }
            }
            InputEvent::KeyPress { key } => {
                if self.base.state.focused {
                    match key {
                        KeyCode::Backspace => {
                            if self.cursor_position > 0 && !self.text.is_empty() {
                                self.text.remove(self.cursor_position - 1);
                                self.cursor_position -= 1;
                            }
                        }
                        KeyCode::Delete => {
                            if self.cursor_position < self.text.len() {
                                self.text.remove(self.cursor_position);
                            }
                        }
                        KeyCode::Left => {
                            if self.cursor_position > 0 {
                                self.cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if self.cursor_position < self.text.len() {
                                self.cursor_position += 1;
                            }
                        }
                        KeyCode::Home => {
                            self.cursor_position = 0;
                        }
                        KeyCode::End => {
                            self.cursor_position = self.text.len();
                        }
                        KeyCode::Enter => {
                            self.base.state.focused = false;
                            // Could emit a "submit" event here
                        }
                        KeyCode::Escape => {
                            self.base.state.focused = false;
                        }
                        _ => {
                            // Handle character input (simplified)
                            if let Some(ch) = key_to_char(*key) {
                                self.text.insert(self.cursor_position, ch);
                                self.cursor_position += 1;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(None)
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

impl Stateful for TextInput {
    fn set_enabled(&mut self, enabled: bool) {
        self.base.state.enabled = enabled;
    }

    fn is_enabled(&self) -> bool {
        self.base.state.enabled
    }

    fn set_focused(&mut self, focused: bool) {
        self.base.state.focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.base.state.focused
    }
}

impl TextInput {
    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = text.clone();
        self.cursor_position = text.len();
        self
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn get_layout(&self) -> &Layout {
        self.base.get_layout()
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.base.set_layout(layout);
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_position = self.text.len();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new(0.0, 100.0, 50.0)
    }
}

// Helper function to convert KeyCode to character (simplified)
fn key_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::A => Some('a'),
        KeyCode::B => Some('b'),
        KeyCode::C => Some('c'),
        KeyCode::D => Some('d'),
        KeyCode::E => Some('e'),
        KeyCode::F => Some('f'),
        KeyCode::G => Some('g'),
        KeyCode::H => Some('h'),
        KeyCode::I => Some('i'),
        KeyCode::J => Some('j'),
        KeyCode::K => Some('k'),
        KeyCode::L => Some('l'),
        KeyCode::M => Some('m'),
        KeyCode::N => Some('n'),
        KeyCode::O => Some('o'),
        KeyCode::P => Some('p'),
        KeyCode::Q => Some('q'),
        KeyCode::R => Some('r'),
        KeyCode::S => Some('s'),
        KeyCode::T => Some('t'),
        KeyCode::U => Some('u'),
        KeyCode::V => Some('v'),
        KeyCode::W => Some('w'),
        KeyCode::X => Some('x'),
        KeyCode::Y => Some('y'),
        KeyCode::Z => Some('z'),
        KeyCode::Key0 => Some('0'),
        KeyCode::Key1 => Some('1'),
        KeyCode::Key2 => Some('2'),
        KeyCode::Key3 => Some('3'),
        KeyCode::Key4 => Some('4'),
        KeyCode::Key5 => Some('5'),
        KeyCode::Key6 => Some('6'),
        KeyCode::Key7 => Some('7'),
        KeyCode::Key8 => Some('8'),
        KeyCode::Key9 => Some('9'),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some('-'),
        KeyCode::Equal => Some('='),
        KeyCode::Period => Some('.'),
        KeyCode::Comma => Some(','),
        _ => None,
    }
}