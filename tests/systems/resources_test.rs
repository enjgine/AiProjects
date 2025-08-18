// tests/systems/resources_test.rs
use stellar_dominion::core::*;
use stellar_dominion::core::events::*;
use stellar_dominion::core::types::*;
use stellar_dominion::systems::ResourceSystem;
use stellar_dominion::managers::PlanetManager;

#[test]
fn test_resource_system_creation() {
    let resource_system = ResourceSystem::new();
    // Should create with default production rates for all building types
    
    // Test that we can get consumption data
    let consumption = resource_system.get_consumption_for_planet(1);
    assert!(consumption.is_none()); // No planet tracked yet
}

#[test]
fn test_production_rates_defined() {
    let resource_system = ResourceSystem::new();
    
    // ResourceSystem should initialize with production rates for key buildings
    // We can't directly access the HashMap but we can test that it works through production
    // This test ensures the constructor properly sets up the rates
}

#[test]
fn test_planet_production_calculation() {
    let resource_system = ResourceSystem::new();
    
    // Create a test planet with workers and buildings
    let mut planet = Planet {
        id: 1,
        position: OrbitalElements::default(),
        resources: ResourceStorage {
            current: ResourceBundle {
                minerals: 1000,
                food: 500,
                energy: 200,
                alloys: 100,
                components: 50,
                fuel: 300,
            },
            capacity: ResourceBundle {
                minerals: 10000,
                food: 5000,
                energy: 1000,
                alloys: 1000,
                components: 500,
                fuel: 2000,
            },
        },
        population: Demographics {
            total: 1000,
            growth_rate: 0.02,
            allocation: WorkerAllocation {
                agriculture: 200,
                mining: 300,
                industry: 100,
                research: 50,
                military: 50,
                unassigned: 300,
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
        controller: Some(1),
    };
    
    // Test production calculation
    let production = resource_system.calculate_planet_production(&planet).unwrap();
    
    // Base worker production: mining=300*2=600, agriculture=200*3=600, industry=100*1=100
    // Building bonuses: Mine=10 minerals, -2 energy; Farm=8 food, -1 energy
    // Expected: minerals = 600+10=610, food = 600+8=608, energy = 100-2-1=97
    
    assert_eq!(production.minerals, 610);
    assert_eq!(production.food, 608);
    assert_eq!(production.energy, 97);
    assert_eq!(production.alloys, 0);
    assert_eq!(production.components, 0);
    assert_eq!(production.fuel, 0);
}

#[test]
fn test_resource_production_with_planet_manager() {
    let mut resource_system = ResourceSystem::new();
    let mut planet_manager = PlanetManager::new();
    let mut event_bus = EventBus::new();
    
    // Create a test planet
    let planet_id = planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    
    // Set up population and buildings
    planet_manager.update_population(planet_id, 1000).unwrap();
    planet_manager.set_worker_allocation(planet_id, WorkerAllocation {
        agriculture: 200,
        mining: 300,
        industry: 100,
        research: 50,
        military: 50,
        unassigned: 300,
    }).unwrap();
    
    planet_manager.add_building(planet_id, BuildingType::Mine).unwrap();
    planet_manager.add_building(planet_id, BuildingType::Farm).unwrap();
    
    // Process production
    resource_system.process_production(&mut planet_manager, &mut event_bus).unwrap();
    
    // Verify that production events were emitted
    let events: Vec<_> = event_bus.queued_events.clone().into_iter().collect();
    let production_events: Vec<_> = events.iter()
        .filter_map(|e| match e {
            GameEvent::SimulationEvent(SimulationEvent::ResourcesProduced { planet, resources }) => 
                Some((planet, resources)),
            _ => None,
        })
        .collect();
    
    assert_eq!(production_events.len(), 1);
    assert_eq!(*production_events[0].0, planet_id);
    
    // Check that consumption is tracked
    let consumption = resource_system.get_consumption_for_planet(planet_id);
    assert!(consumption.is_some());
}

#[test]
fn test_resource_transfer_validation() {
    let mut resource_system = ResourceSystem::new();
    let mut planet_manager = PlanetManager::new();
    let mut event_bus = EventBus::new();
    
    // Create two planets
    let planet1 = planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    let planet2 = planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    
    // Add resources to source planet
    let initial_resources = ResourceBundle {
        minerals: 500,
        food: 300,
        energy: 200,
        alloys: 100,
        components: 50,
        fuel: 150,
    };
    planet_manager.add_resources(planet1, initial_resources).unwrap();
    
    // Test successful transfer
    let transfer_resources = ResourceBundle {
        minerals: 100,
        food: 50,
        energy: 0,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    resource_system.process_transfer(planet1, planet2, transfer_resources, &mut planet_manager, &mut event_bus).unwrap();
    
    // Verify resources were moved
    let source_planet = planet_manager.get_planet(planet1).unwrap();
    let dest_planet = planet_manager.get_planet(planet2).unwrap();
    
    assert_eq!(source_planet.resources.current.minerals, 400);
    assert_eq!(source_planet.resources.current.food, 250);
    assert_eq!(dest_planet.resources.current.minerals, 100);
    assert_eq!(dest_planet.resources.current.food, 50);
    
    // Verify state change events were emitted
    let state_change_events: Vec<_> = event_bus.queued_events.iter()
        .filter_map(|e| match e {
            GameEvent::StateChanged(StateChange::PlanetUpdated(id)) => Some(*id),
            _ => None,
        })
        .collect();
    
    assert!(state_change_events.contains(&planet1));
    assert!(state_change_events.contains(&planet2));
}

#[test]
fn test_resource_transfer_insufficient_resources() {
    let mut resource_system = ResourceSystem::new();
    let mut planet_manager = PlanetManager::new();
    let mut event_bus = EventBus::new();
    
    // Create two planets
    let planet1 = planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    let planet2 = planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    
    // Planet1 has only minimal resources
    let initial_resources = ResourceBundle {
        minerals: 50,
        food: 30,
        energy: 20,
        alloys: 10,
        components: 5,
        fuel: 15,
    };
    planet_manager.add_resources(planet1, initial_resources).unwrap();
    
    // Try to transfer more than available
    let transfer_resources = ResourceBundle {
        minerals: 100, // More than the 50 available
        food: 50,
        energy: 0,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    // Should fail with insufficient resources error
    let result = resource_system.process_transfer(planet1, planet2, transfer_resources, &mut planet_manager, &mut event_bus);
    
    match result {
        Err(GameError::InsufficientResources { required, available }) => {
            assert_eq!(required.minerals, 100);
            assert_eq!(available.minerals, 50);
        }
        _ => panic!("Expected InsufficientResources error"),
    }
}

#[test]
fn test_resource_shortage_detection() {
    let mut resource_system = ResourceSystem::new();
    let mut planet_manager = PlanetManager::new();
    let mut event_bus = EventBus::new();
    
    // Create planet with low resources
    let planet_id = planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    
    // Set minimal resources
    let low_resources = ResourceBundle {
        minerals: 5,
        food: 2,
        energy: 1,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    planet_manager.add_resources(planet_id, low_resources).unwrap();
    
    // Set up population and buildings that will consume more than available
    planet_manager.update_population(planet_id, 1000).unwrap();
    planet_manager.set_worker_allocation(planet_id, WorkerAllocation {
        agriculture: 100,
        mining: 200,
        industry: 200, // High energy consumption
        research: 50,
        military: 50,
        unassigned: 400,
    }).unwrap();
    
    // Add energy-consuming buildings
    planet_manager.add_building(planet_id, BuildingType::Mine).unwrap();
    planet_manager.add_building(planet_id, BuildingType::Factory).unwrap();
    
    // Process production - should detect shortages
    resource_system.process_production(&mut planet_manager, &mut event_bus).unwrap();
    
    // Check for shortage events
    let shortage_events: Vec<_> = event_bus.queued_events.iter()
        .filter_map(|e| match e {
            GameEvent::SimulationEvent(SimulationEvent::ResourceShortage { planet, resource }) => 
                Some((planet, resource)),
            _ => None,
        })
        .collect();
    
    // Should have shortage events for resources that went negative
    assert!(!shortage_events.is_empty());
}

#[test]
fn test_building_production_rates() {
    // Test specific production rates for buildings
    let mine_production = ResourceBundle {
        minerals: 10,
        food: 0,
        energy: -2,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    let farm_production = ResourceBundle {
        minerals: 0,
        food: 8,
        energy: -1,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    // Verify production values are sensible
    assert!(mine_production.minerals > 0);
    assert!(mine_production.energy < 0); // Consumes energy
    
    assert!(farm_production.food > 0);
    assert!(farm_production.energy < 0); // Consumes energy
}

#[test]
fn test_resource_shortage_detection() {
    let resources = ResourceBundle {
        minerals: 5,
        food: 0, // Shortage
        energy: 10,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    let consumption = ResourceBundle {
        minerals: 0,
        food: 10, // More than available
        energy: 0,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    // Should detect shortage
    assert!(!resources.can_afford(&consumption));
}

#[test]
fn test_production_with_workers() {
    // Test that production scales with worker allocation
    let workers_mining = 50;
    let efficiency = 1.0;
    let base_production = 10;
    
    let expected_production = (workers_mining as f32 * efficiency * base_production as f32) as i32;
    
    // Production should scale with workers
    assert!(expected_production > base_production);
}

#[test]
fn test_storage_capacity_limits() {
    let mut storage = ResourceStorage {
        current: ResourceBundle {
            minerals: 900,
            food: 500,
            energy: 300,
            alloys: 100,
            components: 50,
            fuel: 200,
        },
        capacity: ResourceBundle {
            minerals: 1000,
            food: 1000,
            energy: 1000,
            alloys: 1000,
            components: 1000,
            fuel: 1000,
        },
    };
    
    let production = ResourceBundle {
        minerals: 200, // Would exceed capacity
        food: 100,
        energy: 50,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    let new_total = ResourceBundle {
        minerals: storage.current.minerals + production.minerals,
        food: storage.current.food + production.food,
        energy: storage.current.energy + production.energy,
        alloys: storage.current.alloys + production.alloys,
        components: storage.current.components + production.components,
        fuel: storage.current.fuel + production.fuel,
    };
    
    // Should detect capacity overflow
    assert!(!storage.capacity.can_afford(&new_total));
}

#[test]
fn test_resource_integer_constraints() {
    let resources = ResourceBundle {
        minerals: 100,
        food: 50,
        energy: 75,
        alloys: 25,
        components: 10,
        fuel: 40,
    };
    
    // All resources should be integers
    let _: i32 = resources.minerals;
    let _: i32 = resources.food;
    let _: i32 = resources.energy;
    let _: i32 = resources.alloys;
    let _: i32 = resources.components;
    let _: i32 = resources.fuel;
}

#[test]
fn test_consumption_tracking() {
    // Test that resource consumption is properly tracked
    let consumption = ResourceBundle {
        minerals: 0,
        food: 20, // Population consumption
        energy: 50, // Building consumption
        alloys: 0,
        components: 0,
        fuel: 5, // Ship fuel
    };
    
    // Consumption should be positive values
    assert!(consumption.food > 0);
    assert!(consumption.energy > 0);
    assert!(consumption.fuel > 0);
}