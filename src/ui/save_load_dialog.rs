// src/ui/save_load_dialog.rs
use crate::core::types::*;
use crate::core::events::*;
use crate::systems::save_system::SaveMetadata;
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub enum DialogType {
    NewGame,
    SaveGame, 
    LoadGame,
}

pub struct SaveLoadDialog {
    dialog_type: DialogType,
    active: bool,
    input_text: String,
    cursor_pos: usize,
    saves_list: Vec<SaveMetadata>,
    selected_save: Option<usize>,
    scroll_offset: f32,
    error_message: Option<String>,
}

impl SaveLoadDialog {
    pub fn new() -> Self {
        Self {
            dialog_type: DialogType::NewGame,
            active: false,
            input_text: String::new(),
            cursor_pos: 0,
            saves_list: Vec::new(),
            selected_save: None,
            scroll_offset: 0.0,
            error_message: None,
        }
    }

    pub fn show_new_game_dialog(&mut self) {
        self.dialog_type = DialogType::NewGame;
        self.active = true;
        self.input_text = "New Game".to_string();
        self.cursor_pos = self.input_text.len();
        self.error_message = None;
    }

    pub fn show_save_dialog(&mut self) {
        self.dialog_type = DialogType::SaveGame;
        self.active = true;
        self.input_text = String::new();
        self.cursor_pos = 0;
        self.error_message = None;
    }

    pub fn show_load_dialog(&mut self, saves: Vec<SaveMetadata>) {
        self.dialog_type = DialogType::LoadGame;
        self.active = true;
        self.saves_list = saves;
        self.selected_save = if !self.saves_list.is_empty() { Some(0) } else { None };
        self.scroll_offset = 0.0;
        self.error_message = None;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn close(&mut self) {
        self.active = false;
        self.input_text.clear();
        self.saves_list.clear();
        self.selected_save = None;
        self.error_message = None;
    }

    pub fn handle_input(&mut self) -> GameResult<Vec<GameEvent>> {
        if !self.active {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();

        // Handle escape key
        if is_key_pressed(KeyCode::Escape) {
            self.close();
            return Ok(events);
        }

        match self.dialog_type {
            DialogType::NewGame | DialogType::SaveGame => {
                self.handle_text_input()?;
                
                if is_key_pressed(KeyCode::Enter) {
                    if !self.input_text.trim().is_empty() {
                        let event = match self.dialog_type {
                            DialogType::NewGame => GameEvent::PlayerCommand(PlayerCommand::NewGameNamed(self.input_text.trim().to_string())),
                            DialogType::SaveGame => GameEvent::PlayerCommand(PlayerCommand::SaveGameAs(self.input_text.trim().to_string())),
                            _ => unreachable!(),
                        };
                        events.push(event);
                        self.close();
                    } else {
                        self.error_message = Some("Please enter a valid name".to_string());
                    }
                }
            }
            DialogType::LoadGame => {
                self.handle_list_input(&mut events)?;
            }
        }

        Ok(events)
    }

    fn handle_text_input(&mut self) -> GameResult<()> {
        // Handle character input - process all available chars
        while let Some(character) = get_char_pressed() {
            if character.is_alphanumeric() || " -_()[]{}!@#$%^&*+=<>?/.,".contains(character) {
                // Limit input length to prevent overly long names
                if self.input_text.len() < 50 {
                    self.input_text.insert(self.cursor_pos, character);
                    self.cursor_pos += 1;
                    self.error_message = None;
                }
            }
        }

        // Handle backspace
        if is_key_pressed(KeyCode::Backspace) && self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.input_text.remove(self.cursor_pos);
            self.error_message = None;
        }

        // Handle delete
        if is_key_pressed(KeyCode::Delete) && self.cursor_pos < self.input_text.len() {
            self.input_text.remove(self.cursor_pos);
        }

        // Handle cursor movement
        if is_key_pressed(KeyCode::Left) && self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
        if is_key_pressed(KeyCode::Right) && self.cursor_pos < self.input_text.len() {
            self.cursor_pos += 1;
        }

        Ok(())
    }

    fn handle_list_input(&mut self, events: &mut Vec<GameEvent>) -> GameResult<()> {
        if self.saves_list.is_empty() {
            return Ok(());
        }

        // Handle up/down navigation
        if is_key_pressed(KeyCode::Up) {
            if let Some(selected) = self.selected_save {
                self.selected_save = Some(if selected > 0 { selected - 1 } else { self.saves_list.len() - 1 });
            }
        }
        if is_key_pressed(KeyCode::Down) {
            if let Some(selected) = self.selected_save {
                self.selected_save = Some((selected + 1) % self.saves_list.len());
            }
        }

        // Handle enter key
        if is_key_pressed(KeyCode::Enter) {
            if let Some(selected) = self.selected_save {
                let save_name = &self.saves_list[selected].save_name;
                events.push(GameEvent::PlayerCommand(PlayerCommand::LoadGameFrom(save_name.clone())));
                self.close();
            }
        }

        Ok(())
    }

    pub fn render(&mut self) -> GameResult<()> {
        if !self.active {
            return Ok(());
        }

        let screen_width = screen_width();
        let screen_height = screen_height();

        // Draw backdrop
        draw_rectangle(0.0, 0.0, screen_width, screen_height, Color::new(0.0, 0.0, 0.0, 0.7));

        let dialog_width = screen_width.min(400.0);
        let dialog_height = match self.dialog_type {
            DialogType::LoadGame => screen_height.min(500.0),
            _ => 200.0,
        };

        let dialog_x = (screen_width - dialog_width) / 2.0;
        let dialog_y = (screen_height - dialog_height) / 2.0;

        // Draw dialog background
        draw_rectangle(dialog_x, dialog_y, dialog_width, dialog_height, WHITE);
        draw_rectangle_lines(dialog_x, dialog_y, dialog_width, dialog_height, 2.0, BLACK);

        let mut y_offset = dialog_y + 20.0;

        match self.dialog_type {
            DialogType::NewGame => {
                draw_text("New Game", dialog_x + 10.0, y_offset, 20.0, BLACK);
                y_offset += 40.0;
                
                draw_text("Enter game name:", dialog_x + 10.0, y_offset, 16.0, BLACK);
                y_offset += 30.0;
                
                self.draw_text_input(dialog_x + 10.0, y_offset, dialog_width - 20.0)?;
                y_offset += 40.0;
                
                draw_text("Press Enter to create, Escape to cancel", dialog_x + 10.0, y_offset, 14.0, DARKGRAY);
            }
            DialogType::SaveGame => {
                draw_text("Save Game", dialog_x + 10.0, y_offset, 20.0, BLACK);
                y_offset += 40.0;
                
                draw_text("Enter save name:", dialog_x + 10.0, y_offset, 16.0, BLACK);
                y_offset += 30.0;
                
                self.draw_text_input(dialog_x + 10.0, y_offset, dialog_width - 20.0)?;
                y_offset += 40.0;
                
                draw_text("Press Enter to save, Escape to cancel", dialog_x + 10.0, y_offset, 14.0, DARKGRAY);
            }
            DialogType::LoadGame => {
                draw_text("Load Game", dialog_x + 10.0, y_offset, 20.0, BLACK);
                y_offset += 40.0;
                
                if self.saves_list.is_empty() {
                    draw_text("No save games found", dialog_x + 10.0, y_offset, 16.0, DARKGRAY);
                } else {
                    self.draw_saves_list(dialog_x + 10.0, y_offset, dialog_width - 20.0, dialog_height - y_offset + dialog_y - 60.0)?;
                }
                
                let bottom_y = dialog_y + dialog_height - 30.0;
                draw_text("Use arrow keys to select, Enter to load, Escape to cancel", 
                         dialog_x + 10.0, bottom_y, 12.0, DARKGRAY);
            }
        }

        // Draw error message if any
        if let Some(error) = &self.error_message {
            draw_text(error, dialog_x + 10.0, dialog_y + dialog_height - 50.0, 14.0, RED);
        }

        Ok(())
    }

    fn draw_text_input(&self, x: f32, y: f32, width: f32) -> GameResult<()> {
        // Draw input box
        draw_rectangle(x, y - 18.0, width, 25.0, LIGHTGRAY);
        draw_rectangle_lines(x, y - 18.0, width, 25.0, 1.0, BLACK);
        
        // Draw text
        draw_text(&self.input_text, x + 5.0, y, 16.0, BLACK);
        
        // Draw cursor
        if (get_time() * 2.0) as i32 % 2 == 0 {
            let cursor_x = x + 5.0 + measure_text(&self.input_text[..self.cursor_pos], None, 16, 1.0).width;
            draw_line(cursor_x, y - 15.0, cursor_x, y + 5.0, 1.0, BLACK);
        }
        
        Ok(())
    }

    fn draw_saves_list(&mut self, x: f32, y: f32, width: f32, height: f32) -> GameResult<()> {
        let item_height = 60.0;
        let _visible_items = (height / item_height) as usize;
        
        for (i, save) in self.saves_list.iter().enumerate() {
            let item_y = y + (i as f32 * item_height);
            
            // Skip items outside visible area
            if item_y > y + height || item_y + item_height < y {
                continue;
            }
            
            // Highlight selected item
            if Some(i) == self.selected_save {
                draw_rectangle(x - 5.0, item_y - 5.0, width + 10.0, item_height, SKYBLUE);
            }
            
            // Draw save info
            draw_text(&save.save_name, x, item_y + 15.0, 16.0, BLACK);
            
            let info_text = format!("Planets: {} | Ships: {} | Factions: {} | Assets: {}", 
                                  save.total_planets, save.total_ships, save.total_factions, save.total_assets);
            draw_text(&info_text, x, item_y + 35.0, 12.0, DARKGRAY);
            
            let date_text = format!("Play Time: {} seconds", save.play_time_seconds);
            draw_text(&date_text, x, item_y + 50.0, 12.0, DARKGRAY);
            
            // Draw separator
            if i < self.saves_list.len() - 1 {
                draw_line(x, item_y + item_height - 5.0, x + width, item_y + item_height - 5.0, 1.0, LIGHTGRAY);
            }
        }
        
        Ok(())
    }

    fn format_timestamp(&self, timestamp: u64) -> String {
        // Simple timestamp formatting - in a real game you'd use proper date formatting
        let days = timestamp / 86400;
        let hours = (timestamp % 86400) / 3600;
        let minutes = (timestamp % 3600) / 60;
        format!("{}d {}h {}m ago", days, hours, minutes)
    }
}