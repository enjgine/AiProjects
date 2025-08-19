// tests/save_system_integration.rs
use stellar_dominion::core::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

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
    game_state.process_queued_events_for_test().unwrap();
    
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
    let _ = new_game_state.time_manager.set_tick(save_data.tick);
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

// ==== ENHANCED ERROR RECOVERY AND ROBUSTNESS TESTS ====

#[test]
fn test_filesystem_error_disk_full_simulation() {
    // Create test directory
    let test_dir = PathBuf::from("test_saves_disk_full");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    let _planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(0)).unwrap();
    
    // Configure save system with test directory
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Simulate disk full by creating a file that fills the available space
    // We'll create a read-only directory to simulate permission issues
    let _save_path = test_dir.join("quicksave.sav");
    
    // First, save normally to verify it works
    assert!(game_state.save_system.save_game(&game_state).is_ok());
    
    // Now make the directory read-only to simulate permission denied
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&test_dir, perms).unwrap();
        
        // Save should fail with permission error
        let result = game_state.save_system.save_game(&game_state);
        assert!(result.is_err());
        
        // Restore permissions for cleanup
        let mut perms = fs::metadata(&test_dir).unwrap().permissions();
        perms.set_mode(0o755); // Read-write-execute
        fs::set_permissions(&test_dir, perms).unwrap();
    }
    
    #[cfg(windows)]
    {
        // On Windows, we'll fill the disk space instead
        // Create a very large file to simulate disk full
        let blocker_path = test_dir.join("disk_blocker.tmp");
        if let Ok(mut file) = File::create(&blocker_path) {
            // Try to write a large amount of data
            let large_data = vec![0u8; 1024 * 1024]; // 1MB chunks
            for _ in 0..100 { // Try to write 100MB
                if file.write_all(&large_data).is_err() {
                    break;
                }
            }
        }
        
        // Now try to save - may fail due to disk space
        let _result = game_state.save_system.save_game(&game_state);
        // Note: This test may pass if there's sufficient disk space
        
        // Clean up
        let _ = fs::remove_file(&blocker_path);
    }
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_filesystem_error_permission_denied() {
    let test_dir = PathBuf::from("test_saves_permission");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Create a save file and make it read-only
    let save_path = test_dir.join("quicksave.sav");
    File::create(&save_path).unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&save_path).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&save_path, perms).unwrap();
        
        // Save should fail
        let result = game_state.save_system.save_game(&game_state);
        assert!(result.is_err());
        
        // Restore permissions for cleanup
        let mut perms = fs::metadata(&save_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&save_path, perms).unwrap();
    }
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_corrupted_save_recovery() {
    let test_dir = PathBuf::from("test_saves_corruption");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    let _planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(0)).unwrap();
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone())
        .with_backup_count(3);
    
    // Save the game to create initial save and backup
    assert!(game_state.save_system.save_game(&game_state).is_ok());
    
    // Create another save to establish backup chain
    let _ship_id = game_state.ship_manager.create_ship(ShipClass::Scout, Vector2 { x: 10.0, y: 20.0 }, 0).unwrap();
    assert!(game_state.save_system.save_game(&game_state).is_ok());
    
    let save_path = test_dir.join("quicksave.sav");
    
    // Test 1: Completely corrupt the main save file
    {
        fs::write(&save_path, "COMPLETELY_CORRUPTED_DATA").unwrap();
        
        // Load should fall back to backup
        let result = game_state.save_system.load_game();
        assert!(result.is_ok(), "Should recover from backup when main save is corrupted");
    }
    
    // Test 2: Partial corruption - invalid checksum
    {
        let mut valid_save = fs::read_to_string(&test_dir.join("quicksave.bak1")).unwrap();
        // Change the checksum line to corrupt it
        valid_save = valid_save.replace("CHECKSUM:", "CHECKSUM:999999999");
        fs::write(&save_path, valid_save).unwrap();
        
        let result = game_state.save_system.load_game();
        assert!(result.is_ok(), "Should recover from backup when checksum is invalid");
    }
    
    // Test 3: Truncated save file
    {
        let valid_save = fs::read_to_string(&test_dir.join("quicksave.bak1")).unwrap();
        let truncated = &valid_save[..valid_save.len() / 2]; // Take only first half
        fs::write(&save_path, truncated).unwrap();
        
        let result = game_state.save_system.load_game();
        assert!(result.is_ok(), "Should recover from backup when save is truncated");
    }
    
    // Test 4: Invalid data format
    {
        let corrupted_data = "STELLAR_SAVE_V1\nTICK:invalid_number\nPLANETS:not_a_number";
        fs::write(&save_path, corrupted_data).unwrap();
        
        let result = game_state.save_system.load_game();
        assert!(result.is_ok(), "Should recover from backup when data format is invalid");
    }
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_backup_chain_recovery() {
    let test_dir = PathBuf::from("test_saves_backup_chain");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone())
        .with_backup_count(3);
    
    // Create multiple saves to establish backup chain
    for i in 0..5 {
        let _planet_id = game_state.planet_manager.create_planet(
            OrbitalElements { semi_major_axis: i as f32 + 1.0, period: 100.0, phase: 0.0 },
            Some(0)
        ).unwrap();
        assert!(game_state.save_system.save_game(&game_state).is_ok());
    }
    
    let save_path = test_dir.join("quicksave.sav");
    
    // Corrupt main save and first backup
    fs::write(&save_path, "CORRUPTED").unwrap();
    fs::write(&test_dir.join("quicksave.bak1"), "ALSO_CORRUPTED").unwrap();
    
    // Should recover from bak2
    let result = game_state.save_system.load_game();
    assert!(result.is_ok(), "Should recover from deeper backup when earlier backups are corrupted");
    
    // Corrupt all backups
    fs::write(&test_dir.join("quicksave.bak2"), "CORRUPTED_TOO").unwrap();
    fs::write(&test_dir.join("quicksave.bak3"), "ALL_CORRUPTED").unwrap();
    
    // Should fail when all backups are corrupted
    let result = game_state.save_system.load_game();
    assert!(result.is_err(), "Should fail when all saves and backups are corrupted");
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}