// src/ui/panels/resource_panel.rs
use crate::core::{GameResult, EventBus, GameError};
use crate::core::types::*;
use super::Panel;
use macroquad::prelude::*;
use std::collections::HashMap;

// Panel layout constants for better maintainability
const DEFAULT_PANEL_HEIGHT: f32 = 100.0;
const SMALL_SCREEN_PANEL_HEIGHT: f32 = 80.0;
const SMALL_SCREEN_THRESHOLD: f32 = 600.0;
const DEFAULT_MARGIN: f32 = 20.0;
const SMALL_SCREEN_MARGIN: f32 = 10.0;
const SMALL_SCREEN_WIDTH_THRESHOLD: f32 = 800.0;
const FONT_SIZE_THRESHOLD: f32 = 1024.0;
const PANEL_BOTTOM_OFFSET: f32 = 10.0;
const RESOURCE_COUNT: f32 = 6.0;
const FPS_WARNING_THRESHOLD: i32 = 30;
const FPS_MEDIUM_THRESHOLD: i32 = 50;

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
    // Cached FPS string to reduce allocations
    fps_display: String,
    last_fps_update_tick: u64,
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
        
        // Initialize resource display cache with empty strings
        let mut resource_display_cache = HashMap::new();
        resource_display_cache.insert("Minerals", String::with_capacity(32));
        resource_display_cache.insert("Food", String::with_capacity(32));
        resource_display_cache.insert("Energy", String::with_capacity(32));
        resource_display_cache.insert("Alloys", String::with_capacity(32));
        resource_display_cache.insert("Components", String::with_capacity(32));
        resource_display_cache.insert("Fuel", String::with_capacity(32));
        
        Self {
            // Initialize with safe default values - layout will be updated on first render
            panel_rect: Rect::new(DEFAULT_MARGIN, 0.0, 800.0, DEFAULT_PANEL_HEIGHT),
            visible: true,
            cached_empire_totals: ResourceBundle::default(),
            cached_tick: 0,
            tick_display: String::with_capacity(32),
            resource_display_cache,
            resource_colors,
            font_size_large: 16.0,
            font_size_small: 14.0,
            fps_display: String::with_capacity(16),
            last_fps_update_tick: 0,
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
    pub fn render(&mut self, empire_resources: &ResourceBundle, current_tick: u64, is_paused: bool, speed_multiplier: f32, _events: &mut EventBus) -> GameResult<()> {
        if !self.visible {
            return Ok(());
        }
        
        // Input validation - resources should already be validated by the caller,
        // but we validate here for defensive programming
        if let Err(validation_error) = empire_resources.validate_non_negative() {
            return Err(GameError::SystemError(
                format!("ResourcePanel received invalid resource data: {:?}", validation_error)
            ));
        }
        
        // Update panel layout responsively (must be done before drawing)
        self.update_panel_layout();
        
        // Update cached data
        self.update_cache(empire_resources, current_tick);
        
        // Draw enhanced panel background
        self.draw_panel_background();
        
        // Render empire totals
        self.render_empire_resources()?;
        
        // Render game info with performance monitoring
        self.render_game_info(current_tick, is_paused, speed_multiplier)?;
        
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
                self.tick_display.push_str(&format!("Tick: {:.1}k", current_tick as f32 / 1000.0));
            } else {
                self.tick_display.push_str(&format!("Tick: {}", current_tick));
            }
        }
    }

    /// Update panel layout responsively based on screen size
    /// Handles edge cases where screen dimensions might be invalid
    fn update_panel_layout(&mut self) {
        let screen_w = screen_width();
        let screen_h = screen_height();
        
        // Validate screen dimensions to prevent layout issues
        if screen_w <= 0.0 || screen_h <= 0.0 {
            return; // Keep current layout if screen dimensions are invalid
        }
        
        // Responsive panel sizing using constants
        let panel_height = if screen_h < SMALL_SCREEN_THRESHOLD { 
            SMALL_SCREEN_PANEL_HEIGHT 
        } else { 
            DEFAULT_PANEL_HEIGHT 
        };
        let margin = if screen_w < SMALL_SCREEN_WIDTH_THRESHOLD { 
            SMALL_SCREEN_MARGIN 
        } else { 
            DEFAULT_MARGIN 
        };
        
        self.panel_rect = Rect::new(
            margin,
            screen_h - panel_height - PANEL_BOTTOM_OFFSET,
            screen_w - (margin * 2.0),
            panel_height,
        );
        
        // Adjust font sizes based on screen size
        if screen_w < FONT_SIZE_THRESHOLD {
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

    /// Format resource value with appropriate units and separators
    fn format_resource_value(&self, value: i32) -> String {
        Self::format_resource_value_static(value)
    }
    
    /// Static version of format_resource_value for use in caching
    fn format_resource_value_static(value: i32) -> String {
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
        let spacing = available_width / RESOURCE_COUNT;
        
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
                // Fallback if cache is missing - this should rarely happen now
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
                let fallback_color = self.resource_colors.get(resource_name).copied().unwrap_or(WHITE);
                draw_text(&text, x_offset, resource_y, self.font_size_small, fallback_color);
            }
            x_offset += spacing;
        }
        
        Ok(())
    }
    
    fn render_game_info(&mut self, current_tick: u64, is_paused: bool, speed_multiplier: f32) -> GameResult<()> {
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
        
        // Add FPS counter for performance monitoring - update only periodically to reduce overhead
        if current_tick != self.last_fps_update_tick {
            self.last_fps_update_tick = current_tick;
            let fps = macroquad::time::get_fps();
            self.fps_display.clear();
            self.fps_display.push_str(&format!("FPS: {}", fps));
        }
        
        // Determine FPS color based on performance
        let fps_value = macroquad::time::get_fps();
        let fps_color = if fps_value < FPS_WARNING_THRESHOLD {
            Color::new(1.0, 0.3, 0.3, 1.0) // Red for poor performance
        } else if fps_value < FPS_MEDIUM_THRESHOLD {
            Color::new(1.0, 1.0, 0.3, 1.0) // Yellow for medium performance
        } else {
            Color::new(0.3, 1.0, 0.3, 1.0) // Green for good performance
        };
        
        draw_text(
            &self.fps_display,
            right_x,
            y + 45.0,
            self.font_size_small * 0.9,
            fps_color,
        );
        
        Ok(())
    }
    
    /// Invalidate cache when resource data changes externally
    /// This should be called when the UI system detects external resource changes
    pub fn invalidate_cache(&mut self) {
        self.cached_empire_totals = ResourceBundle::default();
        self.cached_tick = 0;
        self.last_fps_update_tick = 0;
        // Clear all cached display strings
        for cached_string in self.resource_display_cache.values_mut() {
            cached_string.clear();
        }
        self.fps_display.clear();
    }
    
    /// Check if panel contains a given point (useful for input handling)
    /// Returns false if panel is not visible to prevent interaction with hidden panels
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        if !self.visible {
            return false;
        }
        
        x >= self.panel_rect.x &&
        x <= self.panel_rect.x + self.panel_rect.w &&
        y >= self.panel_rect.y &&
        y <= self.panel_rect.y + self.panel_rect.h
    }
    
    /// Get current panel bounds for layout calculations
    /// Returns None if panel is not visible
    pub fn get_bounds(&self) -> Option<Rect> {
        if self.visible {
            Some(self.panel_rect)
        } else {
            None
        }
    }
    
    /// Update panel position manually (useful for custom layouts)
    /// Validates that the position is within reasonable screen bounds
    pub fn set_position(&mut self, x: f32, y: f32) -> GameResult<()> {
        let screen_w = screen_width();
        let screen_h = screen_height();
        
        if screen_w <= 0.0 || screen_h <= 0.0 {
            return Err(GameError::SystemError("Invalid screen dimensions for panel positioning".into()));
        }
        
        // Clamp position to keep panel visible
        let clamped_x = x.max(0.0).min(screen_w - self.panel_rect.w);
        let clamped_y = y.max(0.0).min(screen_h - self.panel_rect.h);
        
        self.panel_rect.x = clamped_x;
        self.panel_rect.y = clamped_y;
        
        Ok(())
    }
}