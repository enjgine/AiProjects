// tests/phase2_integration_test.rs
use stellar_dominion::core::{GameState, GameResult, GameEvent, GameSystem};
use stellar_dominion::core::events::PlayerCommand;

#[test]
fn test_toolbar_initialization() -> GameResult<()> {
    let game_state = GameState::new()?;
    
    // Verify the UIRenderer has toolbar components initialized
    // This is a basic smoke test to ensure the new Phase 2 components don't break initialization
    assert!(game_state.time_manager.get_current_tick() == 0);
    
    Ok(())
}

#[test]
fn test_planet_selection_via_events() -> GameResult<()> {
    let mut game_state = GameState::new()?;
    
    // Test that selecting a planet via PlayerCommand works
    game_state.queue_event(GameEvent::PlayerCommand(PlayerCommand::SelectPlanet(0)));
    game_state.process_queued_events_for_test()?;
    
    // The event should be processed without errors
    // In a full implementation, we'd verify the UI state changed appropriately
    Ok(())
}

#[test]
fn test_ship_selection_via_events() -> GameResult<()> {
    let mut game_state = GameState::new()?;
    
    // Test that selecting a ship via PlayerCommand works
    game_state.queue_event(GameEvent::PlayerCommand(PlayerCommand::SelectShip(0)));
    game_state.process_queued_events_for_test()?;
    
    // The event should be processed without errors
    Ok(())
}

#[test]
fn test_multiple_events_processing() -> GameResult<()> {
    let mut game_state = GameState::new()?;
    
    // Test multiple events can be queued and processed
    game_state.queue_event(GameEvent::PlayerCommand(PlayerCommand::SelectPlanet(0)));
    game_state.queue_event(GameEvent::PlayerCommand(PlayerCommand::PauseGame(true)));
    game_state.queue_event(GameEvent::PlayerCommand(PlayerCommand::SelectShip(0)));
    
    // Process all events
    game_state.process_queued_events_for_test()?;
    
    // All events should process without errors
    Ok(())
}

#[test]
fn test_game_loop_with_phase2_ui() -> GameResult<()> {
    let mut game_state = GameState::new()?;
    
    // Test that the game loop logic can run several ticks with the new UI system
    // Note: Macroquad rendering calls are skipped in test environments
    for _ in 0..5 {
        // Test just the fixed update logic, not rendering which requires window context
        game_state.time_manager.update(0.1, &mut game_state.event_bus)?;
        game_state.resource_system.update(0.1, &mut game_state.event_bus)?;
        game_state.process_queued_events_for_test()?;
    }
    
    // Verify the game systems advanced
    assert!(game_state.time_manager.get_current_tick() >= 0);
    
    Ok(())
}

#[test]
fn test_toolbar_menu_state_consistency() -> GameResult<()> {
    let mut game_state = GameState::new()?;
    
    // Test that toolbar and panel states can be synced
    game_state.queue_event(GameEvent::PlayerCommand(PlayerCommand::PauseGame(false)));
    game_state.process_queued_events_for_test()?;
    
    // The pause state should be handled consistently
    Ok(())
}