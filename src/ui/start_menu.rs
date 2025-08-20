// src/ui/start_menu.rs
use crate::core::{GameEvent, GameResult};
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;

pub struct StartMenu {
    selected_button: usize,
    button_count: usize,
    save_exists: bool,
}

#[derive(Debug, Clone, Copy)]
enum MenuButton {
    NewGame = 0,
    LoadGame = 1,
    Exit = 2,
}

impl StartMenu {
    pub fn new() -> Self {
        Self {
            selected_button: 0,
            button_count: 3,
            save_exists: false, // TODO: Check for save file existence
        }
    }
    
    pub fn update_save_status(&mut self, save_exists: bool) {
        self.save_exists = save_exists;
    }
    
    pub fn process_input(&mut self) -> GameResult<Vec<GameEvent>> {
        let mut events = Vec::new();
        
        // Handle keyboard input
        if is_key_pressed(KeyCode::Up) {
            self.selected_button = if self.selected_button == 0 { 
                self.button_count - 1 
            } else { 
                self.selected_button - 1 
            };
        }
        
        if is_key_pressed(KeyCode::Down) {
            self.selected_button = (self.selected_button + 1) % self.button_count;
        }
        
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            match self.selected_button {
                0 => events.push(GameEvent::PlayerCommand(PlayerCommand::NewGame)),
                1 => if self.save_exists {
                    events.push(GameEvent::PlayerCommand(PlayerCommand::LoadGame));
                },
                2 => events.push(GameEvent::PlayerCommand(PlayerCommand::ExitGame)),
                _ => {}
            }
        }
        
        Ok(events)
    }
    
    pub fn render(&mut self) -> GameResult<Vec<GameEvent>> {
        let mut events = Vec::new();
        
        // Clear screen with dark background
        clear_background(Color::from_rgba(10, 15, 25, 255));
        
        // Draw title
        let title = "STELLAR DOMINION";
        let title_font_size = 60.0;
        let title_dims = measure_text(title, None, title_font_size as u16, 1.0);
        let title_x = (screen_width() - title_dims.width) / 2.0;
        let title_y = screen_height() * 0.25;
        
        draw_text(title, title_x, title_y, title_font_size, GOLD);
        
        // Subtitle
        let subtitle = "A Real-Time Space Empire Simulation";
        let subtitle_font_size = 24.0;
        let subtitle_dims = measure_text(subtitle, None, subtitle_font_size as u16, 1.0);
        let subtitle_x = (screen_width() - subtitle_dims.width) / 2.0;
        let subtitle_y = title_y + 80.0;
        
        draw_text(subtitle, subtitle_x, subtitle_y, subtitle_font_size, LIGHTGRAY);
        
        // Menu buttons
        let button_width = 200.0;
        let button_height = 50.0;
        let button_spacing = 20.0;
        let start_y = screen_height() * 0.55;
        let button_x = (screen_width() - button_width) / 2.0;
        
        // New Game button
        let new_game_y = start_y;
        let new_game_selected = self.selected_button == MenuButton::NewGame as usize;
        if self.draw_button("New Game", button_x, new_game_y, button_width, button_height, new_game_selected) {
            events.push(GameEvent::PlayerCommand(PlayerCommand::NewGame));
        }
        
        // Load Game button (disabled if no save exists)
        let load_game_y = new_game_y + button_height + button_spacing;
        let load_game_selected = self.selected_button == MenuButton::LoadGame as usize;
        let load_game_enabled = self.save_exists;
        if self.draw_button_with_state("Load Game", button_x, load_game_y, button_width, button_height, 
                                     load_game_selected, load_game_enabled) {
            if load_game_enabled {
                events.push(GameEvent::PlayerCommand(PlayerCommand::LoadGame));
            }
        }
        
        // Exit button
        let exit_y = load_game_y + button_height + button_spacing;
        let exit_selected = self.selected_button == MenuButton::Exit as usize;
        if self.draw_button("Exit", button_x, exit_y, button_width, button_height, exit_selected) {
            events.push(GameEvent::PlayerCommand(PlayerCommand::ExitGame));
        }
        
        // Draw version info
        let version = "v0.1.0";
        let version_font_size = 16.0;
        let version_x = 10.0;
        let version_y = screen_height() - 10.0;
        draw_text(version, version_x, version_y, version_font_size, GRAY);
        
        Ok(events)
    }
    
    fn draw_button(&self, text: &str, x: f32, y: f32, width: f32, height: f32, selected: bool) -> bool {
        self.draw_button_with_state(text, x, y, width, height, selected, true)
    }
    
    fn draw_button_with_state(&self, text: &str, x: f32, y: f32, width: f32, height: f32, 
                            selected: bool, enabled: bool) -> bool {
        let mouse_pos = mouse_position();
        let mouse_over = mouse_pos.0 >= x && mouse_pos.0 <= x + width &&
                        mouse_pos.1 >= y && mouse_pos.1 <= y + height;
        
        let clicked = mouse_over && is_mouse_button_pressed(MouseButton::Left);
        
        // Determine colors based on state
        let (bg_color, border_color, text_color) = if !enabled {
            (DARKGRAY, GRAY, GRAY)
        } else if selected || mouse_over {
            (Color::from_rgba(50, 70, 100, 255), SKYBLUE, WHITE)
        } else {
            (Color::from_rgba(30, 40, 60, 255), LIGHTGRAY, LIGHTGRAY)
        };
        
        // Draw button background
        draw_rectangle(x, y, width, height, bg_color);
        
        // Draw button border
        draw_rectangle_lines(x, y, width, height, 2.0, border_color);
        
        // Draw button text
        let font_size = 24.0;
        let text_dims = measure_text(text, None, font_size as u16, 1.0);
        let text_x = x + (width - text_dims.width) / 2.0;
        let text_y = y + (height + text_dims.height) / 2.0;
        
        draw_text(text, text_x, text_y, font_size, text_color);
        
        clicked && enabled
    }
}

impl Default for StartMenu {
    fn default() -> Self {
        Self::new()
    }
}