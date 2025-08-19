// src/ui/list_menus.rs
use crate::core::{GameResult, GameEvent, EventBus, GameState};
use crate::core::types::*;
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;

pub const MENU_WIDTH: f32 = 220.0;
pub const MENU_ITEM_HEIGHT: f32 = 32.0;

#[derive(Debug, Clone)]
pub struct PlanetListMenu {
    pub open: bool,
    pub selected_planet: Option<PlanetId>,
    pub scroll_offset: f32,
    pub filter_owned_only: bool,
    // Internal state
    pub mouse_over: bool,
}

#[derive(Debug, Clone)]
pub struct ShipListMenu {
    pub open: bool,
    pub selected_ship: Option<ShipId>,
    pub scroll_offset: f32,
    pub filter_owned_only: bool,
    pub group_by_class: bool,
    // Internal state
    pub mouse_over: bool,
}

impl PlanetListMenu {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_planet: None,
            scroll_offset: 0.0,
            filter_owned_only: false,
            mouse_over: false,
        }
    }
    
    pub fn render(&mut self, state: &GameState, toolbar_height: f32, events: &mut EventBus) -> GameResult<()> {
        if !self.open {
            return Ok(());
        }
        
        #[cfg(not(test))]
        {
            self.mouse_over = false;
            
            let menu_x = 10.0; // Same as button position
            let menu_y = toolbar_height + 2.0;
            let menu_height = 300.0f32.min(screen_height() - menu_y - 20.0);
            
            // Menu background
            draw_rectangle(menu_x, menu_y, MENU_WIDTH, menu_height, Color::new(0.05, 0.05, 0.1, 0.95));
            draw_rectangle_lines(menu_x, menu_y, MENU_WIDTH, menu_height, 1.0, Color::new(0.4, 0.4, 0.4, 1.0));
            
            // Check if mouse is over menu
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= menu_x && mouse_x <= menu_x + MENU_WIDTH && 
               mouse_y >= menu_y && mouse_y <= menu_y + menu_height {
                self.mouse_over = true;
            }
            
            // Header
            draw_text("Planet List", menu_x + 10.0, menu_y + 20.0, 16.0, WHITE);
            
            // Filter checkbox (simplified for now)
            let filter_y = menu_y + 35.0;
            let filter_size = 12.0;
            let filter_color = if self.filter_owned_only { GREEN } else { GRAY };
            draw_rectangle(menu_x + 10.0, filter_y, filter_size, filter_size, filter_color);
            draw_rectangle_lines(menu_x + 10.0, filter_y, filter_size, filter_size, 1.0, WHITE);
            draw_text("Player Only", menu_x + 30.0, filter_y + 10.0, 12.0, LIGHTGRAY);
            
            // Handle filter checkbox click
            if mouse_x >= menu_x + 10.0 && mouse_x <= menu_x + 10.0 + filter_size && 
               mouse_y >= filter_y && mouse_y <= filter_y + filter_size && 
               is_mouse_button_pressed(MouseButton::Left) {
                self.filter_owned_only = !self.filter_owned_only;
            }
            
            // Planet list
            let list_start_y = filter_y + 25.0;
            let mut item_y = list_start_y;
            let planets = state.planet_manager.get_all_planets();
            let mut visible_planets = Vec::new();
            
            // Filter planets
            for planet in planets {
                if !self.filter_owned_only || planet.controller == Some(0) {
                    visible_planets.push(planet);
                }
            }
            
            // Render planet list items
            for planet in visible_planets {
                if item_y + MENU_ITEM_HEIGHT > menu_y + menu_height {
                    break; // Don't render items outside menu bounds
                }
                
                let is_selected = Some(planet.id) == self.selected_planet;
                let is_hovered = mouse_x >= menu_x && mouse_x <= menu_x + MENU_WIDTH &&
                               mouse_y >= item_y && mouse_y <= item_y + MENU_ITEM_HEIGHT;
                
                // Item background
                let bg_color = if is_selected {
                    Color::new(0.3, 0.4, 0.6, 0.8)
                } else if is_hovered {
                    Color::new(0.2, 0.2, 0.3, 0.6)
                } else {
                    Color::new(0.1, 0.1, 0.15, 0.4)
                };
                
                draw_rectangle(menu_x + 5.0, item_y, MENU_WIDTH - 10.0, MENU_ITEM_HEIGHT, bg_color);
                
                // Planet info
                let owner_text = match planet.controller {
                    Some(0) => "Player",
                    Some(id) => &format!("Faction {}", id)[..],
                    None => "Neutral",
                };
                
                let planet_text = format!("Planet {} ({})", planet.id, owner_text);
                draw_text(&planet_text, menu_x + 10.0, item_y + 16.0, 14.0, WHITE);
                
                // Enhanced planet info - Population and key resources
                let pop_text = format!("Pop: {} | Min: {} | Food: {}", 
                    planet.population.total,
                    planet.resources.current.minerals,
                    planet.resources.current.food
                );
                draw_text(&pop_text, menu_x + 10.0, item_y + 28.0, 10.0, LIGHTGRAY);
                
                // Buildings info if any
                if !planet.developments.is_empty() {
                    let buildings_text = format!("Buildings: {}", planet.developments.len());
                    draw_text(&buildings_text, menu_x + 130.0, item_y + 28.0, 9.0, YELLOW);
                }
                
                // Handle item click
                if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
                    self.selected_planet = Some(planet.id);
                    events.queue_event(GameEvent::PlayerCommand(
                        PlayerCommand::SelectPlanet(planet.id)
                    ));
                }
                
                item_y += MENU_ITEM_HEIGHT + 2.0;
            }
        }
        
        Ok(())
    }
    
    pub fn is_mouse_over(&self) -> bool {
        self.mouse_over
    }
    
    pub fn close(&mut self) {
        self.open = false;
    }
}

impl ShipListMenu {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_ship: None,
            scroll_offset: 0.0,
            filter_owned_only: false,
            group_by_class: false,
            mouse_over: false,
        }
    }
    
    pub fn render(&mut self, state: &GameState, toolbar_height: f32, events: &mut EventBus) -> GameResult<()> {
        if !self.open {
            return Ok(());
        }
        
        #[cfg(not(test))]
        {
            self.mouse_over = false;
            
            let menu_x = 95.0; // Same as ships button position
            let menu_y = toolbar_height + 2.0;
            let menu_height = 300.0f32.min(screen_height() - menu_y - 20.0);
            
            // Menu background
            draw_rectangle(menu_x, menu_y, MENU_WIDTH, menu_height, Color::new(0.05, 0.05, 0.1, 0.95));
            draw_rectangle_lines(menu_x, menu_y, MENU_WIDTH, menu_height, 1.0, Color::new(0.4, 0.4, 0.4, 1.0));
            
            // Check if mouse is over menu
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= menu_x && mouse_x <= menu_x + MENU_WIDTH && 
               mouse_y >= menu_y && mouse_y <= menu_y + menu_height {
                self.mouse_over = true;
            }
            
            // Header
            draw_text("Ship List", menu_x + 10.0, menu_y + 20.0, 16.0, WHITE);
            
            // Filter checkboxes
            let filter_y = menu_y + 35.0;
            let filter_size = 12.0;
            
            // Player only filter
            let filter_color = if self.filter_owned_only { GREEN } else { GRAY };
            draw_rectangle(menu_x + 10.0, filter_y, filter_size, filter_size, filter_color);
            draw_rectangle_lines(menu_x + 10.0, filter_y, filter_size, filter_size, 1.0, WHITE);
            draw_text("Player Only", menu_x + 30.0, filter_y + 10.0, 12.0, LIGHTGRAY);
            
            if mouse_x >= menu_x + 10.0 && mouse_x <= menu_x + 10.0 + filter_size && 
               mouse_y >= filter_y && mouse_y <= filter_y + filter_size && 
               is_mouse_button_pressed(MouseButton::Left) {
                self.filter_owned_only = !self.filter_owned_only;
            }
            
            // Group by class filter
            let group_color = if self.group_by_class { GREEN } else { GRAY };
            draw_rectangle(menu_x + 120.0, filter_y, filter_size, filter_size, group_color);
            draw_rectangle_lines(menu_x + 120.0, filter_y, filter_size, filter_size, 1.0, WHITE);
            draw_text("Group", menu_x + 140.0, filter_y + 10.0, 12.0, LIGHTGRAY);
            
            if mouse_x >= menu_x + 120.0 && mouse_x <= menu_x + 120.0 + filter_size && 
               mouse_y >= filter_y && mouse_y <= filter_y + filter_size && 
               is_mouse_button_pressed(MouseButton::Left) {
                self.group_by_class = !self.group_by_class;
            }
            
            // Ship list
            let list_start_y = filter_y + 25.0;
            let mut item_y = list_start_y;
            let ships = state.ship_manager.get_all_ships();
            let mut visible_ships = Vec::new();
            
            // Filter ships
            for ship in ships {
                if !self.filter_owned_only || ship.owner == 0 {
                    visible_ships.push(ship);
                }
            }
            
            // Sort by class if grouping is enabled
            if self.group_by_class {
                visible_ships.sort_by(|a, b| a.ship_class.cmp(&b.ship_class));
            }
            
            // Render ship list items
            for ship in visible_ships {
                if item_y + MENU_ITEM_HEIGHT > menu_y + menu_height {
                    break; // Don't render items outside menu bounds
                }
                
                let is_selected = Some(ship.id) == self.selected_ship;
                let is_hovered = mouse_x >= menu_x && mouse_x <= menu_x + MENU_WIDTH &&
                               mouse_y >= item_y && mouse_y <= item_y + MENU_ITEM_HEIGHT;
                
                // Item background
                let bg_color = if is_selected {
                    Color::new(0.3, 0.4, 0.6, 0.8)
                } else if is_hovered {
                    Color::new(0.2, 0.2, 0.3, 0.6)
                } else {
                    Color::new(0.1, 0.1, 0.15, 0.4)
                };
                
                draw_rectangle(menu_x + 5.0, item_y, MENU_WIDTH - 10.0, MENU_ITEM_HEIGHT, bg_color);
                
                // Ship info
                let class_name = match ship.ship_class {
                    ShipClass::Scout => "Scout",
                    ShipClass::Transport => "Transport",
                    ShipClass::Warship => "Warship",
                    ShipClass::Colony => "Colony",
                };
                
                let owner_text = if ship.owner == 0 { "Player" } else { &format!("Enemy {}", ship.owner)[..] };
                let ship_text = format!("{} {} ({})", class_name, ship.id, owner_text);
                draw_text(&ship_text, menu_x + 10.0, item_y + 16.0, 14.0, WHITE);
                
                // Enhanced status info - Movement, fuel, and cargo
                let status_text = if ship.trajectory.is_some() {
                    "Moving"
                } else {
                    "Idle"
                };
                let fuel_text = format!("{} | Fuel: {:.1}", status_text, ship.fuel);
                draw_text(&fuel_text, menu_x + 10.0, item_y + 28.0, 10.0, LIGHTGRAY);
                
                // Cargo info if ship has cargo
                if ship.cargo.population > 0 || ship.cargo.resources.minerals > 0 {
                    let cargo_text = if ship.cargo.population > 0 {
                        format!("Cargo: {}pop", ship.cargo.population)
                    } else {
                        format!("Cargo: {}min", ship.cargo.resources.minerals)
                    };
                    draw_text(&cargo_text, menu_x + 130.0, item_y + 28.0, 9.0, ORANGE);
                }
                
                // Handle item click
                if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
                    self.selected_ship = Some(ship.id);
                    events.queue_event(GameEvent::PlayerCommand(
                        PlayerCommand::SelectShip(ship.id)
                    ));
                }
                
                item_y += MENU_ITEM_HEIGHT + 2.0;
            }
        }
        
        Ok(())
    }
    
    pub fn is_mouse_over(&self) -> bool {
        self.mouse_over
    }
    
    pub fn close(&mut self) {
        self.open = false;
    }
}

impl Default for PlanetListMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ShipListMenu {
    fn default() -> Self {
        Self::new()
    }
}