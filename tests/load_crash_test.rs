use stellar_dominion::core::*;

#[test]
fn test_load_game_direct() {
    println!("Starting direct load game test...");
    
    // Create a new game state (starts in MainMenu mode)
    let mut game_state = GameState::new().expect("Failed to create game state");
    
    println!("Game state created, current mode: {:?}", game_state.current_mode);
    
    // Test the load game functionality directly without going through UI input
    // Switch to the menu event handling logic manually
    
    // First initialize fresh systems (like the LoadGame handler does)
    println!("Initializing fresh systems...");
    game_state.time_manager = stellar_dominion::systems::TimeManager::new();
    game_state.planet_manager = stellar_dominion::managers::PlanetManager::new();
    game_state.ship_manager = stellar_dominion::managers::ShipManager::new();
    game_state.faction_manager = stellar_dominion::managers::FactionManager::new();
    game_state.resource_system = stellar_dominion::systems::ResourceSystem::new();
    game_state.population_system = stellar_dominion::systems::PopulationSystem::new();
    game_state.construction_system = stellar_dominion::systems::ConstructionSystem::new();
    game_state.physics_engine = stellar_dominion::systems::PhysicsEngine::new();
    game_state.combat_resolver = stellar_dominion::systems::CombatResolver::new();
    
    // Now try to load a game directly
    println!("Attempting to load game...");
    match game_state.load_game() {
        Ok(()) => {
            println!("Load game completed successfully");
            game_state.current_mode = GameMode::InGame;
        }
        Err(e) => {
            println!("Load game failed with error: {:?}", e);
            // This is expected if there's no save file or other issues
        }
    }
    
    println!("Test completed");
}