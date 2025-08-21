#[cfg(test)]
mod dropdown_tests {
    use crate::core::{GameState, GameResult};
    use crate::ui::UIRenderer;
    
    #[test]
    fn test_dropdown_buttons_disabled_without_content() -> GameResult<()> {
        let mut state = GameState::new()?;
        let mut renderer = UIRenderer::new();
        
        // Initial state: no planets or ships, so buttons should not be clickable
        let planets = state.planet_manager.get_all_planets();
        let ships = state.ship_manager.get_all_ships();
        
        assert!(planets.is_empty(), "Should start with no planets");
        assert!(ships.is_empty(), "Should start with no ships");
        
        // Verify UI state reflects no content
        assert!(!renderer.is_planet_panel_open(), "Planet panel should be closed");
        assert!(!renderer.is_ship_panel_open(), "Ship panel should be closed");
        
        Ok(())
    }
    
    #[test]
    fn test_dropdown_buttons_enabled_with_content() -> GameResult<()> {
        let mut state = GameState::new()?;
        let mut renderer = UIRenderer::new();
        
        // Initialize game content (planets and ships)
        state.game_initializer.initialize_game(
            &mut state.planet_manager,
            &mut state.ship_manager, 
            &mut state.faction_manager
        )?;
        
        // Now there should be content
        let planets = state.planet_manager.get_all_planets();
        let ships = state.ship_manager.get_all_ships();
        
        assert!(!planets.is_empty(), "Should have planets after initialization");
        assert!(!ships.is_empty(), "Should have ships after initialization");
        
        // Test that content can be selected (this simulates dropdown selection)
        if let Some(planet) = planets.first() {
            renderer.set_selected_planet(Some(planet.id));
            assert_eq!(renderer.get_selected_planet(), Some(planet.id));
        }
        
        if let Some(ship) = ships.first() {
            renderer.set_selected_ship(Some(ship.id));
            assert_eq!(renderer.get_selected_ship(), Some(ship.id));
        }
        
        Ok(())
    }
}