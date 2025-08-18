// src/ui/input_handler.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Input handler responsible for converting user input to PlayerCommand events
/// Maintains strict EventBus architecture - only emits PlayerCommand events
pub struct InputHandler {
    /// Last recorded mouse position for delta calculations
    last_mouse_pos: (f32, f32),
    /// Starting position of mouse drag for camera movement
    mouse_drag_start: Option<(f32, f32)>,
    /// Key press states for debouncing and state tracking
    key_states: HashMap<KeyCode, KeyState>,
    /// Currently selected planet, if any
    selected_planet: Option<PlanetId>,
    /// Currently selected ship, if any
    selected_ship: Option<ShipId>,
    /// Camera position in world coordinates
    camera_position: Vector2,
    /// Camera zoom level (1.0 = default, >1.0 = zoomed in)
    zoom_level: f32,
    /// Game pause state for proper toggle behavior
    is_paused: bool,
    /// Last input event time for rate limiting
    last_input_time: Instant,
    /// Minimum time between input events to prevent spam
    input_cooldown: Duration,
}

#[derive(Debug, Clone)]
struct KeyState {
    pressed: bool,
    last_press_time: Instant,
    debounce_duration: Duration,
}

impl KeyState {
    fn new() -> Self {
        Self {
            pressed: false,
            last_press_time: Instant::now(),
            debounce_duration: Duration::from_millis(100), // 100ms debounce
        }
    }
    
    fn is_just_pressed(&self, current_time: Instant) -> bool {
        self.pressed && current_time.duration_since(self.last_press_time) > self.debounce_duration
    }
}

const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 10.0;
const ZOOM_SENSITIVITY: f32 = 0.1;
const CAMERA_DRAG_SENSITIVITY: f32 = 1.0;
const SELECTION_TOLERANCE: f32 = 20.0; // pixels
const INPUT_RATE_LIMIT_MS: u64 = 16; // ~60 FPS max input rate
const MAX_CAMERA_BOUND: f32 = 10000.0;
const MIN_GAME_SPEED: f32 = 0.1;
const MAX_GAME_SPEED: f32 = 8.0;

impl InputHandler {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: (0.0, 0.0),
            mouse_drag_start: None,
            key_states: HashMap::new(),
            selected_planet: None,
            selected_ship: None,
            camera_position: Vector2::new(0.0, 0.0),
            zoom_level: 1.0,
            is_paused: false,
            last_input_time: Instant::now(),
            input_cooldown: Duration::from_millis(INPUT_RATE_LIMIT_MS),
        }
    }
    
    /// Reset input handler state (useful for game state transitions)
    pub fn reset(&mut self) {
        self.selected_planet = None;
        self.selected_ship = None;
        self.key_states.clear();
        self.mouse_drag_start = None;
    }
    
    /// Get current camera position for rendering
    pub fn camera_position(&self) -> Vector2 {
        self.camera_position
    }
    
    /// Get current zoom level for rendering
    pub fn zoom_level(&self) -> f32 {
        self.zoom_level
    }
    
    /// Get currently selected planet
    pub fn selected_planet(&self) -> Option<PlanetId> {
        self.selected_planet
    }
    
    /// Get currently selected ship
    pub fn selected_ship(&self) -> Option<ShipId> {
        self.selected_ship
    }
    
    pub fn update(&mut self, events: &mut EventBus) -> GameResult<()> {
        // Rate limiting to prevent input spam
        let now = Instant::now();
        if now.duration_since(self.last_input_time) < self.input_cooldown {
            return Ok(());
        }
        
        self.update_key_states();
        self.update_mouse_input(events)?;
        self.update_keyboard_input(events)?;
        
        self.last_input_time = now;
        Ok(())
    }
    
    /// Update key state tracking for debouncing
    fn update_key_states(&mut self) {
        let now = Instant::now();
        
        // Update key states for important keys
        let keys_to_track = [
            KeyCode::Space, KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
            KeyCode::Escape, KeyCode::Delete, KeyCode::F1, KeyCode::F2, KeyCode::F3,
            KeyCode::A, KeyCode::D, KeyCode::S, KeyCode::W,
        ];
        
        for &key in &keys_to_track {
            let is_pressed = is_key_down(key);
            let key_state = self.key_states.entry(key).or_insert_with(KeyState::new);
            
            // Only update last_press_time on fresh key press
            if is_pressed && !key_state.pressed {
                key_state.last_press_time = now;
            }
            key_state.pressed = is_pressed;
        }
    }
    
    fn update_mouse_input(&mut self, events: &mut EventBus) -> GameResult<()> {
        let (mouse_x, mouse_y) = mouse_position();
        
        // Validate mouse coordinates
        if !mouse_x.is_finite() || !mouse_y.is_finite() {
            return Err(GameError::InvalidOperation("Invalid mouse coordinates".into()));
        }
        
        // Left click - entity selection
        if is_mouse_button_pressed(MouseButton::Left) {
            self.handle_left_click(mouse_x, mouse_y, events)?;
        }
        
        // Right click - context commands (move, attack, etc.)
        if is_mouse_button_pressed(MouseButton::Right) {
            self.handle_right_click(mouse_x, mouse_y, events)?;
        }
        
        // Mouse drag - camera movement (middle mouse or alt+left)
        let is_camera_drag = is_mouse_button_down(MouseButton::Middle) || 
                            (is_mouse_button_down(MouseButton::Left) && is_key_down(KeyCode::LeftAlt));
        
        if is_camera_drag {
            if self.mouse_drag_start.is_none() {
                self.mouse_drag_start = Some((mouse_x, mouse_y));
            }
            self.handle_camera_drag(mouse_x, mouse_y, events)?;
        } else {
            self.mouse_drag_start = None;
        }
        
        // Mouse wheel - zoom with bounds checking
        let wheel = mouse_wheel();
        if wheel.1 != 0.0 {
            self.handle_zoom(wheel.1)?;
        }
        
        self.last_mouse_pos = (mouse_x, mouse_y);
        Ok(())
    }
    
    fn update_keyboard_input(&mut self, events: &mut EventBus) -> GameResult<()> {
        // Game speed controls with validation
        if self.is_key_just_pressed(KeyCode::Key1) {
            self.emit_game_speed_command(0.5, events)?;
        }
        if self.is_key_just_pressed(KeyCode::Key2) {
            self.emit_game_speed_command(1.0, events)?;
        }
        if self.is_key_just_pressed(KeyCode::Key3) {
            self.emit_game_speed_command(2.0, events)?;
        }
        if self.is_key_just_pressed(KeyCode::Key4) {
            self.emit_game_speed_command(4.0, events)?;
        }
        
        // Pause toggle - proper state management
        if self.is_key_just_pressed(KeyCode::Space) {
            self.is_paused = !self.is_paused;
            events.queue_event(GameEvent::PlayerCommand(
                PlayerCommand::PauseGame(self.is_paused)
            ));
        }
        
        // Save/Load shortcuts with proper modifier key handling
        if self.is_ctrl_held() {
            if self.is_key_just_pressed(KeyCode::S) {
                events.queue_event(GameEvent::PlayerCommand(PlayerCommand::SaveGame));
            }
            if self.is_key_just_pressed(KeyCode::L) {
                events.queue_event(GameEvent::PlayerCommand(PlayerCommand::LoadGame));
            }
        }
        
        // Clear selection with Escape
        if self.is_key_just_pressed(KeyCode::Escape) {
            self.selected_planet = None;
            self.selected_ship = None;
        }
        
        // Delete selected ship (if any)
        if self.is_key_just_pressed(KeyCode::Delete) && self.selected_ship.is_some() {
            // Note: This would need a new PlayerCommand variant for ship deletion
            // For now, we'll clear the selection
            self.selected_ship = None;
        }
        
        Ok(())
    }
    
    /// Check if a key was just pressed (debounced)
    fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        // Use macroquad's built-in key press detection with our debouncing
        if let Some(key_state) = self.key_states.get(&key) {
            // Key must be currently pressed and passed debounce time
            key_state.pressed && is_key_pressed(key) && 
            key_state.is_just_pressed(Instant::now())
        } else {
            // Fallback to direct macroquad detection for untracked keys
            is_key_pressed(key)
        }
    }
    
    /// Check if control modifier is held
    fn is_ctrl_held(&self) -> bool {
        is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
    }
    
    /// Emit game speed command with validation
    fn emit_game_speed_command(&self, speed: f32, events: &mut EventBus) -> GameResult<()> {
        if !speed.is_finite() || speed < MIN_GAME_SPEED || speed > MAX_GAME_SPEED {
            return Err(GameError::InvalidOperation(
                format!("Invalid game speed: {}. Must be between {} and {}", 
                    speed, MIN_GAME_SPEED, MAX_GAME_SPEED)
            ));
        }
        
        events.queue_event(GameEvent::PlayerCommand(
            PlayerCommand::SetGameSpeed(speed)
        ));
        Ok(())
    }
    
    fn handle_left_click(&mut self, x: f32, y: f32, events: &mut EventBus) -> GameResult<()> {
        let world_pos = self.screen_to_world(x, y)?;
        
        // Check for ship selection first (ships have priority over planets)
        if let Some(ship_id) = self.find_ship_at_position(world_pos) {
            self.selected_ship = Some(ship_id);
            self.selected_planet = None; // Clear planet selection
            events.queue_event(GameEvent::PlayerCommand(
                PlayerCommand::SelectShip(ship_id)
            ));
            return Ok(());
        }
        
        // Check for planet selection
        if let Some(planet_id) = self.find_planet_at_position(world_pos) {
            self.selected_planet = Some(planet_id);
            self.selected_ship = None; // Clear ship selection
            events.queue_event(GameEvent::PlayerCommand(
                PlayerCommand::SelectPlanet(planet_id)
            ));
            return Ok(());
        }
        
        // Click on empty space - clear selection
        self.selected_planet = None;
        self.selected_ship = None;
        
        Ok(())
    }
    
    fn handle_right_click(&mut self, x: f32, y: f32, events: &mut EventBus) -> GameResult<()> {
        let world_pos = self.screen_to_world(x, y)?;
        
        // Right-click behavior depends on what's currently selected
        if let Some(ship_id) = self.selected_ship {
            // Check if right-clicking on a target for attack/interaction
            if let Some(target_ship_id) = self.find_ship_at_position(world_pos) {
                if target_ship_id != ship_id { // Can't target self
                    events.queue_event(GameEvent::PlayerCommand(
                        PlayerCommand::AttackTarget {
                            attacker: ship_id,
                            target: target_ship_id,
                        }
                    ));
                }
            } else if let Some(planet_id) = self.find_planet_at_position(world_pos) {
                // Right-click planet with ship selected - could be colonize or load/unload
                // For now, we'll emit a colonize command
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::ColonizePlanet {
                        ship: ship_id,
                        planet: planet_id,
                    }
                ));
            } else {
                // Right-click empty space with ship selected - move command
                events.queue_event(GameEvent::PlayerCommand(
                    PlayerCommand::MoveShip {
                        ship: ship_id,
                        target: world_pos,
                    }
                ));
            }
        }
        // Note: Could add planet-specific right-click actions here in the future
        
        Ok(())
    }
    
    fn handle_camera_drag(&mut self, x: f32, y: f32, _events: &mut EventBus) -> GameResult<()> {
        if let Some((start_x, start_y)) = self.mouse_drag_start {
            let dx = (start_x - x) * CAMERA_DRAG_SENSITIVITY / self.zoom_level;
            let dy = (start_y - y) * CAMERA_DRAG_SENSITIVITY / self.zoom_level;
            
            // Validate movement deltas
            if !dx.is_finite() || !dy.is_finite() {
                return Err(GameError::InvalidOperation("Invalid camera movement".into()));
            }
            
            // Apply camera movement with reasonable bounds
            self.camera_position.x = (self.camera_position.x + dx).clamp(-MAX_CAMERA_BOUND, MAX_CAMERA_BOUND);
            self.camera_position.y = (self.camera_position.y + dy).clamp(-MAX_CAMERA_BOUND, MAX_CAMERA_BOUND);
            
            // Update drag start for smooth continuous movement
            self.mouse_drag_start = Some((x, y));
        }
        
        Ok(())
    }
    
    fn handle_zoom(&mut self, delta: f32) -> GameResult<()> {
        // Validate zoom delta
        if !delta.is_finite() {
            return Err(GameError::InvalidOperation("Invalid zoom delta".into()));
        }
        
        // Apply zoom with sensitivity and bounds checking
        let zoom_factor = 1.0 + (delta * ZOOM_SENSITIVITY);
        let new_zoom = self.zoom_level * zoom_factor;
        
        // Clamp zoom to reasonable bounds
        self.zoom_level = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        
        Ok(())
    }
    
    /// Convert screen coordinates to world coordinates
    fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> GameResult<Vector2> {
        if !screen_x.is_finite() || !screen_y.is_finite() {
            return Err(GameError::InvalidOperation("Invalid screen coordinates".into()));
        }
        
        let screen_center_x = screen_width() * 0.5;
        let screen_center_y = screen_height() * 0.5;
        
        let world_x = self.camera_position.x + (screen_x - screen_center_x) / self.zoom_level;
        let world_y = self.camera_position.y + (screen_y - screen_center_y) / self.zoom_level;
        
        Ok(Vector2::new(world_x, world_y))
    }
    
    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vector2) -> (f32, f32) {
        let screen_center_x = screen_width() * 0.5;
        let screen_center_y = screen_height() * 0.5;
        
        let screen_x = screen_center_x + (world_pos.x - self.camera_position.x) * self.zoom_level;
        let screen_y = screen_center_y + (world_pos.y - self.camera_position.y) * self.zoom_level;
        
        (screen_x, screen_y)
    }
    
    /// Find ship at given world position (placeholder - needs actual ship manager integration)
    /// Returns None until ship manager is integrated
    fn find_ship_at_position(&self, world_pos: Vector2) -> Option<ShipId> {
        // Validate input position
        if !world_pos.x.is_finite() || !world_pos.y.is_finite() {
            return None;
        }
        
        // TODO: This requires integration with ship manager to query ship positions
        // The implementation should:
        // 1. Query ship manager for ships within SELECTION_TOLERANCE of world_pos
        // 2. Return the closest ship ID within tolerance, prioritizing player-owned ships
        // 3. Use efficient spatial indexing for large numbers of ships
        None
    }
    
    /// Find planet at given world position (placeholder - needs actual planet manager integration)
    /// Returns None until planet manager is integrated
    fn find_planet_at_position(&self, world_pos: Vector2) -> Option<PlanetId> {
        // Validate input position
        if !world_pos.x.is_finite() || !world_pos.y.is_finite() {
            return None;
        }
        
        // TODO: This requires integration with planet manager to query planet positions
        // The implementation should:
        // 1. Query planet manager for planets within SELECTION_TOLERANCE of world_pos
        // 2. Calculate orbital positions based on current game tick
        // 3. Return the closest planet ID within tolerance
        None
    }
    
    /// Set camera position (for external camera control)
    pub fn set_camera_position(&mut self, position: Vector2) -> GameResult<()> {
        if !position.x.is_finite() || !position.y.is_finite() {
            return Err(GameError::InvalidOperation("Invalid camera position coordinates".into()));
        }
        
        self.camera_position.x = position.x.clamp(-MAX_CAMERA_BOUND, MAX_CAMERA_BOUND);
        self.camera_position.y = position.y.clamp(-MAX_CAMERA_BOUND, MAX_CAMERA_BOUND);
        Ok(())
    }
    
    /// Set zoom level with bounds checking
    pub fn set_zoom_level(&mut self, zoom: f32) -> GameResult<()> {
        if !zoom.is_finite() || zoom <= 0.0 {
            return Err(GameError::InvalidOperation("Invalid zoom level".into()));
        }
        
        self.zoom_level = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        Ok(())
    }
    
    /// Update game pause state (called when pause state changes externally)
    pub fn set_pause_state(&mut self, is_paused: bool) {
        self.is_paused = is_paused;
    }
    
    /// Get current pause state
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }
    
    /// Clear current selection (useful for UI state management)
    pub fn clear_selection(&mut self) {
        self.selected_planet = None;
        self.selected_ship = None;
    }
}