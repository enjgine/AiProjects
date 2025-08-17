// src/systems/physics_engine.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use crate::core::events::*;
use std::collections::HashMap;

pub struct PhysicsEngine {
    orbital_cache: Vec<(PlanetId, Vector2)>,
    transfer_windows: HashMap<(PlanetId, PlanetId), u64>,
    trajectories: HashMap<ShipId, Trajectory>,
    current_tick: u64,
    needs_tick_processing: bool,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            orbital_cache: Vec::new(),
            transfer_windows: HashMap::new(),
            trajectories: HashMap::new(),
            current_tick: 0,
            needs_tick_processing: false,
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
                self.current_tick = *tick;
                // Store that we need to process a tick, but emit events in update()
                self.needs_tick_processing = true;
            }
            GameEvent::PlayerCommand(PlayerCommand::MoveShip { ship, target }) => {
                self.plan_trajectory(*ship, *target)?;
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
    
    fn update_orbital_positions(&mut self, _event_bus: &mut EventBus) -> GameResult<()> {
        // Clear cache and recalculate orbital positions for all planets
        self.orbital_cache.clear();
        
        // Note: In actual implementation, we'd get planet data from PlanetManager
        // For now this is a placeholder that calculates circular orbits
        // Each planet follows: position = (radius * cos(phase + time/period), radius * sin(phase + time/period))
        
        Ok(())
    }
    
    pub fn calculate_orbital_position(&self, orbital_elements: &OrbitalElements, tick: u64) -> Vector2 {
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
    
    pub fn get_ship_position(&self, ship_id: ShipId, base_position: Vector2) -> Vector2 {
        if let Some(trajectory) = self.trajectories.get(&ship_id) {
            self.interpolate_trajectory_position(trajectory, self.current_tick)
        } else {
            base_position
        }
    }
    
    fn interpolate_trajectory_position(&self, trajectory: &Trajectory, current_tick: u64) -> Vector2 {
        if current_tick <= trajectory.departure_time {
            return trajectory.origin;
        }
        if current_tick >= trajectory.arrival_time {
            return trajectory.destination;
        }
        
        let total_time = trajectory.arrival_time - trajectory.departure_time;
        let elapsed_time = current_tick - trajectory.departure_time;
        let progress = (elapsed_time as f32) / (total_time as f32);
        
        Vector2 {
            x: trajectory.origin.x + (trajectory.destination.x - trajectory.origin.x) * progress,
            y: trajectory.origin.y + (trajectory.destination.y - trajectory.origin.y) * progress,
        }
    }
    
    fn check_transfer_windows(&mut self, event_bus: &mut EventBus) -> GameResult<()> {
        // Calculate transfer windows between planets based on orbital positions
        // Transfer windows occur when planets are properly aligned for efficient travel
        
        // For simplicity, we'll emit transfer window events every 50 ticks between any two planets
        if self.current_tick % 50 == 0 {
            // In real implementation, we'd calculate actual orbital mechanics
            // For now, emit periodic transfer windows for demonstration
            for planet_a in 0..5 { // Assuming 5 planets max for demo
                for planet_b in (planet_a + 1)..5 {
                    self.transfer_windows.insert((planet_a, planet_b), self.current_tick + 20);
                    event_bus.queue_event(GameEvent::SimulationEvent(
                        SimulationEvent::TransferWindowOpen {
                            from: planet_a,
                            to: planet_b,
                        }
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    fn plan_trajectory(&mut self, ship_id: ShipId, target: Vector2) -> GameResult<()> {
        // Calculate trajectory from current position to target
        // For now, using placeholder positions - in real implementation we'd get from ShipManager
        let origin = Vector2 { x: 0.0, y: 0.0 }; // Would get from ship's current position
        let distance = self.calculate_distance(origin, target);
        
        // Calculate travel time based on distance (arbitrary speed for now)
        let travel_time = (distance / 10.0) as u64; // 10 units per tick speed
        
        // Calculate fuel cost
        let fuel_cost = self.calculate_fuel_cost(origin, target, ship_id);
        
        let trajectory = Trajectory {
            origin,
            destination: target,
            departure_time: self.current_tick,
            arrival_time: self.current_tick + travel_time,
            fuel_cost,
        };
        
        self.trajectories.insert(ship_id, trajectory);
        Ok(())
    }
    
    fn calculate_distance(&self, from: Vector2, to: Vector2) -> f32 {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    fn calculate_fuel_cost(&self, from: Vector2, to: Vector2, ship_id: ShipId) -> f32 {
        let distance = self.calculate_distance(from, to);
        let base_cost = distance / 100.0; // Base fuel consumption
        
        // Check if there's an active transfer window that reduces cost
        let window_modifier = if self.is_transfer_window_active(from, to) {
            0.5 // 50% fuel reduction during transfer window
        } else {
            1.0
        };
        
        base_cost * window_modifier
    }
    
    fn is_transfer_window_active(&self, _from: Vector2, _to: Vector2) -> bool {
        // Check if current positions are within a transfer window
        // This would involve more complex orbital mechanics calculations
        // For now, return false as placeholder
        false
    }
    
    pub fn get_orbital_position(&self, planet_id: PlanetId) -> Option<Vector2> {
        self.orbital_cache.iter()
            .find(|(id, _)| *id == planet_id)
            .map(|(_, pos)| *pos)
    }
}