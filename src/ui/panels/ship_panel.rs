// src/ui/panels/ship_panel.rs
use crate::core::{GameResult, GameError, GameEvent, EventBus};
use crate::core::types::*;
use crate::core::events::{PlayerCommand};
use super::Panel;
use macroquad::prelude::*;

pub struct ShipPanel {
    panel_rect: Rect,
    visible: bool,
    selected_ship_id: Option<ShipId>,
    scroll_offset: f32,
    button_states: ButtonStates,
}

#[derive(Default)]
struct ButtonStates {
    move_button_rect: Option<Rect>,
    cargo_button_rect: Option<Rect>,
    recall_button_rect: Option<Rect>,
}

impl ShipPanel {
    pub fn new() -> Self {
        Self {
            panel_rect: Rect::new(320.0, 10.0, 280.0, 400.0), // Increased size for better layout
            visible: false,
            selected_ship_id: None,
            scroll_offset: 0.0,
            button_states: ButtonStates::default(),
        }
    }
    
    pub fn render(&mut self, ship: Option<&Ship>, events: &mut EventBus) -> GameResult<()> {
        if !self.visible {
            return Ok(());
        }
        
        // Validate ship data if provided
        if let Some(ship) = ship {
            ship.validate().map_err(|e| GameError::SystemError(format!("Invalid ship data: {}", e)))?;
        }
        
        // Draw panel background with subtle gradient effect
        self.draw_panel_background();
        
        match ship {
            Some(ship) => {
                // Render ship information sections
                let mut y_position = self.panel_rect.y + 10.0;
                
                y_position = self.render_ship_header(ship, y_position)?;
                y_position = self.render_ship_status(ship, y_position)?;
                y_position = self.render_cargo_info(ship, y_position)?;
                y_position = self.render_ship_actions(ship, y_position, events)?;
                
                // Handle user interactions
                self.handle_input(ship, events)?;
            },
            None => {
                self.render_no_ship_selected();
            }
        }
        
        Ok(())
    }
    
    pub fn show(&mut self, ship_id: ShipId) {
        self.visible = true;
        self.selected_ship_id = Some(ship_id);
        self.scroll_offset = 0.0; // Reset scroll when showing new ship
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
        self.selected_ship_id = None;
        self.scroll_offset = 0.0;
        self.button_states = ButtonStates::default();
    }
    
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    pub fn selected_ship_id(&self) -> Option<ShipId> {
        self.selected_ship_id
    }
    
    fn draw_panel_background(&self) {
        // Main background
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            Color::new(0.15, 0.15, 0.25, 0.95),
        );
        
        // Header background
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            30.0,
            Color::new(0.2, 0.3, 0.4, 0.8),
        );
        
        // Border
        draw_rectangle_lines(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            2.0,
            Color::new(0.6, 0.7, 0.8, 1.0),
        );
    }
    
    fn render_ship_header(&self, ship: &Ship, y_start: f32) -> GameResult<f32> {
        let header_text = format!("Ship {} - {:?}", ship.id, ship.ship_class);
        
        draw_text(
            &header_text,
            self.panel_rect.x + 10.0,
            y_start + 20.0,
            18.0,
            WHITE,
        );
        
        // Owner information
        draw_text(
            &format!("Owner: Faction {}", ship.owner),
            self.panel_rect.x + 10.0,
            y_start + 40.0,
            14.0,
            Color::new(0.8, 0.8, 0.8, 1.0),
        );
        
        Ok(y_start + 60.0)
    }
    
    fn render_ship_status(&self, ship: &Ship, y_start: f32) -> GameResult<f32> {
        let mut y_offset = y_start;
        
        // Section header
        draw_text("Status:", self.panel_rect.x + 10.0, y_offset, 16.0, Color::new(0.9, 0.9, 1.0, 1.0));
        y_offset += 25.0;
        
        // Position with better formatting
        draw_text(
            &format!("Position: ({:.1}, {:.1})", ship.position.x, ship.position.y),
            self.panel_rect.x + 15.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += 20.0;
        
        // Fuel with color coding based on level
        let fuel_color = if ship.fuel < 20.0 {
            RED
        } else if ship.fuel < 50.0 {
            YELLOW
        } else {
            GREEN
        };
        
        draw_text(
            &format!("Fuel: {:.1}", ship.fuel),
            self.panel_rect.x + 15.0,
            y_offset,
            14.0,
            fuel_color,
        );
        y_offset += 20.0;
        
        // Movement status with trajectory info
        let (status_text, status_color) = match &ship.trajectory {
            Some(trajectory) => {
                let eta = if trajectory.arrival_time > trajectory.departure_time {
                    trajectory.arrival_time - trajectory.departure_time
                } else {
                    0
                };
                (format!("Moving (ETA: {} ticks)", eta), Color::new(0.5, 1.0, 0.5, 1.0))
            },
            None => ("Stationary".to_string(), Color::new(0.8, 0.8, 0.8, 1.0)),
        };
        
        draw_text(
            &format!("Status: {}", status_text),
            self.panel_rect.x + 15.0,
            y_offset,
            14.0,
            status_color,
        );
        y_offset += 20.0;
        
        // Trajectory details if moving
        if let Some(trajectory) = &ship.trajectory {
            draw_text(
                &format!("Destination: ({:.1}, {:.1})", trajectory.destination.x, trajectory.destination.y),
                self.panel_rect.x + 15.0,
                y_offset,
                12.0,
                Color::new(0.7, 0.7, 0.9, 1.0),
            );
            y_offset += 18.0;
            
            draw_text(
                &format!("Fuel Cost: {:.1}", trajectory.fuel_cost),
                self.panel_rect.x + 15.0,
                y_offset,
                12.0,
                Color::new(0.7, 0.7, 0.9, 1.0),
            );
            y_offset += 18.0;
        }
        
        Ok(y_offset + 10.0)
    }
    
    fn render_cargo_info(&self, ship: &Ship, y_start: f32) -> GameResult<f32> {
        let mut y_offset = y_start;
        
        // Section header
        draw_text("Cargo:", self.panel_rect.x + 10.0, y_offset, 16.0, Color::new(0.9, 0.9, 1.0, 1.0));
        y_offset += 25.0;
        
        // Capacity with visual indicator
        let current_load = ship.cargo.current_load();
        let capacity = ship.cargo.capacity;
        let load_percentage = if capacity > 0 {
            (current_load as f32 / capacity as f32 * 100.0).min(100.0)
        } else {
            0.0
        };
        
        let capacity_color = if load_percentage >= 100.0 {
            RED
        } else if load_percentage >= 80.0 {
            YELLOW
        } else {
            WHITE
        };
        
        draw_text(
            &format!("Capacity: {}/{} ({:.1}%)", current_load, capacity, load_percentage),
            self.panel_rect.x + 15.0,
            y_offset,
            14.0,
            capacity_color,
        );
        y_offset += 20.0;
        
        // Visual capacity bar
        let bar_width = self.panel_rect.w - 30.0;
        let bar_height = 8.0;
        let bar_x = self.panel_rect.x + 15.0;
        
        // Background bar
        draw_rectangle(bar_x, y_offset, bar_width, bar_height, Color::new(0.3, 0.3, 0.3, 1.0));
        
        // Filled portion
        if capacity > 0 {
            let filled_width = bar_width * (current_load as f32 / capacity as f32).min(1.0);
            draw_rectangle(bar_x, y_offset, filled_width, bar_height, capacity_color);
        }
        
        y_offset += 25.0;
        
        // Population
        if ship.cargo.population > 0 {
            draw_text(
                &format!("Population: {}", ship.cargo.population),
                self.panel_rect.x + 15.0,
                y_offset,
                14.0,
                Color::new(0.8, 1.0, 0.8, 1.0),
            );
            y_offset += 18.0;
        }
        
        // Resources breakdown
        let resources = &ship.cargo.resources;
        if resources.minerals + resources.food + resources.alloys + resources.components + resources.fuel > 0 {
            draw_text("Resources:", self.panel_rect.x + 15.0, y_offset, 14.0, Color::new(0.9, 0.9, 1.0, 1.0));
            y_offset += 18.0;
            
            let resource_items = [
                ("Minerals", resources.minerals, Color::new(0.8, 0.6, 0.4, 1.0)),
                ("Food", resources.food, Color::new(0.6, 0.8, 0.4, 1.0)),
                ("Alloys", resources.alloys, Color::new(0.6, 0.6, 0.8, 1.0)),
                ("Components", resources.components, Color::new(0.8, 0.4, 0.8, 1.0)),
                ("Fuel", resources.fuel, Color::new(0.8, 0.8, 0.4, 1.0)),
            ];
            
            for (name, amount, color) in resource_items.iter() {
                if *amount > 0 {
                    draw_text(
                        &format!("  {}: {}", name, amount),
                        self.panel_rect.x + 20.0,
                        y_offset,
                        12.0,
                        *color,
                    );
                    y_offset += 16.0;
                }
            }
        }
        
        Ok(y_offset + 10.0)
    }
    
    fn render_ship_actions(&mut self, ship: &Ship, y_start: f32, events: &mut EventBus) -> GameResult<f32> {
        let mut y_offset = y_start;
        
        // Section header
        draw_text("Actions:", self.panel_rect.x + 10.0, y_offset, 16.0, Color::new(0.9, 0.9, 1.0, 1.0));
        y_offset += 25.0;
        
        let button_width = 80.0;
        let button_height = 25.0;
        let button_spacing = 10.0;
        
        // Move button (only if stationary or player wants to change destination)
        let move_button_rect = Rect::new(
            self.panel_rect.x + 15.0,
            y_offset,
            button_width,
            button_height,
        );
        self.button_states.move_button_rect = Some(move_button_rect);
        
        let move_button_color = if ship.trajectory.is_some() {
            Color::new(0.6, 0.4, 0.4, 1.0) // Darker if already moving
        } else {
            Color::new(0.4, 0.6, 0.4, 1.0) // Brighter if can move
        };
        
        draw_rectangle(
            move_button_rect.x,
            move_button_rect.y,
            move_button_rect.w,
            move_button_rect.h,
            move_button_color,
        );
        draw_rectangle_lines(
            move_button_rect.x,
            move_button_rect.y,
            move_button_rect.w,
            move_button_rect.h,
            1.0,
            WHITE,
        );
        draw_text(
            "Move",
            move_button_rect.x + 25.0,
            move_button_rect.y + 16.0,
            14.0,
            WHITE,
        );
        
        // Cargo actions button
        let cargo_button_rect = Rect::new(
            self.panel_rect.x + 15.0 + button_width + button_spacing,
            y_offset,
            button_width,
            button_height,
        );
        self.button_states.cargo_button_rect = Some(cargo_button_rect);
        
        draw_rectangle(
            cargo_button_rect.x,
            cargo_button_rect.y,
            cargo_button_rect.w,
            cargo_button_rect.h,
            Color::new(0.4, 0.4, 0.6, 1.0),
        );
        draw_rectangle_lines(
            cargo_button_rect.x,
            cargo_button_rect.y,
            cargo_button_rect.w,
            cargo_button_rect.h,
            1.0,
            WHITE,
        );
        draw_text(
            "Cargo",
            cargo_button_rect.x + 20.0,
            cargo_button_rect.y + 16.0,
            14.0,
            WHITE,
        );
        
        y_offset += button_height + button_spacing;
        
        // Recall button (only if moving)
        if ship.trajectory.is_some() {
            let recall_button_rect = Rect::new(
                self.panel_rect.x + 15.0,
                y_offset,
                button_width,
                button_height,
            );
            self.button_states.recall_button_rect = Some(recall_button_rect);
            
            draw_rectangle(
                recall_button_rect.x,
                recall_button_rect.y,
                recall_button_rect.w,
                recall_button_rect.h,
                Color::new(0.6, 0.4, 0.4, 1.0),
            );
            draw_rectangle_lines(
                recall_button_rect.x,
                recall_button_rect.y,
                recall_button_rect.w,
                recall_button_rect.h,
                1.0,
                WHITE,
            );
            draw_text(
                "Recall",
                recall_button_rect.x + 20.0,
                recall_button_rect.y + 16.0,
                14.0,
                WHITE,
            );
            
            y_offset += button_height + button_spacing;
        } else {
            self.button_states.recall_button_rect = None;
        }
        
        Ok(y_offset + 10.0)
    }
    
    fn render_no_ship_selected(&self) {
        let center_x = self.panel_rect.x + self.panel_rect.w / 2.0;
        let center_y = self.panel_rect.y + self.panel_rect.h / 2.0;
        
        draw_text(
            "No Ship Selected",
            center_x - 80.0,
            center_y - 10.0,
            18.0,
            Color::new(0.6, 0.6, 0.6, 1.0),
        );
        
        draw_text(
            "Click on a ship to view details",
            center_x - 100.0,
            center_y + 15.0,
            14.0,
            Color::new(0.5, 0.5, 0.5, 1.0),
        );
    }
    
    fn handle_input(&self, ship: &Ship, events: &mut EventBus) -> GameResult<()> {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return Ok(());
        }
        
        let mouse_pos = mouse_position();
        let mouse_point = macroquad::math::Vec2::new(mouse_pos.0, mouse_pos.1);
        
        // Check move button
        if let Some(rect) = &self.button_states.move_button_rect {
            if self.point_in_rect(mouse_point, rect) {
                // For now, just emit a generic move command - UI should handle destination selection
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::MoveShip { 
                        ship: ship.id, 
                        target: Vector2::new(mouse_pos.0, mouse_pos.1) // Placeholder - should use proper destination
                    }
                ));
            }
        }
        
        // Check cargo button
        if let Some(rect) = &self.button_states.cargo_button_rect {
            if self.point_in_rect(mouse_point, rect) {
                // Cargo management would need a separate dialog/panel
                // For now, just log the action intent
                // This could emit an event to open a cargo management panel
            }
        }
        
        // Check recall button
        if let Some(rect) = &self.button_states.recall_button_rect {
            if self.point_in_rect(mouse_point, rect) {
                // Recall ship by setting target to current position
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::MoveShip { 
                        ship: ship.id, 
                        target: ship.position 
                    }
                ));
            }
        }
        
        Ok(())
    }
    
    fn point_in_rect(&self, point: macroquad::math::Vec2, rect: &Rect) -> bool {
        point.x >= rect.x && point.x <= rect.x + rect.w &&
        point.y >= rect.y && point.y <= rect.y + rect.h
    }
}