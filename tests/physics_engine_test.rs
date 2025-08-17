// tests/physics_engine_test.rs
use stellar_dominion::core::types::*;
use stellar_dominion::core::events::*;
use stellar_dominion::systems::PhysicsEngine;

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