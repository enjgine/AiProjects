// src/ui/panels/ship_panel.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use macroquad::prelude::*;

pub struct ShipPanel {
    panel_rect: Rect,
    visible: bool,
}

impl ShipPanel {
    pub fn new() -> Self {
        Self {
            panel_rect: Rect::new(320.0, 10.0, 250.0, 300.0),
            visible: false,
        }
    }
    
    pub fn render(&mut self, ship: &Ship, events: &mut EventBus) -> GameResult<()> {
        if !self.visible {
            return Ok();
        }
        
        // Draw panel background
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            Color::new(0.2, 0.2, 0.3, 0.9),
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
        
        // Render ship information
        self.render_ship_info(ship)?;
        
        // Render ship status
        self.render_ship_status(ship)?;
        
        // Render cargo information
        self.render_cargo(ship)?;
        
        Ok(())
    }
    
    pub fn show(&mut self, ship_id: ShipId) {
        self.visible = true;
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    fn render_ship_info(&self, ship: &Ship) -> GameResult<()> {
        let y_offset = self.panel_rect.y + 20.0;
        
        draw_text(
            &format!("Ship {} ({:?})", ship.id, ship.ship_class),
            self.panel_rect.x + 10.0,
            y_offset,
            18.0,
            WHITE,
        );
        
        Ok(())
    }
    
    fn render_ship_status(&self, ship: &Ship) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 60.0;
        
        draw_text("Status:", self.panel_rect.x + 10.0, y_offset, 16.0, WHITE);
        y_offset += 25.0;
        
        draw_text(
            &format!("Position: ({:.1}, {:.1})", ship.position.x, ship.position.y),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += 20.0;
        
        draw_text(
            &format!("Fuel: {:.1}", ship.fuel),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += 20.0;
        
        let status = if ship.trajectory.is_some() {
            "Moving"
        } else {
            "Stationary"
        };
        draw_text(
            &format!("Status: {}", status),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        
        Ok(())
    }
    
    fn render_cargo(&self, ship: &Ship) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 180.0;
        
        draw_text("Cargo:", self.panel_rect.x + 10.0, y_offset, 16.0, WHITE);
        y_offset += 25.0;
        
        draw_text(
            &format!("Capacity: {}/{}", 
                self.calculate_cargo_used(&ship.cargo), 
                ship.cargo.capacity
            ),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += 20.0;
        
        if ship.cargo.population > 0 {
            draw_text(
                &format!("Population: {}", ship.cargo.population),
                self.panel_rect.x + 20.0,
                y_offset,
                14.0,
                WHITE,
            );
        }
        
        Ok(())
    }
    
    fn calculate_cargo_used(&self, cargo: &CargoHold) -> i32 {
        cargo.population + 
        cargo.resources.minerals + 
        cargo.resources.food + 
        cargo.resources.alloys + 
        cargo.resources.components
    }
}