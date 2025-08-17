// tests/systems/time_manager_test.rs
use stellar_dominion::core::*;
use stellar_dominion::core::events::*;
use stellar_dominion::core::types::*;
use stellar_dominion::systems::TimeManager;

#[test]
fn test_tick_events_emit_exactly_once_per_timestep() {
    let mut time_manager = TimeManager::new();
    let mut event_bus = EventBus::new();
    
    // Simulate exactly one timestep (0.1s) at 1x speed
    let result = time_manager.update(0.1, &mut event_bus);
    assert!(result.is_ok());
    
    // Should have exactly one tick event
    assert_eq!(event_bus.queued_events.len(), 1);
    
    if let Some(GameEvent::SimulationEvent(SimulationEvent::TickCompleted(tick))) = event_bus.queued_events.front() {
        assert_eq!(*tick, 1);
    } else {
        panic!("Expected TickCompleted event");
    }
    
    // Tick counter should advance
    assert_eq!(time_manager.get_tick(), 1);
}

#[test]
fn test_no_tick_events_when_paused() {
    let mut time_manager = TimeManager::new();
    let mut event_bus = EventBus::new();
    
    // Pause the game
    let pause_event = GameEvent::PlayerCommand(PlayerCommand::PauseGame(true));
    let result = time_manager.handle_event(&pause_event);
    assert!(result.is_ok());
    
    // Try to update with a full timestep
    let result = time_manager.update(0.1, &mut event_bus);
    assert!(result.is_ok());
    
    // Should have no events and tick should not advance
    assert_eq!(event_bus.queued_events.len(), 0);
    assert_eq!(time_manager.get_tick(), 0);
}

#[test]
fn test_speed_multiplier_scales_tick_rate() {
    let mut time_manager = TimeManager::new();
    let mut event_bus = EventBus::new();
    
    // Set speed to 2x
    let speed_event = GameEvent::PlayerCommand(PlayerCommand::SetGameSpeed(2.0));
    let result = time_manager.handle_event(&speed_event);
    assert!(result.is_ok());
    
    // Update with 0.1s real time (should be 0.2s game time)
    let result = time_manager.update(0.1, &mut event_bus);
    assert!(result.is_ok());
    
    // Should have 2 tick events at 2x speed
    assert_eq!(event_bus.queued_events.len(), 2);
    assert_eq!(time_manager.get_tick(), 2);
}

#[test]
fn test_fractional_timesteps_accumulate() {
    let mut time_manager = TimeManager::new();
    let mut event_bus = EventBus::new();
    
    // Update with 0.05s (half a timestep)
    let result = time_manager.update(0.05, &mut event_bus);
    assert!(result.is_ok());
    
    // No tick should occur yet
    assert_eq!(event_bus.queued_events.len(), 0);
    assert_eq!(time_manager.get_tick(), 0);
    
    // Update with another 0.05s (completing one timestep)
    let result = time_manager.update(0.05, &mut event_bus);
    assert!(result.is_ok());
    
    // Now should have one tick
    assert_eq!(event_bus.queued_events.len(), 1);
    assert_eq!(time_manager.get_tick(), 1);
}

#[test]
fn test_deterministic_tick_progression() {
    let mut time_manager1 = TimeManager::new();
    let mut time_manager2 = TimeManager::new();
    let mut event_bus1 = EventBus::new();
    let mut event_bus2 = EventBus::new();
    
    // Same sequence of updates should produce identical results
    let updates = [0.03, 0.07, 0.05, 0.02, 0.08];
    
    for &delta in &updates {
        time_manager1.update(delta, &mut event_bus1).unwrap();
        time_manager2.update(delta, &mut event_bus2).unwrap();
    }
    
    // Both should have identical tick counts
    assert_eq!(time_manager1.get_tick(), time_manager2.get_tick());
    assert_eq!(event_bus1.queued_events.len(), event_bus2.queued_events.len());
}

#[test]
fn test_pause_and_unpause() {
    let mut time_manager = TimeManager::new();
    let mut event_bus = EventBus::new();
    
    // Advance one tick
    time_manager.update(0.1, &mut event_bus).unwrap();
    assert_eq!(time_manager.get_tick(), 1);
    
    // Pause
    let pause_event = GameEvent::PlayerCommand(PlayerCommand::PauseGame(true));
    time_manager.handle_event(&pause_event).unwrap();
    
    // Try to advance (should not work)
    time_manager.update(0.1, &mut event_bus).unwrap();
    assert_eq!(time_manager.get_tick(), 1);
    
    // Unpause
    let unpause_event = GameEvent::PlayerCommand(PlayerCommand::PauseGame(false));
    time_manager.handle_event(&unpause_event).unwrap();
    
    // Now should advance again
    time_manager.update(0.1, &mut event_bus).unwrap();
    assert_eq!(time_manager.get_tick(), 2);
}

#[test]
fn test_speed_change_during_accumulation() {
    let mut time_manager = TimeManager::new();
    let mut event_bus = EventBus::new();
    
    // Accumulate half a timestep at 1x speed
    time_manager.update(0.05, &mut event_bus).unwrap();
    assert_eq!(time_manager.get_tick(), 0);
    
    // Change speed to 2x
    let speed_event = GameEvent::PlayerCommand(PlayerCommand::SetGameSpeed(2.0));
    time_manager.handle_event(&speed_event).unwrap();
    
    // Add another 0.025s real time (0.05s game time at 2x speed)
    // Total: 0.05 + 0.05 = 0.1s game time = 1 tick
    time_manager.update(0.025, &mut event_bus).unwrap();
    assert_eq!(time_manager.get_tick(), 1);
}

#[test]
fn test_initial_state() {
    let time_manager = TimeManager::new();
    
    assert_eq!(time_manager.get_tick(), 0);
    assert!(!time_manager.paused);
    assert_eq!(time_manager.speed_multiplier, 1.0);
    assert_eq!(time_manager.accumulated_time, 0.0);
    assert_eq!(time_manager.tick_duration, 0.1);
}

#[test]
fn test_zero_speed_prevents_ticks() {
    let mut time_manager = TimeManager::new();
    let mut event_bus = EventBus::new();
    
    // Set speed to 0
    let speed_event = GameEvent::PlayerCommand(PlayerCommand::SetGameSpeed(0.0));
    time_manager.handle_event(&speed_event).unwrap();
    
    // Try to advance time
    time_manager.update(1.0, &mut event_bus).unwrap();
    
    // Should not advance
    assert_eq!(time_manager.get_tick(), 0);
    assert_eq!(event_bus.queued_events.len(), 0);
}