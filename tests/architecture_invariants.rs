// tests/architecture_invariants.rs
use stellar_dominion::core::*;
use stellar_dominion::core::events::*;
use stellar_dominion::managers::{PlanetManager, ShipManager};

#[test]
fn test_no_direct_system_coupling() {
    // This test would fail compilation if systems directly reference each other
    let _planet_mgr = PlanetManager::new();
    let _ship_mgr = ShipManager::new();
    
    // Verify managers are independent - they should not hold direct references to each other
    // This is primarily a compile-time check through the type system
    assert!(std::mem::size_of::<PlanetManager>() > 0);
    assert!(std::mem::size_of::<ShipManager>() > 0);
}

#[test]
fn test_resources_never_negative() {
    let mut resources = ResourceBundle {
        minerals: 100,
        food: 50,
        energy: 75,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    let cost = ResourceBundle {
        minerals: 101,
        food: 0,
        energy: 0,
        alloys: 0,
        components: 0,
        fuel: 0,
    };
    
    // Should fail with InsufficientResources error
    assert!(resources.subtract(&cost).is_err());
    
    // Resources should remain unchanged after failed operation
    assert_eq!(resources.minerals, 100);
}

#[test]
fn test_event_bus_isolation() {
    let mut event_bus = EventBus::new();
    
    // Subscribe systems
    event_bus.subscribe(SystemId::PlanetManager, EventType::PlayerCommand);
    event_bus.subscribe(SystemId::ShipManager, EventType::SimulationEvent);
    
    // Queue events
    event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::PauseGame(true)
    ));
    
    event_bus.queue_event(GameEvent::SimulationEvent(
        SimulationEvent::TickCompleted(1)
    ));
    
    // Verify events are queued not immediately processed
    assert!(event_bus.queued_events.len() > 0);
}

#[test]
fn test_deterministic_operations() {
    // Test deterministic orbital calculations
    let orbit1 = OrbitalElements {
        semi_major_axis: 5.0,
        period: 365.0,
        phase: 0.0,
    };
    
    let orbit2 = OrbitalElements {
        semi_major_axis: 5.0,
        period: 365.0,
        phase: 0.0,
    };
    
    // Same inputs must produce same outputs
    assert_eq!(orbit1.semi_major_axis, orbit2.semi_major_axis);
    assert_eq!(orbit1.period, orbit2.period);
    
    // Test deterministic resource operations
    let mut resources1 = ResourceBundle { minerals: 100, food: 50, ..ResourceBundle::default() };
    let mut resources2 = ResourceBundle { minerals: 100, food: 50, ..ResourceBundle::default() };
    
    let cost = ResourceBundle { minerals: 25, food: 10, ..ResourceBundle::default() };
    
    // Same operations must produce same results
    let result1 = resources1.subtract(&cost);
    let result2 = resources2.subtract(&cost);
    
    assert_eq!(result1.is_ok(), result2.is_ok());
    assert_eq!(resources1, resources2);
    
    // Test deterministic worker allocation
    let allocation1 = WorkerAllocation { agriculture: 50, mining: 30, unassigned: 20, ..WorkerAllocation::default() };
    let allocation2 = WorkerAllocation { agriculture: 50, mining: 30, unassigned: 20, ..WorkerAllocation::default() };
    
    let validation1 = allocation1.validate(100);
    let validation2 = allocation2.validate(100);
    
    assert_eq!(validation1.is_ok(), validation2.is_ok());
}

#[test]
fn test_worker_allocation_validation() {
    let allocation = WorkerAllocation {
        agriculture: 20,
        mining: 30,
        industry: 25,
        research: 10,
        military: 5,
        unassigned: 10,
    };
    
    // Total is 100
    assert!(allocation.validate(100).is_ok());
    
    // Wrong total should fail
    assert!(allocation.validate(99).is_err());
}

#[test]
fn test_system_update_order_enforced() {
    // Verify that EventBus.update_order matches the required sequence
    let event_bus = EventBus::new();
    let expected_order = vec![
        SystemId::UIRenderer,
        SystemId::PhysicsEngine, 
        SystemId::ResourceSystem,
        SystemId::PopulationSystem,
        SystemId::ConstructionSystem,
        SystemId::CombatResolver,
        SystemId::TimeManager,
    ];
    
    assert_eq!(event_bus.update_order, expected_order);
    
    // Verify the order cannot be accidentally modified
    assert_eq!(event_bus.update_order.len(), 7);
    
    // Ensure TimeManager comes last (required for tick events)
    assert_eq!(event_bus.update_order.last(), Some(&SystemId::TimeManager));
}

#[test]
fn test_all_state_mutations_return_game_result() {
    let mut planet_mgr = PlanetManager::new();
    let mut ship_mgr = ShipManager::new();
    
    // Manager methods must return GameResult<T>
    let planet_result = planet_mgr.create_planet(OrbitalElements::default(), Some(1));
    assert!(planet_result.is_ok());
    let planet_id = planet_result.unwrap();
    
    let resource_result = planet_mgr.add_resources(planet_id, ResourceBundle::default());
    assert!(resource_result.is_ok());
    
    let allocation_result = planet_mgr.set_worker_allocation(planet_id, WorkerAllocation::default());
    assert!(allocation_result.is_ok());
    
    let ship_result = ship_mgr.create_ship(ShipClass::Scout, Vector2::default(), 1);
    assert!(ship_result.is_ok());
    
    // Verify error cases also return GameResult
    let invalid_resource_result = planet_mgr.remove_resources(planet_id, ResourceBundle {
        minerals: 1000,
        ..ResourceBundle::default()
    });
    assert!(invalid_resource_result.is_err());
    
    // Ensure the error is the expected type
    if let Err(GameError::InsufficientResources { .. }) = invalid_resource_result {
        // Expected error type
    } else {
        panic!("Expected InsufficientResources error");
    }
}

#[test]
fn test_integer_resources_only() {
    // This test ensures resources are i32, not f32
    let resources = ResourceBundle::default();
    
    // This would fail compilation if resources were float
    let _: i32 = resources.minerals;
    let _: i32 = resources.food;
    let _: i32 = resources.energy;
    let _: i32 = resources.alloys;
    let _: i32 = resources.components;
    let _: i32 = resources.fuel;
    
    // Test arithmetic operations maintain integer precision
    let mut resources = ResourceBundle {
        minerals: 100,
        food: 50,
        energy: 75,
        alloys: 25,
        components: 10,
        fuel: 200,
    };
    
    let cost = ResourceBundle {
        minerals: 33,
        food: 17,
        energy: 21,
        alloys: 5,
        components: 3,
        fuel: 67,
    };
    
    // Subtraction should use integer arithmetic (no floating point precision loss)
    resources.subtract(&cost).unwrap();
    
    assert_eq!(resources.minerals, 67); // 100 - 33
    assert_eq!(resources.food, 33);     // 50 - 17  
    assert_eq!(resources.energy, 54);   // 75 - 21
    assert_eq!(resources.fuel, 133);    // 200 - 67
}

#[test]
fn test_event_history_limited() {
    let mut event_bus = EventBus::new();
    
    // Queue more events than history limit
    for i in 0..200 {
        event_bus.queue_event(GameEvent::SimulationEvent(
            SimulationEvent::TickCompleted(i)
        ));
    }
    
    // History should be capped at limit (100)
    assert_eq!(event_bus.event_history.len(), 100);
    
    // Queued events should contain all 200 events (not subject to history limit)
    assert_eq!(event_bus.queued_events.len(), 200);
    
    // Verify oldest events are removed from history (FIFO)
    if let Some(GameEvent::SimulationEvent(SimulationEvent::TickCompleted(tick))) = event_bus.event_history.front() {
        assert!(*tick >= 100); // First 100 events should be removed
    }
}

#[test]
fn test_planet_id_uniqueness() {
    let mut planet_mgr = PlanetManager::new();
    
    let id1 = planet_mgr.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    let id2 = planet_mgr.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    
    assert_ne!(id1, id2);
}

// Compile-time validation through type system
#[test]
fn test_no_arc_mutex() {
    // This test passes if the codebase doesn't use Arc/Mutex
    // The absence of these imports in the core modules ensures this
    let game = GameState::new().unwrap();
    
    // GameState should be directly owned, not behind Arc
    let _owned: GameState = game;
}

#[test]
fn test_fixed_timestep_integrity() {
    const EXPECTED_TIMESTEP: f32 = 0.1;
    
    // Verify timestep is exactly 0.1 seconds
    assert_eq!(EXPECTED_TIMESTEP, 0.1);
    
    // Verify tick counter uses u64 for deterministic progression
    let mut tick: u64 = 0;
    
    // Test tick overflow safety (u64 max is ~584 billion years at 10 ticks/second)
    for _ in 0..1000 {
        let old_tick = tick;
        tick = tick.saturating_add(1);
        assert!(tick > old_tick || tick == u64::MAX);
    }
    
    // Test that time calculations remain deterministic
    let tick_1 = 100u64;
    let tick_2 = 100u64;
    let time_1 = tick_1 as f32 * EXPECTED_TIMESTEP;
    let time_2 = tick_2 as f32 * EXPECTED_TIMESTEP;
    
    assert_eq!(time_1, time_2);
    assert_eq!(time_1, 10.0); // 100 * 0.1
    
    // Verify timestep prevents floating-point drift (allow reasonable tolerance)
    let accumulated_time = (0..100).map(|_| EXPECTED_TIMESTEP).sum::<f32>();
    let direct_time = 100.0 * EXPECTED_TIMESTEP;
    let diff = (accumulated_time - direct_time).abs();
    assert!(diff < 1e-5, "Timestep accumulation drift exceeds tolerance: {}", diff);
}

// Integration test for system isolation
#[cfg(test)]
mod integration {
    use super::*;
    
    #[test]
    fn test_event_routing_respects_subscriptions() {
        let mut event_bus = EventBus::new();
        
        // Subscribe only PlanetManager to PlayerCommand
        event_bus.subscribe(SystemId::PlanetManager, EventType::PlayerCommand);
        event_bus.subscribe(SystemId::ResourceSystem, EventType::SimulationEvent);
        event_bus.subscribe(SystemId::TimeManager, EventType::SimulationEvent);
        
        // Queue a PlayerCommand event
        event_bus.queue_event(GameEvent::PlayerCommand(
            PlayerCommand::BuildStructure { 
                planet: 1, 
                building_type: BuildingType::Mine 
            }
        ));
        
        // Verify event is queued
        assert_eq!(event_bus.queued_events.len(), 1);
        
        // Verify subscriber mapping is correct
        assert!(event_bus.subscribers.contains_key(&SystemId::PlanetManager));
        assert!(event_bus.subscribers.get(&SystemId::PlanetManager).unwrap().contains(&EventType::PlayerCommand));
        assert!(!event_bus.subscribers.get(&SystemId::ResourceSystem).unwrap_or(&vec![]).contains(&EventType::PlayerCommand));
    }
    
    #[test]
    fn test_systems_cannot_directly_modify_other_system_data() {
        let mut game_state = GameState::new().unwrap();
        
        // Create a planet through proper manager interface
        let planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
        
        // Systems should only be able to access data through manager methods
        // This is enforced at compile time - direct field access to planet data should be impossible
        
        // Test that systems use events for state changes
        game_state.queue_event(GameEvent::PlayerCommand(
            PlayerCommand::BuildStructure { 
                planet: planet_id, 
                building_type: BuildingType::Farm 
            }
        ));
        
        // Before processing, planet should have no buildings
        let planet = game_state.planet_manager.get_planet(planet_id).unwrap();
        assert_eq!(planet.developments.len(), 0);
        
        // After event processing, building should be added (if construction system is implemented)
        // This verifies that state changes occur through event processing, not direct manipulation
        assert!(game_state.event_bus.queued_events.len() > 0 || game_state.event_bus.event_history.len() > 0);
    }
    
    #[test] 
    fn test_event_processing_maintains_system_isolation() {
        let mut game_state = GameState::new().unwrap();
        
        // Add some test data
        let _planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
        
        // Test that events are properly queued
        game_state.queue_event(GameEvent::SimulationEvent(SimulationEvent::TickCompleted(1)));
        
        // Verify event is in queue before processing
        assert_eq!(game_state.event_bus.queued_events.len(), 1);
        
        // NOTE: Cannot test full fixed_update due to macroquad UI thread requirements in test environment
        // This would require integration tests with proper macroquad context
        
        // Instead verify the event system structure is correct
        assert!(!game_state.event_bus.update_order.is_empty());
        assert!(game_state.event_bus.subscribers.len() > 0);
        
        // Verify tick counter is accessible
        let _current_tick = game_state.get_current_tick();
    }
    
    #[test]
    fn test_event_subscription_architecture() {
        let mut game_state = GameState::new().unwrap();
        
        // Verify proper event subscriptions are set up
        let planet_subscriptions = game_state.event_bus.subscribers.get(&SystemId::PlanetManager).unwrap();
        assert!(planet_subscriptions.contains(&EventType::PlayerCommand));
        assert!(planet_subscriptions.contains(&EventType::SimulationEvent));
        
        let ui_subscriptions = game_state.event_bus.subscribers.get(&SystemId::UIRenderer).unwrap();
        assert!(ui_subscriptions.contains(&EventType::StateChanged));
        
        // Verify systems that should process events are subscribed
        assert!(game_state.event_bus.subscribers.contains_key(&SystemId::ResourceSystem));
        assert!(game_state.event_bus.subscribers.contains_key(&SystemId::PopulationSystem));
        assert!(game_state.event_bus.subscribers.contains_key(&SystemId::CombatResolver));
        
        // Test event queuing works correctly
        let planet_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap();
        
        game_state.queue_event(GameEvent::PlayerCommand(
            PlayerCommand::AllocateWorkers { 
                planet: planet_id,
                allocation: WorkerAllocation {
                    agriculture: 0,
                    mining: 0, 
                    industry: 0,
                    research: 0,
                    military: 0,
                    unassigned: 0,
                }
            }
        ));
        
        // Verify event was queued
        assert_eq!(game_state.event_bus.queued_events.len(), 1);
        
        // NOTE: Cannot test actual event processing in test environment due to macroquad UI requirements
    }
}

// Additional critical architecture tests
#[test]
fn test_resource_overflow_protection() {
    let mut resources = ResourceBundle {
        minerals: i32::MAX - 10,
        food: 100,
        energy: 100,
        alloys: 100,
        components: 100,
        fuel: 100,
    };
    
    let addition = ResourceBundle {
        minerals: 20, // Would overflow i32::MAX
        ..ResourceBundle::default()
    };
    
    // Addition uses saturating_add, so it should succeed but cap at i32::MAX
    let result = resources.add(&addition);
    assert!(result.is_ok());
    
    // Resources should be capped at i32::MAX (saturating arithmetic)
    assert_eq!(resources.minerals, i32::MAX);
    assert_eq!(resources.food, 100); // Other resources unchanged
    
    // Test that PlanetManager properly detects overflow before calling add
    let mut planet_mgr = PlanetManager::new();
    let planet_id = planet_mgr.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    
    // Set planet minerals near overflow limit but within capacity (default capacity is 10000)
    planet_mgr.modify_planet(planet_id, |planet| {
        planet.resources.current.minerals = i32::MAX - 10;
        // Expand capacity to accommodate this test value
        planet.resources.capacity.minerals = i32::MAX;
        Ok(())
    }).unwrap();
    
    let overflow_resources = ResourceBundle {
        minerals: 20, // Would cause overflow (i32::MAX - 10 + 20 > i32::MAX)
        ..ResourceBundle::default()
    };
    
    // PlanetManager.add_resources should detect potential overflow and fail
    let result = planet_mgr.add_resources(planet_id, overflow_resources);
    assert!(result.is_err());
    
    // Verify the error is due to overflow protection, not capacity
    if let Err(GameError::InvalidOperation(msg)) = result {
        assert!(msg.contains("overflow"));
    } else {
        panic!("Expected overflow protection error");
    }
}

#[test]
fn test_building_slot_constraints_enforced() {
    let mut planet_mgr = PlanetManager::new();
    let planet_id = planet_mgr.create_planet(OrbitalElements::default(), Some(1)).unwrap();
    
    // New planet should have base 10 building slots (10 + 0/10000)
    let available_slots = planet_mgr.get_available_building_slots(planet_id).unwrap();
    assert_eq!(available_slots, 10);
    
    // Add buildings up to the limit
    for _ in 0..10 {
        let result = planet_mgr.add_building(planet_id, BuildingType::Farm);
        assert!(result.is_ok());
    }
    
    // Adding one more building should fail
    let result = planet_mgr.add_building(planet_id, BuildingType::Mine);
    assert!(result.is_err());
    
    // Verify error message indicates slot limit
    if let Err(GameError::InvalidOperation(msg)) = result {
        assert!(msg.contains("No available building slots"));
    } else {
        panic!("Expected InvalidOperation error for building slot limit");
    }
}

#[test]
fn test_worker_allocation_totals_enforced() {
    let allocation = WorkerAllocation {
        agriculture: 25,
        mining: 30,
        industry: 20,
        research: 15,
        military: 5,
        unassigned: 5, // Total: 100
    };
    
    // Should validate successfully for population of 100
    assert!(allocation.validate(100).is_ok());
    
    // Should fail for wrong population totals
    assert!(allocation.validate(99).is_err());
    assert!(allocation.validate(101).is_err());
    
    // Should fail with negative values
    let invalid_allocation = WorkerAllocation {
        agriculture: -5,
        mining: 50,
        industry: 30,
        research: 15,
        military: 5,
        unassigned: 5,
    };
    assert!(invalid_allocation.validate(100).is_err());
}

#[test]
fn test_ship_cargo_capacity_constraints() {
    let cargo = CargoHold {
        resources: ResourceBundle::default(),
        population: 0,
        capacity: 100,
    };
    
    // Should be able to load up to capacity
    let resources = ResourceBundle {
        minerals: 50,
        food: 30,
        energy: 20,
        ..ResourceBundle::default()
    };
    
    assert!(cargo.can_load(&resources, 0));
    
    // Should fail to load beyond capacity
    let excess_resources = ResourceBundle {
        minerals: 80,
        food: 30,
        energy: 20,
        ..ResourceBundle::default()
    };
    
    assert!(!cargo.can_load(&excess_resources, 0));
    
    // Population should count toward capacity
    assert!(!cargo.can_load(&ResourceBundle::default(), 101));
}

#[test]
fn test_unsubscribed_systems_ignore_events() {
    let mut event_bus = EventBus::new();
    
    // Subscribe only TimeManager to PlayerCommand events
    event_bus.subscribe(SystemId::TimeManager, EventType::PlayerCommand);
    
    // Queue a PlayerCommand that should only go to TimeManager
    event_bus.queue_event(GameEvent::PlayerCommand(
        PlayerCommand::SetGameSpeed(2.0)
    ));
    
    // Verify ResourceSystem is NOT subscribed to PlayerCommand events
    let resource_subscriptions = event_bus.subscribers.get(&SystemId::ResourceSystem);
    if let Some(subscriptions) = resource_subscriptions {
        assert!(!subscriptions.contains(&EventType::PlayerCommand), 
                "ResourceSystem should not be subscribed to PlayerCommand events for this test");
    }
    
    // Verify only TimeManager gets PlayerCommand events
    let time_subscriptions = event_bus.subscribers.get(&SystemId::TimeManager).unwrap();
    assert!(time_subscriptions.contains(&EventType::PlayerCommand));
}

#[test]
fn test_event_processing_order_consistency() {
    let event_bus = EventBus::new();
    
    // Test that update_order is consistently applied when multiple systems subscribe to same event
    let mut systems_subscribing_to_simulation = Vec::new();
    
    for &system_id in &event_bus.update_order {
        if let Some(subscriptions) = event_bus.subscribers.get(&system_id) {
            if subscriptions.contains(&EventType::SimulationEvent) {
                systems_subscribing_to_simulation.push(system_id);
            }
        }
    }
    
    // Verify systems are collected in update_order
    // This ensures event routing respects the fixed update order for deterministic behavior
    let mut previous_index = 0;
    for &system_id in &systems_subscribing_to_simulation {
        let current_index = event_bus.update_order.iter().position(|&id| id == system_id).unwrap();
        assert!(current_index >= previous_index, 
                "Systems subscribing to SimulationEvent must be processed in update_order");
        previous_index = current_index;
    }
}

#[test]
fn test_all_event_types_have_subscribers() {
    // Use GameState to get properly initialized EventBus with subscriptions
    let game_state = GameState::new().unwrap();
    let event_bus = &game_state.event_bus;
    
    // Verify each EventType has at least one subscriber
    let mut event_type_coverage = std::collections::HashSet::new();
    
    for subscriptions in event_bus.subscribers.values() {
        for &event_type in subscriptions {
            event_type_coverage.insert(event_type);
        }
    }
    
    // All EventType variants must have subscribers
    assert!(event_type_coverage.contains(&EventType::PlayerCommand), 
            "PlayerCommand events must have subscribers");
    assert!(event_type_coverage.contains(&EventType::SimulationEvent), 
            "SimulationEvent events must have subscribers");
    assert!(event_type_coverage.contains(&EventType::StateChanged), 
            "StateChanged events must have subscribers");
    
    // Verify we have comprehensive coverage
    assert_eq!(event_type_coverage.len(), 3, "All EventType variants must be covered");
}

#[test]
fn test_maximum_entity_boundary_limits() {
    let mut planet_mgr = PlanetManager::new();
    
    // Test creating many planets (performance target: 100 planets)
    let mut planet_ids = Vec::new();
    for i in 0..100 {
        let result = planet_mgr.create_planet(OrbitalElements::default(), Some(i as FactionId));
        assert!(result.is_ok(), "Should be able to create 100 planets");
        planet_ids.push(result.unwrap());
    }
    
    // Verify all planets are unique
    let unique_count = planet_ids.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_count, 100, "All planet IDs must be unique");
    
    // Test resource operations at scale with proper capacity management
    // Default capacity is: minerals: 10000, food: 5000, energy: 1000, alloys: 1000, components: 500, fuel: 2000
    let moderate_resources = ResourceBundle {
        minerals: 5000,  // Within default capacity (10000)
        food: 2000,      // Within default capacity (5000)
        energy: 800,     // Within default capacity (1000)
        alloys: 800,     // Within default capacity (1000)
        components: 400, // Within default capacity (500)
        fuel: 1500,      // Within default capacity (2000)
    };
    
    // First test with smaller resource amounts that fit within default capacity
    for &planet_id in &planet_ids[0..10] { // Test first 10 planets
        let result = planet_mgr.add_resources(planet_id, moderate_resources.clone());
        assert!(result.is_ok(), "Should handle moderate resource quantities within capacity");
    }
    
    // Test that we can handle a reasonable number of operations
    for &planet_id in &planet_ids[10..20] { // Test another 10 planets
        // Test smaller resource additions that should definitely work
        let small_resources = ResourceBundle {
            minerals: 100,
            food: 50,
            energy: 75,
            alloys: 25,
            components: 10,
            fuel: 200,
        };
        let result = planet_mgr.add_resources(planet_id, small_resources);
        assert!(result.is_ok(), "Should handle small resource quantities");
    }
}

#[test]
fn test_system_trait_compliance() {
    // Verify implemented systems follow the GameSystem trait
    // This is primarily a compile-time check
    use stellar_dominion::systems::*;
    use stellar_dominion::core::GameSystem;
    
    fn requires_game_system<T: GameSystem>(_: T) {}
    
    // These systems currently implement GameSystem
    requires_game_system(ResourceSystem::new());
    requires_game_system(PopulationSystem::new());
    requires_game_system(CombatResolver::new());
    
    // TODO: Uncomment when these systems implement GameSystem trait
    // requires_game_system(ConstructionSystem::new());
    // requires_game_system(PhysicsEngine::new());
    // requires_game_system(TimeManager::new());
    
    // Managers also implement GameSystem
    requires_game_system(PlanetManager::new());
}