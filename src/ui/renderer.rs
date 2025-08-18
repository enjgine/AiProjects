// src/ui/renderer.rs
use crate::core::{GameResult, GameEvent, EventBus, GameState};
use crate::core::types::*;
use crate::core::events::{PlayerCommand, StateChange};
use macroquad::prelude::*;
use std::f32::consts::PI;
use std::collections::HashMap;

// Constants for improved maintainability and performance tuning
const DEFAULT_SCREEN_WIDTH: f32 = 800.0;
const DEFAULT_SCREEN_HEIGHT: f32 = 600.0;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 10.0;
const ZOOM_FACTOR: f32 = 1.1;
const DOUBLE_CLICK_THRESHOLD: f32 = 0.3;
const _STAR_COUNT_DIVISOR: f32 = 4000.0;
const MAX_STARS: usize = 300;
const ORBIT_VISIBILITY_THRESHOLD: f32 = 0.3;
const DETAIL_ZOOM_THRESHOLD: f32 = 0.5;
const HIGH_DETAIL_ZOOM_THRESHOLD: f32 = 0.8;
const FRUSTUM_CULLING_MARGIN: f32 = 50.0;
const CACHE_INVALIDATION_ZOOM_THRESHOLD: f32 = 0.05;
const _PLANET_SELECTION_RADIUS: f32 = 20.0;
const _SHIP_SELECTION_RADIUS: f32 = 15.0;
const ORBITAL_SCALE_FACTOR: f32 = 100.0;

// Performance optimization constants
const MIN_VISIBLE_SIZE: f32 = 1.0;
const MAX_TRAJECTORY_LINE_DISTANCE: f32 = 1000.0;
const CACHE_CLEANUP_INTERVAL: u64 = 300; // Clean cache every 300 frames

pub struct UIRenderer {
    selected_planet: Option<PlanetId>,
    selected_ship: Option<ShipId>,
    camera_position: Vector2,
    _ui_scale: f32,
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
    #[allow(dead_code)]
    resource_panel_open: bool,
    build_menu_open: bool,
    selected_building_type: Option<BuildingType>,
    #[allow(dead_code)]
    worker_allocation_temp: WorkerAllocation,
    #[allow(dead_code)]
    resource_transfer_temp: ResourceBundle,
    #[allow(dead_code)]
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
            _ui_scale: 1.0,
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
        self._ui_scale
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
    
    pub fn render(&mut self, _state: &GameState, _interpolation: f32) -> GameResult<()> {
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
            
            // Periodic cache cleanup for better memory management
            if self.frame_counter % CACHE_CLEANUP_INTERVAL == 0 {
                self.cleanup_old_cache_entries(_state);
            }
            
            clear_background(Color::new(0.05, 0.05, 0.1, 1.0));
            
            // Render space elements with frustum culling
            self.render_space()?;
            self.render_planets(_state, _interpolation)?;
            self.render_ships(_state, _interpolation)?;
            
            // Render UI panels
            self.render_ui_panels(_state)?;
            
            // Render HUD last to ensure it's on top
            self.render_hud(_state)?;
            
            // Reset UI interaction state for next frame
            self.ui_context.mouse_over_ui = false;
        }
        
        Ok(())
    }
    
    pub fn process_input(&mut self, _events: &mut EventBus) -> GameResult<()> {
        // In test environments, macroquad functions aren't available
        #[cfg(not(test))]
        {
            let current_time = get_time() as f32;
            
            // Validate current time to prevent issues with time-based calculations
            if !current_time.is_finite() || current_time < 0.0 {
                return Err(GameError::SystemError("Invalid system time".into()));
            }
            
            // Handle mouse input with double-click detection
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                
                // Validate mouse position
                if !mouse_x.is_finite() || !mouse_y.is_finite() {
                    return Err(GameError::InvalidOperation("Invalid mouse position".into()));
                }
                
                let _is_double_click = current_time - self.ui_context.last_click_time < self.ui_context.double_click_threshold;
                self.ui_context.last_click_time = current_time;
                
                if !self.ui_context.mouse_over_ui {
                    // Store click for later processing when GameState is available
                    // This is a temporary architectural limitation
                    _events.queue_event(GameEvent::PlayerCommand(
                        PlayerCommand::SelectPlanet(0) // Placeholder - needs proper implementation
                    ));
                }
            }
            
            if is_mouse_button_pressed(MouseButton::Right) {
                let (mouse_x, mouse_y) = mouse_position();
                
                // Validate mouse position
                if !mouse_x.is_finite() || !mouse_y.is_finite() {
                    return Err(GameError::InvalidOperation("Invalid mouse position for right-click".into()));
                }
                
                if !self.ui_context.mouse_over_ui {
                    // Convert to world coordinates for command
                    let world_pos = self.screen_to_world(Vector2 { x: mouse_x, y: mouse_y });
                    
                    // Emit a generic move command if a ship is selected
                    if let Some(ship_id) = self.selected_ship {
                        _events.queue_event(GameEvent::PlayerCommand(
                            PlayerCommand::MoveShip { ship: ship_id, target: world_pos }
                        ));
                    }
                }
            }
            
            // Handle mouse wheel for zoom with smooth scaling and validation
            let wheel = mouse_wheel().1;
            if wheel != 0.0 && wheel.is_finite() {
                let zoom_factor = if wheel > 0.0 { ZOOM_FACTOR } else { 1.0 / ZOOM_FACTOR };
                let new_zoom = self.zoom_level * zoom_factor;
                
                // Validate new zoom level
                if new_zoom.is_finite() && new_zoom > 0.0 {
                    // Only update if zoom actually changed
                    if (new_zoom - self.zoom_level).abs() > f32::EPSILON {
                        // Invalidate position cache when zoom changes significantly
                        if (new_zoom - self.last_zoom_for_cache).abs() > CACHE_INVALIDATION_ZOOM_THRESHOLD {
                            self.invalidate_position_cache();
                            self.last_zoom_for_cache = new_zoom;
                        }
                        self.zoom_level = new_zoom;
                        self.validate_zoom_level(); // Ensure zoom stays within bounds
                    }
                }
            }
            
            // Handle camera movement with validation
            self.handle_camera_movement();
            
            // Handle keyboard input
            self.handle_keyboard_input(_events)?;
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
        // Validate screen dimensions to prevent overflow and division by zero
        if screen_w <= 0.0 || screen_h <= 0.0 {
            return;
        }
        
        self.cached_starfield.clear();
        let star_count = ((screen_w * screen_h) / _STAR_COUNT_DIVISOR) as usize;
        
        // Use a simple LCG (Linear Congruential Generator) for better distribution
        let mut seed = 12345u32;
        
        for _i in 0..star_count.min(MAX_STARS) {
            // Better pseudorandom number generation with proper modular arithmetic
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let x = (seed as f32 / u32::MAX as f32) * screen_w;
            
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let y = (seed as f32 / u32::MAX as f32) * screen_h;
            
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let brightness = 0.2 + 0.8 * (seed as f32 / u32::MAX as f32);
            
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let size = if (seed % 20) == 0 { 1.5 } else { 0.8 };
            
            self.cached_starfield.push((x, y, brightness, size));
        }
    }
    
    fn render_planets(&mut self, _state: &GameState, _interpolation: f32) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let planets = _state.planet_manager.get_all_planets();
            let current_tick = _state.time_manager.get_current_tick();
            
            for planet in planets {
                // Use cached position if available and still valid
                let world_pos = if let Some((cached_pos, cached_tick)) = self.cached_planet_positions.get(&planet.id) {
                    if *cached_tick == current_tick {
                        *cached_pos
                    } else {
                        let pos = self.calculate_planet_position(planet, current_tick, _interpolation);
                        self.cached_planet_positions.insert(planet.id, (pos, current_tick));
                        pos
                    }
                } else {
                    let pos = self.calculate_planet_position(planet, current_tick, _interpolation);
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
    
    fn render_ships(&mut self, _state: &GameState, _interpolation: f32) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let ships = _state.ship_manager.get_all_ships();
            let current_tick = _state.time_manager.get_current_tick();
            
            // Early exit if no ships to render
            if ships.is_empty() {
                return Ok(());
            }
            
            // Pre-calculate screen bounds for improved frustum culling
            let screen_bounds = self.get_screen_bounds();
        
            for ship in ships {
                // Use cached position if available and valid
                let world_pos = self.get_ship_world_position(ship, current_tick, _interpolation);
                let screen_pos = self.world_to_screen(world_pos);
                
                // Enhanced frustum culling with screen bounds
                if !self.is_position_in_bounds(screen_pos, &screen_bounds) {
                    continue;
                }
                
                // Calculate ship rendering properties
                let (color, base_size) = self.get_ship_display_properties(ship);
                let size = (base_size * self.zoom_level).max(MIN_VISIBLE_SIZE);
                
                // Skip rendering very small ships for performance
                if size < MIN_VISIBLE_SIZE {
                    continue;
                }
                
                // Draw ship with optimized rendering based on class
                self.draw_ship_shape(screen_pos, size, color, ship.ship_class)?;
                
                // Draw selection highlight
                if Some(ship.id) == self.selected_ship {
                    let highlight_thickness = (2.0 * self.zoom_level).max(1.0);
                    draw_circle_lines(screen_pos.x, screen_pos.y, size + 4.0, highlight_thickness, YELLOW);
                }
                
                // Draw trajectory line with distance optimization
                if let Some(trajectory) = &ship.trajectory {
                    self.draw_trajectory_line(screen_pos, trajectory, color)?;
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
    
    fn render_hud(&self, _state: &GameState) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Game status
            let status_text = if self.paused { "PAUSED" } else { "RUNNING" };
            draw_text(status_text, 10.0, 30.0, 24.0, if self.paused { RED } else { GREEN });
        
        // Tick counter
        let tick_text = format!("Tick: {}", _state.time_manager.get_current_tick());
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
            // Validate screen dimensions to prevent rendering issues
            let screen_w = screen_width();
            let screen_h = screen_height();
            if screen_w <= 0.0 || screen_h <= 0.0 {
                return Ok(()); // Skip rendering with invalid screen dimensions
            }
            
            let panel_x = screen_w - 220.0;
            let panel_y = 10.0;
            let panel_w = 210.0;
            let panel_h = 150.0;
            
            // Ensure panel fits on screen
            if panel_x < 0.0 || panel_y + panel_h > screen_h {
                return Ok(()); // Skip rendering if panel would be off-screen
            }
            
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
                        // Format large numbers more readably
                        let formatted_amount = if amount >= 1_000_000 {
                            format!("{:.1}M", amount as f64 / 1_000_000.0)
                        } else if amount >= 1_000 {
                            format!("{:.1}K", amount as f64 / 1_000.0)
                        } else {
                            amount.to_string()
                        };
                        
                        draw_text(&format!("{}: {}", name, formatted_amount), 
                            panel_x + 10.0, panel_y + y_offset, 14.0, color);
                        y_offset += line_height;
                    }
                }
                Err(err) => {
                    let error_msg = format!("Resource Error: {}", err);
                    draw_text(&error_msg, panel_x + 10.0, panel_y + 45.0, 12.0, RED);
                }
            }
        }
        
        Ok(())
    }
    
    fn render_planet_panel(&mut self, state: &GameState) -> GameResult<()> {
        #[cfg(not(test))]
        {
            if let Some(planet_id) = self.selected_planet {
                match state.planet_manager.get_planet(planet_id) {
                    Ok(planet) => {
                        // Validate screen dimensions
                        let screen_w = screen_width();
                        let screen_h = screen_height();
                        if screen_w <= 0.0 || screen_h <= 0.0 {
                            return Ok(());
                        }
                        
                        let panel_x = screen_w - 350.0;
                        let panel_y = 170.0;
                        let panel_w = 340.0;
                        let panel_h = 300.0;
                        
                        // Ensure panel fits on screen
                        if panel_x < 0.0 || panel_y + panel_h > screen_h {
                            return Ok(());
                        }
                        
                        // Mark mouse over UI for interaction tracking
                        let (mouse_x, mouse_y) = mouse_position();
                        if mouse_x >= panel_x && mouse_x <= panel_x + panel_w && 
                           mouse_y >= panel_y && mouse_y <= panel_y + panel_h {
                            self.ui_context.mouse_over_ui = true;
                        }
                        
                        // Panel background
                        draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::new(0.0, 0.0, 0.0, 0.8));
                        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, WHITE);
                        
                        let mut y_offset = 25.0;
                        
                        // Title with owner information
                        let title = if let Some(controller) = planet.controller {
                            format!("Planet {} (Faction {})", planet_id, controller)
                        } else {
                            format!("Planet {} (Neutral)", planet_id)
                        };
                        draw_text(&title, panel_x + 10.0, panel_y + y_offset, 18.0, WHITE);
                        y_offset += 25.0;
                        
                        // Population with growth indicator
                        let pop_color = if planet.population.total > 0 { WHITE } else { GRAY };
                        draw_text(&format!("Population: {}", planet.population.total), 
                            panel_x + 10.0, panel_y + y_offset, 14.0, pop_color);
                        y_offset += 20.0;
                        
                        // Resources with capacity indicators
                        draw_text("Resources:", panel_x + 10.0, panel_y + y_offset, 14.0, WHITE);
                        y_offset += 15.0;
                        
                        let resources = [
                            ("Minerals", planet.resources.current.minerals, planet.resources.capacity.minerals),
                            ("Food", planet.resources.current.food, planet.resources.capacity.food),
                            ("Energy", planet.resources.current.energy, planet.resources.capacity.energy),
                            ("Alloys", planet.resources.current.alloys, planet.resources.capacity.alloys),
                            ("Components", planet.resources.current.components, planet.resources.capacity.components),
                            ("Fuel", planet.resources.current.fuel, planet.resources.capacity.fuel),
                        ];
                        
                        for (name, current, capacity) in resources {
                            // Color code based on resource levels
                            let color = if current <= 0 {
                                RED
                            } else if capacity > 0 && current >= capacity {
                                YELLOW
                            } else {
                                LIGHTGRAY
                            };
                            
                            let text = if capacity > 0 {
                                format!("  {}: {}/{}", name, current, capacity)
                            } else {
                                format!("  {}: {}", name, current)
                            };
                            
                            draw_text(&text, panel_x + 20.0, panel_y + y_offset, 12.0, color);
                            y_offset += 15.0;
                        }
                        
                        // Buildings with details
                        y_offset += 10.0;
                        draw_text(&format!("Buildings: {}", planet.developments.len()), 
                            panel_x + 10.0, panel_y + y_offset, 14.0, WHITE);
                        y_offset += 20.0;
                        
                        // Action buttons with improved spacing
                        if self.render_button(panel_x + 10.0, panel_y + y_offset, 100.0, 25.0, "Build") {
                            self.ui_context.build_menu_open = true;
                        }
                        
                        if self.render_button(panel_x + 120.0, panel_y + y_offset, 100.0, 25.0, "Close") {
                            self.ui_context.planet_panel_open = false;
                        }
                    }
                    Err(_) => {
                        // Planet no longer exists, clear selection
                        self.selected_planet = None;
                        self.ui_context.planet_panel_open = false;
                    }
                }
            }
        }
        Ok(())
    }
    
    fn render_ship_panel(&mut self, _state: &GameState) -> GameResult<()> {
        if let Some(ship_id) = self.selected_ship {
            if let Ok(ship) = _state.ship_manager.get_ship(ship_id) {
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
                    // y_offset += 20.0; // Removed unused assignment
                }
                
                // Close button
                if self.render_button(panel_x + panel_w - 60.0, panel_y + 10.0, 50.0, 20.0, "Close") {
                    self.ui_context.ship_panel_open = false;
                }
            }
        }
        Ok(())
    }
    
    fn render_build_menu(&mut self, _state: &GameState) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Validate screen dimensions
            let screen_w = screen_width();
            let screen_h = screen_height();
            if screen_w <= 0.0 || screen_h <= 0.0 {
                return Ok(());
            }
            
            let panel_w = 400.0;
            let panel_h = 300.0;
            let panel_x = screen_w / 2.0 - panel_w / 2.0;
            let panel_y = screen_h / 2.0 - panel_h / 2.0;
            
            // Ensure panel fits on screen
            if panel_x < 0.0 || panel_y < 0.0 || panel_x + panel_w > screen_w || panel_y + panel_h > screen_h {
                return Ok(());
            }
            
            // Mark mouse over UI for interaction tracking
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x >= panel_x && mouse_x <= panel_x + panel_w && 
               mouse_y >= panel_y && mouse_y <= panel_y + panel_h {
                self.ui_context.mouse_over_ui = true;
            }
            
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
                    if let Some(_planet_id) = self.selected_planet {
                        // Store selected building type for later command emission
                        self.ui_context.selected_building_type = Some(building_type);
                        // Note: In a complete implementation, this would emit a PlayerCommand::BuildStructure event
                    }
                }
                
                draw_text(description, panel_x + 170.0, panel_y + y_offset + 20.0, 12.0, LIGHTGRAY);
                y_offset += 40.0;
            }
            
            // Close button
            if self.render_button(panel_x + panel_w - 80.0, panel_y + 10.0, 70.0, 25.0, "Close") {
                self.ui_context.build_menu_open = false;
            }
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
    
    fn handle_click(&mut self, x: f32, y: f32, _events: &mut EventBus, is_double_click: bool, _state: &GameState) -> GameResult<()> {
        // Validate input coordinates
        if !x.is_finite() || !y.is_finite() {
            return Err(GameError::InvalidOperation("Invalid click coordinates".into()));
        }
        
        let _world_pos = self.screen_to_world(Vector2 { x, y });
        let screen_pos = Vector2 { x, y };
        
        // Clear current selection first
        let mut found_selection = false;
        
        // Check for ship selection first (ships are typically smaller and should have priority)
        if let Some(ship_id) = self.find_ship_at_position(screen_pos, _state)? {
            self.selected_ship = Some(ship_id);
            self.selected_planet = None;
            self.ui_context.ship_panel_open = true;
            self.ui_context.planet_panel_open = false;
            
            _events.queue_event(GameEvent::PlayerCommand(
                PlayerCommand::SelectShip(ship_id)
            ));
            found_selection = true;
        }
        // Check for planet selection if no ship was clicked
        else if let Some(planet_id) = self.find_planet_at_position(screen_pos, _state)? {
            self.selected_planet = Some(planet_id);
            self.selected_ship = None;
            self.ui_context.planet_panel_open = true;
            self.ui_context.ship_panel_open = false;
            
            // Double-click opens build menu for planets
            if is_double_click {
                self.ui_context.build_menu_open = true;
            }
            
            _events.queue_event(GameEvent::PlayerCommand(
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
    
    fn handle_right_click(&mut self, x: f32, y: f32, _events: &mut EventBus, _state: &GameState) -> GameResult<()> {
        // Validate input coordinates
        if !x.is_finite() || !y.is_finite() {
            return Err(GameError::InvalidOperation("Invalid right-click coordinates".into()));
        }
        
        let world_pos = self.screen_to_world(Vector2 { x, y });
        let screen_pos = Vector2 { x, y };
        
        // Context-sensitive right-click behavior
        if let Some(ship_id) = self.selected_ship {
            // Verify the ship still exists before issuing commands
            if _state.ship_manager.get_ship(ship_id).is_err() {
                self.selected_ship = None;
                self.ui_context.ship_panel_open = false;
                return Ok(());
            }
            
            // Check if right-clicking on a planet for docking/colonization
            if let Some(_target_planet) = self.find_planet_at_position(screen_pos, _state)? {
                // Emit appropriate command based on ship type
                // For now, just move to planet orbit
                _events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::MoveShip { ship: ship_id, target: world_pos }
                ));
            } else {
                // Move to empty space
                _events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::MoveShip { ship: ship_id, target: world_pos }
                ));
            }
        } else if let Some(_planet_id) = self.selected_planet {
            // Right-click on planet could show context menu for future features
            // For now, just close any open menus
            self.ui_context.build_menu_open = false;
        }
        
        Ok(())
    }
    
    fn handle_camera_movement(&mut self) {
        #[cfg(not(test))]
        {
            let base_move_speed = 5.0;
            let move_speed = base_move_speed / self.zoom_level.max(0.1); // Prevent division issues
            
            // Store original position for validation
            let original_pos = self.camera_position;
            
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
            
            // Validate camera position to prevent overflow or invalid values
            if !self.camera_position.x.is_finite() || !self.camera_position.y.is_finite() {
                self.camera_position = original_pos; // Restore if invalid
            }
            
            // Optionally clamp camera to reasonable bounds to prevent infinite scrolling
            const MAX_CAMERA_DISTANCE: f32 = 100000.0;
            self.camera_position.x = self.camera_position.x.clamp(-MAX_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
            self.camera_position.y = self.camera_position.y.clamp(-MAX_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
        }
    }
    
    fn handle_keyboard_input(&mut self, _events: &mut EventBus) -> GameResult<()> {
        #[cfg(not(test))]
        {
            // Pause/resume
            if is_key_pressed(KeyCode::Space) {
                _events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::PauseGame(!self.paused)
                ));
            }
            
            // Speed controls
            if is_key_pressed(KeyCode::Key1) {
                _events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::SetGameSpeed(0.5)
                ));
            }
            if is_key_pressed(KeyCode::Key2) {
                _events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::SetGameSpeed(1.0)
                ));
            }
            if is_key_pressed(KeyCode::Key3) {
                _events.queue_event(GameEvent::PlayerCommand(
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
        // Validate input position
        if !world_pos.x.is_finite() || !world_pos.y.is_finite() {
            // Return screen center for invalid world position
            #[cfg(not(test))]
            return Vector2 { x: screen_width() / 2.0, y: screen_height() / 2.0 };
            #[cfg(test)]
            return Vector2 { x: DEFAULT_SCREEN_WIDTH / 2.0, y: DEFAULT_SCREEN_HEIGHT / 2.0 };
        }
        
        #[cfg(not(test))]
        {
            let screen_w = screen_width();
            let screen_h = screen_height();
            
            // Validate screen dimensions
            if screen_w <= 0.0 || screen_h <= 0.0 || !screen_w.is_finite() || !screen_h.is_finite() {
                return Vector2 { x: 0.0, y: 0.0 };
            }
            
            let result = Vector2 {
                x: (world_pos.x - self.camera_position.x) * self.zoom_level + screen_w / 2.0,
                y: (world_pos.y - self.camera_position.y) * self.zoom_level + screen_h / 2.0,
            };
            
            // Validate result
            if result.x.is_finite() && result.y.is_finite() {
                result
            } else {
                Vector2 { x: screen_w / 2.0, y: screen_h / 2.0 }
            }
        }
        #[cfg(test)]
        {
            let result = Vector2 {
                x: (world_pos.x - self.camera_position.x) * self.zoom_level + DEFAULT_SCREEN_WIDTH / 2.0,
                y: (world_pos.y - self.camera_position.y) * self.zoom_level + DEFAULT_SCREEN_HEIGHT / 2.0,
            };
            
            // Validate result
            if result.x.is_finite() && result.y.is_finite() {
                result
            } else {
                Vector2 { x: DEFAULT_SCREEN_WIDTH / 2.0, y: DEFAULT_SCREEN_HEIGHT / 2.0 }
            }
        }
    }
    
    fn screen_to_world(&self, screen_pos: Vector2) -> Vector2 {
        // Validate input position
        if !screen_pos.x.is_finite() || !screen_pos.y.is_finite() {
            return self.camera_position; // Return camera position for invalid screen position
        }
        
        // Prevent division by zero or very small zoom levels
        let safe_zoom = self.zoom_level.max(f32::EPSILON);
        
        #[cfg(not(test))]
        {
            let screen_w = screen_width();
            let screen_h = screen_height();
            
            // Validate screen dimensions
            if screen_w <= 0.0 || screen_h <= 0.0 || !screen_w.is_finite() || !screen_h.is_finite() {
                return self.camera_position;
            }
            
            let result = Vector2 {
                x: (screen_pos.x - screen_w / 2.0) / safe_zoom + self.camera_position.x,
                y: (screen_pos.y - screen_h / 2.0) / safe_zoom + self.camera_position.y,
            };
            
            // Validate result
            if result.x.is_finite() && result.y.is_finite() {
                result
            } else {
                self.camera_position
            }
        }
        #[cfg(test)]
        {
            let result = Vector2 {
                x: (screen_pos.x - DEFAULT_SCREEN_WIDTH / 2.0) / safe_zoom + self.camera_position.x,
                y: (screen_pos.y - DEFAULT_SCREEN_HEIGHT / 2.0) / safe_zoom + self.camera_position.y,
            };
            
            // Validate result
            if result.x.is_finite() && result.y.is_finite() {
                result
            } else {
                self.camera_position
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
    
    /// Calculates a planet's current position based on its orbital parameters.
    /// 
    /// This method implements a simplified circular orbit calculation using
    /// the planet's orbital period and phase offset.
    /// 
    /// # Arguments
    /// * `planet` - The planet whose position to calculate
    /// * `tick` - Current simulation tick
    /// * `interpolation` - Interpolation factor for smooth animation (0.0-1.0)
    /// 
    /// # Returns
    /// The planet's world position as a Vector2
    fn calculate_planet_position(&self, planet: &Planet, tick: u64, interpolation: f32) -> Vector2 {
        // Validate and clamp interpolation
        let clamped_interpolation = interpolation.clamp(0.0, 1.0);
        let time = tick as f32 + clamped_interpolation;
        
        // Validate planet orbital parameters
        if !planet.position.semi_major_axis.is_finite() || 
           !planet.position.period.is_finite() || 
           !planet.position.phase.is_finite() {
            // Return origin for invalid orbital parameters
            return Vector2 { x: 0.0, y: 0.0 };
        }
        
        // Prevent division by zero and handle very small periods
        let angle = if planet.position.period > f32::EPSILON {
            let cycles = time / planet.position.period;
            // Prevent potential overflow in angle calculation
            let normalized_cycles = cycles % 1000.0; // Normalize to prevent huge values
            normalized_cycles * 2.0 * PI + planet.position.phase
        } else {
            planet.position.phase
        };
        
        // Validate angle calculation
        if !angle.is_finite() {
            return Vector2 { x: 0.0, y: 0.0 };
        }
        
        // Scale factor for visual representation
        let scale = ORBITAL_SCALE_FACTOR;
        
        // Calculate position with bounds checking
        let x = planet.position.semi_major_axis * scale * angle.cos();
        let y = planet.position.semi_major_axis * scale * angle.sin();
        
        // Validate final result
        if x.is_finite() && y.is_finite() {
            Vector2 { x, y }
        } else {
            // Fallback to origin if calculation produces invalid result
            Vector2 { x: 0.0, y: 0.0 }
        }
    }
    
    /// Interpolates a ship's position along its trajectory.
    /// 
    /// This method provides smooth movement animation between trajectory waypoints
    /// using linear interpolation. It handles edge cases for ships that haven't
    /// departed yet or have already arrived.
    /// 
    /// # Arguments
    /// * `trajectory` - The ship's movement trajectory
    /// * `tick` - Current simulation tick
    /// * `interpolation` - Interpolation factor for smooth animation (0.0-1.0)
    /// 
    /// # Returns
    /// The interpolated world position as a Vector2
    fn interpolate_ship_position(&self, trajectory: &Trajectory, tick: u64, interpolation: f32) -> Vector2 {
        // Validate input parameters
        let clamped_interpolation = interpolation.clamp(0.0, 1.0);
        
        // Check for valid trajectory times
        if trajectory.departure_time > trajectory.arrival_time {
            // Invalid trajectory, return origin
            return trajectory.origin;
        }
        
        let current_time = tick as f32 + clamped_interpolation;
        let departure_time = trajectory.departure_time as f32;
        let arrival_time = trajectory.arrival_time as f32;
        
        // Handle edge cases
        if current_time < departure_time {
            return trajectory.origin;
        }
        if current_time >= arrival_time {
            return trajectory.destination;
        }
        
        let travel_time = arrival_time - departure_time;
        // Prevent division by zero for instantaneous travel
        if travel_time <= f32::EPSILON {
            return trajectory.destination;
        }
        
        let elapsed = current_time - departure_time;
        let progress = (elapsed / travel_time).clamp(0.0, 1.0);
        
        // Validate trajectory endpoints before interpolation
        if !trajectory.origin.x.is_finite() || !trajectory.origin.y.is_finite() ||
           !trajectory.destination.x.is_finite() || !trajectory.destination.y.is_finite() {
            // Invalid coordinates, return origin as fallback
            return trajectory.origin;
        }
        
        // Linear interpolation between origin and destination
        let result = Vector2 {
            x: trajectory.origin.x + (trajectory.destination.x - trajectory.origin.x) * progress,
            y: trajectory.origin.y + (trajectory.destination.y - trajectory.origin.y) * progress,
        };
        
        // Validate the result
        if result.x.is_finite() && result.y.is_finite() {
            result
        } else {
            // Return origin if interpolation produced invalid result
            trajectory.origin
        }
    }
    
    /// Draws a planet's orbital path as a circle.
    /// 
    /// Only draws orbits that are visible and within reasonable size limits
    /// to avoid performance issues with very large or very small orbits.
    fn draw_orbit(&self, planet: &Planet) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let center = Vector2 { x: 0.0, y: 0.0 };
            let screen_center = self.world_to_screen(center);
            let radius = planet.position.semi_major_axis * ORBITAL_SCALE_FACTOR * self.zoom_level;
            
            // Only draw orbits that are visible and reasonably sized
            const MIN_ORBIT_RADIUS: f32 = 10.0;
            const MAX_ORBIT_RADIUS: f32 = 3000.0;
            
            if radius > MIN_ORBIT_RADIUS && radius < MAX_ORBIT_RADIUS {
                let line_thickness = (1.0 * self.zoom_level).max(0.5);
                draw_circle_lines(
                    screen_center.x, 
                    screen_center.y, 
                    radius, 
                    line_thickness, 
                    Color::new(0.3, 0.3, 0.3, 0.4)
                );
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
    
    /// Calculates total empire resources across all player-controlled planets.
    /// 
    /// This method aggregates resources from all planets controlled by the player
    /// (faction ID 0) using saturating arithmetic to prevent integer overflow.
    /// 
    /// # Arguments
    /// * `state` - Current game state containing planet data
    /// 
    /// # Returns
    /// * `Ok(ResourceBundle)` - Total empire resources
    /// * `Err(GameError)` - If an error occurs during calculation
    fn calculate_empire_resources(&self, state: &GameState) -> GameResult<ResourceBundle> {
        let planets = state.planet_manager.get_all_planets();
        let mut total = ResourceBundle::default();
        
        const PLAYER_FACTION_ID: FactionId = 0;
        
        for planet in planets {
            if planet.controller == Some(PLAYER_FACTION_ID) {
                // Use saturating arithmetic to prevent overflow
                // This ensures the game remains stable even with extreme resource values
                total.minerals = total.minerals.saturating_add(planet.resources.current.minerals);
                total.food = total.food.saturating_add(planet.resources.current.food);
                total.energy = total.energy.saturating_add(planet.resources.current.energy);
                total.alloys = total.alloys.saturating_add(planet.resources.current.alloys);
                total.components = total.components.saturating_add(planet.resources.current.components);
                total.fuel = total.fuel.saturating_add(planet.resources.current.fuel);
            }
        }
        
        // Validate the final result
        total.validate_non_negative()?;
        
        Ok(total)
    }
    
    // Performance optimization methods
    fn invalidate_position_cache(&mut self) {
        self.cached_planet_positions.clear();
        self.cached_ship_positions.clear();
    }
    
    /// Validates and clamps zoom level to safe bounds.
    /// 
    /// Ensures zoom levels remain within acceptable limits to prevent
    /// rendering issues and maintain usability.
    fn validate_zoom_level(&mut self) {
        self.zoom_level = self.zoom_level.clamp(MIN_ZOOM, MAX_ZOOM);
        
        // Additional safety check for NaN or infinite values
        if !self.zoom_level.is_finite() {
            self.zoom_level = 1.0;
        }
    }
    
    /// Cleans up old cache entries to prevent memory bloat.
    /// 
    /// Removes cache entries that are more than a few ticks old,
    /// as they are likely no longer relevant. Also performs memory compaction
    /// when cache sizes become excessive.
    fn cleanup_old_cache_entries(&mut self, state: &GameState) {
        let current_tick = state.time_manager.get_current_tick();
        const MAX_CACHE_AGE: u64 = 5; // Keep cache for 5 ticks
        const MAX_CACHE_SIZE: usize = 1000; // Maximum entries before forced cleanup
        
        // Clean planet position cache
        let initial_planet_cache_size = self.cached_planet_positions.len();
        self.cached_planet_positions.retain(|_, (_, tick)| {
            current_tick.saturating_sub(*tick) <= MAX_CACHE_AGE
        });
        
        // Force cleanup if cache is still too large
        if self.cached_planet_positions.len() > MAX_CACHE_SIZE {
            self.cached_planet_positions.clear();
        }
        
        // Clean ship position cache
        let initial_ship_cache_size = self.cached_ship_positions.len();
        self.cached_ship_positions.retain(|_, (_, tick)| {
            current_tick.saturating_sub(*tick) <= MAX_CACHE_AGE
        });
        
        // Force cleanup if cache is still too large
        if self.cached_ship_positions.len() > MAX_CACHE_SIZE {
            self.cached_ship_positions.clear();
        }
        
        // Clean resource cache if it's too old
        if let Some((_, cached_tick)) = &self.cached_empire_resources {
            if current_tick.saturating_sub(*cached_tick) > MAX_CACHE_AGE {
                self.cached_empire_resources = None;
            }
        }
        
        // Shrink cache capacity if we cleaned up a significant amount
        if initial_planet_cache_size > 100 && self.cached_planet_positions.len() < initial_planet_cache_size / 2 {
            self.cached_planet_positions.shrink_to_fit();
        }
        if initial_ship_cache_size > 100 && self.cached_ship_positions.len() < initial_ship_cache_size / 2 {
            self.cached_ship_positions.shrink_to_fit();
        }
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
    
    /// Calculates screen bounds for improved frustum culling
    fn get_screen_bounds(&self) -> (f32, f32, f32, f32) {
        #[cfg(not(test))]
        {
            let margin = FRUSTUM_CULLING_MARGIN;
            (-margin, -margin, screen_width() + margin, screen_height() + margin)
        }
        #[cfg(test)]
        {
            let margin = FRUSTUM_CULLING_MARGIN;
            (-margin, -margin, DEFAULT_SCREEN_WIDTH + margin, DEFAULT_SCREEN_HEIGHT + margin)
        }
    }
    
    /// Checks if a position is within the given bounds
    fn is_position_in_bounds(&self, pos: Vector2, bounds: &(f32, f32, f32, f32)) -> bool {
        pos.x >= bounds.0 && pos.y >= bounds.1 && pos.x <= bounds.2 && pos.y <= bounds.3
    }
    
    /// Gets a ship's world position with caching and interpolation
    fn get_ship_world_position(&mut self, ship: &Ship, current_tick: u64, interpolation: f32) -> Vector2 {
        // Use cached position if available and valid
        if let Some((cached_pos, cached_tick)) = self.cached_ship_positions.get(&ship.id) {
            if *cached_tick == current_tick && ship.trajectory.is_none() {
                // Use cached position for stationary ships
                return *cached_pos;
            }
        }
        
        // Recalculate position
        let pos = if let Some(trajectory) = &ship.trajectory {
            self.interpolate_ship_position(trajectory, current_tick, interpolation)
        } else {
            ship.position
        };
        
        // Cache the new position
        self.cached_ship_positions.insert(ship.id, (pos, current_tick));
        pos
    }
    
    /// Gets ship display properties (color and size) based on class and ownership
    fn get_ship_display_properties(&self, ship: &Ship) -> (Color, f32) {
        let color = if ship.owner == 0 { BLUE } else { ORANGE };
        let base_size = match ship.ship_class {
            ShipClass::Scout => 3.0,
            ShipClass::Transport => 4.0,
            ShipClass::Warship => 6.0,
            ShipClass::Colony => 5.0,
        };
        (color, base_size)
    }
    
    /// Draws a ship shape based on its class with optimized rendering
    fn draw_ship_shape(&self, screen_pos: Vector2, size: f32, color: Color, ship_class: ShipClass) -> GameResult<()> {
        #[cfg(not(test))]
        {
            match ship_class {
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
        }
        Ok(())
    }
    
    /// Draws a trajectory line with distance and zoom optimizations
    fn draw_trajectory_line(&self, screen_pos: Vector2, trajectory: &Trajectory, color: Color) -> GameResult<()> {
        #[cfg(not(test))]
        {
            if self.zoom_level > ORBIT_VISIBILITY_THRESHOLD {
                let dest_screen = self.world_to_screen(trajectory.destination);
                let distance = ((dest_screen.x - screen_pos.x).powi(2) + 
                               (dest_screen.y - screen_pos.y).powi(2)).sqrt();
                
                // Only draw reasonable length trajectory lines
                if distance < MAX_TRAJECTORY_LINE_DISTANCE && distance > 1.0 {
                    let line_thickness = (1.0 * self.zoom_level).max(0.5);
                    draw_line(screen_pos.x, screen_pos.y, dest_screen.x, dest_screen.y, line_thickness, color);
                }
            }
        }
        Ok(())
    }
    
    /// Finds a planet at the given screen position.
    /// 
    /// This method iterates through all planets, converts their world positions
    /// to screen coordinates, and finds the closest planet within selection range.
    /// 
    /// # Arguments
    /// * `screen_pos` - The screen coordinate where the user clicked
    /// * `state` - Game state containing planet data
    /// 
    /// # Returns
    /// * `Ok(Some(PlanetId))` if a planet is found at the position
    /// * `Ok(None)` if no planet is found
    /// * `Err(GameError)` if an error occurs during selection
    fn find_planet_at_position(&self, _screen_pos: Vector2, _state: &GameState) -> GameResult<Option<PlanetId>> {
        let _planets = _state.planet_manager.get_all_planets();
        let _current_tick = _state.time_manager.get_current_tick();
        let selection_radius = _PLANET_SELECTION_RADIUS * self.zoom_level;
        
        let mut closest_planet: Option<PlanetId> = None;
        let mut closest_distance = f32::INFINITY;
        
        for planet in _planets {
            // Calculate planet's current screen position
            let world_pos = self.calculate_planet_position(planet, _current_tick, 0.0);
            let planet_screen_pos = self.world_to_screen(world_pos);
            
            // Calculate distance from click to planet center
            let dx = _screen_pos.x - planet_screen_pos.x;
            let dy = _screen_pos.y - planet_screen_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            // Check if within selection radius and closer than previous candidates
            if distance <= selection_radius && distance < closest_distance {
                closest_distance = distance;
                closest_planet = Some(planet.id);
            }
        }
        
        Ok(closest_planet)
    }
    
    /// Finds a ship at the given screen position.
    /// 
    /// This method iterates through all ships, converts their world positions
    /// to screen coordinates (including trajectory interpolation), and finds
    /// the closest ship within selection range. Ships have priority over planets
    /// due to their smaller size and more precise control requirements.
    /// 
    /// # Arguments
    /// * `screen_pos` - The screen coordinate where the user clicked
    /// * `state` - Game state containing ship data
    /// 
    /// # Returns
    /// * `Ok(Some(ShipId))` if a ship is found at the position
    /// * `Ok(None)` if no ship is found
    /// * `Err(GameError)` if an error occurs during selection
    fn find_ship_at_position(&self, _screen_pos: Vector2, _state: &GameState) -> GameResult<Option<ShipId>> {
        let _ships = _state.ship_manager.get_all_ships();
        let _current_tick = _state.time_manager.get_current_tick();
        let selection_radius = _SHIP_SELECTION_RADIUS * self.zoom_level;
        
        let mut closest_ship: Option<ShipId> = None;
        let mut closest_distance = f32::INFINITY;
        
        for ship in _ships {
            // Calculate ship's current screen position (with trajectory interpolation)
            let world_pos = if let Some(trajectory) = &ship.trajectory {
                self.interpolate_ship_position(trajectory, _current_tick, 0.0)
            } else {
                ship.position
            };
            
            let ship_screen_pos = self.world_to_screen(world_pos);
            
            // Skip ships that are off-screen for performance
            if !self.is_on_screen(ship_screen_pos) {
                continue;
            }
            
            // Calculate distance from click to ship center
            let dx = _screen_pos.x - ship_screen_pos.x;
            let dy = _screen_pos.y - ship_screen_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            // Check if within selection radius and closer than previous candidates
            if distance <= selection_radius && distance < closest_distance {
                closest_distance = distance;
                closest_ship = Some(ship.id);
            }
        }
        
        Ok(closest_ship)
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