// src/ui/renderer.rs
use crate::core::{GameResult, GameEvent, EventBus, GameState};
use crate::core::types::*;
use crate::core::events::{PlayerCommand, StateChange};
use macroquad::prelude::*;
use std::f32::consts::PI;
use std::collections::HashMap;

// Constants for improved maintainability
const DEFAULT_SCREEN_WIDTH: f32 = 800.0;
const DEFAULT_SCREEN_HEIGHT: f32 = 600.0;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 10.0;
const ZOOM_FACTOR: f32 = 1.1;
const DOUBLE_CLICK_THRESHOLD: f32 = 0.3;
const STAR_COUNT_DIVISOR: f32 = 4000.0;
const MAX_STARS: usize = 300;
const ORBIT_VISIBILITY_THRESHOLD: f32 = 0.3;
const DETAIL_ZOOM_THRESHOLD: f32 = 0.5;
const HIGH_DETAIL_ZOOM_THRESHOLD: f32 = 0.8;
const FRUSTUM_CULLING_MARGIN: f32 = 50.0;
const CACHE_INVALIDATION_ZOOM_THRESHOLD: f32 = 0.05;

pub struct UIRenderer {
    selected_planet: Option<PlanetId>,
    selected_ship: Option<ShipId>,
    camera_position: Vector2,
    ui_scale: f32,
    zoom_level: f32,
    paused: bool,
    show_orbits: bool,
    ui_context: UIContext,
    // Performance optimizations
    cached_planet_positions: HashMap<PlanetId, (Vector2, u64)>, // Position cache with tick
    cached_ship_positions: HashMap<ShipId, (Vector2, u64)>,     // Position cache with tick
    cached_empire_resources: Option<(ResourceBundle, u64)>,     // Resource cache with tick
    cached_starfield: Vec<(f32, f32, f32, f32)>,               // Pre-calculated star positions
    frame_counter: u64,
    last_screen_size: (f32, f32),
    last_zoom_for_cache: f32,
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
    // UI interaction state
    mouse_over_ui: bool,
    last_click_time: f32,
    double_click_threshold: f32,
}

impl UIRenderer {
    pub fn new() -> Self {
        let mut ui_context = UIContext::default();
        ui_context.double_click_threshold = DOUBLE_CLICK_THRESHOLD;
        
        Self {
            selected_planet: None,
            selected_ship: None,
            camera_position: Vector2 { x: 0.0, y: 0.0 },
            ui_scale: 1.0,
            zoom_level: 1.0,
            paused: false,
            show_orbits: true,
            ui_context,
            cached_planet_positions: HashMap::with_capacity(100), // Pre-allocate for typical game
            cached_ship_positions: HashMap::with_capacity(500),   // Pre-allocate for typical game
            cached_empire_resources: None,
            cached_starfield: Vec::with_capacity(MAX_STARS),
            frame_counter: 0,
            last_screen_size: (DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT),
            last_zoom_for_cache: 1.0,
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
            // Performance optimization: Check if screen size changed
            let current_screen_size = (screen_width(), screen_height());
            if current_screen_size != self.last_screen_size {
                self.last_screen_size = current_screen_size;
                self.invalidate_position_cache();
            }
            
            self.frame_counter = self.frame_counter.wrapping_add(1);
            
            clear_background(Color::new(0.05, 0.05, 0.1, 1.0));
            
            // Render space elements with frustum culling
            self.render_space()?;
            self.render_planets(state, interpolation)?;
            self.render_ships(state, interpolation)?;
            
            // Render UI panels
            self.render_ui_panels(state)?;
            
            // Render HUD last to ensure it's on top
            self.render_hud(state)?;
            
            // Reset UI interaction state for next frame
            self.ui_context.mouse_over_ui = false;
        }
        
        Ok(())
    }
    
    pub fn process_input(&mut self, events: &mut EventBus) -> GameResult<()> {
        // In test environments, macroquad functions aren't available
        #[cfg(not(test))]
        {
            let current_time = get_time() as f32;
            
            // Handle mouse input with double-click detection
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                let is_double_click = current_time - self.ui_context.last_click_time < self.ui_context.double_click_threshold;
                self.ui_context.last_click_time = current_time;
                
                if !self.ui_context.mouse_over_ui {
                    self.handle_click(mouse_x, mouse_y, events, is_double_click)?;
                }
            }
            
            if is_mouse_button_pressed(MouseButton::Right) {
                let (mouse_x, mouse_y) = mouse_position();
                if !self.ui_context.mouse_over_ui {
                    self.handle_right_click(mouse_x, mouse_y, events)?;
                }
            }
            
            // Handle mouse wheel for zoom with smooth scaling
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                let zoom_factor = if wheel > 0.0 { ZOOM_FACTOR } else { 1.0 / ZOOM_FACTOR };
                let new_zoom = (self.zoom_level * zoom_factor).clamp(MIN_ZOOM, MAX_ZOOM);
                
                // Only update if zoom actually changed
                if (new_zoom - self.zoom_level).abs() > f32::EPSILON {
                    // Invalidate position cache when zoom changes significantly
                    if (new_zoom - self.last_zoom_for_cache).abs() > CACHE_INVALIDATION_ZOOM_THRESHOLD {
                        self.invalidate_position_cache();
                        self.last_zoom_for_cache = new_zoom;
                    }
                    self.zoom_level = new_zoom;
                }
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
    
    fn render_space(&mut self) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Lazy initialize starfield if empty or screen size changed
            let screen_w = screen_width();
            let screen_h = screen_height();
            
            if self.cached_starfield.is_empty() || self.last_screen_size != (screen_w, screen_h) {
                self.generate_starfield(screen_w, screen_h);
            }
            
            // Render pre-calculated starfield with parallax
            let parallax_x = (self.camera_position.x * 0.05).fract();
            let parallax_y = (self.camera_position.y * 0.05).fract();
            
            for &(base_x, base_y, brightness, size) in &self.cached_starfield {
                let x = (base_x + parallax_x * screen_w) % screen_w;
                let y = (base_y + parallax_y * screen_h) % screen_h;
                
                draw_circle(
                    x, y, size,
                    Color::new(brightness, brightness, brightness * 0.95, brightness * 0.8)
                );
            }
        }
        Ok(())
    }
    
    fn generate_starfield(&mut self, screen_w: f32, screen_h: f32) {
        self.cached_starfield.clear();
        let star_count = ((screen_w * screen_h) / STAR_COUNT_DIVISOR) as usize;
        
        for i in 0..star_count.min(MAX_STARS) {
            let seed_x = (i * 127) as f32;
            let seed_y = (i * 311) as f32;
            
            let x = seed_x % screen_w;
            let y = seed_y % screen_h;
            let brightness = 0.2 + 0.8 * ((i * 17) % 100) as f32 / 100.0;
            let size = if i % 20 == 0 { 1.5 } else { 0.8 };
            
            self.cached_starfield.push((x, y, brightness, size));
        }
    }
    
    fn render_planets(&mut self, state: &GameState, interpolation: f32) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let planets = state.planet_manager.get_all_planets();
            let current_tick = state.time_manager.get_current_tick();
            
            for planet in planets {
                // Use cached position if available and still valid
                let world_pos = if let Some((cached_pos, cached_tick)) = self.cached_planet_positions.get(&planet.id) {
                    if *cached_tick == current_tick {
                        *cached_pos
                    } else {
                        let pos = self.calculate_planet_position(planet, current_tick, interpolation);
                        self.cached_planet_positions.insert(planet.id, (pos, current_tick));
                        pos
                    }
                } else {
                    let pos = self.calculate_planet_position(planet, current_tick, interpolation);
                    self.cached_planet_positions.insert(planet.id, (pos, current_tick));
                    pos
                };
                
                let screen_pos = self.world_to_screen(world_pos);
                
                // Frustum culling: skip if off-screen
                if !self.is_on_screen(screen_pos) {
                    continue;
                }
                
                // Render orbital path if enabled and zoom allows
                if self.show_orbits && self.zoom_level > ORBIT_VISIBILITY_THRESHOLD {
                    self.draw_orbit(planet)?;
                }
                
                // Determine planet color based on ownership
                let color = match planet.controller {
                    Some(faction_id) => {
                        if faction_id == 0 { GREEN } else { RED }
                    }
                    None => GRAY,
                };
                
                // Draw planet with LOD scaling
                let base_size = 8.0;
                let size = (base_size * self.zoom_level).max(2.0); // Minimum visible size
                draw_circle(screen_pos.x, screen_pos.y, size, color);
                
                // Draw selection highlight
                if Some(planet.id) == self.selected_planet {
                    let highlight_thickness = (2.0 * self.zoom_level).max(1.0);
                    draw_circle_lines(screen_pos.x, screen_pos.y, size + 4.0, highlight_thickness, YELLOW);
                }
                
                // Draw planet label only if zoom level allows readability
                if self.zoom_level > DETAIL_ZOOM_THRESHOLD {
                    let text = format!("P{}", planet.id);
                    let text_size = (16.0 * self.zoom_level).clamp(10.0, 20.0);
                    draw_text(&text, screen_pos.x - 10.0, screen_pos.y - size - 10.0, text_size, WHITE);
                    
                    // Draw resource indicators only at higher zoom
                    if planet.controller.is_some() && self.zoom_level > HIGH_DETAIL_ZOOM_THRESHOLD {
                        self.draw_planet_indicators(screen_pos, planet)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    fn render_ships(&mut self, state: &GameState, interpolation: f32) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let ships = state.ship_manager.get_all_ships();
            let current_tick = state.time_manager.get_current_tick();
        
            for ship in ships {
                // Use cached position if available and valid
                let world_pos = if let Some((cached_pos, cached_tick)) = self.cached_ship_positions.get(&ship.id) {
                    if *cached_tick == current_tick && ship.trajectory.is_none() {
                        // Use cached position for stationary ships
                        *cached_pos
                    } else {
                        // Recalculate for moving ships or new tick
                        let pos = if let Some(trajectory) = &ship.trajectory {
                            self.interpolate_ship_position(trajectory, current_tick, interpolation)
                        } else {
                            ship.position
                        };
                        self.cached_ship_positions.insert(ship.id, (pos, current_tick));
                        pos
                    }
                } else {
                    let pos = if let Some(trajectory) = &ship.trajectory {
                        self.interpolate_ship_position(trajectory, current_tick, interpolation)
                    } else {
                        ship.position
                    };
                    self.cached_ship_positions.insert(ship.id, (pos, current_tick));
                    pos
                };
                
                let screen_pos = self.world_to_screen(world_pos);
                
                // Frustum culling
                if !self.is_on_screen(screen_pos) {
                    continue;
                }
                
                // Determine ship color and shape based on class and ownership
                let color = if ship.owner == 0 { BLUE } else { ORANGE };
                let base_size = match ship.ship_class {
                    ShipClass::Scout => 3.0,
                    ShipClass::Transport => 4.0,
                    ShipClass::Warship => 6.0,
                    ShipClass::Colony => 5.0,
                };
                let size = (base_size * self.zoom_level).max(1.5); // Minimum visible size
                
                // Draw ship based on class with optimized rendering
                match ship.ship_class {
                    ShipClass::Scout => {
                        draw_circle(screen_pos.x, screen_pos.y, size, color);
                    }
                    ShipClass::Transport => {
                        draw_rectangle(screen_pos.x - size, screen_pos.y - size, size * 2.0, size * 2.0, color);
                    }
                    ShipClass::Warship => {
                        if size > 2.0 { // Only draw detailed shape at sufficient zoom
                            let points = [
                                Vec2::new(screen_pos.x, screen_pos.y - size),
                                Vec2::new(screen_pos.x - size, screen_pos.y + size),
                                Vec2::new(screen_pos.x + size, screen_pos.y + size),
                            ];
                            draw_triangle(points[0], points[1], points[2], color);
                        } else {
                            draw_circle(screen_pos.x, screen_pos.y, size, color);
                        }
                    }
                    ShipClass::Colony => {
                        draw_circle(screen_pos.x, screen_pos.y, size, color);
                        if size > 3.0 {
                            draw_circle_lines(screen_pos.x, screen_pos.y, size + 2.0, 1.0, color);
                        }
                    }
                }
                
                // Draw selection highlight
                if Some(ship.id) == self.selected_ship {
                    let highlight_thickness = (2.0 * self.zoom_level).max(1.0);
                    draw_circle_lines(screen_pos.x, screen_pos.y, size + 4.0, highlight_thickness, YELLOW);
                }
                
                // Draw trajectory line if ship is moving and zoom allows
                if let Some(trajectory) = &ship.trajectory {
                    if self.zoom_level > ORBIT_VISIBILITY_THRESHOLD {
                        let dest_screen = self.world_to_screen(trajectory.destination);
                        let line_thickness = (1.0 * self.zoom_level).max(0.5);
                        draw_line(screen_pos.x, screen_pos.y, dest_screen.x, dest_screen.y, line_thickness, color);
                    }
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
    
    fn render_resource_panel(&mut self, state: &GameState) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let panel_x = screen_width() - 220.0;
            let panel_y = 10.0;
            let panel_w = 210.0;
            let panel_h = 150.0;
            
            // Mark mouse over UI if hovering
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= panel_x && mouse_x <= panel_x + panel_w && 
               mouse_y >= panel_y && mouse_y <= panel_y + panel_h {
                self.ui_context.mouse_over_ui = true;
            }
            
            // Panel background with improved visual design
            draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::new(0.0, 0.0, 0.0, 0.8));
            draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, Color::new(0.8, 0.8, 0.8, 1.0));
            
            // Title
            draw_text("Empire Resources", panel_x + 10.0, panel_y + 25.0, 16.0, WHITE);
            
            // Get total empire resources with caching
            match self.get_cached_empire_resources(state) {
                Ok(total_resources) => {
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
                Err(_) => {
                    draw_text("Resource Error", panel_x + 10.0, panel_y + 45.0, 14.0, RED);
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
    
    fn render_button(&mut self, x: f32, y: f32, w: f32, h: f32, text: &str) -> bool {
        #[cfg(not(test))]
        {
            let (mouse_x, mouse_y) = mouse_position();
            let hovered = mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h;
            let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);
            
            // Mark that mouse is over UI
            if hovered {
                self.ui_context.mouse_over_ui = true;
            }
            
            let color = if hovered { 
                Color::new(0.3, 0.3, 0.4, 1.0) 
            } else { 
                Color::new(0.2, 0.2, 0.3, 1.0) 
            };
            
            draw_rectangle(x, y, w, h, color);
            draw_rectangle_lines(x, y, w, h, 1.0, WHITE);
            
            let text_size = 14.0;
            let text_dims = measure_text(text, None, text_size as u16, 1.0);
            draw_text(text, 
                x + (w - text_dims.width) / 2.0, 
                y + (h + text_dims.height) / 2.0, 
                text_size, WHITE);
            
            return clicked;
        }
        
        #[cfg(test)]
        false
    }
    
    fn handle_click(&mut self, x: f32, y: f32, events: &mut EventBus, is_double_click: bool) -> GameResult<()> {
        let world_pos = self.screen_to_world(Vector2 { x, y });
        let screen_pos = Vector2 { x, y };
        
        // Clear current selection first
        let mut found_selection = false;
        
        // Check for ship selection first (ships are typically smaller and should have priority)
        if let Some(ship_id) = self.find_ship_at_position(screen_pos, &events)? {
            self.selected_ship = Some(ship_id);
            self.selected_planet = None;
            self.ui_context.ship_panel_open = true;
            self.ui_context.planet_panel_open = false;
            
            events.queue_event(GameEvent::PlayerCommand(
                PlayerCommand::SelectShip(ship_id)
            ));
            found_selection = true;
        }
        // Check for planet selection if no ship was clicked
        else if let Some(planet_id) = self.find_planet_at_position(screen_pos, &events)? {
            self.selected_planet = Some(planet_id);
            self.selected_ship = None;
            self.ui_context.planet_panel_open = true;
            self.ui_context.ship_panel_open = false;
            
            // Double-click opens build menu for planets
            if is_double_click {
                self.ui_context.build_menu_open = true;
            }
            
            events.queue_event(GameEvent::PlayerCommand(
                PlayerCommand::SelectPlanet(planet_id)
            ));
            found_selection = true;
        }
        
        // If nothing was selected, clear selection
        if !found_selection {
            self.selected_planet = None;
            self.selected_ship = None;
            self.ui_context.planet_panel_open = false;
            self.ui_context.ship_panel_open = false;
            self.ui_context.build_menu_open = false;
        }
        
        Ok(())
    }
    
    fn handle_right_click(&mut self, x: f32, y: f32, events: &mut EventBus) -> GameResult<()> {
        let world_pos = self.screen_to_world(Vector2 { x, y });
        let screen_pos = Vector2 { x, y };
        
        // Context-sensitive right-click behavior
        if let Some(ship_id) = self.selected_ship {
            // Check if right-clicking on a planet for docking/colonization
            if let Some(target_planet) = self.find_planet_at_position(screen_pos, &events)? {
                // Emit appropriate command based on ship type
                // For now, just move to planet orbit
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::MoveShip { ship: ship_id, target: world_pos }
                ));
            } else {
                // Move to empty space
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::MoveShip { ship: ship_id, target: world_pos }
                ));
            }
        } else if let Some(planet_id) = self.selected_planet {
            // Right-click on planet could show context menu for future features
            // For now, just close any open menus
            self.ui_context.build_menu_open = false;
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
                x: (world_pos.x - self.camera_position.x) * self.zoom_level + DEFAULT_SCREEN_WIDTH / 2.0,
                y: (world_pos.y - self.camera_position.y) * self.zoom_level + DEFAULT_SCREEN_HEIGHT / 2.0,
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
                x: (screen_pos.x - DEFAULT_SCREEN_WIDTH / 2.0) / self.zoom_level + self.camera_position.x,
                y: (screen_pos.y - DEFAULT_SCREEN_HEIGHT / 2.0) / self.zoom_level + self.camera_position.y,
            }
        }
    }
    
    fn is_on_screen(&self, screen_pos: Vector2) -> bool {
        #[cfg(not(test))]
        {
            let margin = FRUSTUM_CULLING_MARGIN;
            screen_pos.x >= -margin && screen_pos.x <= screen_width() + margin &&
            screen_pos.y >= -margin && screen_pos.y <= screen_height() + margin
        }
        #[cfg(test)]
        {
            let margin = FRUSTUM_CULLING_MARGIN;
            screen_pos.x >= -margin && screen_pos.x <= DEFAULT_SCREEN_WIDTH + margin &&
            screen_pos.y >= -margin && screen_pos.y <= DEFAULT_SCREEN_HEIGHT + margin
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
        let departure_time = trajectory.departure_time as f32;
        let arrival_time = trajectory.arrival_time as f32;
        
        if current_time < departure_time {
            return trajectory.origin;
        }
        if current_time >= arrival_time {
            return trajectory.destination;
        }
        
        let travel_time = arrival_time - departure_time;
        // Prevent division by zero
        if travel_time <= f32::EPSILON {
            return trajectory.destination;
        }
        
        let elapsed = current_time - departure_time;
        let progress = (elapsed / travel_time).clamp(0.0, 1.0);
        
        Vector2 {
            x: trajectory.origin.x + (trajectory.destination.x - trajectory.origin.x) * progress,
            y: trajectory.origin.y + (trajectory.destination.y - trajectory.origin.y) * progress,
        }
    }
    
    fn draw_orbit(&self, planet: &Planet) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let center = Vector2 { x: 0.0, y: 0.0 };
            let screen_center = self.world_to_screen(center);
            let radius = planet.position.semi_major_axis * 100.0 * self.zoom_level;
            
            // Only draw orbits that are visible and reasonably sized
            if radius > 10.0 && radius < 3000.0 {
                let line_thickness = (1.0 * self.zoom_level).max(0.5);
                draw_circle_lines(screen_center.x, screen_center.y, radius, line_thickness, 
                    Color::new(0.3, 0.3, 0.3, 0.4));
            }
        }
        Ok(())
    }
    
    fn draw_planet_indicators(&self, screen_pos: Vector2, planet: &Planet) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let base_y = screen_pos.y + 15.0;
            let indicator_size = (3.0 * self.zoom_level).max(1.5);
            
            // Population indicator (yellow dot for high population)
            if planet.population.total > 1000 {
                draw_circle(screen_pos.x - 15.0, base_y, indicator_size, YELLOW);
            }
            
            // Resource richness indicators (green dot for mineral wealth)
            if planet.resources.current.minerals > 500 {
                draw_circle(screen_pos.x - 5.0, base_y, indicator_size * 0.7, GREEN);
            }
            
            // Building indicator (gray square for developed planets)
            if !planet.developments.is_empty() {
                let building_size = indicator_size * 1.2;
                draw_rectangle(screen_pos.x + 5.0, base_y - building_size/2.0, 
                    building_size, building_size, GRAY);
            }
        }
        
        Ok(())
    }
    
    fn calculate_empire_resources(&self, state: &GameState) -> GameResult<ResourceBundle> {
        let planets = state.planet_manager.get_all_planets();
        let mut total = ResourceBundle::default();
        
        for planet in planets {
            if planet.controller == Some(0) {  // Player faction ID
                // Use checked arithmetic to prevent overflow
                total.minerals = total.minerals.saturating_add(planet.resources.current.minerals);
                total.food = total.food.saturating_add(planet.resources.current.food);
                total.energy = total.energy.saturating_add(planet.resources.current.energy);
                total.alloys = total.alloys.saturating_add(planet.resources.current.alloys);
                total.components = total.components.saturating_add(planet.resources.current.components);
                total.fuel = total.fuel.saturating_add(planet.resources.current.fuel);
            }
        }
        
        Ok(total)
    }
    
    // Performance optimization methods
    fn invalidate_position_cache(&mut self) {
        self.cached_planet_positions.clear();
        self.cached_ship_positions.clear();
    }
    
    fn get_cached_empire_resources(&mut self, state: &GameState) -> GameResult<ResourceBundle> {
        let current_tick = state.time_manager.get_current_tick();
        
        // Check if cache is valid
        if let Some((cached_resources, cached_tick)) = &self.cached_empire_resources {
            if *cached_tick == current_tick {
                return Ok(*cached_resources);
            }
        }
        
        // Recalculate and cache
        let resources = self.calculate_empire_resources(state)?;
        self.cached_empire_resources = Some((resources, current_tick));
        Ok(resources)
    }
    
    fn find_planet_at_position(&self, screen_pos: Vector2, events: &EventBus) -> GameResult<Option<PlanetId>> {
        // Note: This method needs access to GameState to be functional
        // For now, returning None as the full implementation would require 
        // passing GameState or restructuring the click handling
        // TODO: Implement planet selection logic with proper distance checking
        let _ = (screen_pos, events); // Suppress unused parameter warnings
        Ok(None)
    }
    
    fn find_ship_at_position(&self, screen_pos: Vector2, events: &EventBus) -> GameResult<Option<ShipId>> {
        // Note: This method needs access to GameState to be functional
        // For now, returning None as the full implementation would require 
        // passing GameState or restructuring the click handling
        // TODO: Implement ship selection logic with proper distance checking
        let _ = (screen_pos, events); // Suppress unused parameter warnings
        Ok(None)
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