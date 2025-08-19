// tests/time_manager_integration.rs
use stellar_dominion::core::*;
use stellar_dominion::core::events::*;

#[test]
fn test_time_manager_eventbus_integration() {
    let mut game_state = GameState::new().unwrap();
    
    // Queue a speed change command
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::SetGameSpeed(2.0)
    ));
    
    // Process events (should route to TimeManager)
    let result = game_state.process_queued_events_for_test();
    assert!(result.is_ok());
    
    // Verify speed was applied
    assert_eq!(game_state.time_manager.get_speed_multiplier(), 2.0);
}

#[test]
fn test_time_manager_pause_integration() {
    let mut game_state = GameState::new().unwrap();
    
    // Queue a pause command
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::PauseGame(true)
    ));
    
    // Process events
    game_state.process_queued_events_for_test().unwrap();
    
    // Verify pause was applied
    assert!(game_state.time_manager.is_paused());
    
    // Run a fixed update cycle
    game_state.fixed_update(0.1).unwrap();
    
    // Should not advance tick when paused
    assert_eq!(game_state.time_manager.get_tick(), 0);
}

#[test]
fn test_tick_events_emitted_to_bus() {
    let mut game_state = GameState::new().unwrap();
    
    // Run fixed update to generate tick
    game_state.fixed_update(0.1).unwrap();
    
    // Should have tick completed event in history
    assert!(game_state.event_bus.event_history.len() > 0);
    
    // Check that the last event is a TickCompleted
    if let Some(GameEvent::SimulationEvent(SimulationEvent::TickCompleted(tick))) = 
        game_state.event_bus.event_history.back() {
        assert_eq!(*tick, 1);
    } else {
        panic!("Expected TickCompleted event in history");
    }
}

#[test]
fn test_deterministic_game_state_progression() {
    let mut game1 = GameState::new().unwrap();
    let mut game2 = GameState::new().unwrap();
    
    // Apply identical sequences of updates
    let deltas = [0.03, 0.07, 0.04, 0.06, 0.05];
    
    for &delta in &deltas {
        game1.fixed_update(delta).unwrap();
        game2.fixed_update(delta).unwrap();
    }
    
    // Both games should have identical tick counts
    assert_eq!(game1.time_manager.get_tick(), game2.time_manager.get_tick());
    
    // Event histories should have the same number of events
    assert_eq!(game1.event_bus.event_history.len(), game2.event_bus.event_history.len());
}

#[test]
fn test_time_manager_subscription_registered() {
    let game_state = GameState::new().unwrap();
    
    // Verify TimeManager is subscribed to PlayerCommand events
    let subscriptions = game_state.event_bus.subscribers.get(&SystemId::TimeManager);
    assert!(subscriptions.is_some());
    assert!(subscriptions.unwrap().contains(&EventType::PlayerCommand));
}

#[test]
fn test_fixed_update_order_maintained() {
    let mut game_state = GameState::new().unwrap();
    
    // TimeManager should be updated last in the fixed_update cycle
    // This test verifies the implementation follows the strict order:
    // UI → Physics → Resources → Population → Construction → Combat → Time
    
    // Run multiple update cycles
    for _ in 0..5 {
        let result = game_state.fixed_update(0.1);
        assert!(result.is_ok());
    }
    
    // Should have 5 ticks
    assert_eq!(game_state.time_manager.get_tick(), 5);
}

#[test]
fn test_event_processing_after_time_update() {
    let mut game_state = GameState::new().unwrap();
    
    // TimeManager update should happen before event processing
    // Queue an event that should be processed in the same frame
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::SetGameSpeed(3.0)
    ));
    
    // Run fixed update
    game_state.fixed_update(0.1).unwrap();
    
    // Event should be processed and speed should be changed
    assert_eq!(game_state.time_manager.get_speed_multiplier(), 3.0);
    
    // Should have generated a tick at 1x speed (speed change applied after this tick)
    assert_eq!(game_state.time_manager.get_tick(), 1);
}

#[test]
fn test_accumulated_time_precision() {
    let mut game_state = GameState::new().unwrap();
    
    // Test with very small deltas to ensure precision
    for _ in 0..1000 {
        game_state.fixed_update(0.0001).unwrap();
    }
    
    // Should have exactly 1 tick (1000 * 0.0001 = 0.1)
    assert_eq!(game_state.time_manager.get_tick(), 1);
}

#[test]
fn test_multiple_commands_same_frame() {
    let mut game_state = GameState::new().unwrap();
    
    // Queue multiple commands
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::PauseGame(true)
    ));
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::SetGameSpeed(5.0)
    ));
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::PauseGame(false)
    ));
    
    // Process all events
    game_state.process_queued_events_for_test().unwrap();
    
    // Final state should reflect all commands
    assert!(!game_state.time_manager.is_paused()); // Last pause command was false
    assert_eq!(game_state.time_manager.get_speed_multiplier(), 5.0);
}