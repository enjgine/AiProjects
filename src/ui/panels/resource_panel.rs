// src/ui/panels/resource_panel.rs
use crate::core::{GameResult, EventBus};
use crate::core::types::*;
use super::Panel;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct ResourcePanel {
    panel_rect: Rect,
    visible: bool,
    cached_empire_totals: ResourceBundle,
    cached_tick: u64,
    tick_display: String,
    // Performance and UI enhancements
    resource_display_cache: HashMap<&'static str, String>,
    resource_colors: HashMap<&'static str, Color>,
    font_size_large: f32,
    font_size_small: f32,
}

impl Panel for ResourcePanel {
    fn new() -> Self {
        let mut resource_colors = HashMap::new();
        resource_colors.insert("Minerals", YELLOW);
        resource_colors.insert("Food", GREEN);
        resource_colors.insert("Energy", Color::new(0.3, 0.6, 1.0, 1.0)); // Light blue for better visibility
        resource_colors.insert("Alloys", Color::new(1.0, 0.5, 0.2, 1.0)); // Orange instead of red
        resource_colors.insert("Components", Color::new(0.8, 0.4, 1.0, 1.0)); // Purple
        resource_colors.insert("Fuel", Color::new(0.6, 0.6, 0.6, 1.0)); // Gray
        
        Self {
            panel_rect: Rect::new(10.0, screen_height() - 120.0, screen_width() - 20.0, 100.0),
            visible: true,
            cached_empire_totals: ResourceBundle::default(),
            cached_tick: 0,
            tick_display: String::with_capacity(32),
            resource_display_cache: HashMap::new(),
            resource_colors,
            font_size_large: 16.0,
            font_size_small: 14.0,
        }
    }

    fn show(&mut self) {
        self.visible = true;
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}

impl ResourcePanel {
    
    /// Render the resource panel with cached data
    /// This method should be called with empire totals and game info pre-calculated
    /// by the UIRenderer to maintain EventBus architecture compliance
    pub fn render(&mut self, empire_resources: &ResourceBundle, current_tick: u64, is_paused: bool, speed_multiplier: f32, events: &mut EventBus) -> GameResult<()> {
        if !self.visible {
            return Ok(());
        }
        
        // Validate input resources before display
        empire_resources.validate_non_negative()
            .map_err(|_| GameError::SystemError("Invalid resource data received for display".into()))?;
        
        // Update cached data
        self.update_cache(empire_resources, current_tick);
        
        // Update panel layout responsively
        self.update_panel_layout();
        
        // Draw enhanced panel background
        self.draw_panel_background();
        
        // Render empire totals
        self.render_empire_resources()?;
        
        // Render game info with performance monitoring
        self.render_game_info(is_paused, speed_multiplier)?;
        
        Ok(())
    }
    
    /// Update cached data to avoid string allocations every frame
    fn update_cache(&mut self, empire_resources: &ResourceBundle, current_tick: u64) {
        // Only update if resources actually changed
        if self.cached_empire_totals != *empire_resources {
            self.cached_empire_totals = *empire_resources;
            
            // Update resource display strings with formatting
            let resources = [
                ("Minerals", empire_resources.minerals),
                ("Food", empire_resources.food),
                ("Energy", empire_resources.energy),
                ("Alloys", empire_resources.alloys),
                ("Components", empire_resources.components),
                ("Fuel", empire_resources.fuel),
            ];
            
            for (name, value) in &resources {
                if let Some(cached_string) = self.resource_display_cache.get_mut(name) {
                    cached_string.clear();
                    cached_string.push_str(name);
                    cached_string.push_str(": ");
                    let formatted_value = Self::format_resource_value_static(*value);
                    cached_string.push_str(&formatted_value);
                }
            }
        }
        
        if self.cached_tick != current_tick {
            self.cached_tick = current_tick;
            self.tick_display.clear();
            if current_tick > 10000 {
                self.tick_display.push_str(&format!("Tick: {:.1}k", current_tick as f64 / 1000.0));
            } else {
                self.tick_display.push_str(&format!("Tick: {}", current_tick));
            }
        }
    }

    /// Update panel layout to stay at bottom of screen
    fn update_panel_layout(&mut self) {
        self.panel_rect.y = screen_height() - 120.0;
        self.panel_rect.w = screen_width() - 20.0;
    }

    /// Draw the panel background and border
    fn draw_panel_background(&self) {
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            Color::new(0.1, 0.1, 0.2, 0.8),
        );

        draw_rectangle_lines(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            2.0,
            WHITE,
        );
    }

    /// Format resource value with appropriate units and separators
    fn format_resource_value(&self, value: i32) -> String {
        if value >= 1_000_000 {
            format!("{:.1}M", value as f32 / 1_000_000.0)
        } else if value >= 1_000 {
            format!("{:.1}K", value as f32 / 1_000.0)
        } else {
            value.to_string()
        }
    }
    
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    
    fn render_empire_resources(&self) -> GameResult<()> {
        let start_x = self.panel_rect.x + 20.0;
        let y = self.panel_rect.y + 20.0;
        
        draw_text("Empire Resources:", start_x, y, self.font_size_large, WHITE);
        
        // Calculate responsive spacing
        let available_width = self.panel_rect.w - 40.0; // Margins on both sides
        let resource_count = 6.0; // Including fuel
        let spacing = available_width / resource_count;
        
        let resource_y = y + 25.0;
        let mut x_offset = start_x;
        
        // Resource order and their cached display strings
        let resources = [
            "Minerals", "Food", "Energy", "Alloys", "Components", "Fuel"
        ];
        
        for &resource_name in &resources {
            if let (Some(cached_text), Some(&color)) = (
                self.resource_display_cache.get(resource_name),
                self.resource_colors.get(resource_name)
            ) {
                draw_text(cached_text, x_offset, resource_y, self.font_size_small, color);
            } else {
                // Fallback if cache is missing
                let value = match resource_name {
                    "Minerals" => self.cached_empire_totals.minerals,
                    "Food" => self.cached_empire_totals.food,
                    "Energy" => self.cached_empire_totals.energy,
                    "Alloys" => self.cached_empire_totals.alloys,
                    "Components" => self.cached_empire_totals.components,
                    "Fuel" => self.cached_empire_totals.fuel,
                    _ => 0,
                };
                let text = format!("{}: {}", resource_name, self.format_resource_value(value));
                draw_text(&text, x_offset, resource_y, self.font_size_small, WHITE);
            }
            x_offset += spacing;
        }
        
        Ok(())
    }
    
    fn render_game_info(&self, is_paused: bool, speed_multiplier: f32) -> GameResult<()> {
        let right_x = self.panel_rect.x + self.panel_rect.w - 200.0;
        let y = self.panel_rect.y + 20.0;
        
        // Game speed and pause status with color coding
        if is_paused {
            draw_text("PAUSED", right_x, y, self.font_size_large, Color::new(1.0, 0.3, 0.3, 1.0));
        } else {
            let speed_text = format!("Speed: {:.1}x", speed_multiplier);
            let speed_color = if speed_multiplier > 1.0 {
                Color::new(0.3, 1.0, 0.3, 1.0) // Green for fast speed
            } else {
                WHITE // White for normal speed
            };
            draw_text(&speed_text, right_x, y, self.font_size_large, speed_color);
        }
        
        // Use cached tick display
        draw_text(&self.tick_display, right_x, y + 25.0, self.font_size_small, LIGHTGRAY);
        
        // Add FPS counter for performance monitoring
        let fps = macroquad::time::get_fps();
        let fps_color = if fps < 30 {
            Color::new(1.0, 0.3, 0.3, 1.0) // Red for poor performance
        } else if fps < 50 {
            Color::new(1.0, 1.0, 0.3, 1.0) // Yellow for medium performance
        } else {
            Color::new(0.3, 1.0, 0.3, 1.0) // Green for good performance
        };
        
        draw_text(
            &format!("FPS: {}", fps),
            right_x,
            y + 45.0,
            self.font_size_small * 0.9,
            fps_color,
        );
        
        Ok(())
    }
}
    
    /// Update panel layout responsively based on screen size
    fn update_panel_layout(&mut self) {
        let screen_w = screen_width();
        let screen_h = screen_height();
        
        // Responsive panel sizing
        let panel_height = if screen_h < 600.0 { 80.0 } else { 100.0 };
        let margin = if screen_w < 800.0 { 10.0 } else { 20.0 };
        
        self.panel_rect = Rect::new(
            margin,
            screen_h - panel_height - 10.0,
            screen_w - (margin * 2.0),
            panel_height,
        );
        
        // Adjust font sizes based on screen size
        if screen_w < 1024.0 {
            self.font_size_large = 14.0;
            self.font_size_small = 12.0;
        } else {
            self.font_size_large = 16.0;
            self.font_size_small = 14.0;
        }
    }
    
    /// Draw enhanced panel background with subtle visual improvements
    fn draw_panel_background(&self) {
        // Main background with slight transparency
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            Color::new(0.08, 0.08, 0.15, 0.92),
        );
        
        // Subtle top highlight
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            2.0,
            Color::new(0.2, 0.2, 0.4, 0.6),
        );
        
        // Clean border
        draw_rectangle_lines(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            1.0,
            Color::new(0.4, 0.4, 0.6, 0.8),
        );
    }
    
    /// Format resource values with appropriate units (K, M)
    fn format_resource_value(&self, value: i32) -> String {
        if value >= 1_000_000 {
            format!("{:.1}M", value as f64 / 1_000_000.0)
        } else if value >= 1_000 {
            format!("{:.1}K", value as f64 / 1_000.0)
        } else {
            value.to_string()
        }
    }
    
    /// Invalidate cache when resource data changes externally
    pub fn invalidate_cache(&mut self) {
        self.cached_empire_totals = ResourceBundle::default();
        self.cached_tick = 0;
        // Clear all cached display strings
        for cached_string in self.resource_display_cache.values_mut() {
            cached_string.clear();
        }
    }
    
    /// Check if panel contains a given point (useful for input handling)
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.panel_rect.x &&
        x <= self.panel_rect.x + self.panel_rect.w &&
        y >= self.panel_rect.y &&
        y <= self.panel_rect.y + self.panel_rect.h
    }
