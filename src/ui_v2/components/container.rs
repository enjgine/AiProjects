// src/ui_v2/components/container.rs
//! Container components for organizing and grouping UI elements

use super::base_component::{UIComponent, BaseComponent, ComponentState, Stateful};
use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, Layout};
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;
// Vec2 is already available from macroquad::prelude::*

/// Generic panel container for grouping related UI elements
pub struct Panel {
    base: BaseComponent,
    title: String,
    children: Vec<Box<dyn UIComponent<()>>>,
    collapsible: bool,
    collapsed: bool,
    show_border: bool,
    background_override: Option<Color>,
}

impl Panel {
    pub fn new(title: String) -> Self {
        Self {
            base: BaseComponent::new(),
            title,
            children: Vec::new(),
            collapsible: false,
            collapsed: false,
            show_border: true,
            background_override: None,
        }
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.base = self.base.with_layout(layout);
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.background_override = Some(color);
        self
    }

    pub fn borderless(mut self) -> Self {
        self.show_border = false;
        self
    }

    pub fn get_layout(&self) -> &Layout {
        self.base.get_layout()
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.base.set_layout(layout);
    }

    pub fn add_child(&mut self, child: Box<dyn UIComponent<()>>) {
        self.children.push(child);
    }

    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    pub fn toggle_collapse(&mut self) {
        if self.collapsible {
            self.collapsed = !self.collapsed;
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    fn get_title_height(&self) -> f32 {
        if self.title.is_empty() { 0.0 } else { 30.0 }
    }

    fn get_content_area(&self) -> Rect {
        let rect = self.base.state.layout.get_rect();
        let title_height = self.get_title_height();
        
        Rect::new(
            rect.x + self.base.state.layout.padding,
            rect.y + title_height + self.base.state.layout.padding,
            rect.w - self.base.state.layout.padding * 2.0,
            rect.h - title_height - self.base.state.layout.padding * 2.0
        )
    }

    fn render_title_bar(&self, context: &RenderContext) -> ComponentResult {
        if self.title.is_empty() {
            return Ok(None);
        }

        let rect = self.base.state.layout.get_rect();
        let title_height = self.get_title_height();

        // Title background
        draw_rectangle(
            rect.x, rect.y, rect.w, title_height,
            context.theme.primary_color
        );

        // Title text
        let text_size = measure_text(&self.title, None, context.font_size as u16, context.scale_factor);
        let text_x = rect.x + 10.0;
        let text_y = rect.y + (title_height + text_size.height) / 2.0;

        draw_text(&self.title, text_x, text_y, context.font_size, context.theme.text_color);

        // Collapse indicator
        if self.collapsible {
            let indicator = if self.collapsed { "+" } else { "-" };
            let indicator_x = rect.x + rect.w - 20.0;
            let indicator_y = rect.y + (title_height + text_size.height) / 2.0;

            draw_text(indicator, indicator_x, indicator_y, context.font_size, context.theme.text_color);
        }

        Ok(None)
    }

    fn render_content(&mut self, context: &RenderContext) -> ComponentResult {
        if self.collapsed {
            return Ok(None);
        }

        let unit_data = ();
        for child in &mut self.children {
            child.render(&unit_data, context)?;
        }

        Ok(None)
    }

    fn handle_title_click(&mut self, mouse_pos: Vec2) -> bool {
        if !self.collapsible || self.title.is_empty() {
            return false;
        }

        let rect = self.base.state.layout.get_rect();
        let title_height = self.get_title_height();
        let title_rect = Rect::new(rect.x, rect.y, rect.w, title_height);

        if title_rect.contains(mouse_pos) {
            self.toggle_collapse();
            return true;
        }

        false
    }
}

impl UIComponent<()> for Panel {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        self.base.update_hover_state(context.mouse_position);
        let rect = self.base.state.layout.get_rect();

        // Render background
        let bg_color = self.background_override.unwrap_or(context.theme.panel_background);
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg_color);

        // Render border
        if self.show_border {
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
        }

        // Render title bar
        self.render_title_bar(context)?;

        // Render content
        self.render_content(context)?;

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        if !self.base.state.enabled || !self.base.state.visible {
            return Ok(None);
        }

        // Handle title bar clicks for collapsible panels
        if let InputEvent::MouseClick { x, y, button } = input {
            if *button == MouseButton::Left {
                let mouse_pos = Vec2::new(*x, *y);
                if self.handle_title_click(mouse_pos) {
                    return Ok(None);
                }
            }
        }

        // Handle child input (reverse order for top-most first)
        if !self.collapsed {
            for child in self.children.iter_mut().rev() {
                if let Ok(Some(command)) = child.handle_input(input) {
                    return Ok(Some(command));
                }
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

impl Stateful for Panel {
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

/// Generic list view for displaying collections of items
pub struct ListView<T> {
    base: BaseComponent,
    items: Vec<T>,
    item_renderer: Box<dyn Fn(&T, usize, Rect, &RenderContext) -> ComponentResult>,
    selected_index: Option<usize>,
    scroll_offset: f32,
    item_height: f32,
    show_selection: bool,
    selectable: bool,
    on_selection: Option<Box<dyn Fn(&T, usize) -> PlayerCommand>>,
}

impl<T> ListView<T> {
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(),
            items: Vec::new(),
            item_renderer: Box::new(|_, _, _, _| Ok(None)),
            selected_index: None,
            scroll_offset: 0.0,
            item_height: 25.0,
            show_selection: true,
            selectable: true,
            on_selection: None,
        }
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

    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.items = items;
        self
    }

    pub fn with_item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    pub fn with_item_renderer<F>(mut self, renderer: F) -> Self
    where
        F: Fn(&T, usize, Rect, &RenderContext) -> ComponentResult + 'static,
    {
        self.item_renderer = Box::new(renderer);
        self
    }

    pub fn with_selection_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&T, usize) -> PlayerCommand + 'static,
    {
        self.on_selection = Some(Box::new(handler));
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.selected_index = None;
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_index = None;
        self.scroll_offset = 0.0;
    }

    pub fn get_selected(&self) -> Option<&T> {
        self.selected_index.and_then(|idx| self.items.get(idx))
    }

    pub fn set_selected_index(&mut self, index: Option<usize>) {
        if index.map_or(true, |i| i < self.items.len()) {
            self.selected_index = index;
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        let content_area = self.get_content_area();
        let total_height = self.items.len() as f32 * self.item_height;
        let max_scroll = (total_height - content_area.h).max(0.0);
        
        self.scroll_offset = (self.scroll_offset - delta * 30.0).clamp(0.0, max_scroll);
    }

    fn handle_click(&mut self, mouse_pos: Vec2) -> ComponentResult {
        if !self.selectable {
            return Ok(None);
        }

        let content_area = self.get_content_area();
        if !content_area.contains(mouse_pos) {
            return Ok(None);
        }

        let relative_y = mouse_pos.y - content_area.y + self.scroll_offset;
        let item_index = (relative_y / self.item_height) as usize;

        if item_index < self.items.len() {
            self.selected_index = Some(item_index);

            if let Some(handler) = &self.on_selection {
                if let Some(item) = self.items.get(item_index) {
                    return Ok(Some(handler(item, item_index)));
                }
            }
        }

        Ok(None)
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

    fn render_items(&self, context: &RenderContext) -> ComponentResult {
        let content_area = self.get_content_area();
        let visible_items = (content_area.h / self.item_height) as usize + 2;
        let start_item = (self.scroll_offset / self.item_height) as usize;

        for (i, item) in self.items.iter()
            .enumerate()
            .skip(start_item)
            .take(visible_items) {
            
            let item_y = content_area.y + (i as f32 * self.item_height) - self.scroll_offset;
            
            if item_y < content_area.y - self.item_height || item_y > content_area.y + content_area.h {
                continue;
            }

            let item_rect = Rect::new(content_area.x, item_y, content_area.w, self.item_height);

            // Render selection background
            if self.show_selection && self.selected_index == Some(i) {
                draw_rectangle(
                    item_rect.x, item_rect.y, item_rect.w, item_rect.h,
                    context.theme.primary_color
                );
            } else if i % 2 == 0 {
                draw_rectangle(
                    item_rect.x, item_rect.y, item_rect.w, item_rect.h,
                    context.theme.dimmed(context.theme.panel_background)
                );
            }

            // Render item using custom renderer
            (self.item_renderer)(item, i, item_rect, context)?;
        }

        Ok(None)
    }
}

impl<T> UIComponent<()> for ListView<T> {
    fn render(&mut self, _data: &(), context: &RenderContext) -> ComponentResult {
        if !self.base.state.visible {
            return Ok(None);
        }

        self.base.update_hover_state(context.mouse_position);
        
        // Render background
        self.base.render_background(context);

        // Render items
        self.render_items(context)?;

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
                    return self.handle_click(mouse_pos);
                }
            }
            InputEvent::Scroll { x, y, delta } => {
                let mouse_pos = Vec2::new(*x, *y);
                if self.base.is_mouse_over(mouse_pos) {
                    self.handle_scroll(*delta);
                    return Ok(None);
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

impl<T> Default for ListView<T> {
    fn default() -> Self {
        Self::new()
    }
}