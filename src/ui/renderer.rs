// src/ui/renderer.rs
use crate::core::{GameResult, GameEvent, EventBus, GameState};
use crate::core::types::*;
use crate::core::events::{PlayerCommand, StateChange};
use macroquad::prelude::*;
use std::f32::consts::PI;

pub struct UIRenderer {
    selected_planet: Option<PlanetId>,
    selected_ship: Option<ShipId>,
    camera_position: Vector2,
    ui_scale: f32,
    zoom_level: f32,
    paused: bool,
    show_orbits: bool,
    ui_context: UIContext,
}

#[derive(Default)]
struct UIContext {
    planet_panel_open: bool,
    ship_panel_open: bool,
    resource_panel_open: bool,
    build_menu_open: bool,
    selected_building_type: Option<BuildingType>,
    worker_allocation_temp: WorkerAllocation,
    resource_transfer_temp: ResourceBundle,
    transfer_target: Option<PlanetId>,
}

impl UIRenderer {
    pub fn new() -> Self {
        Self {
            selected_planet: None,
            selected_ship: None,
            camera_position: Vector2 { x: 0.0, y: 0.0 },
            ui_scale: 1.0,
            zoom_level: 1.0,
            paused: false,
            show_orbits: true,
            ui_context: UIContext::default(),
        }
    }
    
    // Public accessors for testing
    #[cfg(test)]
    pub fn get_selected_planet(&self) -> Option<PlanetId> {
        self.selected_planet
    }
    
    #[cfg(test)]
    pub fn get_selected_ship(&self) -> Option<ShipId> {
        self.selected_ship
    }
    
    #[cfg(test)]
    pub fn set_selected_ship(&mut self, ship_id: Option<ShipId>) {
        self.selected_ship = ship_id;
    }
    
    #[cfg(test)]
    pub fn set_selected_planet(&mut self, planet_id: Option<PlanetId>) {
        self.selected_planet = planet_id;
    }
    
    #[cfg(test)]
    pub fn get_zoom_level(&self) -> f32 {
        self.zoom_level
    }
    
    #[cfg(test)]
    pub fn get_ui_scale(&self) -> f32 {
        self.ui_scale
    }
    
    #[cfg(test)]
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    #[cfg(test)]
    pub fn is_planet_panel_open(&self) -> bool {
        self.ui_context.planet_panel_open
    }
    
    #[cfg(test)]
    pub fn is_ship_panel_open(&self) -> bool {
        self.ui_context.ship_panel_open
    }
    
    pub fn render(&mut self, state: &GameState, interpolation: f32) -> GameResult<()> {
        // In test environments, macroquad rendering functions aren't available
        #[cfg(not(test))]
        {
            clear_background(Color::new(0.05, 0.05, 0.1, 1.0));
            
            // Render space elements
            self.render_space()?;
            self.render_planets(state, interpolation)?;
            self.render_ships(state, interpolation)?;
            
            // Render UI panels
            self.render_ui_panels(state)?;
            
            // Render HUD
            self.render_hud(state)?;
        }
        
        Ok(())
    }
    
    pub fn process_input(&mut self, events: &mut EventBus) -> GameResult<()> {
        // In test environments, macroquad functions aren't available
        #[cfg(not(test))]
        {
            // Handle mouse input
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                self.handle_click(mouse_x, mouse_y, events)?;
            }
            
            if is_mouse_button_pressed(MouseButton::Right) {
                let (mouse_x, mouse_y) = mouse_position();
                self.handle_right_click(mouse_x, mouse_y, events)?;
            }
            
            // Handle mouse wheel for zoom
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                self.zoom_level = (self.zoom_level * (1.0 + wheel * 0.1)).clamp(0.1, 5.0);
            }
            
            // Handle camera movement
            self.handle_camera_movement();
            
            // Handle keyboard input
            self.handle_keyboard_input(events)?;
        }
        
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::StateChanged(state_change) => {
                match state_change {
                    StateChange::PlanetUpdated(planet_id) => {
                        if Some(*planet_id) == self.selected_planet {
                            self.ui_context.planet_panel_open = true;
                        }
                    }
                    StateChange::ShipUpdated(ship_id) => {
                        if Some(*ship_id) == self.selected_ship {
                            self.ui_context.ship_panel_open = true;
                        }
                    }
                    _ => {}
                }
            }
            GameEvent::PlayerCommand(PlayerCommand::PauseGame(paused)) => {
                self.paused = *paused;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn render_space(&self) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Render starfield background
            for i in 0..200 {
                let x = (i * 127) % screen_width() as i32;
                let y = (i * 311) % screen_height() as i32;
                let brightness = 0.3 + 0.7 * ((i * 17) % 100) as f32 / 100.0;
                
                draw_circle(
                    x as f32 + self.camera_position.x * 0.1,
                    y as f32 + self.camera_position.y * 0.1,
                    1.0,
                    Color::new(brightness, brightness, brightness * 0.9, 1.0)
                );
            }
        }
        Ok(())
    }
    
    fn render_planets(&self, state: &GameState, interpolation: f32) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let planets = state.planet_manager.get_all_planets();
            
            for planet in planets {
                let world_pos = self.calculate_planet_position(planet, state.time_manager.get_current_tick(), interpolation);
                let screen_pos = self.world_to_screen(world_pos);
                
                // Don't render if off-screen
                if !self.is_on_screen(screen_pos) {
                    continue;
                }
                
                // Render orbital path if enabled
                if self.show_orbits {
                    self.draw_orbit(planet)?;
                }
                
                // Determine planet color based on ownership
                let color = match planet.controller {
                    Some(faction_id) => {
                        if faction_id == 0 { GREEN } else { RED }
                    }
                    None => GRAY,
                };
                
                // Draw planet
                let size = 8.0 * self.zoom_level;
                draw_circle(screen_pos.x, screen_pos.y, size, color);
                
                // Draw selection highlight
                if Some(planet.id) == self.selected_planet {
                    draw_circle_lines(screen_pos.x, screen_pos.y, size + 4.0, 2.0, YELLOW);
                }
                
                // Draw planet label
                let text = format!("P{}", planet.id);
                draw_text(&text, screen_pos.x - 10.0, screen_pos.y - size - 10.0, 16.0, WHITE);
                
                // Draw resource indicators
                if planet.controller.is_some() {
                    self.draw_planet_indicators(screen_pos, planet)?;
                }
            }
        }
        Ok(())
    }
    
    fn render_ships(&self, state: &GameState, interpolation: f32) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let ships = state.ship_manager.get_all_ships();
        
        for ship in ships {
            let world_pos = if let Some(trajectory) = &ship.trajectory {
                // Interpolate ship position along trajectory
                self.interpolate_ship_position(trajectory, state.time_manager.get_current_tick(), interpolation)
            } else {
                ship.position
            };
            
            let screen_pos = self.world_to_screen(world_pos);
            
            if !self.is_on_screen(screen_pos) {
                continue;
            }
            
            // Determine ship color and shape based on class and ownership
            let color = if ship.owner == 0 { BLUE } else { ORANGE };
            let size = match ship.ship_class {
                ShipClass::Scout => 3.0,
                ShipClass::Transport => 4.0,
                ShipClass::Warship => 6.0,
                ShipClass::Colony => 5.0,
            } * self.zoom_level;
            
            // Draw ship based on class
            match ship.ship_class {
                ShipClass::Scout => draw_circle(screen_pos.x, screen_pos.y, size, color),
                ShipClass::Transport => draw_rectangle(screen_pos.x - size, screen_pos.y - size, size * 2.0, size * 2.0, color),
                ShipClass::Warship => {
                    let points = [
                        Vec2::new(screen_pos.x, screen_pos.y - size),
                        Vec2::new(screen_pos.x - size, screen_pos.y + size),
                        Vec2::new(screen_pos.x + size, screen_pos.y + size),
                    ];
                    draw_triangle(points[0], points[1], points[2], color);
                }
                ShipClass::Colony => {
                    draw_circle(screen_pos.x, screen_pos.y, size, color);
                    draw_circle_lines(screen_pos.x, screen_pos.y, size + 2.0, 1.0, color);
                }
            }
            
            // Draw selection highlight
            if Some(ship.id) == self.selected_ship {
                draw_circle_lines(screen_pos.x, screen_pos.y, size + 4.0, 2.0, YELLOW);
            }
            
            // Draw trajectory line if ship is moving
            if let Some(trajectory) = &ship.trajectory {
                let dest_screen = self.world_to_screen(trajectory.destination);
                draw_line(screen_pos.x, screen_pos.y, dest_screen.x, dest_screen.y, 1.0, color);
            }
        }
        }
        
        Ok(())
    }
    
    fn render_ui_panels(&mut self, state: &GameState) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Resource panel (always visible)
            self.render_resource_panel(state)?;
            
            // Planet panel
            if self.ui_context.planet_panel_open && self.selected_planet.is_some() {
                self.render_planet_panel(state)?;
            }
            
            // Ship panel
            if self.ui_context.ship_panel_open && self.selected_ship.is_some() {
                self.render_ship_panel(state)?;
            }
            
            // Build menu
            if self.ui_context.build_menu_open && self.selected_planet.is_some() {
                self.render_build_menu(state)?;
            }
        }
        
        Ok(())
    }
    
    fn render_hud(&self, state: &GameState) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Game status
            let status_text = if self.paused { "PAUSED" } else { "RUNNING" };
            draw_text(status_text, 10.0, 30.0, 24.0, if self.paused { RED } else { GREEN });
        
        // Tick counter
        let tick_text = format!("Tick: {}", state.time_manager.get_current_tick());
        draw_text(&tick_text, 10.0, 55.0, 20.0, WHITE);
        
        // Camera position and zoom
        let cam_text = format!("Cam: ({:.1}, {:.1}) Zoom: {:.1}x", 
            self.camera_position.x, self.camera_position.y, self.zoom_level);
        draw_text(&cam_text, 10.0, 80.0, 16.0, LIGHTGRAY);
        
        // Controls help
        let help_y = screen_height() - 120.0;
        draw_text("Controls:", 10.0, help_y, 16.0, WHITE);
        draw_text("Space: Pause/Resume", 10.0, help_y + 20.0, 14.0, LIGHTGRAY);
        draw_text("WASD: Move Camera", 10.0, help_y + 35.0, 14.0, LIGHTGRAY);
        draw_text("Mouse Wheel: Zoom", 10.0, help_y + 50.0, 14.0, LIGHTGRAY);
        draw_text("Left Click: Select", 10.0, help_y + 65.0, 14.0, LIGHTGRAY);
        draw_text("Right Click: Orders", 10.0, help_y + 80.0, 14.0, LIGHTGRAY);
        }
        
        Ok(())
    }
    
    fn render_resource_panel(&self, state: &GameState) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let panel_x = screen_width() - 220.0;
        let panel_y = 10.0;
        let panel_w = 210.0;
        let panel_h = 150.0;
        
        // Panel background
        draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, WHITE);
        
        // Title
        draw_text("Empire Resources", panel_x + 10.0, panel_y + 25.0, 16.0, WHITE);
        
        // Get total empire resources
        if let Ok(total_resources) = self.calculate_empire_resources(state) {
            let mut y_offset = 45.0;
            let line_height = 18.0;
            
            let resources = [
                ("Minerals", total_resources.minerals, GREEN),
                ("Food", total_resources.food, YELLOW),
                ("Energy", total_resources.energy, BLUE),
                ("Alloys", total_resources.alloys, GRAY),
                ("Components", total_resources.components, PURPLE),
                ("Fuel", total_resources.fuel, ORANGE),
            ];
            
            for (name, amount, color) in resources {
                draw_text(&format!("{}: {}", name, amount), 
                    panel_x + 10.0, panel_y + y_offset, 14.0, color);
                y_offset += line_height;
            }
        }
        }
        
        Ok(())
    }
    
    fn render_planet_panel(&mut self, state: &GameState) -> GameResult<()> {
        if let Some(planet_id) = self.selected_planet {
            if let Ok(planet) = state.planet_manager.get_planet(planet_id) {
                let panel_x = screen_width() - 350.0;
                let panel_y = 170.0;
                let panel_w = 340.0;
                let panel_h = 300.0;
                
                // Panel background
                draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::new(0.0, 0.0, 0.0, 0.8));
                draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, WHITE);
                
                let mut y_offset = 25.0;
                
                // Title
                draw_text(&format!("Planet {}", planet_id), panel_x + 10.0, panel_y + y_offset, 18.0, WHITE);
                y_offset += 25.0;
                
                // Population
                draw_text(&format!("Population: {}", planet.population.total), 
                    panel_x + 10.0, panel_y + y_offset, 14.0, WHITE);
                y_offset += 20.0;
                
                // Resources
                draw_text("Resources:", panel_x + 10.0, panel_y + y_offset, 14.0, WHITE);
                y_offset += 15.0;
                
                let resources = [
                    ("Minerals", planet.resources.current.minerals),
                    ("Food", planet.resources.current.food),
                    ("Energy", planet.resources.current.energy),
                ];
                
                for (name, amount) in resources {
                    draw_text(&format!("  {}: {}", name, amount), 
                        panel_x + 20.0, panel_y + y_offset, 12.0, LIGHTGRAY);
                    y_offset += 15.0;
                }
                
                // Buildings
                y_offset += 10.0;
                draw_text(&format!("Buildings: {}", planet.developments.len()), 
                    panel_x + 10.0, panel_y + y_offset, 14.0, WHITE);
                y_offset += 20.0;
                
                // Action buttons
                if self.render_button(panel_x + 10.0, panel_y + y_offset, 100.0, 25.0, "Build") {
                    self.ui_context.build_menu_open = true;
                }
                
                if self.render_button(panel_x + 120.0, panel_y + y_offset, 100.0, 25.0, "Close") {
                    self.ui_context.planet_panel_open = false;
                }
            }
        }
        Ok(())
    }
    
    fn render_ship_panel(&mut self, state: &GameState) -> GameResult<()> {
        if let Some(ship_id) = self.selected_ship {
            if let Ok(ship) = state.ship_manager.get_ship(ship_id) {
                let panel_x = 50.0;
                let panel_y = screen_height() - 200.0;
                let panel_w = 300.0;
                let panel_h = 150.0;
                
                // Panel background
                draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::new(0.0, 0.0, 0.0, 0.8));
                draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, WHITE);
                
                let mut y_offset = 25.0;
                
                // Title
                let class_name = match ship.ship_class {
                    ShipClass::Scout => "Scout",
                    ShipClass::Transport => "Transport",
                    ShipClass::Warship => "Warship",
                    ShipClass::Colony => "Colony Ship",
                };
                draw_text(&format!("{} {}", class_name, ship_id), 
                    panel_x + 10.0, panel_y + y_offset, 16.0, WHITE);
                y_offset += 25.0;
                
                // Status
                draw_text(&format!("Fuel: {:.1}", ship.fuel), 
                    panel_x + 10.0, panel_y + y_offset, 14.0, WHITE);
                y_offset += 20.0;
                
                if ship.cargo.population > 0 || ship.cargo.resources.minerals > 0 {
                    draw_text(&format!("Cargo: {} pop, {} min", 
                        ship.cargo.population, ship.cargo.resources.minerals), 
                        panel_x + 10.0, panel_y + y_offset, 14.0, WHITE);
                    y_offset += 20.0;
                }
                
                // Close button
                if self.render_button(panel_x + panel_w - 60.0, panel_y + 10.0, 50.0, 20.0, "Close") {
                    self.ui_context.ship_panel_open = false;
                }
            }
        }
        Ok(())
    }
    
    fn render_build_menu(&mut self, state: &GameState) -> GameResult<()> {
        let panel_x = screen_width() / 2.0 - 200.0;
        let panel_y = screen_height() / 2.0 - 150.0;
        let panel_w = 400.0;
        let panel_h = 300.0;
        
        // Panel background
        draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::new(0.0, 0.0, 0.0, 0.9));
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, WHITE);
        
        // Title
        draw_text("Build Menu", panel_x + 10.0, panel_y + 25.0, 18.0, WHITE);
        
        // Building options
        let buildings = [
            (BuildingType::Mine, "Mine", "Increases mineral production"),
            (BuildingType::Farm, "Farm", "Increases food production"),
            (BuildingType::PowerPlant, "Power Plant", "Increases energy production"),
            (BuildingType::Factory, "Factory", "Produces alloys and components"),
            (BuildingType::Spaceport, "Spaceport", "Allows ship construction"),
        ];
        
        let mut y_offset = 50.0;
        for (building_type, name, description) in buildings {
            if self.render_button(panel_x + 10.0, panel_y + y_offset, 150.0, 30.0, name) {
                if let Some(planet_id) = self.selected_planet {
                    // Queue build command
                    // This would emit a PlayerCommand::BuildStructure event
                    self.ui_context.selected_building_type = Some(building_type);
                }
            }
            
            draw_text(description, panel_x + 170.0, panel_y + y_offset + 20.0, 12.0, LIGHTGRAY);
            y_offset += 40.0;
        }
        
        // Close button
        if self.render_button(panel_x + panel_w - 80.0, panel_y + 10.0, 70.0, 25.0, "Close") {
            self.ui_context.build_menu_open = false;
        }
        
        Ok(())
    }
    
    fn render_button(&self, x: f32, y: f32, w: f32, h: f32, text: &str) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        let hovered = mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h;
        let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);
        
        let color = if hovered { Color::new(0.3, 0.3, 0.4, 1.0) } else { Color::new(0.2, 0.2, 0.3, 1.0) };
        
        draw_rectangle(x, y, w, h, color);
        draw_rectangle_lines(x, y, w, h, 1.0, WHITE);
        
        let text_size = 14.0;
        let text_dims = measure_text(text, None, text_size as u16, 1.0);
        draw_text(text, 
            x + (w - text_dims.width) / 2.0, 
            y + (h + text_dims.height) / 2.0, 
            text_size, WHITE);
        
        clicked
    }
    
    fn handle_click(&mut self, x: f32, y: f32, events: &mut EventBus) -> GameResult<()> {
        let world_pos = self.screen_to_world(Vector2 { x, y });
        
        // Check for planet selection
        // This would require access to planet positions
        // For now, emit a generic selection command
        
        // Check for ship selection
        // Similar logic for ships
        
        Ok(())
    }
    
    fn handle_right_click(&mut self, x: f32, y: f32, events: &mut EventBus) -> GameResult<()> {
        let world_pos = self.screen_to_world(Vector2 { x, y });
        
        // If ship selected, move to clicked position
        if let Some(ship_id) = self.selected_ship {
            events.queue_event(GameEvent::PlayerCommand(
                PlayerCommand::MoveShip { ship: ship_id, target: world_pos }
            ));
        }
        
        Ok(())
    }
    
    fn handle_camera_movement(&mut self) {
        #[cfg(not(test))]
        {
            let move_speed = 5.0 / self.zoom_level;
            
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                self.camera_position.y -= move_speed;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                self.camera_position.y += move_speed;
            }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                self.camera_position.x -= move_speed;
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                self.camera_position.x += move_speed;
            }
        }
    }
    
    fn handle_keyboard_input(&mut self, events: &mut EventBus) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Pause/resume
            if is_key_pressed(KeyCode::Space) {
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::PauseGame(!self.paused)
                ));
            }
            
            // Speed controls
            if is_key_pressed(KeyCode::Key1) {
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::SetGameSpeed(0.5)
                ));
            }
            if is_key_pressed(KeyCode::Key2) {
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::SetGameSpeed(1.0)
                ));
            }
            if is_key_pressed(KeyCode::Key3) {
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::SetGameSpeed(2.0)
                ));
            }
            
            // Toggle orbit display
            if is_key_pressed(KeyCode::O) {
                self.show_orbits = !self.show_orbits;
            }
            
            // UI toggles
            if is_key_pressed(KeyCode::P) && self.selected_planet.is_some() {
                self.ui_context.planet_panel_open = !self.ui_context.planet_panel_open;
            }
            
            if is_key_pressed(KeyCode::B) && self.selected_planet.is_some() {
                self.ui_context.build_menu_open = !self.ui_context.build_menu_open;
            }
        }
        
        Ok(())
    }
    
    // Helper methods
    fn world_to_screen(&self, world_pos: Vector2) -> Vector2 {
        #[cfg(not(test))]
        {
            Vector2 {
                x: (world_pos.x - self.camera_position.x) * self.zoom_level + screen_width() / 2.0,
                y: (world_pos.y - self.camera_position.y) * self.zoom_level + screen_height() / 2.0,
            }
        }
        #[cfg(test)]
        {
            // Use default screen size for tests
            Vector2 {
                x: (world_pos.x - self.camera_position.x) * self.zoom_level + 800.0 / 2.0,
                y: (world_pos.y - self.camera_position.y) * self.zoom_level + 600.0 / 2.0,
            }
        }
    }
    
    fn screen_to_world(&self, screen_pos: Vector2) -> Vector2 {
        #[cfg(not(test))]
        {
            Vector2 {
                x: (screen_pos.x - screen_width() / 2.0) / self.zoom_level + self.camera_position.x,
                y: (screen_pos.y - screen_height() / 2.0) / self.zoom_level + self.camera_position.y,
            }
        }
        #[cfg(test)]
        {
            Vector2 {
                x: (screen_pos.x - 800.0 / 2.0) / self.zoom_level + self.camera_position.x,
                y: (screen_pos.y - 600.0 / 2.0) / self.zoom_level + self.camera_position.y,
            }
        }
    }
    
    fn is_on_screen(&self, screen_pos: Vector2) -> bool {
        #[cfg(not(test))]
        {
            screen_pos.x >= -50.0 && screen_pos.x <= screen_width() + 50.0 &&
            screen_pos.y >= -50.0 && screen_pos.y <= screen_height() + 50.0
        }
        #[cfg(test)]
        {
            screen_pos.x >= -50.0 && screen_pos.x <= 800.0 + 50.0 &&
            screen_pos.y >= -50.0 && screen_pos.y <= 600.0 + 50.0
        }
    }
    
    fn calculate_planet_position(&self, planet: &Planet, tick: u64, interpolation: f32) -> Vector2 {
        let time = tick as f32 + interpolation;
        let angle = (time / planet.position.period) * 2.0 * PI + planet.position.phase;
        
        Vector2 {
            x: planet.position.semi_major_axis * 100.0 * angle.cos(),
            y: planet.position.semi_major_axis * 100.0 * angle.sin(),
        }
    }
    
    fn interpolate_ship_position(&self, trajectory: &Trajectory, tick: u64, interpolation: f32) -> Vector2 {
        let current_time = tick as f32 + interpolation;
        
        if current_time < trajectory.departure_time as f32 {
            return trajectory.origin;
        }
        if current_time >= trajectory.arrival_time as f32 {
            return trajectory.destination;
        }
        
        let travel_time = trajectory.arrival_time as f32 - trajectory.departure_time as f32;
        let elapsed = current_time - trajectory.departure_time as f32;
        let progress = elapsed / travel_time;
        
        Vector2 {
            x: trajectory.origin.x + (trajectory.destination.x - trajectory.origin.x) * progress,
            y: trajectory.origin.y + (trajectory.destination.y - trajectory.origin.y) * progress,
        }
    }
    
    fn draw_orbit(&self, planet: &Planet) -> GameResult<()> {
        let center = Vector2 { x: 0.0, y: 0.0 };
        let screen_center = self.world_to_screen(center);
        let radius = planet.position.semi_major_axis * 100.0 * self.zoom_level;
        
        if radius > 10.0 && radius < 2000.0 {
            draw_circle_lines(screen_center.x, screen_center.y, radius, 1.0, 
                Color::new(0.3, 0.3, 0.3, 0.5));
        }
        Ok(())
    }
    
    fn draw_planet_indicators(&self, screen_pos: Vector2, planet: &Planet) -> GameResult<()> {
        let base_y = screen_pos.y + 15.0;
        
        // Population indicator
        if planet.population.total > 1000 {
            draw_circle(screen_pos.x - 15.0, base_y, 3.0, YELLOW);
        }
        
        // Resource richness indicators
        if planet.resources.current.minerals > 500 {
            draw_circle(screen_pos.x - 5.0, base_y, 2.0, GREEN);
        }
        
        // Building indicator
        if !planet.developments.is_empty() {
            draw_rectangle(screen_pos.x + 5.0, base_y - 2.0, 4.0, 4.0, GRAY);
        }
        
        Ok(())
    }
    
    fn calculate_empire_resources(&self, state: &GameState) -> GameResult<ResourceBundle> {
        let planets = state.planet_manager.get_all_planets();
        let mut total = ResourceBundle::default();
        
        for planet in planets {
            if planet.controller == Some(0) {  // Player faction ID
                total.minerals += planet.resources.current.minerals;
                total.food += planet.resources.current.food;
                total.energy += planet.resources.current.energy;
                total.alloys += planet.resources.current.alloys;
                total.components += planet.resources.current.components;
                total.fuel += planet.resources.current.fuel;
            }
        }
        
        Ok(total)
    }
}

// Implement GameSystem trait for architecture compliance
impl crate::core::GameSystem for UIRenderer {
    fn update(&mut self, _delta: f32, _events: &mut EventBus) -> GameResult<()> {
        // UIRenderer doesn't need tick updates since it's driven by input and rendering
        Ok(())
    }
    
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        self.handle_event(event)
    }
}