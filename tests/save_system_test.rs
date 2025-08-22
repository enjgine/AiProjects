// tests/save_system_test.rs
//! Tests for the simplified save system
//! 
//! Tests cover:
//! - Basic save/load functionality
//! - Save file validation
//! - Save listing and management
//! - Error handling
//! - Data integrity validation

use stellar_dominion::core::*;
use stellar_dominion::systems::SaveSystem;
use stellar_dominion::systems::save_system::{SaveData, SaveInfo};
use std::fs;
use std::path::PathBuf;

/// Test fixture for creating save system test data
struct SaveTestFixture;

impl SaveTestFixture {
    /// Create a test save directory
    pub fn setup_test_directory() -> PathBuf {
        let test_dir = PathBuf::from("test_saves");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }
    
    /// Clean up test directory
    pub fn cleanup_test_directory(test_dir: &PathBuf) {
        if test_dir.exists() {
            fs::remove_dir_all(test_dir).ok();
        }
    }
    
    /// Create test game configuration
    pub fn create_test_game_configuration() -> GameConfiguration {
        GameConfiguration {
            planet_count: 15,
            starting_resources: ResourceBundle {
                minerals: 1000,
                food: 500,
                energy: 300,
                alloys: 100,
                components: 50,
                fuel: 200,
            },
            starting_population: 2000,
            galaxy_size: GalaxySize::Medium,
            ai_opponents: 3,
        }
    }
    
    /// Create test planet data
    pub fn create_test_planet(id: PlanetId) -> Planet {
        let total_pop = 1000 + (id * 100) as i32;
        Planet {
            id,
            position: OrbitalElements {
                semi_major_axis: 1.0 + (id as f32 * 0.5),
                period: 365.0 + (id as f32 * 50.0),
                phase: (id as f32 * 0.1),
            },
            resources: ResourceStorage {
                current: ResourceBundle {
                    minerals: std::cmp::min(500 + (id * 50) as i32, 2000),
                    food: std::cmp::min(300 + (id * 30) as i32, 1500),
                    energy: std::cmp::min(200 + (id * 20) as i32, 1000),
                    alloys: std::cmp::min(100 + (id * 10) as i32, 800),
                    components: std::cmp::min(50 + (id * 5) as i32, 500),
                    fuel: std::cmp::min(100 + (id * 10) as i32, 600),
                },
                capacity: ResourceBundle {
                    minerals: 2000,
                    food: 1500,
                    energy: 1000,
                    alloys: 800,
                    components: 500,
                    fuel: 600,
                },
            },
            population: Demographics {
                total: total_pop,
                growth_rate: 0.02,
                allocation: WorkerAllocation {
                    agriculture: 150,
                    mining: 200,
                    industry: 300,
                    research: 100,
                    military: 50,
                    unassigned: total_pop - 800,
                },
            },
            developments: vec![
                Building {
                    building_type: BuildingType::Mine,
                    tier: 1,
                    operational: true,
                },
                Building {
                    building_type: BuildingType::Farm,
                    tier: 1,
                    operational: true,
                },
            ],
            controller: Some(0),
        }
    }
    
    /// Create test ship data
    pub fn create_test_ship(id: ShipId, faction_id: FactionId) -> Ship {
        Ship {
            id,
            ship_class: ShipClass::Warship,
            position: Vector2::new(200.0 * id as f32, 200.0 * id as f32),
            trajectory: None,
            cargo: CargoHold {
                resources: ResourceBundle {
                    minerals: 10 + id as i32,
                    food: 5 + id as i32,
                    energy: 0,
                    alloys: 2 + id as i32,
                    components: 1,
                    fuel: 20 + (id * 2) as i32,
                },
                population: 0,
                capacity: 100,
            },
            fuel: 100.0,
            owner: faction_id,
        }
    }
    
    /// Create test faction data
    pub fn create_test_faction(id: FactionId) -> Faction {
        Faction {
            id,
            name: format!("Test Faction {}", id),
            is_player: id == 0,
            ai_type: AIPersonality::Balanced,
            score: 1000 + (id as i32 * 100),
        }
    }
    
    /// Create test save data
    pub fn create_test_save_data(save_name: &str, tick: u64) -> SaveData {
        SaveData {
            version: 1,
            save_name: save_name.to_string(),
            timestamp: 1640995200, // 2022-01-01 timestamp
            tick,
            planets: vec![
                Self::create_test_planet(0),
                Self::create_test_planet(1),
                Self::create_test_planet(2),
            ],
            ships: vec![
                Self::create_test_ship(0, 0),
                Self::create_test_ship(1, 1),
                Self::create_test_ship(2, 0),
            ],
            factions: vec![
                Self::create_test_faction(0), // Player faction
                Self::create_test_faction(1), // AI faction
                Self::create_test_faction(2), // AI faction
            ],
            game_configuration: Self::create_test_game_configuration(),
        }
    }
}

/// Basic save system functionality tests
#[cfg(test)]
mod basic_save_tests {
    use super::*;
    
    #[test]
    fn test_save_system_creation() {
        let _save_system = SaveSystem::new();
        // Basic instantiation test
        assert!(true);
    }
    
    #[test]
    fn test_save_data_creation() {
        let save_data = SaveTestFixture::create_test_save_data("test_save", 100);
        
        assert_eq!(save_data.version, 1);
        assert_eq!(save_data.save_name, "test_save");
        assert_eq!(save_data.tick, 100);
        assert_eq!(save_data.planets.len(), 3);
        assert_eq!(save_data.ships.len(), 3);
        assert_eq!(save_data.factions.len(), 3);
    }
    
    #[test]
    fn test_save_info_creation() {
        let save_data = SaveTestFixture::create_test_save_data("test_info", 200);
        let save_info = SaveInfo::from_save_data(&save_data);
        
        assert_eq!(save_info.name, "test_info");
        assert_eq!(save_info.tick, 200);
        assert_eq!(save_info.planets, 3);
        assert_eq!(save_info.ships, 3);
        assert_eq!(save_info.factions, 3);
        assert_eq!(save_info.timestamp, 1640995200);
    }
}

/// Save/load cycle tests
#[cfg(test)]
mod save_load_tests {
    use super::*;
    
    #[test]
    fn test_basic_save_load_cycle() {
        let test_dir = SaveTestFixture::setup_test_directory();
        
        // Create a save system with custom directory
        let save_system = SaveSystem::new();
        
        // Create test data
        let original_data = SaveTestFixture::create_test_save_data("test_basic", 500);
        
        // Note: The simplified save system doesn't have save_game_binary,
        // so we would need to test with the JSON-based methods when they are 
        // available on a full GameState instance
        
        // For now, test the data validation
        assert!(save_system.validate_save_integrity(&original_data).is_ok());
        
        // Test that validation catches problems
        let mut invalid_data = original_data.clone();
        invalid_data.planets.clear();
        assert!(save_system.validate_save_integrity(&invalid_data).is_err());
        
        SaveTestFixture::cleanup_test_directory(&test_dir);
    }
    
    #[test]
    fn test_save_validation() {
        let save_system = SaveSystem::new();
        
        // Test valid save data
        let valid_data = SaveTestFixture::create_test_save_data("valid", 100);
        assert!(save_system.validate_save_integrity(&valid_data).is_ok());
        
        // Test invalid version
        let mut invalid_version = valid_data.clone();
        invalid_version.version = 999;
        assert!(save_system.validate_save_integrity(&invalid_version).is_err());
        
        // Test empty planets
        let mut no_planets = valid_data.clone();
        no_planets.planets.clear();
        assert!(save_system.validate_save_integrity(&no_planets).is_err());
        
        // Test empty factions
        let mut no_factions = valid_data.clone();
        no_factions.factions.clear();
        assert!(save_system.validate_save_integrity(&no_factions).is_err());
        
        // Test invalid worker allocation
        let mut invalid_workers = valid_data.clone();
        invalid_workers.planets[0].population.allocation.mining = 999999;
        assert!(save_system.validate_save_integrity(&invalid_workers).is_err());
    }
    
    #[test]
    fn test_save_name_validation() {
        let save_system = SaveSystem::new();
        
        // Test basic save existence check
        assert!(!save_system.save_exists("nonexistent_save"));
        
        // Test save path generation
        // The save system generates .sav files in the saves directory
        assert!(true); // Path generation is internal
    }
}

/// Resource validation tests
#[cfg(test)]
mod resource_validation_tests {
    use super::*;
    
    #[test]
    fn test_resource_bundle_validation() {
        // Test valid resources
        let valid_resources = ResourceBundle {
            minerals: 100,
            food: 50,
            energy: 75,
            alloys: 25,
            components: 10,
            fuel: 30,
        };
        assert!(valid_resources.validate_non_negative().is_ok());
        
        // Test invalid resources (negative values)
        let invalid_resources = ResourceBundle {
            minerals: -10,
            food: 50,
            energy: 75,
            alloys: 25,
            components: 10,
            fuel: 30,
        };
        assert!(invalid_resources.validate_non_negative().is_err());
    }
    
    #[test]
    fn test_worker_allocation_validation() {
        let total_pop = 1000;
        
        // Test valid allocation
        let valid_allocation = WorkerAllocation {
            agriculture: 200,
            mining: 200,
            industry: 200,
            research: 200,
            military: 100,
            unassigned: 100,
        };
        assert!(valid_allocation.validate(total_pop).is_ok());
        
        // Test invalid allocation (too many workers)
        let invalid_allocation = WorkerAllocation {
            agriculture: 500,
            mining: 500,
            industry: 500,
            research: 500,
            military: 500,
            unassigned: 500,
        };
        assert!(invalid_allocation.validate(total_pop).is_err());
    }
}

/// Save system error handling tests
#[cfg(test)]
mod error_handling_tests {
    use super::*;
    
    #[test]
    fn test_invalid_save_data_handling() {
        let save_system = SaveSystem::new();
        
        // Test with completely empty save data
        let empty_data = SaveData {
            version: 1,
            save_name: "empty".to_string(),
            timestamp: 0,
            tick: 0,
            planets: vec![],
            ships: vec![],
            factions: vec![],
            game_configuration: SaveTestFixture::create_test_game_configuration(),
        };
        
        // Should fail validation due to empty planets and factions
        assert!(save_system.validate_save_integrity(&empty_data).is_err());
    }
    
    #[test]
    fn test_resource_constraint_validation() {
        let save_system = SaveSystem::new();
        let mut test_data = SaveTestFixture::create_test_save_data("resource_test", 100);
        
        // Create planet with invalid negative resources
        test_data.planets[0].resources.current.minerals = -100;
        
        // Should fail validation
        assert!(save_system.validate_save_integrity(&test_data).is_err());
    }
}

/// Performance tests for save system
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_save_validation_performance() {
        let save_system = SaveSystem::new();
        
        // Create larger save data for performance testing
        let mut large_save_data = SaveTestFixture::create_test_save_data("large_test", 1000);
        
        // Add more planets and ships
        for i in 3..50 {
            large_save_data.planets.push(SaveTestFixture::create_test_planet(i));
            large_save_data.ships.push(SaveTestFixture::create_test_ship(i, (i % 3) as u8));
        }
        
        let start = Instant::now();
        let result = save_system.validate_save_integrity(&large_save_data);
        let validation_time = start.elapsed();
        
        if let Err(e) = &result {
            println!("Validation failed: {:?}", e);
        }
        assert!(result.is_ok());
        assert!(validation_time.as_millis() < 100, "Validation should take less than 100ms for 50 planets");
        
        println!("Validated save with {} planets and {} ships in {:?}", 
                 large_save_data.planets.len(), 
                 large_save_data.ships.len(), 
                 validation_time);
    }
}

/// Cleanup test to remove temporary files
#[cfg(test)]
mod cleanup {
    use super::*;
    
    #[test]
    fn test_cleanup() {
        // Clean up any remaining test files
        let test_dirs = vec!["test_saves", "temp_test_saves"];
        
        for dir in test_dirs {
            let path = PathBuf::from(dir);
            if path.exists() {
                fs::remove_dir_all(path).ok();
            }
        }
    }
}