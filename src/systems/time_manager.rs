// src/systems/time_manager.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;

/// Manages game timing with fixed timesteps for deterministic simulation.
/// Emits TickCompleted events every 0.1 seconds to drive all systems.
pub struct TimeManager {
    tick: u64,
    pub(crate) paused: bool,
    pub(crate) speed_multiplier: f32,
    pub(crate) accumulated_time: f64, // Use f64 for better precision over long periods
    pub(crate) tick_duration: f64, // 0.1 seconds - use f64 for consistency
}

// Constants for timing constraints
const MIN_SPEED_MULTIPLIER: f32 = 0.0;
const MAX_SPEED_MULTIPLIER: f32 = 10.0;
const TICK_DURATION_SECONDS: f64 = 0.1;
const MAX_SAFE_TICK: u64 = u64::MAX - 1000; // Leave buffer for overflow protection

impl TimeManager {
    /// Creates a new TimeManager with default settings.
    /// Fixed timestep is 0.1 seconds for deterministic simulation.
    pub fn new() -> Self {
        Self {
            tick: 0,
            paused: false,
            speed_multiplier: 1.0,
            accumulated_time: 0.0,
            tick_duration: TICK_DURATION_SECONDS,
        }
    }
    
    /// Updates the time manager, advancing ticks and emitting TickCompleted events.
    /// 
    /// # Arguments
    /// * `delta` - Frame time in seconds (from fixed timestep loop)
    /// * `event_bus` - EventBus to emit TickCompleted events
    /// 
    /// # Returns
    /// GameResult indicating success or overflow protection trigger
    pub fn update(&mut self, delta: f32, event_bus: &mut EventBus) -> GameResult<()> {
        if !self.paused {
            // Convert f32 delta to f64 for precision
            let delta_f64 = delta as f64;
            self.accumulated_time += delta_f64 * self.speed_multiplier as f64;
            
            // Process all accumulated ticks
            while self.accumulated_time >= self.tick_duration {
                // Check for potential overflow before incrementing
                if self.tick >= MAX_SAFE_TICK {
                    return Err(GameError::SystemError(
                        "Tick counter approaching overflow limit".to_string()
                    ));
                }
                
                self.tick += 1;
                self.accumulated_time -= self.tick_duration;
                
                // Emit tick completed event
                event_bus.queue_event(GameEvent::SimulationEvent(
                    crate::core::events::SimulationEvent::TickCompleted(self.tick)
                ));
            }
        }
        Ok(())
    }
    
    /// Handles player commands for pause/unpause and speed control.
    /// 
    /// # Arguments
    /// * `event` - Game event to process
    /// 
    /// # Returns
    /// GameResult indicating success or validation failure
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    crate::core::events::PlayerCommand::SetGameSpeed(speed) => {
                        self.set_speed_multiplier(*speed)?;
                    }
                    crate::core::events::PlayerCommand::PauseGame(paused) => {
                        self.paused = *paused;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Returns the current game tick counter.
    /// Each tick represents 0.1 seconds of game time.
    pub fn get_current_tick(&self) -> u64 {
        self.tick
    }
    
    /// Sets the current tick (used for save/load functionality).
    /// Validates the tick value for safety.
    /// 
    /// # Arguments
    /// * `tick` - New tick value to set
    /// 
    /// # Returns
    /// GameResult indicating success or validation failure
    pub fn set_tick(&mut self, tick: u64) -> GameResult<()> {
        if tick >= MAX_SAFE_TICK {
            return Err(GameError::InvalidOperation(
                format!("Tick value {} too large, maximum allowed: {}", tick, MAX_SAFE_TICK)
            ));
        }
        self.tick = tick;
        // Reset accumulated time to prevent timing inconsistencies
        self.accumulated_time = 0.0;
        Ok(())
    }
    
    /// Sets the game speed multiplier with validation.
    /// 
    /// # Arguments
    /// * `speed` - Speed multiplier (0.0 = paused, 1.0 = normal, 2.0 = double speed)
    /// 
    /// # Returns
    /// GameResult indicating success or validation failure
    pub fn set_speed_multiplier(&mut self, speed: f32) -> GameResult<()> {
        if speed < MIN_SPEED_MULTIPLIER || speed > MAX_SPEED_MULTIPLIER {
            return Err(GameError::InvalidOperation(
                format!("Speed multiplier {} out of range [{}, {}]", 
                    speed, MIN_SPEED_MULTIPLIER, MAX_SPEED_MULTIPLIER)
            ));
        }
        if speed.is_nan() || speed.is_infinite() {
            return Err(GameError::InvalidOperation(
                "Speed multiplier cannot be NaN or infinite".to_string()
            ));
        }
        self.speed_multiplier = speed;
        Ok(())
    }
    
    /// Returns the current game speed multiplier.
    pub fn get_speed_multiplier(&self) -> f32 {
        self.speed_multiplier
    }
    
    /// Returns whether the game is currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    /// Returns the fixed timestep duration in seconds.
    pub fn get_tick_duration(&self) -> f64 {
        self.tick_duration
    }
    
    /// Calculates game time in seconds based on current tick.
    /// Each tick represents 0.1 seconds of game time.
    pub fn get_game_time_seconds(&self) -> f64 {
        self.tick as f64 * self.tick_duration
    }
    
    /// Validates the TimeManager's internal state.
    /// Used for debugging and save/load validation.
    pub fn validate(&self) -> GameResult<()> {
        if self.tick >= MAX_SAFE_TICK {
            return Err(GameError::SystemError(
                "Tick counter too large".to_string()
            ));
        }
        
        if self.speed_multiplier < MIN_SPEED_MULTIPLIER || self.speed_multiplier > MAX_SPEED_MULTIPLIER {
            return Err(GameError::SystemError(
                "Speed multiplier out of valid range".to_string()
            ));
        }
        
        if self.speed_multiplier.is_nan() || self.speed_multiplier.is_infinite() {
            return Err(GameError::SystemError(
                "Speed multiplier is not a valid number".to_string()
            ));
        }
        
        if self.accumulated_time < 0.0 {
            return Err(GameError::SystemError(
                "Accumulated time cannot be negative".to_string()
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{EventBus, GameEvent};
    use crate::core::events::{SimulationEvent, PlayerCommand};

    #[test]
    fn test_time_manager_creation() {
        let time_manager = TimeManager::new();
        assert_eq!(time_manager.get_current_tick(), 0);
        assert!(!time_manager.is_paused());
        assert_eq!(time_manager.get_speed_multiplier(), 1.0);
        assert_eq!(time_manager.get_tick_duration(), 0.1);
        assert!(time_manager.validate().is_ok());
    }

    #[test]
    fn test_tick_advancement() {
        let mut time_manager = TimeManager::new();
        let mut event_bus = EventBus::new();
        
        // Advance by exactly one tick duration
        let result = time_manager.update(0.1, &mut event_bus);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_current_tick(), 1);
        
        // Check that TickCompleted event was queued
        assert_eq!(event_bus.queued_events.len(), 1);
        match &event_bus.queued_events[0] {
            GameEvent::SimulationEvent(SimulationEvent::TickCompleted(tick)) => {
                assert_eq!(*tick, 1);
            }
            _ => panic!("Expected TickCompleted event"),
        }
    }

    #[test]
    fn test_partial_tick_accumulation() {
        let mut time_manager = TimeManager::new();
        let mut event_bus = EventBus::new();
        
        // Advance by half tick duration - should not emit event
        let result = time_manager.update(0.05, &mut event_bus);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_current_tick(), 0);
        assert_eq!(event_bus.queued_events.len(), 0);
        
        // Advance by another half - should emit one event
        let result = time_manager.update(0.05, &mut event_bus);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_current_tick(), 1);
        assert_eq!(event_bus.queued_events.len(), 1);
    }

    #[test]
    fn test_multiple_ticks_in_one_update() {
        let mut time_manager = TimeManager::new();
        let mut event_bus = EventBus::new();
        
        // Advance by 2.5 tick durations - should emit 2 events
        let result = time_manager.update(0.25, &mut event_bus);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_current_tick(), 2);
        assert_eq!(event_bus.queued_events.len(), 2);
        
        // Verify both events
        match &event_bus.queued_events[0] {
            GameEvent::SimulationEvent(SimulationEvent::TickCompleted(tick)) => {
                assert_eq!(*tick, 1);
            }
            _ => panic!("Expected first TickCompleted event"),
        }
        match &event_bus.queued_events[1] {
            GameEvent::SimulationEvent(SimulationEvent::TickCompleted(tick)) => {
                assert_eq!(*tick, 2);
            }
            _ => panic!("Expected second TickCompleted event"),
        }
    }

    #[test]
    fn test_pause_functionality() {
        let mut time_manager = TimeManager::new();
        let mut event_bus = EventBus::new();
        
        // Pause the game
        let pause_event = GameEvent::PlayerCommand(PlayerCommand::PauseGame(true));
        let result = time_manager.handle_event(&pause_event);
        assert!(result.is_ok());
        assert!(time_manager.is_paused());
        
        // Advance time while paused - should not emit events
        let result = time_manager.update(0.2, &mut event_bus);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_current_tick(), 0);
        assert_eq!(event_bus.queued_events.len(), 0);
        
        // Unpause and advance
        let unpause_event = GameEvent::PlayerCommand(PlayerCommand::PauseGame(false));
        let result = time_manager.handle_event(&unpause_event);
        assert!(result.is_ok());
        assert!(!time_manager.is_paused());
        
        let result = time_manager.update(0.1, &mut event_bus);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_current_tick(), 1);
        assert_eq!(event_bus.queued_events.len(), 1);
    }

    #[test]
    fn test_speed_multiplier() {
        let mut time_manager = TimeManager::new();
        let mut event_bus = EventBus::new();
        
        // Set double speed
        let speed_event = GameEvent::PlayerCommand(PlayerCommand::SetGameSpeed(2.0));
        let result = time_manager.handle_event(&speed_event);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_speed_multiplier(), 2.0);
        
        // Advance by 0.1 seconds real time = 0.2 seconds game time = 2 ticks
        let result = time_manager.update(0.1, &mut event_bus);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_current_tick(), 2);
        assert_eq!(event_bus.queued_events.len(), 2);
    }

    #[test]
    fn test_speed_multiplier_validation() {
        let mut time_manager = TimeManager::new();
        
        // Test valid speed
        assert!(time_manager.set_speed_multiplier(5.0).is_ok());
        assert_eq!(time_manager.get_speed_multiplier(), 5.0);
        
        // Test invalid speeds
        assert!(time_manager.set_speed_multiplier(-1.0).is_err());
        assert!(time_manager.set_speed_multiplier(11.0).is_err());
        assert!(time_manager.set_speed_multiplier(f32::NAN).is_err());
        assert!(time_manager.set_speed_multiplier(f32::INFINITY).is_err());
        
        // Speed should remain at previous valid value
        assert_eq!(time_manager.get_speed_multiplier(), 5.0);
    }

    #[test]
    fn test_set_tick_validation() {
        let mut time_manager = TimeManager::new();
        
        // Test valid tick
        let result = time_manager.set_tick(1000);
        assert!(result.is_ok());
        assert_eq!(time_manager.get_current_tick(), 1000);
        
        // Test overflow protection
        let result = time_manager.set_tick(u64::MAX - 500);
        assert!(result.is_err());
        assert_eq!(time_manager.get_current_tick(), 1000); // Should remain unchanged
    }

    #[test]
    fn test_overflow_protection() {
        let mut time_manager = TimeManager::new();
        let mut event_bus = EventBus::new();
        
        // Set tick near overflow limit
        let result = time_manager.set_tick(u64::MAX - 100);
        assert!(result.is_err()); // Should fail due to overflow protection
        
        // Advance from a safe high value
        let result = time_manager.set_tick(u64::MAX - 1500);
        assert!(result.is_ok());
        
        // Try to advance beyond overflow protection
        let result = time_manager.update(100.0, &mut event_bus); // 1000 ticks
        assert!(result.is_err()); // Should fail due to overflow protection
    }

    #[test]
    fn test_game_time_calculation() {
        let mut time_manager = TimeManager::new();
        
        assert_eq!(time_manager.get_game_time_seconds(), 0.0);
        
        let _ = time_manager.set_tick(100);
        if time_manager.set_tick(100).is_ok() {
            assert_eq!(time_manager.get_game_time_seconds(), 10.0); // 100 * 0.1
        }
    }

    #[test]
    fn test_precision_with_f64() {
        let mut time_manager = TimeManager::new();
        let mut event_bus = EventBus::new();
        
        // Test many small updates to verify f64 precision prevents drift
        for _ in 0..1000 {
            let result = time_manager.update(0.0001, &mut event_bus); // 0.1ms updates
            assert!(result.is_ok());
        }
        
        // 1000 * 0.0001 = 0.1 seconds = 1 tick
        assert_eq!(time_manager.get_current_tick(), 1);
        assert_eq!(event_bus.queued_events.len(), 1);
    }

    #[test]
    fn test_validation_comprehensive() {
        let time_manager = TimeManager::new();
        assert!(time_manager.validate().is_ok());
        
        // Test with modified state
        let mut time_manager = TimeManager {
            tick: 1000,
            paused: true,
            speed_multiplier: 5.0,
            accumulated_time: 0.05,
            tick_duration: 0.1,
        };
        assert!(time_manager.validate().is_ok());
        
        // Test invalid state
        time_manager.speed_multiplier = -1.0;
        assert!(time_manager.validate().is_err());
        
        time_manager.speed_multiplier = f32::NAN;
        assert!(time_manager.validate().is_err());
        
        time_manager.speed_multiplier = 1.0;
        time_manager.accumulated_time = -0.1;
        assert!(time_manager.validate().is_err());
        
        time_manager.accumulated_time = 0.0;
        time_manager.tick = u64::MAX;
        assert!(time_manager.validate().is_err());
    }
}