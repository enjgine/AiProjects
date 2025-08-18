// tests/save_system_integration.rs
use stellar_dominion::core::*;
use stellar_dominion::core::events::*;
use stellar_dominion::core::types::*;

#[test]
fn test_save_system_integration() {
    let mut game_state = GameState::new().unwrap();
    
    // Create test data
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    let planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(0)).unwrap();
    let ship_id = game_state.ship_manager.create_ship(ShipClass::Scout, Vector2 { x: 0.0, y: 0.0 }, 0).unwrap();
    
    // Advance time by one tick (0.1 seconds)
    let mut temp_event_bus = EventBus::new();
    game_state.time_manager.update(0.1, &mut temp_event_bus).unwrap();
    let current_tick = game_state.time_manager.get_current_tick();
    
    // Test SaveGame event
    game_state.event_bus.queue_event(GameEvent::PlayerCommand(PlayerCommand::SaveGame));
    
    // Process the save event
    let mut temp_event_bus = std::mem::replace(&mut game_state.event_bus, EventBus::new());
    temp_event_bus.process_events(&mut game_state).unwrap();
    game_state.event_bus = temp_event_bus;
    
    // Verify save file was created by attempting to load
    let load_result = game_state.save_system.load_game();
    assert!(load_result.is_ok());
    
    let save_data = load_result.unwrap();
    assert_eq!(save_data.tick, current_tick);
    assert_eq!(save_data.planets.len(), 1);
    assert_eq!(save_data.ships.len(), 1);
    assert_eq!(save_data.factions.len(), 1);
    
    // Verify planet data
    assert_eq!(save_data.planets[0].id, planet_id);
    assert_eq!(save_data.planets[0].controller, Some(0));
    
    // Verify ship data  
    assert_eq!(save_data.ships[0].id, ship_id);
    assert_eq!(save_data.ships[0].ship_class, ShipClass::Scout);
    assert_eq!(save_data.ships[0].owner, 0);
    
    // Verify faction data
    assert_eq!(save_data.factions[0].id, 0);
    assert_eq!(save_data.factions[0].name, "Test Empire");
    assert!(save_data.factions[0].is_player);
}

#[test]
fn test_load_game_restores_state() {
    let mut original_game_state = GameState::new().unwrap();
    
    // Create test data
    let _faction_id = original_game_state.faction_manager.create_faction("Load Test".to_string(), false, AIPersonality::Economic).unwrap();
    let planet_id = original_game_state.planet_manager.create_planet(
        OrbitalElements { semi_major_axis: 5.0, period: 100.0, phase: 0.5 }, 
        Some(0)
    ).unwrap();
    
    // Advance time to tick 42 (each update with 0.1 advances one tick)
    let mut temp_event_bus = EventBus::new();
    for _ in 0..42 {
        original_game_state.time_manager.update(0.1, &mut temp_event_bus).unwrap();
    }
    
    // Save the game state
    original_game_state.save_system.save_game(&original_game_state).unwrap();
    
    // Create new game state and load
    let mut new_game_state = GameState::new().unwrap();
    let save_data = new_game_state.save_system.load_game().unwrap();
    
    // Apply the loaded data
    new_game_state.time_manager.set_tick(save_data.tick);
    new_game_state.planet_manager.load_planets(save_data.planets).unwrap();
    new_game_state.ship_manager.load_ships(save_data.ships).unwrap();
    new_game_state.faction_manager.load_factions(save_data.factions).unwrap();
    
    // Verify state was restored correctly
    assert_eq!(new_game_state.time_manager.get_current_tick(), 42);
    
    let planet = new_game_state.planet_manager.get_planet(planet_id).unwrap();
    assert_eq!(planet.position.semi_major_axis, 5.0);
    assert_eq!(planet.position.period, 100.0);
    assert_eq!(planet.controller, Some(0));
    
    let faction = new_game_state.faction_manager.get_faction(0).unwrap();
    assert_eq!(faction.name, "Load Test");
    assert!(!faction.is_player);
}

#[test]
fn test_save_load_preserves_game_integrity() {
    let mut game_state = GameState::new().unwrap();
    
    // Create a complex game state
    let faction1 = game_state.faction_manager.create_faction("Empire A".to_string(), true, AIPersonality::Aggressive).unwrap();
    let faction2 = game_state.faction_manager.create_faction("Empire B".to_string(), false, AIPersonality::Balanced).unwrap();
    
    let planet1 = game_state.planet_manager.create_planet(
        OrbitalElements { semi_major_axis: 3.0, period: 50.0, phase: 0.0 }, 
        Some(faction1)
    ).unwrap();
    
    let planet2 = game_state.planet_manager.create_planet(
        OrbitalElements { semi_major_axis: 7.0, period: 200.0, phase: 1.5 }, 
        Some(faction2)
    ).unwrap();
    
    let ship1 = game_state.ship_manager.create_ship(ShipClass::Transport, Vector2 { x: 100.0, y: 200.0 }, faction1).unwrap();
    let ship2 = game_state.ship_manager.create_ship(ShipClass::Warship, Vector2 { x: -50.0, y: 75.0 }, faction2).unwrap();
    
    // Save and load
    game_state.save_system.save_game(&game_state).unwrap();
    let save_data = game_state.save_system.load_game().unwrap();
    
    // Verify all relationships are preserved
    assert_eq!(save_data.planets[0].controller, Some(faction1));
    assert_eq!(save_data.planets[1].controller, Some(faction2));
    assert_eq!(save_data.ships[0].owner, faction1);
    assert_eq!(save_data.ships[1].owner, faction2);
    
    // Verify IDs are consistent
    assert_eq!(save_data.planets[0].id, planet1);
    assert_eq!(save_data.planets[1].id, planet2);
    assert_eq!(save_data.ships[0].id, ship1);
    assert_eq!(save_data.ships[1].id, ship2);
}