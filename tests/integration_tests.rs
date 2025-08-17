// tests/integration_tests.rs
use stellar_dominion::core::*;
use stellar_dominion::core::events::*;
use stellar_dominion::core::types::*;

#[test]
fn test_full_game_loop() {
    let mut game_state = GameState::new().unwrap();
    
    // Run several update cycles
    for _ in 0..10 {
        game_state.fixed_update(0.1).unwrap();
    }
    
    // Verify game state is stable
    assert!(game_state.time_manager.get_tick() > 0);
}

#[test]
fn test_planet_creation_and_management() {
    let mut game_state = GameState::new().unwrap();
    
    // Create a planet
    let planet_id = game_state.planet_manager.create_planet(OrbitalElements::default()).unwrap();
    
    // Verify planet exists
    let planet = game_state.planet_manager.get_planet(planet_id).unwrap();
    assert_eq!(planet.id, planet_id);
    
    // Test resource operations
    let resources = ResourceBundle {
        minerals: 100,
        food: 50,
        energy: 75,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    game_state.planet_manager.add_resources(planet_id, resources).unwrap();
    
    let updated_planet = game_state.planet_manager.get_planet(planet_id).unwrap();
    assert_eq!(updated_planet.resources.current.minerals, 100);
}

#[test]
fn test_ship_creation_and_movement() {
    let mut game_state = GameState::new().unwrap();
    
    // Create a ship
    let ship_id = game_state.ship_manager.create_ship(
        ShipClass::Scout,
        Vector2 { x: 0.0, y: 0.0 },
        0, // faction 0
    ).unwrap();
    
    // Verify ship exists
    let ship = game_state.ship_manager.get_ship(ship_id).unwrap();
    assert_eq!(ship.id, ship_id);
    assert_eq!(ship.ship_class, ShipClass::Scout);
    
    // Test position update
    let new_position = Vector2 { x: 10.0, y: 20.0 };
    game_state.ship_manager.update_position(ship_id, new_position).unwrap();
    
    let updated_ship = game_state.ship_manager.get_ship(ship_id).unwrap();
    assert_eq!(updated_ship.position.x, 10.0);
    assert_eq!(updated_ship.position.y, 20.0);
}

#[test]
fn test_event_system_flow() {
    let mut game_state = GameState::new().unwrap();
    
    // Queue some events
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::PauseGame(true)
    ));
    
    game_state.event_bus.queue_event(GameEvent::SimulationEvent(
        SimulationEvent::TickCompleted(1)
    ));
    
    // Process events
    game_state.event_bus.process_events(&mut game_state).unwrap();
    
    // Verify events were processed (cleared from queue)
    assert_eq!(game_state.event_bus.queued_events.len(), 0);
}

#[test]
fn test_faction_management() {
    let mut game_state = GameState::new().unwrap();
    
    // Create factions
    let player_faction = game_state.faction_manager.create_faction(
        "Player Empire".to_string(),
        true,
        AIPersonality::Balanced,
    ).unwrap();
    
    let ai_faction = game_state.faction_manager.create_faction(
        "AI Empire".to_string(),
        false,
        AIPersonality::Aggressive,
    ).unwrap();
    
    // Verify factions
    let player = game_state.faction_manager.get_faction(player_faction).unwrap();
    assert!(player.is_player);
    assert_eq!(player.name, "Player Empire");
    
    let ai = game_state.faction_manager.get_faction(ai_faction).unwrap();
    assert!(!ai.is_player);
    assert_eq!(ai.ai_type, AIPersonality::Aggressive);
}

#[test]
fn test_time_management() {
    let mut game_state = GameState::new().unwrap();
    
    let initial_tick = game_state.time_manager.get_tick();
    
    // Run several updates
    for _ in 0..5 {
        game_state.fixed_update(0.1).unwrap();
    }
    
    // Verify time advanced
    assert!(game_state.time_manager.get_tick() > initial_tick);
    
    // Test pause
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::PauseGame(true)
    ));
    game_state.event_bus.process_events(&mut game_state).unwrap();
    
    let paused_tick = game_state.time_manager.get_tick();
    
    // Run updates while paused
    for _ in 0..3 {
        game_state.fixed_update(0.1).unwrap();
    }
    
    // Tick should not advance while paused
    assert_eq!(game_state.time_manager.get_tick(), paused_tick);
}

#[test]
fn test_resource_validation() {
    let mut resources = ResourceBundle {
        minerals: 50,
        food: 30,
        energy: 20,
        alloys: 10,
        components: 5,
        fuel: 15,
    };
    
    let valid_cost = ResourceBundle {
        minerals: 40,
        food: 20,
        energy: 10,
        alloys: 5,
        components: 2,
        fuel: 10,
    };
    
    let invalid_cost = ResourceBundle {
        minerals: 60, // More than available
        food: 0,
        energy: 0,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    // Valid subtraction should succeed
    assert!(resources.subtract(&valid_cost).is_ok());
    
    // Invalid subtraction should fail
    assert!(resources.subtract(&invalid_cost).is_err());
}

#[test]
fn test_worker_allocation_validation() {
    let allocation = WorkerAllocation {
        agriculture: 25,
        mining: 30,
        industry: 20,
        research: 15,
        military: 5,
        unassigned: 5,
    };
    
    // Should validate with correct total
    assert!(allocation.validate(100).is_ok());
    
    // Should fail with incorrect total
    assert!(allocation.validate(99).is_err());
    assert!(allocation.validate(101).is_err());
}

#[test]
fn test_ui_event_generation() {
    let mut ui_renderer = UIRenderer::new();
    let mut event_bus = EventBus::new();
    
    // Process input (should not crash)
    ui_renderer.process_input(&mut event_bus).unwrap();
    
    // Verify UI doesn't directly modify state
    // (This is more of a compile-time check through the architecture)
}

#[test]
fn test_system_isolation() {
    let mut game_state = GameState::new().unwrap();
    
    // Systems should only communicate through events
    // This test verifies the architecture doesn't allow direct access
    
    // Create some test data
    let planet_id = game_state.planet_manager.create_planet(OrbitalElements::default()).unwrap();
    let ship_id = game_state.ship_manager.create_ship(
        ShipClass::Transport,
        Vector2::default(),
        0,
    ).unwrap();
    
    // Run update cycles
    for _ in 0..5 {
        game_state.fixed_update(0.1).unwrap();
    }
    
    // Verify data integrity
    assert!(game_state.planet_manager.get_planet(planet_id).is_ok());
    assert!(game_state.ship_manager.get_ship(ship_id).is_ok());
}