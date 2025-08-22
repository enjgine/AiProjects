// src/ui_v2/core/input_controller.rs
//! Centralized input processing and event generation

use super::InputEvent;
use crate::core::events::PlayerCommand;
use macroquad::prelude::Vec2;
use macroquad::prelude::*;

/// Processes raw input and converts it to UI events
pub struct InputController {
    last_mouse_pos: Vec2,
    mouse_pressed: [bool; 3], // Left, Right, Middle
    keys_pressed: std::collections::HashSet<KeyCode>,
    double_click_timer: f32,
    last_click_pos: Vec2,
    double_click_threshold: f32,
}

impl InputController {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: Vec2::new(0.0, 0.0),
            mouse_pressed: [false; 3],
            keys_pressed: std::collections::HashSet::new(),
            double_click_timer: 0.0,
            last_click_pos: Vec2::new(0.0, 0.0),
            double_click_threshold: 0.3, // 300ms
        }
    }

    /// Process all input events for this frame
    pub fn process_input(&mut self, delta_time: f32) -> Vec<InputEvent> {
        let mut events = Vec::new();

        // Update double-click timer
        if self.double_click_timer > 0.0 {
            self.double_click_timer -= delta_time;
        }

        // Mouse position
        let current_mouse = Vec2::from(mouse_position());
        if current_mouse != self.last_mouse_pos {
            events.push(InputEvent::MouseMove {
                x: current_mouse.x,
                y: current_mouse.y,
            });
            self.last_mouse_pos = current_mouse;
        }

        // Mouse buttons
        self.process_mouse_buttons(&mut events, current_mouse);
        
        // Keyboard
        self.process_keyboard(&mut events);

        // Mouse wheel
        let wheel_delta = mouse_wheel().1;
        if wheel_delta != 0.0 {
            events.push(InputEvent::Scroll {
                x: current_mouse.x,
                y: current_mouse.y,
                delta: wheel_delta,
            });
        }

        events
    }

    fn process_mouse_buttons(&mut self, events: &mut Vec<InputEvent>, mouse_pos: Vec2) {
        let buttons = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
        
        for (i, button) in buttons.iter().enumerate() {
            let pressed = is_mouse_button_pressed(*button);
            let released = is_mouse_button_released(*button);

            if pressed && !self.mouse_pressed[i] {
                self.mouse_pressed[i] = true;
                
                // Check for double-click
                let is_double_click = self.double_click_timer > 0.0 &&
                    mouse_pos.distance(self.last_click_pos) < 5.0;

                if is_double_click {
                    // Generate double-click event (could be added to InputEvent enum)
                    self.double_click_timer = 0.0;
                } else {
                    self.double_click_timer = self.double_click_threshold;
                    self.last_click_pos = mouse_pos;
                }

                events.push(InputEvent::MouseClick {
                    x: mouse_pos.x,
                    y: mouse_pos.y,
                    button: *button,
                });
            }

            if released && self.mouse_pressed[i] {
                self.mouse_pressed[i] = false;
                events.push(InputEvent::MouseRelease {
                    x: mouse_pos.x,
                    y: mouse_pos.y,
                    button: *button,
                });
            }
        }
    }

    fn process_keyboard(&mut self, events: &mut Vec<InputEvent>) {
        // Get all currently pressed keys
        let current_keys = get_keys_pressed();
        
        // Check for new key presses
        for key in &current_keys {
            if !self.keys_pressed.contains(key) {
                self.keys_pressed.insert(*key);
                events.push(InputEvent::KeyPress { key: *key });
            }
        }

        // Check for key releases
        let released_keys: Vec<KeyCode> = self.keys_pressed
            .difference(&current_keys.into_iter().collect())
            .copied()
            .collect();
        
        for key in released_keys {
            self.keys_pressed.remove(&key);
            events.push(InputEvent::KeyRelease { key });
        }
    }

    /// Generate common UI commands from input patterns
    pub fn generate_ui_commands(&self, events: &[InputEvent]) -> Vec<PlayerCommand> {
        let mut commands = Vec::new();

        for event in events {
            match event {
                InputEvent::KeyPress { key } => {
                    match key {
                        KeyCode::Escape => {
                            // Could close current dialog/panel
                            commands.push(PlayerCommand::ClosePlanetPanel);
                        }
                        KeyCode::F1 => {
                            // Could open help
                            commands.push(PlayerCommand::GameOptions);
                        }
                        KeyCode::Space => {
                            // Could pause/unpause
                            commands.push(PlayerCommand::PauseGame(true));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        commands
    }

    /// Check if a specific key is currently held
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Get current mouse position
    pub fn get_mouse_position(&self) -> Vec2 {
        self.last_mouse_pos
    }

    /// Check if any mouse button is currently pressed
    pub fn is_any_mouse_button_down(&self) -> bool {
        self.mouse_pressed.iter().any(|&pressed| pressed)
    }
}

impl Default for InputController {
    fn default() -> Self {
        Self::new()
    }
}