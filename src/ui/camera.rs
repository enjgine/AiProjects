// src/ui/camera.rs
//! Camera system for managing viewport and world-to-screen transformations.
//!
//! The Camera handles:
//! - World coordinate to screen coordinate conversion
//! - Zoom level management with constraints
//! - Camera movement and positioning
//! - Viewport culling calculations

use crate::core::types::{Vector2, GameResult, GameError};
use crate::ui::ui_config::{MIN_ZOOM, MAX_ZOOM, CAMERA_SPEED_BASE};
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vector2,
    zoom_level: f32,
    screen_width: f32,
    screen_height: f32,
    /// Maximum allowed camera position from origin (prevents infinite scrolling)
    max_world_bounds: f32,
}

impl Camera {
    /// World bounds limit to prevent excessive camera movement
    const MAX_WORLD_BOUNDS: f32 = 100000.0;
    /// Minimum zoom level to prevent division issues
    const MIN_ZOOM_SAFE: f32 = 0.001;

    pub fn new() -> Self {
        Self {
            position: Vector2 { x: 0.0, y: 0.0 },
            zoom_level: 1.0,
            screen_width: 800.0,  // Default, updated in update_screen_size
            screen_height: 600.0,
            max_world_bounds: Self::MAX_WORLD_BOUNDS,
        }
    }
    
    /// Create camera with custom world bounds
    pub fn new_with_bounds(max_bounds: f32) -> GameResult<Self> {
        if max_bounds <= 0.0 {
            return Err(GameError::InvalidInput("Camera bounds must be positive".to_string()));
        }
        
        Ok(Self {
            position: Vector2 { x: 0.0, y: 0.0 },
            zoom_level: 1.0,
            screen_width: 800.0,
            screen_height: 600.0,
            max_world_bounds: max_bounds,
        })
    }
    
    /// Get current zoom level (read-only access)
    pub fn zoom_level(&self) -> f32 {
        self.zoom_level
    }
    
    /// Update screen dimensions (call once per frame)
    pub fn update_screen_size(&mut self) {
        #[cfg(not(test))]
        {
            self.screen_width = screen_width();
            self.screen_height = screen_height();
        }
    }
    
    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vector2) -> Vector2 {
        // Use safe zoom level to prevent overflow/underflow
        let safe_zoom = self.zoom_level.max(Self::MIN_ZOOM_SAFE);
        Vector2 {
            x: (world_pos.x - self.position.x) * safe_zoom + self.screen_width * 0.5,
            y: (world_pos.y - self.position.y) * safe_zoom + self.screen_height * 0.5,
        }
    }
    
    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vector2) -> GameResult<Vector2> {
        // Prevent division by zero
        if self.zoom_level <= Self::MIN_ZOOM_SAFE {
            return Err(GameError::InvalidInput("Zoom level too small for screen conversion".to_string()));
        }
        
        Ok(Vector2 {
            x: (screen_pos.x - self.screen_width * 0.5) / self.zoom_level + self.position.x,
            y: (screen_pos.y - self.screen_height * 0.5) / self.zoom_level + self.position.y,
        })
    }
    
    /// Check if a world position is visible on screen (with margin)
    /// Optimized version that avoids coordinate transformation when possible
    pub fn is_visible(&self, world_pos: Vector2, margin: f32) -> bool {
        // Calculate world-space bounds for visibility check (more efficient)
        let safe_zoom = self.zoom_level.max(Self::MIN_ZOOM_SAFE);
        let margin_world = margin / safe_zoom;
        let half_screen_world_x = (self.screen_width * 0.5) / safe_zoom;
        let half_screen_world_y = (self.screen_height * 0.5) / safe_zoom;
        
        // Check bounds in world space
        world_pos.x >= self.position.x - half_screen_world_x - margin_world
            && world_pos.x <= self.position.x + half_screen_world_x + margin_world
            && world_pos.y >= self.position.y - half_screen_world_y - margin_world
            && world_pos.y <= self.position.y + half_screen_world_y + margin_world
    }
    
    /// Move camera by delta in world coordinates with bounds checking
    pub fn move_by(&mut self, delta: Vector2) -> GameResult<()> {
        let new_pos = Vector2 {
            x: self.position.x + delta.x,
            y: self.position.y + delta.y,
        };
        self.set_position(new_pos)
    }
    
    /// Set camera position in world coordinates with bounds validation
    pub fn set_position(&mut self, position: Vector2) -> GameResult<()> {
        // Validate position is within world bounds
        if position.x.abs() > self.max_world_bounds || position.y.abs() > self.max_world_bounds {
            return Err(GameError::InvalidInput(
                format!("Camera position ({}, {}) exceeds world bounds (±{})", 
                    position.x, position.y, self.max_world_bounds)
            ));
        }
        
        // Check for NaN or infinite values
        if !position.x.is_finite() || !position.y.is_finite() {
            return Err(GameError::InvalidInput("Camera position contains invalid float values".to_string()));
        }
        
        self.position = position;
        Ok(())
    }
    
    /// Adjust zoom level with constraints and validation
    pub fn adjust_zoom(&mut self, delta: f32) -> GameResult<()> {
        if !delta.is_finite() {
            return Err(GameError::InvalidInput("Zoom delta must be finite".to_string()));
        }
        
        // Prevent extreme zoom changes that could cause overflow
        let clamped_delta = delta.clamp(-0.9, 10.0);
        let new_zoom = (self.zoom_level * (1.0 + clamped_delta)).clamp(MIN_ZOOM, MAX_ZOOM);
        self.set_zoom(new_zoom)
    }
    
    /// Set zoom level with constraints and validation
    pub fn set_zoom(&mut self, zoom: f32) -> GameResult<()> {
        if !zoom.is_finite() || zoom <= 0.0 {
            return Err(GameError::InvalidInput("Zoom level must be positive and finite".to_string()));
        }
        
        self.zoom_level = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        Ok(())
    }
    
    /// Get current movement speed adjusted for zoom (safe division)
    pub fn get_movement_speed(&self) -> f32 {
        let safe_zoom = self.zoom_level.max(Self::MIN_ZOOM_SAFE);
        CAMERA_SPEED_BASE / safe_zoom
    }
    
    /// Calculate visible world bounds (returns None if zoom too small)
    pub fn get_visible_bounds(&self) -> Option<(Vector2, Vector2)> {
        let top_left = self.screen_to_world(Vector2 { x: 0.0, y: 0.0 }).ok()?;
        let bottom_right = self.screen_to_world(Vector2 {
            x: self.screen_width,
            y: self.screen_height,
        }).ok()?;
        Some((top_left, bottom_right))
    }
    
    /// Focus camera on a world position with bounds validation
    pub fn focus_on(&mut self, world_pos: Vector2) -> GameResult<()> {
        self.set_position(world_pos)
    }
    
    /// Get viewport center in world coordinates
    pub fn get_center(&self) -> Vector2 {
        self.position
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_camera_new() {
        let camera = Camera::new();
        assert_eq!(camera.position.x, 0.0);
        assert_eq!(camera.position.y, 0.0);
        assert_eq!(camera.zoom_level, 1.0);
    }
    
    #[test]
    fn test_world_to_screen_conversion() {
        let camera = Camera::new();
        let world_pos = Vector2 { x: 0.0, y: 0.0 };
        let screen_pos = camera.world_to_screen(world_pos);
        
        // Origin should be at screen center
        assert_eq!(screen_pos.x, 400.0);
        assert_eq!(screen_pos.y, 300.0);
    }
    
    #[test]
    fn test_screen_to_world_conversion() {
        let camera = Camera::new();
        let screen_pos = Vector2 { x: 400.0, y: 300.0 };
        let world_pos = camera.screen_to_world(screen_pos).unwrap();
        
        // Screen center should be at world origin
        assert_eq!(world_pos.x, 0.0);
        assert_eq!(world_pos.y, 0.0);
    }
    
    #[test]
    fn test_zoom_constraints() {
        let mut camera = Camera::new();
        
        // Test minimum zoom
        camera.adjust_zoom(-10.0).unwrap();
        assert_eq!(camera.zoom_level(), MIN_ZOOM);
        
        // Test maximum zoom
        camera.set_zoom(100.0).unwrap();
        assert_eq!(camera.zoom_level(), MAX_ZOOM);
    }
    
    #[test]
    fn test_camera_movement() {
        let mut camera = Camera::new();
        let delta = Vector2 { x: 10.0, y: -5.0 };
        
        camera.move_by(delta).unwrap();
        assert_eq!(camera.position.x, 10.0);
        assert_eq!(camera.position.y, -5.0);
    }
    
    #[test]
    fn test_visibility_check() {
        let camera = Camera::new();
        
        // Point at world origin should be visible (screen center)
        assert!(camera.is_visible(Vector2 { x: 0.0, y: 0.0 }, 0.0));
        
        // Point far away should not be visible
        assert!(!camera.is_visible(Vector2 { x: 10000.0, y: 10000.0 }, 0.0));
    }
    
    #[test]
    fn test_bounds_validation() {
        let mut camera = Camera::new();
        
        // Test world bounds enforcement
        let far_position = Vector2 { x: 200000.0, y: 200000.0 };
        assert!(camera.set_position(far_position).is_err());
        
        // Test valid position
        let valid_position = Vector2 { x: 1000.0, y: 1000.0 };
        assert!(camera.set_position(valid_position).is_ok());
    }
    
    #[test]
    fn test_zoom_edge_cases() {
        let mut camera = Camera::new();
        
        // Test invalid zoom values
        assert!(camera.set_zoom(0.0).is_err());
        assert!(camera.set_zoom(-1.0).is_err());
        assert!(camera.set_zoom(f32::NAN).is_err());
        assert!(camera.set_zoom(f32::INFINITY).is_err());
        
        // Test valid zoom
        assert!(camera.set_zoom(2.0).is_ok());
        assert_eq!(camera.zoom_level(), 2.0);
    }
    
    #[test]
    fn test_screen_to_world_edge_cases() {
        let mut camera = Camera::new();
        
        // Set very small zoom level
        camera.zoom_level = 0.0001;
        let screen_pos = Vector2 { x: 400.0, y: 300.0 };
        
        // Should return error due to very small zoom
        assert!(camera.screen_to_world(screen_pos).is_err());
    }
    
    #[test]
    fn test_movement_speed_safety() {
        let mut camera = Camera::new();
        
        // Test with normal zoom
        let normal_speed = camera.get_movement_speed();
        assert!(normal_speed.is_finite());
        
        // Test with very small zoom (should not panic or return infinity)
        camera.zoom_level = 0.0001;
        let small_zoom_speed = camera.get_movement_speed();
        assert!(small_zoom_speed.is_finite());
        assert!(small_zoom_speed > 0.0);
    }
    
    #[test]
    fn test_visible_bounds_safety() {
        let mut camera = Camera::new();
        
        // Normal case should work
        assert!(camera.get_visible_bounds().is_some());
        
        // Very small zoom should return None
        camera.zoom_level = 0.0001;
        assert!(camera.get_visible_bounds().is_none());
    }
}