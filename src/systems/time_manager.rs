// src/systems/time_manager.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;

pub struct TimeManager {
    tick: u64,
    pub(crate) paused: bool,
    pub(crate) speed_multiplier: f32,
    pub(crate) accumulated_time: f32,
    pub(crate) tick_duration: f32, // 0.1 seconds
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            tick: 0,
            paused: false,
            speed_multiplier: 1.0,
            accumulated_time: 0.0,
            tick_duration: 0.1,
        }
    }
    
    pub fn update(&mut self, delta: f32, event_bus: &mut EventBus) -> GameResult<()> {
        if !self.paused {
            self.accumulated_time += delta * self.speed_multiplier;
            
            while self.accumulated_time >= self.tick_duration {
                self.tick += 1;
                self.accumulated_time -= self.tick_duration;
                
                event_bus.queue_event(GameEvent::SimulationEvent(
                    crate::core::events::SimulationEvent::TickCompleted(self.tick)
                ));
            }
        }
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    crate::core::events::PlayerCommand::SetGameSpeed(speed) => {
                        self.speed_multiplier = *speed;
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
    
    pub fn get_tick(&self) -> u64 {
        self.tick
    }
    
    pub fn get_current_tick(&self) -> u64 {
        self.tick
    }
    
    pub fn set_tick(&mut self, tick: u64) {
        self.tick = tick;
    }
}