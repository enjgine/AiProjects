// src/ui/start_menu.rs
use crate::core::{GameEvent, GameResult};
use crate::core::events::PlayerCommand;
use crate::core::types::*;
use crate::systems::save_system::SaveSystem;
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
    GameOptions = 2,
    Exit = 3,
}

impl StartMenu {
    pub fn new() -> Self {
        // Check for any save files using the proper save system
        let save_exists = Self::check_for_saves();
        Self {
            selected_button: 0,
            button_count: 4,
            save_exists,
        }
    }
    
    /// Check if any save files exist using the save system
    fn check_for_saves() -> bool {
        let save_system = SaveSystem::new();
        match save_system.list_saves() {
            Ok(saves) => !saves.is_empty(),
            Err(_) => false, // If we can't read saves, assume none exist
        }
    }
    
    pub fn update_save_status(&mut self, save_exists: bool) {
        self.save_exists = save_exists;
    }
    
    /// Refresh save status by checking the save system
    pub fn refresh_save_status(&mut self) {
        self.save_exists = Self::check_for_saves();
    }
    
    pub fn process_input(&mut self) -> GameResult<Vec<GameEvent>> {
        let mut events = Vec::new();
        
        // Skip input processing in test environments where macroquad isn't initialized
        #[cfg(not(test))]
        {
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
                    2 => events.push(GameEvent::PlayerCommand(PlayerCommand::GameOptions)),
                    3 => events.push(GameEvent::PlayerCommand(PlayerCommand::ExitGame)),
                    _ => {}
                }
            }
        }
        
        Ok(events)
    }
    
    pub fn render(&mut self, game_config: Option<&GameConfiguration>) -> GameResult<Vec<GameEvent>> {
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
        
        // Game Options button
        let options_y = load_game_y + button_height + button_spacing;
        let options_selected = self.selected_button == MenuButton::GameOptions as usize;
        if self.draw_button("Game Options", button_x, options_y, button_width, button_height, options_selected) {
            events.push(GameEvent::PlayerCommand(PlayerCommand::GameOptions));
        }
        
        // Exit button
        let exit_y = options_y + button_height + button_spacing;
        let exit_selected = self.selected_button == MenuButton::Exit as usize;
        if self.draw_button("Exit", button_x, exit_y, button_width, button_height, exit_selected) {
            events.push(GameEvent::PlayerCommand(PlayerCommand::ExitGame));
        }
        
        // Draw game configuration info (if provided)
        if let Some(config) = game_config {
            self.draw_config_info(config);
        }
        
        // Draw version info
        let version = "v0.1.0";
        let version_font_size = 16.0;
        let version_x = 10.0;
        let version_y = screen_height() - 10.0;
        draw_text(version, version_x, version_y, version_font_size, GRAY);
        
        Ok(events)
    }
    
    fn draw_config_info(&self, config: &GameConfiguration) {
        let info_x = screen_width() - 300.0;
        let info_y = 50.0;
        let font_size = 14.0;
        let line_height = 18.0;
        
        draw_text("Current Game Settings:", info_x, info_y, font_size, WHITE);
        
        let galaxy_size_text = match config.galaxy_size {
            GalaxySize::Small => "Small Galaxy",
            GalaxySize::Medium => "Medium Galaxy",
            GalaxySize::Large => "Large Galaxy",
        };
        draw_text(galaxy_size_text, info_x, info_y + line_height, font_size, LIGHTGRAY);
        
        let planet_text = format!("{} Planets", config.planet_count);
        draw_text(&planet_text, info_x, info_y + line_height * 2.0, font_size, LIGHTGRAY);
        
        let ai_text = format!("{} AI Opponents", config.ai_opponents);
        draw_text(&ai_text, info_x, info_y + line_height * 3.0, font_size, LIGHTGRAY);
        
        let pop_text = format!("Start Population: {}", config.starting_population);
        draw_text(&pop_text, info_x, info_y + line_height * 4.0, font_size, LIGHTGRAY);
        
        draw_text("(Press Game Options to change)", info_x, info_y + line_height * 6.0, 12.0, GRAY);
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