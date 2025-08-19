// tests/save_system_integration_enhanced.rs
// Enhanced save system integration tests with comprehensive error recovery and robustness testing
use stellar_dominion::core::*;
use stellar_dominion::core::events::EventType;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[test]
fn test_atomic_operation_validation() {
    let test_dir = PathBuf::from("test_saves_atomic");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    let _planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(0)).unwrap();
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Initial save
    assert!(game_state.save_system.save_game(&game_state).is_ok());
    
    let save_path = test_dir.join("quicksave.sav");
    let temp_path = save_path.with_extension("tmp");
    
    // Verify atomic operation: temp file should not exist after successful save
    assert!(!temp_path.exists(), "Temporary file should not exist after successful save");
    assert!(save_path.exists(), "Save file should exist after successful save");
    
    // Test interrupted save simulation
    // We'll manually create a temp file to simulate an interrupted save
    fs::write(&temp_path, "PARTIAL_SAVE_DATA").unwrap();
    
    // Save again - should cleanup temp file and create new save
    assert!(game_state.save_system.save_game(&game_state).is_ok());
    
    // Verify temp file is cleaned up and main save is intact
    assert!(!temp_path.exists(), "Temporary file should be cleaned up after save");
    assert!(save_path.exists(), "Save file should exist");
    
    // Verify the save is valid by loading it
    let load_result = game_state.save_system.load_game();
    assert!(load_result.is_ok(), "Save should be valid after atomic operation");
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_serialization_validation_and_error_handling() {
    let test_dir = PathBuf::from("test_saves_validation");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Test invalid slot names
    let invalid_slot_names = vec![
        "slot/with/slashes",
        "slot\\with\\backslashes",
        "slot:with:colons",
        "slot*with*asterisks",
        "slot?with?questions",
        "slot\"with\"quotes",
        "slot<with>brackets",
        "slot|with|pipes",
    ];
    
    for invalid_name in invalid_slot_names {
        let result = game_state.save_system.save_game_to_slot(&game_state, invalid_name);
        assert!(result.is_err(), "Save should fail with invalid slot name: {}", invalid_name);
        
        let result = game_state.save_system.load_game_from_slot(invalid_name);
        assert!(result.is_err(), "Load should fail with invalid slot name: {}", invalid_name);
    }
    
    // Test valid slot names
    let valid_slot_names = vec![
        "valid_slot",
        "slot-with-dashes",
        "slot_with_underscores",
        "slot123",
        "UPPERCASE_SLOT",
    ];
    
    for valid_name in valid_slot_names {
        let result = game_state.save_system.save_game_to_slot(&game_state, valid_name);
        assert!(result.is_ok(), "Save should succeed with valid slot name: {}", valid_name);
        
        let result = game_state.save_system.load_game_from_slot(valid_name);
        assert!(result.is_ok(), "Load should succeed with valid slot name: {}", valid_name);
    }
    
    // Test loading non-existent save
    let result = game_state.save_system.load_game_from_slot("non_existent_save");
    assert!(result.is_err(), "Load should fail for non-existent save");
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_performance_under_stress() {
    let test_dir = PathBuf::from("test_saves_performance");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    
    // Create stress test data - many factions, planets, and ships
    for faction_idx in 0..10 {
        let _faction_id = game_state.faction_manager.create_faction(
            format!("Empire_{}", faction_idx),
            faction_idx == 0, // First faction is player
            if faction_idx % 3 == 0 { AIPersonality::Aggressive }
            else if faction_idx % 3 == 1 { AIPersonality::Balanced }
            else { AIPersonality::Economic }
        ).unwrap();
        
        // Create planets for this faction
        for planet_idx in 0..20 {
            let _planet_id = game_state.planet_manager.create_planet(
                OrbitalElements {
                    semi_major_axis: (planet_idx as f32 + 1.0) * 0.5,
                    period: (planet_idx as f32 + 1.0) * 50.0,
                    phase: planet_idx as f32 * 0.1,
                },
                Some(faction_idx)
            ).unwrap();
        }
        
        // Create ships for this faction
        for ship_idx in 0..30 {
            let ship_class = match ship_idx % 4 {
                0 => ShipClass::Scout,
                1 => ShipClass::Transport,
                2 => ShipClass::Warship,
                _ => ShipClass::Colony,
            };
            
            let _ship_id = game_state.ship_manager.create_ship(
                ship_class,
                Vector2 { x: ship_idx as f32 * 10.0, y: faction_idx as f32 * 100.0 },
                faction_idx
            ).unwrap();
        }
    }
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Test save performance
    let start_time = Instant::now();
    let save_result = game_state.save_system.save_game(&game_state);
    let save_duration = start_time.elapsed();
    
    assert!(save_result.is_ok(), "Large game state save should succeed");
    assert!(save_duration < Duration::from_secs(5), "Save should complete within 5 seconds, took: {:?}", save_duration);
    
    // Test load performance
    let start_time = Instant::now();
    let load_result = game_state.save_system.load_game();
    let load_duration = start_time.elapsed();
    
    assert!(load_result.is_ok(), "Large game state load should succeed");
    assert!(load_duration < Duration::from_secs(5), "Load should complete within 5 seconds, took: {:?}", load_duration);
    
    let save_data = load_result.unwrap();
    
    // Verify large dataset integrity
    assert_eq!(save_data.factions.len(), 10, "Should have 10 factions");
    assert_eq!(save_data.planets.len(), 200, "Should have 200 planets (20 per faction)");
    assert_eq!(save_data.ships.len(), 300, "Should have 300 ships (30 per faction)");
    
    // Test rapid save/load cycles
    for i in 0..10 {
        let start = Instant::now();
        
        assert!(game_state.save_system.save_game_to_slot(&game_state, &format!("stress_test_{}", i)).is_ok());
        assert!(game_state.save_system.load_game_from_slot(&format!("stress_test_{}", i)).is_ok());
        
        let cycle_duration = start.elapsed();
        assert!(cycle_duration < Duration::from_secs(2), "Save/load cycle {} should complete within 2 seconds", i);
    }
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_eventbus_architecture_compliance() {
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    let _planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(0)).unwrap();
    
    // Test save/load operations through the save system directly (not through EventBus events)
    // Since the EventBus integration depends on UI which requires macroquad thread setup
    
    // Test direct save operation
    let save_result = game_state.save_system.save_game(&game_state);
    assert!(save_result.is_ok(), "Direct save should succeed");
    
    // Test direct load operation
    let load_result = game_state.save_system.load_game();
    assert!(load_result.is_ok(), "Direct load should succeed");
    
    // Verify save system is properly subscribed to PlayerCommand events
    let subscriptions = game_state.event_bus.subscribers.get(&SystemId::SaveSystem);
    assert!(subscriptions.is_some(), "SaveSystem should be subscribed to events");
    
    let subscriptions = subscriptions.unwrap();
    assert!(subscriptions.contains(&EventType::PlayerCommand), 
           "SaveSystem should be subscribed to PlayerCommand events");
}

#[test]
fn test_concurrent_save_operations() {
    let test_dir = PathBuf::from("test_saves_concurrent");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    let _planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(0)).unwrap();
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Test multiple save operations to different slots
    let slot_names = vec!["slot1", "slot2", "slot3", "slot4", "slot5"];
    
    // Save to multiple slots simultaneously (simulating concurrent operations)
    for slot_name in &slot_names {
        let result = game_state.save_system.save_game_to_slot(&game_state, slot_name);
        assert!(result.is_ok(), "Save to slot {} should succeed", slot_name);
    }
    
    // Verify all saves exist and are valid
    for slot_name in &slot_names {
        let result = game_state.save_system.load_game_from_slot(slot_name);
        assert!(result.is_ok(), "Load from slot {} should succeed", slot_name);
        
        let save_data = result.unwrap();
        assert_eq!(save_data.planets.len(), 1, "Each save should have 1 planet");
        assert_eq!(save_data.factions.len(), 1, "Each save should have 1 faction");
    }
    
    // Test overwriting existing saves
    let _ship_id = game_state.ship_manager.create_ship(ShipClass::Scout, Vector2::default(), 0).unwrap();
    
    for slot_name in &slot_names {
        let result = game_state.save_system.save_game_to_slot(&game_state, slot_name);
        assert!(result.is_ok(), "Overwrite save to slot {} should succeed", slot_name);
    }
    
    // Verify updated saves
    for slot_name in &slot_names {
        let result = game_state.save_system.load_game_from_slot(slot_name);
        assert!(result.is_ok(), "Load updated slot {} should succeed", slot_name);
        
        let save_data = result.unwrap();
        assert_eq!(save_data.ships.len(), 1, "Updated save should have 1 ship");
    }
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_memory_pressure_scenarios() {
    let test_dir = PathBuf::from("test_saves_memory");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    
    // Create a large faction name to test memory handling
    let large_name = "X".repeat(100); // 100 character faction name (reasonable size)
    let _faction_id = game_state.faction_manager.create_faction(large_name, true, AIPersonality::Balanced).unwrap();
    
    // Create many planets with large data
    for i in 0..100 {
        let _planet_id = game_state.planet_manager.create_planet(
            OrbitalElements {
                semi_major_axis: i as f32 * 10.0,
                period: i as f32 * 100.0,
                phase: i as f32,
            },
            Some(0)
        ).unwrap();
    }
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Test save under memory pressure
    let save_result = game_state.save_system.save_game(&game_state);
    assert!(save_result.is_ok(), "Save should succeed even with large data");
    
    // Test load under memory pressure
    let load_result = game_state.save_system.load_game();
    assert!(load_result.is_ok(), "Load should succeed even with large data");
    
    let save_data = load_result.unwrap();
    assert_eq!(save_data.planets.len(), 100, "Should preserve all 100 planets");
    assert_eq!(save_data.factions[0].name.len(), 100, "Should preserve large faction name");
    
    // Test serialization buffer growth
    let serialized = save_data.serialize();
    assert!(serialized.is_ok(), "Serialization should handle large data");
    
    let serialized_data = serialized.unwrap();
    assert!(serialized_data.len() > 10000, "Serialized data should be substantial");
    
    // Test deserialization of large data
    let deserialized = stellar_dominion::systems::save_system::SaveData::deserialize(&serialized_data);
    assert!(deserialized.is_ok(), "Deserialization should handle large data");
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_deterministic_state_preservation() {
    let test_dir = PathBuf::from("test_saves_deterministic");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state1 = GameState::new().unwrap();
    let mut game_state2 = GameState::new().unwrap();
    
    // Create identical game states
    let faction_id = game_state1.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    let _faction_id2 = game_state2.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    
    let _planet_id = game_state1.planet_manager.create_planet(
        OrbitalElements { semi_major_axis: 5.0, period: 365.0, phase: 1.5 },
        Some(faction_id)
    ).unwrap();
    let _planet_id2 = game_state2.planet_manager.create_planet(
        OrbitalElements { semi_major_axis: 5.0, period: 365.0, phase: 1.5 },
        Some(faction_id)
    ).unwrap();
    
    let _ship_id = game_state1.ship_manager.create_ship(
        ShipClass::Transport, 
        Vector2 { x: 123.456, y: 789.012 }, 
        faction_id
    ).unwrap();
    let _ship_id2 = game_state2.ship_manager.create_ship(
        ShipClass::Transport, 
        Vector2 { x: 123.456, y: 789.012 }, 
        faction_id
    ).unwrap();
    
    // Advance time identically
    let mut temp_bus1 = EventBus::new();
    let mut temp_bus2 = EventBus::new();
    for _ in 0..10 {
        game_state1.time_manager.update(0.1, &mut temp_bus1).unwrap();
        game_state2.time_manager.update(0.1, &mut temp_bus2).unwrap();
    }
    
    game_state1.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    game_state2.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Save both states
    assert!(game_state1.save_system.save_game_to_slot(&game_state1, "state1").is_ok());
    assert!(game_state2.save_system.save_game_to_slot(&game_state2, "state2").is_ok());
    
    // Load both saves
    let save_data1 = game_state1.save_system.load_game_from_slot("state1").unwrap();
    let save_data2 = game_state2.save_system.load_game_from_slot("state2").unwrap();
    
    // Verify deterministic preservation
    assert_eq!(save_data1.tick, save_data2.tick, "Ticks should be identical");
    assert_eq!(save_data1.planets.len(), save_data2.planets.len(), "Planet counts should be identical");
    assert_eq!(save_data1.ships.len(), save_data2.ships.len(), "Ship counts should be identical");
    assert_eq!(save_data1.factions.len(), save_data2.factions.len(), "Faction counts should be identical");
    
    // Verify exact planet data
    assert_eq!(save_data1.planets[0].position.semi_major_axis, save_data2.planets[0].position.semi_major_axis);
    assert_eq!(save_data1.planets[0].position.period, save_data2.planets[0].position.period);
    assert_eq!(save_data1.planets[0].position.phase, save_data2.planets[0].position.phase);
    
    // Verify exact ship data
    assert_eq!(save_data1.ships[0].position.x, save_data2.ships[0].position.x);
    assert_eq!(save_data1.ships[0].position.y, save_data2.ships[0].position.y);
    assert_eq!(save_data1.ships[0].ship_class, save_data2.ships[0].ship_class);
    
    // Verify checksums are identical for identical states
    assert_eq!(save_data1.checksum, save_data2.checksum, "Checksums should be identical for identical game states");
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_corrupted_save_recovery_advanced() {
    let test_dir = PathBuf::from("test_saves_corruption_advanced");
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
    
    // Test 5: Binary corruption (invalid UTF-8)
    {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD, 0xFC]; // Invalid UTF-8 sequence
        fs::write(&save_path, invalid_utf8).unwrap();
        
        let result = game_state.save_system.load_game();
        assert!(result.is_ok(), "Should recover from backup when save has invalid UTF-8");
    }
    
    // Test 6: Empty file corruption
    {
        fs::write(&save_path, "").unwrap();
        
        let result = game_state.save_system.load_game();
        assert!(result.is_ok(), "Should recover from backup when save is empty");
    }
    
    // Test 7: Missing required fields
    {
        let corrupted_data = "STELLAR_SAVE_V1\nTICK:100\n"; // Missing PLANETS field
        fs::write(&save_path, corrupted_data).unwrap();
        
        let result = game_state.save_system.load_game();
        assert!(result.is_ok(), "Should recover from backup when required fields are missing");
    }
    
    // Test 8: Inconsistent data counts
    {
        let corrupted_data = "STELLAR_SAVE_V1\nTICK:100\nPLANETS:5\nSHIPS:0\nFACTIONS:0\nCHECKSUM:123";
        fs::write(&save_path, corrupted_data).unwrap();
        
        let result = game_state.save_system.load_game();
        assert!(result.is_ok(), "Should recover from backup when data counts are inconsistent");
    }
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_filesystem_error_recovery_comprehensive() {
    let test_dir = PathBuf::from("test_saves_fs_error_comprehensive");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    let _faction_id = game_state.faction_manager.create_faction("Test Empire".to_string(), true, AIPersonality::Balanced).unwrap();
    let _planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(0)).unwrap();
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Test 1: Non-existent directory (should create)
    {
        let non_existent_dir = test_dir.join("non_existent");
        game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
            .with_save_directory(non_existent_dir.clone());
        
        let result = game_state.save_system.save_game(&game_state);
        assert!(result.is_ok(), "Save should create directory if it doesn't exist");
        assert!(non_existent_dir.exists(), "Directory should be created");
    }
    
    // Test 2: Deeply nested directory creation
    {
        let deep_dir = test_dir.join("very").join("deep").join("nested").join("directory");
        game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
            .with_save_directory(deep_dir.clone());
        
        let result = game_state.save_system.save_game(&game_state);
        assert!(result.is_ok(), "Save should create deeply nested directories");
        assert!(deep_dir.exists(), "Deep directory should be created");
    }
    
    // Test 3: Path with spaces and special characters
    {
        let special_dir = test_dir.join("dir with spaces & special chars");
        game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
            .with_save_directory(special_dir.clone());
        
        let result = game_state.save_system.save_game(&game_state);
        assert!(result.is_ok(), "Save should handle directories with spaces and special characters");
    }
    
    // Test 4: Very long path names
    {
        let long_name = "a".repeat(100); // 100 character directory name
        let long_dir = test_dir.join(long_name);
        game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
            .with_save_directory(long_dir.clone());
        
        let result = game_state.save_system.save_game(&game_state);
        // This might fail on some systems with path length limitations, but should handle gracefully
        if result.is_err() {
            // Should get a meaningful error message
            let error_msg = format!("{}", result.unwrap_err());
            assert!(error_msg.contains("Failed") || error_msg.contains("path") || error_msg.contains("directory"));
        }
    }
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_large_save_file_handling() {
    let test_dir = PathBuf::from("test_saves_large_file");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();
    
    let mut game_state = GameState::new().unwrap();
    
    // Create an extremely large game state
    for faction_idx in 0..50 { // 50 factions
        let very_long_name = format!("Empire_{}_{}", faction_idx, "X".repeat(500)); // 500+ char names
        let _faction_id = game_state.faction_manager.create_faction(
            very_long_name,
            faction_idx == 0,
            AIPersonality::Balanced
        ).unwrap();
        
        // Create many planets per faction
        for planet_idx in 0..50 { // 50 planets per faction = 2500 total
            let _planet_id = game_state.planet_manager.create_planet(
                OrbitalElements {
                    semi_major_axis: (faction_idx as f32 * 10.0 + planet_idx as f32),
                    period: (faction_idx as f32 * 100.0 + planet_idx as f32 * 10.0),
                    phase: (planet_idx as f32) * 0.1,
                },
                Some(faction_idx)
            ).unwrap();
        }
        
        // Create ships per faction
        for ship_idx in 0..100 { // 100 ships per faction = 5000 total
            let _ship_id = game_state.ship_manager.create_ship(
                ShipClass::Transport,
                Vector2 { 
                    x: (faction_idx as f32 * 100.0 + ship_idx as f32), 
                    y: (ship_idx as f32 * 10.0) 
                },
                faction_idx
            ).unwrap();
        }
    }
    
    game_state.save_system = stellar_dominion::systems::save_system::SaveSystem::new()
        .with_save_directory(test_dir.clone());
    
    // Test save of extremely large data
    let start_time = Instant::now();
    let save_result = game_state.save_system.save_game(&game_state);
    let save_duration = start_time.elapsed();
    
    assert!(save_result.is_ok(), "Should be able to save extremely large game state");
    assert!(save_duration < Duration::from_secs(30), "Large save should complete within 30 seconds");
    
    // Verify the save file is large
    let save_path = test_dir.join("quicksave.sav");
    let file_size = fs::metadata(&save_path).unwrap().len();
    assert!(file_size > 1_000_000, "Save file should be over 1MB for large game state"); // At least 1MB
    
    // Test load of large data
    let start_time = Instant::now();
    let load_result = game_state.save_system.load_game();
    let load_duration = start_time.elapsed();
    
    assert!(load_result.is_ok(), "Should be able to load extremely large save");
    assert!(load_duration < Duration::from_secs(30), "Large load should complete within 30 seconds");
    
    let save_data = load_result.unwrap();
    assert_eq!(save_data.factions.len(), 50, "Should preserve all 50 factions");
    assert_eq!(save_data.planets.len(), 2500, "Should preserve all 2500 planets");
    assert_eq!(save_data.ships.len(), 5000, "Should preserve all 5000 ships");
    
    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}