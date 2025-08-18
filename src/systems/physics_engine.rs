// src/systems/physics_engine.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use crate::core::events::*;
use std::collections::HashMap;

/// PhysicsEngine handles orbital mechanics, ship trajectories, and spatial physics
/// in the game world. It operates on a deterministic fixed timestep system.
/// 
/// Key responsibilities:
/// - Calculate orbital positions for all planets based on their OrbitalElements
/// - Manage ship trajectories and detect arrivals
/// - Calculate transfer windows between planets for optimal ship movement
/// - Provide spatial position queries for rendering and game logic
pub struct PhysicsEngine {
    /// Cache of calculated orbital positions to avoid redundant calculations
    orbital_cache: HashMap<PlanetId, Vector2>,
    /// Active transfer windows with their expiration times
    transfer_windows: HashMap<(PlanetId, PlanetId), u64>,
    /// Ship trajectories indexed by ship ID
    trajectories: HashMap<ShipId, Trajectory>,
    /// Current simulation tick for deterministic calculations
    current_tick: u64,
    /// Flag to defer event emission until update() phase
    needs_tick_processing: bool,
    /// Cached distances between planets for performance
    planet_distances: HashMap<(PlanetId, PlanetId), f32>,
    /// Maximum number of planets to avoid unbounded iterations
    max_planets: u32,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            orbital_cache: HashMap::with_capacity(64), // Pre-allocate for performance
            transfer_windows: HashMap::with_capacity(32),
            trajectories: HashMap::with_capacity(128),
            current_tick: 0,
            needs_tick_processing: false,
            planet_distances: HashMap::with_capacity(64),
            max_planets: 100, // Safety limit to prevent unbounded operations
        }
    }
    
    pub fn update(&mut self, _delta: f32, event_bus: &mut EventBus) -> GameResult<()> {
        // Process any pending tick processing
        if self.needs_tick_processing {
            self.process_tick(event_bus)?;
            self.needs_tick_processing = false;
        }
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::SimulationEvent(SimulationEvent::TickCompleted(tick)) => {
                // Validate tick progression to ensure deterministic behavior
                if *tick < self.current_tick {
                    return Err(GameError::InvalidOperation(
                        format!("Tick went backwards: {} < {}", tick, self.current_tick)
                    ));
                }
                self.current_tick = *tick;
                // Store that we need to process a tick, but emit events in update()
                self.needs_tick_processing = true;
            }
            GameEvent::PlayerCommand(PlayerCommand::MoveShip { ship, target }) => {
                self.plan_trajectory(*ship, *target)?;
            }
            GameEvent::SimulationEvent(SimulationEvent::ShipArrived { ship, destination: _ }) => {
                // Clean up completed trajectory when ship arrives
                self.trajectories.remove(ship);
            }
            _ => {}
        }
        Ok(())
    }
    
    fn process_tick(&mut self, event_bus: &mut EventBus) -> GameResult<()> {
        // Update orbital positions based on current tick
        self.update_orbital_positions(event_bus)?;
        
        // Update ship trajectories and check arrivals
        self.update_ship_trajectories(event_bus)?;
        
        // Check for transfer windows
        self.check_transfer_windows(event_bus)?;
        
        Ok(())
    }
    
    fn update_orbital_positions(&mut self, event_bus: &mut EventBus) -> GameResult<()> {
        // Clear previous cache to ensure fresh calculations
        self.orbital_cache.clear();
        
        // Note: In actual implementation, we'd get planet data from PlanetManager
        // For now this calculates orbital positions for demonstration planets
        // Each planet follows: position = (radius * cos(phase + time/period), radius * sin(phase + time/period))
        
        // Demonstrate orbital calculations for first few planets
        for planet_id in 0..5.min(self.max_planets) {
            let orbital_elements = self.get_demo_orbital_elements(planet_id);
            let position = self.calculate_orbital_position(&orbital_elements, self.current_tick);
            
            self.orbital_cache.insert(planet_id, position);
            
            // Emit position update event for other systems
            event_bus.queue_event(GameEvent::StateChanged(
                StateChange::PlanetUpdated(planet_id)
            ));
        }
        
        Ok(())
    }
    
    /// Get demonstration orbital elements for a planet ID
    /// In production, this would come from PlanetManager
    fn get_demo_orbital_elements(&self, planet_id: PlanetId) -> OrbitalElements {
        match planet_id {
            0 => OrbitalElements { semi_major_axis: 1.0, period: 100.0, phase: 0.0 },
            1 => OrbitalElements { semi_major_axis: 1.5, period: 150.0, phase: 0.5 },
            2 => OrbitalElements { semi_major_axis: 2.0, period: 200.0, phase: 1.0 },
            3 => OrbitalElements { semi_major_axis: 2.5, period: 300.0, phase: 1.5 },
            4 => OrbitalElements { semi_major_axis: 3.0, period: 400.0, phase: 2.0 },
            _ => OrbitalElements::default(),
        }
    }
    
    pub fn calculate_orbital_position(&self, orbital_elements: &OrbitalElements, tick: u64) -> Vector2 {
        // Validate orbital elements to prevent invalid calculations
        if orbital_elements.period <= 0.0 {
            // Return origin for invalid period to avoid division by zero
            return Vector2::default();
        }
        
        if orbital_elements.semi_major_axis < 0.0 {
            // Return origin for negative radius
            return Vector2::default();
        }
        
        // Calculate orbital position using circular orbit approximation
        let time_ratio = (tick as f32) / orbital_elements.period;
        let angle = orbital_elements.phase + 2.0 * std::f32::consts::PI * time_ratio;
        
        Vector2 {
            x: orbital_elements.semi_major_axis * angle.cos(),
            y: orbital_elements.semi_major_axis * angle.sin(),
        }
    }
    
    fn update_ship_trajectories(&mut self, event_bus: &mut EventBus) -> GameResult<()> {
        let mut completed_trajectories = Vec::new();
        
        for (ship_id, trajectory) in &self.trajectories {
            if self.current_tick >= trajectory.arrival_time {
                // Ship has arrived
                event_bus.queue_event(GameEvent::SimulationEvent(
                    SimulationEvent::ShipArrived {
                        ship: *ship_id,
                        destination: trajectory.destination,
                    }
                ));
                completed_trajectories.push(*ship_id);
            }
        }
        
        // Remove completed trajectories
        for ship_id in completed_trajectories {
            self.trajectories.remove(&ship_id);
        }
        
        Ok(())
    }
    
    /// Get the current interpolated position of a ship
    /// Returns the ship's position along its trajectory, or base_position if not moving
    pub fn get_ship_position(&self, ship_id: ShipId, base_position: Vector2) -> Vector2 {
        if let Some(trajectory) = self.trajectories.get(&ship_id) {
            self.interpolate_trajectory_position(trajectory, self.current_tick)
        } else {
            base_position
        }
    }
    
    fn interpolate_trajectory_position(&self, trajectory: &Trajectory, current_tick: u64) -> Vector2 {
        // Handle edge cases for trajectory timing
        if current_tick <= trajectory.departure_time {
            return trajectory.origin;
        }
        if current_tick >= trajectory.arrival_time {
            return trajectory.destination;
        }
        
        let total_time = trajectory.arrival_time - trajectory.departure_time;
        if total_time == 0 {
            // Instant travel edge case
            return trajectory.destination;
        }
        
        let elapsed_time = current_tick - trajectory.departure_time;
        let progress = (elapsed_time as f32) / (total_time as f32);
        
        // Clamp progress to [0, 1] for safety
        let progress = progress.max(0.0).min(1.0);
        
        // Linear interpolation between origin and destination
        Vector2 {
            x: trajectory.origin.x + (trajectory.destination.x - trajectory.origin.x) * progress,
            y: trajectory.origin.y + (trajectory.destination.y - trajectory.origin.y) * progress,
        }
    }
    
    fn check_transfer_windows(&mut self, event_bus: &mut EventBus) -> GameResult<()> {
        // Calculate transfer windows between planets based on orbital positions
        // Transfer windows occur when planets are properly aligned for efficient travel
        
        // Clean up expired transfer windows first
        self.transfer_windows.retain(|_, &mut expiry_tick| expiry_tick > self.current_tick);
        
        // Check for new transfer windows every 25 ticks for better responsiveness
        if self.current_tick % 25 == 0 {
            let max_planets = 5.min(self.max_planets); // Safety limit
            
            for planet_a in 0..max_planets {
                for planet_b in (planet_a + 1)..max_planets {
                    // Check if this pair already has an active transfer window
                    let pair = (planet_a, planet_b);
                    if self.transfer_windows.contains_key(&pair) {
                        continue;
                    }
                    
                    // Calculate if planets are aligned for efficient transfer
                    if self.are_planets_aligned_for_transfer(planet_a, planet_b) {
                        let window_duration = 30; // 30 tick window
                        self.transfer_windows.insert(pair, self.current_tick + window_duration);
                        
                        event_bus.queue_event(GameEvent::SimulationEvent(
                            SimulationEvent::TransferWindowOpen {
                                from: planet_a,
                                to: planet_b,
                            }
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if two planets are aligned for an efficient transfer window
    fn are_planets_aligned_for_transfer(&self, planet_a: PlanetId, planet_b: PlanetId) -> bool {
        // Get orbital positions from cache
        let pos_a = self.orbital_cache.get(&planet_a);
        let pos_b = self.orbital_cache.get(&planet_b);
        
        match (pos_a, pos_b) {
            (Some(pos_a), Some(pos_b)) => {
                // Calculate angular separation
                let angle_a = pos_a.y.atan2(pos_a.x);
                let angle_b = pos_b.y.atan2(pos_b.x);
                let angular_diff = (angle_a - angle_b).abs();
                
                // Normalize to [0, PI]
                let normalized_diff = if angular_diff > std::f32::consts::PI {
                    2.0 * std::f32::consts::PI - angular_diff
                } else {
                    angular_diff
                };
                
                // Transfer window occurs when planets are somewhat aligned (within 45 degrees)
                normalized_diff < std::f32::consts::PI / 4.0
            }
            _ => false, // Can't determine alignment without positions
        }
    }
    
    fn plan_trajectory(&mut self, ship_id: ShipId, target: Vector2) -> GameResult<()> {
        // Validate input parameters
        if target.x.is_nan() || target.y.is_nan() || target.x.is_infinite() || target.y.is_infinite() {
            return Err(GameError::InvalidTarget(
                "Target position contains invalid coordinates".to_string()
            ));
        }
        
        // For now, using placeholder positions - in real implementation we'd get from ShipManager
        let origin = Vector2 { x: 0.0, y: 0.0 }; // Would get from ship's current position
        let distance = self.calculate_distance(origin, target);
        
        // Validate distance is reasonable
        if distance <= 0.0 {
            return Err(GameError::InvalidOperation(
                "Cannot plan trajectory with zero distance".to_string()
            ));
        }
        
        if distance > 1000.0 { // Maximum reasonable travel distance
            return Err(GameError::InvalidTarget(
                "Target is too far away".to_string()
            ));
        }
        
        // Calculate travel time based on distance (base speed: 10 units per tick)
        let base_speed = 10.0;
        let travel_time = (distance / base_speed).ceil() as u64;
        
        // Ensure minimum travel time of 1 tick
        let travel_time = travel_time.max(1);
        
        // Calculate fuel cost
        let fuel_cost = self.calculate_fuel_cost(origin, target, ship_id);
        
        // Validate fuel cost is reasonable
        if fuel_cost < 0.0 {
            return Err(GameError::SystemError(
                "Fuel cost calculation resulted in negative value".to_string()
            ));
        }
        
        let trajectory = Trajectory {
            origin,
            destination: target,
            departure_time: self.current_tick,
            arrival_time: self.current_tick + travel_time,
            fuel_cost,
        };
        
        // Store the trajectory
        self.trajectories.insert(ship_id, trajectory);
        
        Ok(())
    }
    
    fn calculate_distance(&self, from: Vector2, to: Vector2) -> f32 {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    fn calculate_fuel_cost(&self, from: Vector2, to: Vector2, _ship_id: ShipId) -> f32 {
        let distance = self.calculate_distance(from, to);
        let base_cost = distance / 100.0; // Base fuel consumption: 1 fuel per 100 units
        
        // Check if there's an active transfer window that reduces cost
        let window_modifier = if self.is_transfer_window_active(from, to) {
            0.5 // 50% fuel reduction during transfer window
        } else {
            1.0
        };
        
        // Apply ship-specific modifiers (placeholder for now)
        let ship_modifier = 1.0; // Would vary by ship class in full implementation
        
        // Ensure minimum fuel cost to prevent zero-cost travel
        (base_cost * window_modifier * ship_modifier).max(0.1)
    }
    
    fn is_transfer_window_active(&self, _from: Vector2, _to: Vector2) -> bool {
        // Check if current positions are within a transfer window
        // This would involve more complex orbital mechanics calculations
        // For now, return false as placeholder
        false
    }
    
    /// Get the cached orbital position of a planet
    /// Returns None if the planet position hasn't been calculated this tick
    pub fn get_orbital_position(&self, planet_id: PlanetId) -> Option<Vector2> {
        self.orbital_cache.get(&planet_id).copied()
    }
    
    /// Get all active ship trajectories (for debugging/UI)
    pub fn get_active_trajectories(&self) -> &HashMap<ShipId, Trajectory> {
        &self.trajectories
    }
    
    /// Get the number of active transfer windows
    pub fn get_transfer_window_count(&self) -> usize {
        self.transfer_windows.len()
    }
    
    /// Check if a specific transfer window is active
    pub fn is_transfer_window_open(&self, from: PlanetId, to: PlanetId) -> bool {
        let pair = (from.min(to), from.max(to)); // Normalize pair order
        self.transfer_windows.get(&pair)
            .map(|&expiry| expiry > self.current_tick)
            .unwrap_or(false)
    }
    
    /// Calculate estimated travel time between two points
    pub fn estimate_travel_time(&self, from: Vector2, to: Vector2) -> u64 {
        let distance = self.calculate_distance(from, to);
        let base_speed = 10.0;
        (distance / base_speed).ceil() as u64
    }
}

// Include unit tests
#[cfg(test)]
mod physics_unit_tests {
    use super::PhysicsEngine;
    use crate::core::types::*;
    use crate::core::events::*;

    #[test]
    fn test_new_physics_engine() {
        let physics = PhysicsEngine::new();
        assert_eq!(physics.get_transfer_window_count(), 0);
        assert_eq!(physics.get_active_trajectories().len(), 0);
    }

    #[test]
    fn test_orbital_calculation_with_validation() {
        let physics = PhysicsEngine::new();
        
        // Test normal orbital elements
        let normal_orbit = OrbitalElements {
            semi_major_axis: 5.0,
            period: 100.0,
            phase: 0.0,
        };
        let pos = physics.calculate_orbital_position(&normal_orbit, 0);
        assert!((pos.x - 5.0).abs() < 0.01);
        assert!((pos.y - 0.0).abs() < 0.01);

        // Test zero period (should return origin)
        let zero_period_orbit = OrbitalElements {
            semi_major_axis: 5.0,
            period: 0.0,
            phase: 0.0,
        };
        let pos_zero = physics.calculate_orbital_position(&zero_period_orbit, 50);
        assert_eq!(pos_zero.x, 0.0);
        assert_eq!(pos_zero.y, 0.0);

        // Test negative radius (should return origin)
        let negative_radius_orbit = OrbitalElements {
            semi_major_axis: -3.0,
            period: 100.0,
            phase: 0.0,
        };
        let pos_negative = physics.calculate_orbital_position(&negative_radius_orbit, 50);
        assert_eq!(pos_negative.x, 0.0);
        assert_eq!(pos_negative.y, 0.0);
    }

    #[test]
    fn test_error_handling_for_invalid_coordinates() {
        let mut physics = PhysicsEngine::new();
        
        // Test NaN coordinates
        let nan_target = Vector2 { x: f32::NAN, y: 50.0 };
        let move_event = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
            ship: 123,
            target: nan_target,
        });
        assert!(physics.handle_event(&move_event).is_err());

        // Test infinite coordinates
        let inf_target = Vector2 { x: f32::INFINITY, y: 50.0 };
        let move_event2 = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
            ship: 456,
            target: inf_target,
        });
        assert!(physics.handle_event(&move_event2).is_err());
    }

    #[test]
    fn test_tick_progression_validation() {
        let mut physics = PhysicsEngine::new();
        
        // Set initial tick
        let tick1 = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(100));
        assert!(physics.handle_event(&tick1).is_ok());

        // Backwards tick should be rejected
        let backwards_tick = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(50));
        assert!(physics.handle_event(&backwards_tick).is_err());

        // Forward tick should be accepted
        let forward_tick = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(150));
        assert!(physics.handle_event(&forward_tick).is_ok());
    }

    #[test]
    fn test_distance_validation_in_trajectory_planning() {
        let mut physics = PhysicsEngine::new();
        
        // Test too far target
        let far_target = Vector2 { x: 2000.0, y: 0.0 };
        let move_event = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
            ship: 123,
            target: far_target,
        });
        assert!(physics.handle_event(&move_event).is_err());

        // Test same position (zero distance)
        let same_target = Vector2 { x: 0.0, y: 0.0 };
        let move_event2 = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
            ship: 456,
            target: same_target,
        });
        assert!(physics.handle_event(&move_event2).is_err());

        // Test valid distance
        let valid_target = Vector2 { x: 100.0, y: 100.0 };
        let move_event3 = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
            ship: 789,
            target: valid_target,
        });
        assert!(physics.handle_event(&move_event3).is_ok());
    }

    #[test]
    fn test_deterministic_orbital_calculations() {
        let physics1 = PhysicsEngine::new();
        let physics2 = PhysicsEngine::new();
        
        let orbital_elements = OrbitalElements {
            semi_major_axis: 7.5,
            period: 365.0,
            phase: 1.57,
        };
        
        // Same inputs should produce identical outputs
        for &tick in &[0, 100, 1000, 10000] {
            let pos1 = physics1.calculate_orbital_position(&orbital_elements, tick);
            let pos2 = physics2.calculate_orbital_position(&orbital_elements, tick);
            
            assert!((pos1.x - pos2.x).abs() < f32::EPSILON);
            assert!((pos1.y - pos2.y).abs() < f32::EPSILON);
        }
    }
}