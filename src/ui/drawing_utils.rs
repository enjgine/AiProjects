// src/ui/drawing_utils.rs
use crate::core::types::*;
use crate::core::GameResult;
use macroquad::prelude::*;
use std::f32::consts::PI;

const ORBIT_VISIBILITY_THRESHOLD: f32 = 0.3;
const MAX_TRAJECTORY_LINE_DISTANCE: f32 = 1000.0;

/// Utility functions for drawing game objects
pub struct DrawingUtils;

impl DrawingUtils {
    /// Draws orbital path for a planet
    pub fn draw_orbit(planet: &Planet, zoom_level: f32) -> GameResult<()> {
        #[cfg(not(test))]
        {
            if zoom_level > ORBIT_VISIBILITY_THRESHOLD {
                let orbit_radius = planet.position.semi_major_axis;
                let center = Vector2::new(0.0, 0.0);
                
                let segments = (32.0 * zoom_level).min(128.0).max(16.0) as i32;
                let mut last_point = Vector2::new(center.x + orbit_radius, center.y);
                
                for i in 1..=segments {
                    let angle = (i as f32 / segments as f32) * 2.0 * PI;
                    let point = Vector2::new(
                        center.x + orbit_radius * angle.cos(),
                        center.y + orbit_radius * angle.sin()
                    );
                    
                    draw_line(last_point.x, last_point.y, point.x, point.y, 1.0, 
                              Color::new(0.3, 0.3, 0.3, 0.5));
                    last_point = point;
                }
            }
        }
        Ok(())
    }
    
    /// Draws selection indicators and resource info for planets
    pub fn draw_planet_indicators(screen_pos: Vector2, planet: &Planet, zoom_level: f32) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let base_size = 8.0;
            let size = (base_size * zoom_level).max(4.0);
            
            // Planet circle
            let color = match planet.controller {
                Some(0) => GREEN,   // Player
                Some(_) => RED,     // Enemy
                None => GRAY,       // Neutral
            };
            
            draw_circle(screen_pos.x, screen_pos.y, size, color);
            
            // Resource indicators for detailed view
            if zoom_level > 1.0 && planet.controller.is_some() {
                let text_y = screen_pos.y + size + 15.0;
                let resource_text = format!("Pop: {}", planet.population.total);
                draw_text(&resource_text, screen_pos.x - 20.0, text_y, 10.0, WHITE);
            }
        }
        Ok(())
    }
    
    /// Draws ship shape based on class
    pub fn draw_ship_shape(screen_pos: Vector2, size: f32, color: Color, ship_class: ShipClass) -> GameResult<()> {
        #[cfg(not(test))]
        {
            match ship_class {
                ShipClass::Scout => {
                    // Triangle for fighters
                    let points = [
                        Vector2::new(screen_pos.x, screen_pos.y - size),
                        Vector2::new(screen_pos.x - size/2.0, screen_pos.y + size/2.0),
                        Vector2::new(screen_pos.x + size/2.0, screen_pos.y + size/2.0),
                    ];
                    for i in 0..3 {
                        let next = (i + 1) % 3;
                        draw_line(points[i].x, points[i].y, points[next].x, points[next].y, 2.0, color);
                    }
                }
                ShipClass::Transport | ShipClass::Warship => {
                    // Rectangle for larger ships
                    draw_rectangle(screen_pos.x - size/2.0, screen_pos.y - size/3.0, 
                                 size, size * 2.0/3.0, color);
                }
                ShipClass::Colony => {
                    // Large diamond for capital ships
                    let points = [
                        Vector2::new(screen_pos.x, screen_pos.y - size),
                        Vector2::new(screen_pos.x + size, screen_pos.y),
                        Vector2::new(screen_pos.x, screen_pos.y + size),
                        Vector2::new(screen_pos.x - size, screen_pos.y),
                    ];
                    for i in 0..4 {
                        let next = (i + 1) % 4;
                        draw_line(points[i].x, points[i].y, points[next].x, points[next].y, 3.0, color);
                    }
                }
                _ => {
                    // Default circle
                    draw_circle(screen_pos.x, screen_pos.y, size, color);
                }
            }
        }
        Ok(())
    }
    
    /// Draws trajectory line for ship movement
    pub fn draw_trajectory_line(screen_pos: Vector2, trajectory: &Trajectory, color: Color) -> GameResult<()> {
        #[cfg(not(test))]
        {
            let dx = trajectory.destination.x - screen_pos.x;
            let dy = trajectory.destination.y - screen_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < MAX_TRAJECTORY_LINE_DISTANCE && distance > 1.0 {
                // Draw dashed line to destination
                let segments = (distance / 20.0).min(20.0).max(3.0) as i32;
                for i in (0..segments).step_by(2) {
                    let t1 = i as f32 / segments as f32;
                    let t2 = ((i + 1) as f32 / segments as f32).min(1.0);
                    
                    let start = Vector2::new(
                        screen_pos.x + (trajectory.destination.x - screen_pos.x) * t1,
                        screen_pos.y + (trajectory.destination.y - screen_pos.y) * t1
                    );
                    let end = Vector2::new(
                        screen_pos.x + (trajectory.destination.x - screen_pos.x) * t2,
                        screen_pos.y + (trajectory.destination.y - screen_pos.y) * t2
                    );
                    
                    draw_line(start.x, start.y, end.x, end.y, 1.0, color);
                }
            }
        }
        Ok(())
    }
    
    /// Coordinate conversion helper - screen to world
    pub fn screen_to_world(screen_pos: Vector2, camera_position: Vector2, zoom_level: f32) -> Vector2 {
        #[cfg(not(test))]
        {
            let screen_w = screen_width();
            let screen_h = screen_height();
            
            if screen_w <= 0.0 || screen_h <= 0.0 || !screen_w.is_finite() || !screen_h.is_finite() {
                return camera_position;
            }
            
            let safe_zoom = zoom_level.max(f32::EPSILON);
            Vector2::new(
                (screen_pos.x - screen_w / 2.0) / safe_zoom + camera_position.x,
                (screen_pos.y - screen_h / 2.0) / safe_zoom + camera_position.y,
            )
        }
        #[cfg(test)]
        {
            const DEFAULT_SCREEN_WIDTH: f32 = 800.0;
            const DEFAULT_SCREEN_HEIGHT: f32 = 600.0;
            let safe_zoom = zoom_level.max(f32::EPSILON);
            Vector2::new(
                (screen_pos.x - DEFAULT_SCREEN_WIDTH / 2.0) / safe_zoom + camera_position.x,
                (screen_pos.y - DEFAULT_SCREEN_HEIGHT / 2.0) / safe_zoom + camera_position.y,
            )
        }
    }
    
    /// Coordinate conversion helper - world to screen 
    pub fn world_to_screen(world_pos: Vector2, camera_position: Vector2, zoom_level: f32) -> Vector2 {
        #[cfg(not(test))]
        {
            let screen_w = screen_width();
            let screen_h = screen_height();
            
            if screen_w <= 0.0 || screen_h <= 0.0 || !screen_w.is_finite() || !screen_h.is_finite() {
                return Vector2::new(0.0, 0.0);
            }
            
            Vector2::new(
                (world_pos.x - camera_position.x) * zoom_level + screen_w / 2.0,
                (world_pos.y - camera_position.y) * zoom_level + screen_h / 2.0,
            )
        }
        #[cfg(test)]
        {
            const DEFAULT_SCREEN_WIDTH: f32 = 800.0;
            const DEFAULT_SCREEN_HEIGHT: f32 = 600.0;
            Vector2::new(
                (world_pos.x - camera_position.x) * zoom_level + DEFAULT_SCREEN_WIDTH / 2.0,
                (world_pos.y - camera_position.y) * zoom_level + DEFAULT_SCREEN_HEIGHT / 2.0,
            )
        }
    }
}