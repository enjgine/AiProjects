// src/ui_v2/views/data_view.rs
//! Data-focused view for displaying lists and tables

use super::{View, BaseView};
use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, ViewData, Layout};
use crate::ui_v2::components::{UIComponent, Button, Dropdown};
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;

/// View for displaying tabular or list data
pub struct DataView {
    base: BaseView,
    columns: Vec<ColumnDefinition>,
    rows: Vec<Vec<String>>,
    selected_row: Option<usize>,
    scroll_offset: f32,
    row_height: f32,
    header_height: f32,
    sortable: bool,
    sort_column: Option<usize>,
    sort_ascending: bool,
}

#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    pub title: String,
    pub width: f32,
    pub alignment: TextAlign,
    pub sortable: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl DataView {
    pub fn new(title: String) -> Self {
        Self {
            base: BaseView::new(title),
            columns: Vec::new(),
            rows: Vec::new(),
            selected_row: None,
            scroll_offset: 0.0,
            row_height: 25.0,
            header_height: 30.0,
            sortable: true,
            sort_column: None,
            sort_ascending: true,
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

    pub fn with_columns(mut self, columns: Vec<ColumnDefinition>) -> Self {
        self.columns = columns;
        self
    }

    pub fn set_data(&mut self, rows: Vec<Vec<String>>) {
        self.rows = rows;
        self.selected_row = None;
        self.apply_sort();
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
        self.apply_sort();
    }

    pub fn get_selected_row(&self) -> Option<&Vec<String>> {
        self.selected_row.and_then(|idx| self.rows.get(idx))
    }

    pub fn set_sortable(&mut self, sortable: bool) {
        self.sortable = sortable;
    }

    fn apply_sort(&mut self) {
        if let Some(col_idx) = self.sort_column {
            if col_idx < self.columns.len() {
                self.rows.sort_by(|a, b| {
                    let empty_string = String::new();
                    let a_val = a.get(col_idx).unwrap_or(&empty_string);
                    let b_val = b.get(col_idx).unwrap_or(&empty_string);
                    
                    let cmp = a_val.cmp(b_val);
                    if self.sort_ascending { cmp } else { cmp.reverse() }
                });
            }
        }
    }

    fn render_header(&self, context: &RenderContext, content_area: Rect) -> ComponentResult {
        if self.columns.is_empty() {
            return Ok(None);
        }

        let header_y = content_area.y;
        let mut x_offset = content_area.x;

        // Header background
        draw_rectangle(
            content_area.x, header_y, content_area.w, self.header_height,
            context.theme.highlighted(context.theme.panel_background)
        );

        // Column headers
        for (col_idx, column) in self.columns.iter().enumerate() {
            let col_rect = Rect::new(x_offset, header_y, column.width, self.header_height);
            
            // Header border
            draw_rectangle_lines(
                col_rect.x, col_rect.y, col_rect.w, col_rect.h,
                1.0, context.theme.border_color
            );

            // Header text
            let text_x = match column.alignment {
                TextAlign::Left => col_rect.x + 5.0,
                TextAlign::Center => col_rect.x + (col_rect.w - 
                    measure_text(&column.title, None, context.font_size as u16, context.scale_factor).width) / 2.0,
                TextAlign::Right => col_rect.x + col_rect.w - 
                    measure_text(&column.title, None, context.font_size as u16, context.scale_factor).width - 5.0,
            };

            draw_text(
                &column.title,
                text_x,
                col_rect.y + col_rect.h - 8.0,
                context.font_size * 0.9,
                context.theme.text_color
            );

            // Sort indicator
            if self.sortable && self.sort_column == Some(col_idx) {
                let arrow = if self.sort_ascending { "▲" } else { "▼" };
                draw_text(
                    arrow,
                    col_rect.x + col_rect.w - 15.0,
                    col_rect.y + col_rect.h - 8.0,
                    context.font_size * 0.7,
                    context.theme.accent_color
                );
            }

            x_offset += column.width;
        }

        Ok(None)
    }

    fn render_rows(&self, context: &RenderContext, content_area: Rect) -> ComponentResult {
        if self.columns.is_empty() || self.rows.is_empty() {
            return Ok(None);
        }

        let data_start_y = content_area.y + self.header_height;
        let data_height = content_area.h - self.header_height;
        let visible_rows = (data_height / self.row_height) as usize + 2;
        let start_row = (self.scroll_offset / self.row_height) as usize;

        for (row_idx, row) in self.rows.iter()
            .enumerate()
            .skip(start_row)
            .take(visible_rows) {
            
            let row_y = data_start_y + (row_idx as f32 * self.row_height) - self.scroll_offset;
            
            if row_y < data_start_y - self.row_height || row_y > data_start_y + data_height {
                continue;
            }

            // Row background
            let is_selected = self.selected_row == Some(row_idx);
            let row_color = if is_selected {
                context.theme.primary_color
            } else if row_idx % 2 == 0 {
                context.theme.panel_background
            } else {
                context.theme.dimmed(context.theme.panel_background)
            };

            draw_rectangle(
                content_area.x, row_y, content_area.w, self.row_height,
                row_color
            );

            // Row data
            let mut x_offset = content_area.x;
            for (col_idx, column) in self.columns.iter().enumerate() {
                let empty_string = String::new();
                let cell_value = row.get(col_idx).unwrap_or(&empty_string);
                let col_rect = Rect::new(x_offset, row_y, column.width, self.row_height);

                let text_x = match column.alignment {
                    TextAlign::Left => col_rect.x + 5.0,
                    TextAlign::Center => col_rect.x + (col_rect.w - 
                        measure_text(cell_value, None, context.font_size as u16, context.scale_factor).width) / 2.0,
                    TextAlign::Right => col_rect.x + col_rect.w - 
                        measure_text(cell_value, None, context.font_size as u16, context.scale_factor).width - 5.0,
                };

                draw_text(
                    cell_value,
                    text_x,
                    col_rect.y + col_rect.h - 5.0,
                    context.font_size * 0.85,
                    if is_selected { context.theme.highlighted_text_color } else { context.theme.text_color }
                );

                x_offset += column.width;
            }
        }

        Ok(None)
    }

    fn handle_click(&mut self, mouse_pos: Vec2) -> ComponentResult {
        let content_area = self.base.get_content_area();
        
        if !content_area.contains(mouse_pos) {
            return Ok(None);
        }

        // Check header click for sorting
        if mouse_pos.y >= content_area.y && mouse_pos.y <= content_area.y + self.header_height {
            if self.sortable {
                let mut x_offset = content_area.x;
                for (col_idx, column) in self.columns.iter().enumerate() {
                    if mouse_pos.x >= x_offset && mouse_pos.x <= x_offset + column.width {
                        if self.sort_column == Some(col_idx) {
                            self.sort_ascending = !self.sort_ascending;
                        } else {
                            self.sort_column = Some(col_idx);
                            self.sort_ascending = true;
                        }
                        self.apply_sort();
                        break;
                    }
                    x_offset += column.width;
                }
            }
            return Ok(None);
        }

        // Check row click for selection
        let data_start_y = content_area.y + self.header_height;
        if mouse_pos.y >= data_start_y {
            let relative_y = mouse_pos.y - data_start_y + self.scroll_offset;
            let row_idx = (relative_y / self.row_height) as usize;
            
            if row_idx < self.rows.len() {
                self.selected_row = Some(row_idx);
            }
        }

        Ok(None)
    }

    fn handle_scroll(&mut self, delta: f32) {
        let max_scroll = ((self.rows.len() as f32 * self.row_height) - 
            (self.base.get_content_area().h - self.header_height)).max(0.0);
        
        self.scroll_offset = (self.scroll_offset - delta * 30.0).clamp(0.0, max_scroll);
    }
}

impl View for DataView {
    fn render(&mut self, context: &RenderContext) -> ComponentResult {
        self.base.render_base(context)?;
        
        let content_area = self.base.get_content_area();
        self.render_header(context, content_area)?;
        self.render_rows(context, content_area)?;
        
        self.base.render_components(context)?;

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        match input {
            InputEvent::MouseClick { x, y, button } => {
                if *button == MouseButton::Left {
                    let mouse_pos = Vec2::new(*x, *y);
                    if self.base.contains_point(mouse_pos) {
                        return self.handle_click(mouse_pos);
                    }
                }
            }
            InputEvent::Scroll { x, y, delta } => {
                let mouse_pos = Vec2::new(*x, *y);
                if self.base.contains_point(mouse_pos) {
                    self.handle_scroll(*delta);
                    return Ok(None);
                }
            }
            _ => {}
        }

        self.base.handle_input_base(input)
    }

    fn update(&mut self, delta_time: f32) -> ComponentResult {
        self.base.update_components(delta_time)
    }

    fn update_data(&mut self, _data: ViewData) -> ComponentResult {
        // Could update table data based on ViewData
        Ok(None)
    }

    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.base.visible = visible;
    }

    fn refresh(&mut self) -> ComponentResult {
        self.apply_sort();
        Ok(None)
    }

    fn get_view_type(&self) -> &'static str {
        "DataView"
    }
}

impl Default for DataView {
    fn default() -> Self {
        Self::new(String::new())
    }
}