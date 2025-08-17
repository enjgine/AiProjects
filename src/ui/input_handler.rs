// src/ui/input_handler.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use macroquad::prelude::*;

pub struct InputHandler {
    last_mouse_pos: (f32, f32),
    mouse_drag_start: Option<(f32, f32)>,
    key_states: std::collections::HashMap<KeyCode, bool>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: (0.0, 0.0),
            mouse_drag_start: None,
            key_states: std::collections::HashMap::new(),
        }
    }
    
    pub fn update(&mut self, events: &mut EventBus) -> GameResult<()> {
        self.update_mouse_input(events)?;
        self.update_keyboard_input(events)?;
        Ok(())
    }
    
    fn update_mouse_input(&mut self, events: &mut EventBus) -> GameResult<()> {
        let (mouse_x, mouse_y) = mouse_position();
        
        // Left click - selection
        if is_mouse_button_pressed(MouseButton::Left) {
            self.handle_left_click(mouse_x, mouse_y, events)?;
        }
        
        // Right click - commands
        if is_mouse_button_pressed(MouseButton::Right) {
            self.handle_right_click(mouse_x, mouse_y, events)?;
        }
        
        // Mouse drag - camera movement
        if is_mouse_button_down(MouseButton::Middle) {
            if self.mouse_drag_start.is_none() {
                self.mouse_drag_start = Some((mouse_x, mouse_y));
            }
            self.handle_camera_drag(mouse_x, mouse_y, events)?;
        } else {
            self.mouse_drag_start = None;
        }
        
        // Mouse wheel - zoom
        let wheel = mouse_wheel();
        if wheel.1 != 0.0 {
            self.handle_zoom(wheel.1, events)?;
        }
        
        self.last_mouse_pos = (mouse_x, mouse_y);
        Ok(())
    }
    
    fn update_keyboard_input(&mut self, events: &mut EventBus) -> GameResult<()> {
        // Game speed controls
        if is_key_pressed(KeyCode::Key1) {
            events.queue_event(GameEvent::PlayerCommand(
                crate::core::events::PlayerCommand::SetGameSpeed(0.5)
            ));
        }
        if is_key_pressed(KeyCode::Key2) {
            events.queue_event(GameEvent::PlayerCommand(
                crate::core::events::PlayerCommand::SetGameSpeed(1.0)
            ));
        }
        if is_key_pressed(KeyCode::Key3) {
            events.queue_event(GameEvent::PlayerCommand(
                crate::core::events::PlayerCommand::SetGameSpeed(2.0)
            ));
        }
        
        // Pause toggle
        if is_key_pressed(KeyCode::Space) {
            events.queue_event(GameEvent::PlayerCommand(
                crate::core::events::PlayerCommand::PauseGame(true)
            ));
        }
        
        Ok(())
    }
    
    fn handle_left_click(&mut self, x: f32, y: f32, events: &mut EventBus) -> GameResult<()> {
        // Convert to world coordinates and check for entity selection
        // This is a placeholder implementation
        Ok(())
    }
    
    fn handle_right_click(&mut self, x: f32, y: f32, events: &mut EventBus) -> GameResult<()> {
        // Handle context commands (move, attack, etc.)
        // This is a placeholder implementation
        Ok(())
    }
    
    fn handle_camera_drag(&mut self, x: f32, y: f32, events: &mut EventBus) -> GameResult<()> {
        // Update camera position based on drag
        // This is a placeholder implementation
        Ok(())
    }
    
    fn handle_zoom(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()> {
        // Handle zoom in/out
        // This is a placeholder implementation
        Ok(())
    }
}