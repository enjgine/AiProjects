// src/ui/toolbar.rs
use crate::core::{GameResult, EventBus};
use macroquad::prelude::*;

pub const TOOLBAR_HEIGHT: f32 = 40.0;

#[derive(Debug, Clone)]
pub struct Toolbar {
    pub height: f32,
    pub planets_menu_open: bool,
    pub ships_menu_open: bool,
    pub resources_panel_open: bool,
    pub settings_menu_open: bool,
    // Internal state
    mouse_over: bool,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            height: TOOLBAR_HEIGHT,
            planets_menu_open: false,
            ships_menu_open: false,
            resources_panel_open: true, // Start with resources visible
            settings_menu_open: false,
            mouse_over: false,
        }
    }
    
    pub fn render(&mut self, _events: &mut EventBus) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let screen_w = screen_width();
            let toolbar_bg = Color::new(0.1, 0.1, 0.15, 0.95);
            
            // Toolbar background
            draw_rectangle(0.0, 0.0, screen_w, self.height, toolbar_bg);
            draw_line(0.0, self.height, screen_w, self.height, 1.0, Color::new(0.3, 0.3, 0.3, 1.0));
            
            // Reset mouse tracking
            self.mouse_over = false;
            
            let mut button_x = 10.0;
            let button_width = 80.0;
            let button_height = 28.0;
            let button_y = (self.height - button_height) / 2.0;
            let spacing = 85.0;
            
            // Planets menu button
            if self.render_toolbar_button(button_x, button_y, button_width, button_height, 
                "Planets", self.planets_menu_open) {
                self.planets_menu_open = !self.planets_menu_open;
                // Close other dropdowns
                self.ships_menu_open = false;
                self.settings_menu_open = false;
            }
            button_x += spacing;
            
            // Ships menu button
            if self.render_toolbar_button(button_x, button_y, button_width, button_height, 
                "Ships", self.ships_menu_open) {
                self.ships_menu_open = !self.ships_menu_open;
                // Close other dropdowns
                self.planets_menu_open = false;
                self.settings_menu_open = false;
            }
            button_x += spacing;
            
            // Resources button removed - resources now shown in horizontal bar
            
            
            // Settings menu button
            if self.render_toolbar_button(button_x, button_y, button_width, button_height, 
                "Settings", self.settings_menu_open) {
                self.settings_menu_open = !self.settings_menu_open;
                // Close other dropdowns
                self.planets_menu_open = false;
                self.ships_menu_open = false;
            }
        }
        
        Ok(())
    }
    
    fn render_toolbar_button(&mut self, x: f32, y: f32, w: f32, h: f32, text: &str, active: bool) -> bool {
        #[cfg(not(test))]
        {
            let (mouse_x, mouse_y) = mouse_position();
            let hovered = mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h;
            let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);
            
            if hovered {
                self.mouse_over = true;
            }
            
            // Enhanced button styling with gradients and better visual feedback
            let color = if active {
                Color::new(0.4, 0.5, 0.8, 1.0) // Brighter active blue
            } else if hovered {
                Color::new(0.35, 0.35, 0.45, 1.0) // Lighter hover
            } else {
                Color::new(0.2, 0.2, 0.25, 0.9) // Slightly transparent default
            };
            
            let border_color = if active {
                Color::new(0.7, 0.8, 1.0, 1.0) // Bright blue border for active
            } else if hovered {
                Color::new(0.6, 0.6, 0.8, 1.0) // Purple-ish for hover
            } else {
                Color::new(0.4, 0.4, 0.4, 0.8)
            };
            
            // Draw main button rectangle
            draw_rectangle(x, y, w, h, color);
            
            // Add subtle top highlight for 3D effect
            if hovered || active {
                let highlight_color = Color::new(1.0, 1.0, 1.0, 0.1);
                draw_rectangle(x, y, w, 2.0, highlight_color);
            }
            
            draw_rectangle_lines(x, y, w, h, if active { 2.0 } else { 1.0 }, border_color);
            
            // Center text in button
            let text_size = 14.0;
            let text_dims = measure_text(text, None, text_size as u16, 1.0);
            draw_text(text, 
                x + (w - text_dims.width) / 2.0, 
                y + (h + text_dims.height) / 2.0 - 2.0, 
                text_size, WHITE);
            
            return clicked;
        }
        
        #[cfg(test)]
        false
    }
    
    pub fn is_mouse_over(&self) -> bool {
        self.mouse_over
    }
    
    pub fn close_all_dropdowns(&mut self) {
        self.planets_menu_open = false;
        self.ships_menu_open = false;
        self.settings_menu_open = false;
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}