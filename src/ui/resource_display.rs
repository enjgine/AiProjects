// src/ui/resource_display.rs
use crate::core::{GameResult, GameState};
use crate::core::types::*;
use macroquad::prelude::*;

/// Handles all resource display rendering including horizontal bars and side panels
pub struct ResourceDisplay {
    // Resource tracking for change indicators  
    resource_window_history: Option<(ResourceBundle, i32, u64)>, // (prev_resources, prev_population, tick)
    last_window_tick: u64,
    resource_changes: ResourceBundle,
    population_change: i32,
    
    // Cache for performance
    cached_empire_resources: Option<(ResourceBundle, u64)>, // Resource cache with tick
}

impl ResourceDisplay {
    pub fn new() -> Self {
        Self {
            resource_window_history: None,
            last_window_tick: 0,
            resource_changes: ResourceBundle::default(),
            population_change: 0,
            cached_empire_resources: None,
        }
    }

    /// Render horizontal resource bar at the top of the screen
    pub fn render_horizontal_bar(&mut self, state: &GameState) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let screen_w = screen_width();
            if screen_w <= 0.0 {
                return Ok(());
            }
            
            let bar_height = 35.0;
            let bar_y = 40.0; // Just below toolbar
            let bar_x = 0.0;
            let bar_w = screen_w;
            
            // Background
            draw_rectangle(bar_x, bar_y, bar_w, bar_height, Color::new(0.1, 0.1, 0.15, 0.95));
            draw_line(0.0, bar_y + bar_height, screen_w, bar_y + bar_height, 1.0, Color::new(0.3, 0.3, 0.3, 1.0));
            
            // Get resources and population
            let total_resources = self.get_cached_empire_resources(state)?;
            let total_population = self.calculate_empire_population(state);
            
            // Update change tracking
            self.update_240_tick_tracking(state)?;
            
            // Calculate item spacing
            let item_count = 7.0; // 6 resources + 1 population
            let available_width = bar_w - 40.0; // Margins
            let item_width = available_width / item_count;
            let text_y = bar_y + 22.0;
            
            let mut x_offset = 20.0;
            
            // Population first with change indicator
            let pop_text = Self::format_population_text(total_population);
            draw_text(&pop_text, x_offset, text_y, 14.0, Color::new(0.9, 0.9, 1.0, 1.0));
            
            // Population change indicator
            if self.population_change != 0 {
                let (change_text, change_color) = Self::format_change_indicator(self.population_change);
                draw_text(&change_text, x_offset, text_y - 8.0, 10.0, change_color);
            }
            x_offset += item_width;
            
            // Resources with colors and change tracking
            let resources_with_changes = [
                ("Min", total_resources.minerals, self.resource_changes.minerals, Color::new(0.6, 1.0, 0.6, 1.0)), // Light green
                ("Food", total_resources.food, self.resource_changes.food, Color::new(1.0, 0.8, 0.4, 1.0)),      // Orange
                ("Energy", total_resources.energy, self.resource_changes.energy, Color::new(0.4, 0.8, 1.0, 1.0)), // Light blue
                ("Alloys", total_resources.alloys, self.resource_changes.alloys, Color::new(0.8, 0.8, 0.8, 1.0)), // Light gray
                ("Comp", total_resources.components, self.resource_changes.components, Color::new(1.0, 0.6, 1.0, 1.0)), // Light purple
                ("Fuel", total_resources.fuel, self.resource_changes.fuel, Color::new(1.0, 0.6, 0.4, 1.0)),      // Light red
            ];
            
            for (name, amount, change, color) in resources_with_changes {
                let resource_text = Self::format_resource_text(name, amount);
                draw_text(&resource_text, x_offset, text_y, 14.0, color);
                
                // Change indicator
                if change != 0 {
                    let (change_text, change_color) = Self::format_change_indicator(change);
                    draw_text(&change_text, x_offset, text_y - 8.0, 10.0, change_color);
                }
                x_offset += item_width;
            }
        }
        Ok(())
    }

    /// Render side resource panel
    pub fn render_side_panel(&mut self, state: &GameState, mouse_over_ui: &mut bool) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Validate screen dimensions to prevent rendering issues
            let screen_w = screen_width();
            let screen_h = screen_height();
            if screen_w <= 0.0 || screen_h <= 0.0 {
                return Ok(()); // Skip rendering with invalid screen dimensions
            }
            
            let panel_x = screen_w - 220.0;
            let panel_y = 50.0; // Below toolbar
            let panel_w = 210.0;
            let panel_h = 150.0;
            
            // Ensure panel fits on screen
            if panel_x < 0.0 || panel_y + panel_h > screen_h {
                return Ok(()); // Skip rendering if panel would be off-screen
            }
            
            // Mark mouse over UI if hovering
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= panel_x && mouse_x <= panel_x + panel_w && 
               mouse_y >= panel_y && mouse_y <= panel_y + panel_h {
                *mouse_over_ui = true;
            }
            
            // Panel background with improved visual design
            draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::new(0.0, 0.0, 0.0, 0.8));
            draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, Color::new(0.8, 0.8, 0.8, 1.0));
            
            // Title
            draw_text("Empire Resources", panel_x + 10.0, panel_y + 25.0, 16.0, WHITE);
            
            // Get total empire resources with caching
            match self.get_cached_empire_resources(state) {
                Ok(total_resources) => {
                    let mut y_offset = 45.0;
                    let line_height = 18.0;
                    
                    let resources = [
                        ("Minerals", total_resources.minerals, GREEN),
                        ("Food", total_resources.food, YELLOW),
                        ("Energy", total_resources.energy, BLUE),
                        ("Alloys", total_resources.alloys, GRAY),
                        ("Components", total_resources.components, PURPLE),
                        ("Fuel", total_resources.fuel, ORANGE),
                    ];
                    
                    for (name, amount, color) in resources {
                        let text = Self::format_resource_text(name, amount);
                        draw_text(&text, panel_x + 10.0, panel_y + y_offset, 14.0, color);
                        y_offset += line_height;
                    }
                }
                Err(_) => {
                    draw_text("Resource data unavailable", panel_x + 10.0, panel_y + 45.0, 12.0, RED);
                }
            }
        }
        Ok(())
    }

    /// Update resource change tracking over 240-tick windows
    fn update_240_tick_tracking(&mut self, state: &GameState) -> GameResult<()> {
        // Use shorter window for testing, longer for production
        const CALCULATION_WINDOW: u64 = if cfg!(debug_assertions) { 30 } else { 240 };
        
        let current_tick = state.time_manager.get_current_tick();
        let current_resources = self.get_cached_empire_resources(state)?;
        let current_population = self.calculate_empire_population(state);
        
        // Initialize tracking on first run
        if self.resource_window_history.is_none() {
            self.resource_window_history = Some((current_resources, current_population, current_tick));
            self.last_window_tick = current_tick;
            return Ok(());
        }
        
        // Check if we need to update the calculation window
        if current_tick >= self.last_window_tick + CALCULATION_WINDOW {
            if let Some((prev_resources, prev_population, _prev_tick)) = self.resource_window_history {
                // Calculate changes based on the full window
                self.resource_changes.minerals = current_resources.minerals - prev_resources.minerals;
                self.resource_changes.food = current_resources.food - prev_resources.food;
                self.resource_changes.energy = current_resources.energy - prev_resources.energy;
                self.resource_changes.alloys = current_resources.alloys - prev_resources.alloys;
                self.resource_changes.components = current_resources.components - prev_resources.components;
                self.resource_changes.fuel = current_resources.fuel - prev_resources.fuel;
                self.population_change = current_population - prev_population;
            }
            
            // Update tracking data for next window
            self.resource_window_history = Some((current_resources, current_population, current_tick));
            self.last_window_tick = current_tick;
        }
        
        Ok(())
    }

    /// Get cached empire resources with validation
    fn get_cached_empire_resources(&mut self, state: &GameState) -> GameResult<ResourceBundle> {
        let current_tick = state.time_manager.get_current_tick();
        
        // Check if cache is valid (within 5 ticks)
        if let Some((cached_resources, cached_tick)) = self.cached_empire_resources {
            if current_tick <= cached_tick + 5 {
                return Ok(cached_resources);
            }
        }
        
        // Recalculate empire resources
        let total_resources = self.calculate_empire_resources(state)?;
        self.cached_empire_resources = Some((total_resources, current_tick));
        Ok(total_resources)
    }

    /// Calculate total empire resources across all planets
    fn calculate_empire_resources(&self, state: &GameState) -> GameResult<ResourceBundle> {
        let player_faction = state.faction_manager.get_player_faction()
            .ok_or_else(|| GameError::InvalidOperation("No player faction found".into()))?;
        
        let player_planets = state.planet_manager.get_planets_by_faction(player_faction.id);
        
        let mut total_resources = ResourceBundle::default();
        for planet in player_planets {
            total_resources.add(&planet.resources.current)?;
        }
        
        Ok(total_resources)
    }

    /// Calculate total empire population across all planets
    fn calculate_empire_population(&self, state: &GameState) -> i32 {
        let player_faction = match state.faction_manager.get_player_faction() {
            Some(faction) => faction,
            None => return 0,
        };
        
        let player_planets = state.planet_manager.get_planets_by_faction(player_faction.id);
        
        let mut total_population = 0;
        for planet in player_planets {
            total_population += planet.population.total;
        }
        
        total_population
    }

    /// Format population text with appropriate units
    fn format_population_text(population: i32) -> String {
        if population >= 1_000_000 {
            format!("Pop: {:.1}M", population as f32 / 1_000_000.0)
        } else if population >= 1_000 {
            format!("Pop: {:.1}K", population as f32 / 1_000.0)
        } else {
            format!("Pop: {}", population)
        }
    }

    /// Format resource text with appropriate units
    fn format_resource_text(name: &str, amount: i32) -> String {
        if amount >= 1_000_000 {
            format!("{}: {:.1}M", name, amount as f32 / 1_000_000.0)
        } else if amount >= 1_000 {
            format!("{}: {:.1}K", name, amount as f32 / 1_000.0)
        } else {
            format!("{}: {}", name, amount)
        }
    }

    /// Format change indicator with appropriate color
    fn format_change_indicator(change: i32) -> (String, Color) {
        let text = if change > 0 {
            format!("+{}", change)
        } else {
            format!("{}", change)
        };
        let color = if change > 0 { 
            Color::new(0.6, 1.0, 0.6, 1.0) 
        } else { 
            Color::new(1.0, 0.6, 0.6, 1.0) 
        };
        (text, color)
    }
}

impl Default for ResourceDisplay {
    fn default() -> Self {
        Self::new()
    }
}