// tests/physics_engine_test.rs
use stellar_dominion::core::types::*;
use stellar_dominion::core::events::*;
use stellar_dominion::systems::PhysicsEngine;

/// Additional comprehensive tests for the improved PhysicsEngine

#[test]
fn test_orbital_position_calculation() {
    let physics = PhysicsEngine::new();
    
    let orbital_elements = OrbitalElements {
        semi_major_axis: 5.0,
        period: 100.0, // 100 ticks for full orbit
        phase: 0.0,
    };
    
    // Test position at start
    let pos_0 = physics.calculate_orbital_position(&orbital_elements, 0);
    assert!((pos_0.x - 5.0).abs() < 0.01); // Should be at semi_major_axis
    assert!((pos_0.y - 0.0).abs() < 0.01); // Should be at y=0
    
    // Test position at quarter orbit (25 ticks)
    let pos_25 = physics.calculate_orbital_position(&orbital_elements, 25);
    assert!((pos_25.x - 0.0).abs() < 0.01); // Should be near x=0
    assert!((pos_25.y - 5.0).abs() < 0.01); // Should be at semi_major_axis
    
    // Test position at half orbit (50 ticks)
    let pos_50 = physics.calculate_orbital_position(&orbital_elements, 50);
    assert!((pos_50.x - (-5.0)).abs() < 0.01); // Should be at -semi_major_axis
    assert!((pos_50.y - 0.0).abs() < 0.01); // Should be at y=0
}

#[test]
fn test_public_api_functionality() {
    let physics = PhysicsEngine::new();
    
    // Test the public methods that are available
    let orbital_elements = OrbitalElements {
        semi_major_axis: 5.0,
        period: 100.0,
        phase: 0.0,
    };
    
    // Test calculate_orbital_position with different ticks
    let pos1 = physics.calculate_orbital_position(&orbital_elements, 0);
    let pos2 = physics.calculate_orbital_position(&orbital_elements, 25);
    
    // Verify positions are different (orbit is working)
    assert!((pos1.x - pos2.x).abs() > 1.0 || (pos1.y - pos2.y).abs() > 1.0);
    
    // Test get_ship_position
    let ship_position = physics.get_ship_position(123, Vector2 { x: 10.0, y: 20.0 });
    
    // Should return base position when no trajectory exists
    assert!((ship_position.x - 10.0).abs() < 0.01);
    assert!((ship_position.y - 20.0).abs() < 0.01);
    
    // Test get_orbital_position
    let orbital_pos = physics.get_orbital_position(456);
    assert!(orbital_pos.is_none()); // Should be None when no planet cached
}

#[test]
fn test_tick_completed_event_handling() {
    let mut physics = PhysicsEngine::new();
    let mut event_bus = EventBus::new();
    
    // Create tick completed event
    let event = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(42));
    
    // Handle the event
    physics.handle_event(&event).unwrap();
    
    // Verify the event was handled (we can't access private fields directly)
    // The test passes if no error occurred during event handling
    
    // Process tick in update
    physics.update(0.0, &mut event_bus).unwrap();
}

#[test]
fn test_move_ship_command_handling() {
    let mut physics = PhysicsEngine::new();
    
    let target = Vector2 { x: 50.0, y: 50.0 };
    let ship_id = 123;
    
    let event = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
        ship: ship_id,
        target,
    });
    
    // Handle the move command
    physics.handle_event(&event).unwrap();
    
    // Verify trajectory was planned (we can't access private fields directly)
    // The test passes if no error occurred during command handling
}

#[test]
fn test_physics_engine_integration() {
    let mut physics = PhysicsEngine::new();
    let mut event_bus = EventBus::new();
    
    // Test that PhysicsEngine can handle tick events
    let tick_event = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(50));
    physics.handle_event(&tick_event).unwrap();
    physics.update(0.0, &mut event_bus).unwrap();
    
    // Test that PhysicsEngine can handle ship move commands
    let move_event = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
        ship: 123,
        target: Vector2 { x: 100.0, y: 100.0 },
    });
    physics.handle_event(&move_event).unwrap();
    
    // Test passes if no errors occur during event handling
}

#[test]
fn test_improved_error_handling() {
    let mut physics = PhysicsEngine::new();
    
    // Test invalid target coordinates
    let invalid_target = Vector2 { x: f32::NAN, y: 50.0 };
    let move_event = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
        ship: 123,
        target: invalid_target,
    });
    
    // Should return error for invalid coordinates
    assert!(physics.handle_event(&move_event).is_err());
    
    // Test infinite coordinates
    let infinite_target = Vector2 { x: f32::INFINITY, y: 50.0 };
    let move_event2 = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
        ship: 456,
        target: infinite_target,
    });
    
    assert!(physics.handle_event(&move_event2).is_err());
}

#[test]
fn test_tick_validation() {
    let mut physics = PhysicsEngine::new();
    
    // Set initial tick
    let tick1 = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(100));
    physics.handle_event(&tick1).unwrap();
    
    // Test that backwards tick is rejected
    let backwards_tick = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(50));
    assert!(physics.handle_event(&backwards_tick).is_err());
    
    // Test that forward tick is accepted
    let forward_tick = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(150));
    assert!(physics.handle_event(&forward_tick).is_ok());
}

#[test]
fn test_invalid_orbital_elements() {
    let physics = PhysicsEngine::new();
    
    // Test with zero period (should return origin)
    let invalid_orbit1 = OrbitalElements {
        semi_major_axis: 5.0,
        period: 0.0,
        phase: 0.0,
    };
    
    let pos1 = physics.calculate_orbital_position(&invalid_orbit1, 50);
    assert_eq!(pos1.x, 0.0);
    assert_eq!(pos1.y, 0.0);
    
    // Test with negative radius (should return origin)
    let invalid_orbit2 = OrbitalElements {
        semi_major_axis: -5.0,
        period: 100.0,
        phase: 0.0,
    };
    
    let pos2 = physics.calculate_orbital_position(&invalid_orbit2, 50);
    assert_eq!(pos2.x, 0.0);
    assert_eq!(pos2.y, 0.0);
}

#[test]
fn test_new_api_methods() {
    let physics = PhysicsEngine::new();
    
    // Test get_active_trajectories
    let trajectories = physics.get_active_trajectories();
    assert_eq!(trajectories.len(), 0); // Should be empty initially
    
    // Test get_transfer_window_count
    let window_count = physics.get_transfer_window_count();
    assert_eq!(window_count, 0); // Should be zero initially
    
    // Test is_transfer_window_open
    let window_open = physics.is_transfer_window_open(1, 2);
    assert_eq!(window_open, false); // Should be false initially
    
    // Test estimate_travel_time
    let from = Vector2 { x: 0.0, y: 0.0 };
    let to = Vector2 { x: 100.0, y: 0.0 };
    let travel_time = physics.estimate_travel_time(from, to);
    assert_eq!(travel_time, 10); // 100 units / 10 speed = 10 ticks
}

#[test]
fn test_deterministic_behavior() {
    let physics1 = PhysicsEngine::new();
    let physics2 = PhysicsEngine::new();
    
    let orbital_elements = OrbitalElements {
        semi_major_axis: 7.5,
        period: 365.0,
        phase: 1.57, // π/2 radians
    };
    
    // Same inputs should produce identical outputs
    for tick in [0, 100, 1000, 10000] {
        let pos1 = physics1.calculate_orbital_position(&orbital_elements, tick);
        let pos2 = physics2.calculate_orbital_position(&orbital_elements, tick);
        
        assert!((pos1.x - pos2.x).abs() < f32::EPSILON);
        assert!((pos1.y - pos2.y).abs() < f32::EPSILON);
    }
}

#[test]
fn test_distance_validation() {
    let mut physics = PhysicsEngine::new();
    
    // Test extremely far target (should be rejected)
    let far_target = Vector2 { x: 2000.0, y: 0.0 };
    let move_event = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
        ship: 123,
        target: far_target,
    });
    
    assert!(physics.handle_event(&move_event).is_err());
    
    // Test same position (should be rejected)
    let same_target = Vector2 { x: 0.0, y: 0.0 };
    let move_event2 = GameEvent::PlayerCommand(PlayerCommand::MoveShip {
        ship: 456,
        target: same_target,
    });
    
    assert!(physics.handle_event(&move_event2).is_err());
}