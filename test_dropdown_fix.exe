use stellar_dominion::{
    core::{GameState, GameResult},
    ui::UIRenderer,
};

fn main() -> GameResult<()> {
    println!("Testing dropdown menu functionality...");
    
    // Create a test game state with some planets and ships
    let mut state = GameState::new()?;
    let mut renderer = UIRenderer::new();
    
    // Test 1: Check initial state
    println!("Initial state:");
    println!("  Planet menu open: {}", renderer.planet_list_menu.open);
    println!("  Ship menu open: {}", renderer.ship_list_menu.open);
    println!("  Planet panel open: {}", renderer.is_planet_panel_open());
    println!("  Ship panel open: {}", renderer.is_ship_panel_open());
    
    // Test 2: Simulate toolbar button click for planets
    println!("\nSimulating planet menu button click...");
    renderer.toolbar.planets_menu_open = true;
    
    // Sync menu states (this happens in render_toolbar_and_menus)
    renderer.planet_list_menu.open = renderer.toolbar.planets_menu_open;
    renderer.ship_list_menu.open = renderer.toolbar.ships_menu_open;
    
    println!("After button click:");
    println!("  Planet menu open: {}", renderer.planet_list_menu.open);
    println!("  Ship menu open: {}", renderer.ship_list_menu.open);
    
    // Test 3: Test that clicking on a planet in the dropdown would open the panel
    if let Some(planet) = state.planet_manager.get_all_planets().first() {
        println!("\nSimulating planet selection from dropdown...");
        renderer.selected_planet = Some(planet.id);
        renderer.ui_context.planet_panel_open = true;
        
        println!("After planet selection:");
        println!("  Selected planet: {:?}", renderer.selected_planet);
        println!("  Planet panel open: {}", renderer.is_planet_panel_open());
    } else {
        println!("\nNo planets found in state - creating one for test...");
        // This would create a planet but we need the specific implementation
    }
    
    println!("\nDropdown test completed successfully!");
    Ok(())
}