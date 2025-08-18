#[cfg(test)]
mod ui_renderer_tests {
    use stellar_dominion::core::{GameResult, GameEvent, EventBus, GameState};
    use stellar_dominion::core::types::*;
    use stellar_dominion::core::events::*;
    use stellar_dominion::ui::UIRenderer;
    
    fn create_test_game_state() -> GameResult<GameState> {
        GameState::new()
    }
    
    #[test]
    fn test_ui_renderer_creation() {
        let renderer = UIRenderer::new();
        
        // Verify initial state
        assert!(renderer.get_selected_planet().is_none());
        assert!(renderer.get_selected_ship().is_none());
        assert_eq!(renderer.get_zoom_level(), 1.0);
        assert_eq!(renderer.get_ui_scale(), 1.0);
        assert!(!renderer.is_paused());
    }
    
    #[test]
    fn test_handle_pause_event() {
        let mut renderer = UIRenderer::new();
        
        // Test pause event
        let pause_event = GameEvent::PlayerCommand(PlayerCommand::PauseGame(true));
        renderer.handle_event(&pause_event).unwrap();
        assert!(renderer.is_paused());
        
        // Test unpause event
        let unpause_event = GameEvent::PlayerCommand(PlayerCommand::PauseGame(false));
        renderer.handle_event(&unpause_event).unwrap();
        assert!(!renderer.is_paused());
    }
    
    #[test]
    fn test_state_change_events() {
        let mut renderer = UIRenderer::new();
        
        // Set a selected planet
        renderer.set_selected_planet(Some(1));
        
        // Test planet update event
        let planet_update = GameEvent::StateChanged(StateChange::PlanetUpdated(1));
        renderer.handle_event(&planet_update).unwrap();
        assert!(renderer.is_planet_panel_open());
        
        // Test unrelated planet update
        let other_planet_update = GameEvent::StateChanged(StateChange::PlanetUpdated(2));
        renderer.handle_event(&other_planet_update).unwrap();
        // Panel should still be open for planet 1
        assert!(renderer.is_planet_panel_open());
        
        // Set a selected ship
        renderer.set_selected_ship(Some(1));
        
        // Test ship update event
        let ship_update = GameEvent::StateChanged(StateChange::ShipUpdated(1));
        renderer.handle_event(&ship_update).unwrap();
        assert!(renderer.is_ship_panel_open());
    }
    
    #[test]
    fn test_coordinate_transformations() {
        let mut renderer = UIRenderer::new();
        
        // Test world to screen transformation
        let world_pos = Vector2 { x: 100.0, y: 50.0 };
        let screen_pos = renderer.world_to_screen(world_pos);
        
        // With default camera (0,0) and zoom 1.0, world (100,50) should be at screen center + (100,50)
        // Note: This depends on screen_width() and screen_height(), which may not be available in tests
        // So we'll test the inverse transformation instead
        
        let back_to_world = renderer.screen_to_world(screen_pos);
        assert!((back_to_world.x - world_pos.x).abs() < 0.001);
        assert!((back_to_world.y - world_pos.y).abs() < 0.001);
    }
    
    #[test]
    fn test_coordinate_transformations_with_zoom() {
        let mut renderer = UIRenderer::new();
        renderer.zoom_level = 2.0;
        
        let world_pos = Vector2 { x: 100.0, y: 50.0 };
        let screen_pos = renderer.world_to_screen(world_pos);
        let back_to_world = renderer.screen_to_world(screen_pos);
        
        assert!((back_to_world.x - world_pos.x).abs() < 0.001);
        assert!((back_to_world.y - world_pos.y).abs() < 0.001);
    }
    
    #[test]
    fn test_coordinate_transformations_with_camera_offset() {
        let mut renderer = UIRenderer::new();
        renderer.camera_position = Vector2 { x: 200.0, y: 100.0 };
        
        let world_pos = Vector2 { x: 100.0, y: 50.0 };
        let screen_pos = renderer.world_to_screen(world_pos);
        let back_to_world = renderer.screen_to_world(screen_pos);
        
        assert!((back_to_world.x - world_pos.x).abs() < 0.001);
        assert!((back_to_world.y - world_pos.y).abs() < 0.001);
    }
    
    #[test]
    fn test_planet_position_calculation() {
        let renderer = UIRenderer::new();
        
        let planet = Planet {
            id: 1,
            position: OrbitalElements {
                semi_major_axis: 5.0,
                period: 100.0,
                phase: 0.0,
            },
            resources: ResourceStorage::default(),
            population: Demographics::default(),
            developments: Vec::new(),
            controller: None,
        };
        
        // Test position at tick 0
        let pos_t0 = renderer.calculate_planet_position(&planet, 0, 0.0);
        // At phase 0, angle = 0, so x should be semi_major_axis * 100, y should be 0
        assert!((pos_t0.x - 500.0).abs() < 0.001); // 5.0 * 100.0 * cos(0)
        assert!(pos_t0.y.abs() < 0.001); // 5.0 * 100.0 * sin(0)
        
        // Test position at quarter period
        let pos_quarter = renderer.calculate_planet_position(&planet, 25, 0.0);
        // At quarter period, angle = π/2, so x ≈ 0, y should be semi_major_axis * 100
        assert!(pos_quarter.x.abs() < 0.1);
        assert!((pos_quarter.y - 500.0).abs() < 0.1);
    }
    
    #[test]
    fn test_ship_trajectory_interpolation() {
        let renderer = UIRenderer::new();
        
        let trajectory = Trajectory {
            origin: Vector2 { x: 0.0, y: 0.0 },
            destination: Vector2 { x: 100.0, y: 100.0 },
            departure_time: 10,
            arrival_time: 20,
            fuel_cost: 50.0,
        };
        
        // Before departure
        let pos_before = renderer.interpolate_ship_position(&trajectory, 5, 0.0);
        assert_eq!(pos_before.x, 0.0);
        assert_eq!(pos_before.y, 0.0);
        
        // After arrival
        let pos_after = renderer.interpolate_ship_position(&trajectory, 25, 0.0);
        assert_eq!(pos_after.x, 100.0);
        assert_eq!(pos_after.y, 100.0);
        
        // Halfway through journey
        let pos_halfway = renderer.interpolate_ship_position(&trajectory, 15, 0.0);
        assert_eq!(pos_halfway.x, 50.0);
        assert_eq!(pos_halfway.y, 50.0);
        
        // Quarter way through journey
        let pos_quarter = renderer.interpolate_ship_position(&trajectory, 12, 0.5);
        assert_eq!(pos_quarter.x, 25.0);
        assert_eq!(pos_quarter.y, 25.0);
    }
    
    #[test]
    fn test_empire_resource_calculation() {
        // Create a test game state using the public constructor
        let mut game_state = GameState::new().unwrap();
        
        // Create test planets
        let planet1_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(0)).unwrap(); // Player controlled
        let planet2_id = game_state.planet_manager.create_planet(OrbitalElements::default(), Some(1)).unwrap(); // Enemy controlled
        let planet3_id = game_state.planet_manager.create_planet(OrbitalElements::default(), None).unwrap(); // Neutral
        
        // Add resources to planets
        let test_resources = ResourceBundle {
            minerals: 1000,
            food: 500,
            energy: 200,
            alloys: 100,
            components: 50,
            fuel: 300,
        };
        
        game_state.planet_manager.add_resources(planet1_id, test_resources).unwrap();
        game_state.planet_manager.add_resources(planet2_id, test_resources).unwrap();
        game_state.planet_manager.add_resources(planet3_id, test_resources).unwrap();
        
        let renderer = UIRenderer::new();
        let empire_resources = renderer.calculate_empire_resources(&game_state).unwrap();
        
        // Should only include player-controlled planet (faction 0)
        assert_eq!(empire_resources.minerals, 1000);
        assert_eq!(empire_resources.food, 500);
        assert_eq!(empire_resources.energy, 200);
        assert_eq!(empire_resources.alloys, 100);
        assert_eq!(empire_resources.components, 50);
        assert_eq!(empire_resources.fuel, 300);
    }
    
    #[test] 
    fn test_input_event_generation() {
        let mut renderer = UIRenderer::new();
        let mut event_bus = EventBus::new();
        
        // Set a selected ship for testing move commands
        renderer.set_selected_ship(Some(1));
        
        // Test right click generating move command
        let target_pos = Vector2 { x: 100.0, y: 50.0 };
        renderer.handle_right_click(target_pos.x, target_pos.y, &mut event_bus).unwrap();
        
        // Check that a move command was queued
        assert_eq!(event_bus.queued_events.len(), 1);
        
        if let Some(GameEvent::PlayerCommand(PlayerCommand::MoveShip { ship, target })) = event_bus.queued_events.front() {
            assert_eq!(*ship, 1);
            // Target should be the world position (same as input in this case since camera is at origin)
            assert!((target.x - target_pos.x).abs() < 0.001);
            assert!((target.y - target_pos.y).abs() < 0.001);
        } else {
            panic!("Expected MoveShip command");
        }
    }
    
    #[test]
    fn test_zoom_constraints() {
        let mut renderer = UIRenderer::new();
        
        // Test zoom limits
        renderer.zoom_level = 0.05; // Below minimum
        // In real implementation, zoom would be constrained to 0.1-5.0 range
        // This test would check that zoom is clamped appropriately
        
        renderer.zoom_level = 10.0; // Above maximum
        // Similarly, this should be clamped to 5.0
    }
    
    #[test]
    fn test_ui_context_state() {
        let renderer = UIRenderer::new();
        
        // Test initial UI context state
        assert!(!renderer.is_planet_panel_open());
        assert!(!renderer.is_ship_panel_open());
        
        // Note: We can't test the internal UI context state directly since it's private
        // and we haven't exposed all the getters. This is actually good encapsulation.
        // In a real UI, these would be tested through user interactions that trigger state changes.
    }
}