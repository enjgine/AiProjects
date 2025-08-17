// src/ui/panels/resource_panel.rs
use crate::core::{GameResult, GameEvent, EventBus, GameState};
use crate::core::types::*;
use macroquad::prelude::*;

pub struct ResourcePanel {
    panel_rect: Rect,
    visible: bool,
}

impl ResourcePanel {
    pub fn new() -> Self {
        Self {
            panel_rect: Rect::new(10.0, screen_height() - 120.0, screen_width() - 20.0, 100.0),
            visible: true,
        }
    }
    
    pub fn render(&mut self, state: &GameState, events: &mut EventBus) -> GameResult<()> {
        if !self.visible {
            return Ok();
        }
        
        // Update panel position to stay at bottom of screen
        self.panel_rect.y = screen_height() - 120.0;
        self.panel_rect.w = screen_width() - 20.0;
        
        // Draw panel background
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            Color::new(0.1, 0.1, 0.2, 0.8),
        );
        
        // Draw panel border
        draw_rectangle_lines(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            2.0,
            WHITE,
        );
        
        // Render empire totals
        self.render_empire_resources(state)?;
        
        // Render game info
        self.render_game_info(state)?;
        
        Ok(())
    }
    
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    
    fn render_empire_resources(&self, state: &GameState) -> GameResult<()> {
        let start_x = self.panel_rect.x + 20.0;
        let y = self.panel_rect.y + 30.0;
        
        // Calculate total empire resources
        let total_resources = self.calculate_empire_totals(state);
        
        draw_text("Empire Resources:", start_x, y, 16.0, WHITE);
        
        let resource_spacing = 120.0;
        let mut x_offset = start_x;
        
        draw_text(
            &format!("Minerals: {}", total_resources.minerals),
            x_offset,
            y + 25.0,
            14.0,
            YELLOW,
        );
        x_offset += resource_spacing;
        
        draw_text(
            &format!("Food: {}", total_resources.food),
            x_offset,
            y + 25.0,
            14.0,
            GREEN,
        );
        x_offset += resource_spacing;
        
        draw_text(
            &format!("Energy: {}", total_resources.energy),
            x_offset,
            y + 25.0,
            14.0,
            BLUE,
        );
        x_offset += resource_spacing;
        
        draw_text(
            &format!("Alloys: {}", total_resources.alloys),
            x_offset,
            y + 25.0,
            14.0,
            RED,
        );
        x_offset += resource_spacing;
        
        draw_text(
            &format!("Components: {}", total_resources.components),
            x_offset,
            y + 25.0,
            14.0,
            PURPLE,
        );
        
        Ok(())
    }
    
    fn render_game_info(&self, state: &GameState) -> GameResult<()> {
        let right_x = self.panel_rect.x + self.panel_rect.w - 200.0;
        let y = self.panel_rect.y + 30.0;
        
        // Game speed and pause status
        let speed_text = if state.time_manager.paused {
            "PAUSED"
        } else {
            &format!("Speed: {:.1}x", state.time_manager.speed_multiplier)
        };
        
        draw_text(speed_text, right_x, y, 16.0, WHITE);
        
        // Current tick
        draw_text(
            &format!("Tick: {}", state.time_manager.tick),
            right_x,
            y + 25.0,
            14.0,
            LIGHTGRAY,
        );
        
        Ok(())
    }
    
    fn calculate_empire_totals(&self, state: &GameState) -> ResourceBundle {
        let mut total = ResourceBundle::default();
        
        // Sum resources from all player-controlled planets
        for planet in &state.planet_manager.planets {
            if let Some(controller) = planet.controller {
                // Assuming player is faction 0
                if controller == 0 {
                    total.minerals += planet.resources.current.minerals;
                    total.food += planet.resources.current.food;
                    total.energy += planet.resources.current.energy;
                    total.alloys += planet.resources.current.alloys;
                    total.components += planet.resources.current.components;
                    total.fuel += planet.resources.current.fuel;
                }
            }
        }
        
        total
    }
}