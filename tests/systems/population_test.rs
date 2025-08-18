// tests/systems/population_test.rs
use stellar_dominion::core::{GameResult, GameEvent, EventBus};
use stellar_dominion::core::types::*;
use stellar_dominion::core::events::*;
use stellar_dominion::systems::PopulationSystem;
use stellar_dominion::managers::PlanetManager;

#[test]
fn test_population_system_creation() {
    let system = PopulationSystem::new();
    // Basic creation test - system should initialize without errors
    assert!(true);
}

#[test]
fn test_population_growth_with_food_surplus() -> GameResult<()> {
    let mut population_system = PopulationSystem::new();
    let mut planet_manager = PlanetManager::new();
    let mut event_bus = EventBus::new();
    
    // Create a test planet with population and adequate food
    let planet_id = planet_manager.create_planet(OrbitalElements::default(), Some(1))?;
    
    // Set up initial population and food
    planet_manager.update_population(planet_id, 1000)?;
    planet_manager.add_resources(planet_id, ResourceBundle {
        food: 1500, // 50% surplus (1000 consumption + 500 surplus)
        ..ResourceBundle::default()
    })?;
    
    // Process growth
    population_system.process_growth(1, &mut planet_manager, &mut event_bus)?;
    
    // Verify population grew by 2% (1000 * 0.02 = 20)
    let planet = planet_manager.get_planet(planet_id)?;
    assert_eq!(planet.population.total, 1020);
    
    // Verify growth event was emitted
    assert_eq!(event_bus.queued_events.len(), 2); // PopulationGrowth + PlanetUpdated
    
    Ok(())
}

#[test]
fn test_population_no_growth_without_food_surplus() -> GameResult<()> {
    let mut population_system = PopulationSystem::new();
    let mut planet_manager = PlanetManager::new();
    let mut event_bus = EventBus::new();
    
    // Create a test planet with population but insufficient food
    let planet_id = planet_manager.create_planet(OrbitalElements::default(), Some(1))?;
    
    // Set up initial population and insufficient food (only 10% surplus)
    planet_manager.update_population(planet_id, 1000)?;
    planet_manager.add_resources(planet_id, ResourceBundle {
        food: 1100, // Only 10% surplus (less than 20% requirement)
        ..ResourceBundle::default()
    })?;
    
    // Process growth
    population_system.process_growth(1, &mut planet_manager, &mut event_bus)?;
    
    // Verify population didn't grow
    let planet = planet_manager.get_planet(planet_id)?;
    assert_eq!(planet.population.total, 1000);
    
    // Verify no growth events were emitted
    assert_eq!(event_bus.queued_events.len(), 0);
    
    Ok(())
}

#[test]
fn test_worker_allocation_validation() -> GameResult<()> {
    let mut population_system = PopulationSystem::new();
    let mut planet_manager = PlanetManager::new();
    let mut event_bus = EventBus::new();
    
    // Create a test planet with population
    let planet_id = planet_manager.create_planet(OrbitalElements::default(), Some(1))?;
    planet_manager.update_population(planet_id, 1000)?;
    
    // Test valid allocation with minimum 10% unassigned workers
    let valid_allocation = WorkerAllocation {
        agriculture: 200,
        mining: 200,
        industry: 200,
        research: 200,
        military: 100,
        unassigned: 100, // 10% unassigned
    };
    
    let result = population_system.process_allocation(planet_id, valid_allocation, &mut planet_manager, &mut event_bus);
    assert!(result.is_ok());
    
    // Test invalid allocation with insufficient unassigned workers
    let invalid_allocation = WorkerAllocation {
        agriculture: 250,
        mining: 250,
        industry: 250,
        research: 200,
        military: 50,
        unassigned: 0, // Less than 10% unassigned
    };
    
    let result = population_system.process_allocation(planet_id, invalid_allocation, &mut planet_manager, &mut event_bus);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_worker_allocation_total_mismatch() -> GameResult<()> {
    let mut population_system = PopulationSystem::new();
    let mut planet_manager = PlanetManager::new();
    let mut event_bus = EventBus::new();
    
    // Create a test planet with population
    let planet_id = planet_manager.create_planet(OrbitalElements::default(), Some(1))?;
    planet_manager.update_population(planet_id, 1000)?;
    
    // Test allocation that doesn't sum to total population
    let invalid_allocation = WorkerAllocation {
        agriculture: 200,
        mining: 200,
        industry: 200,
        research: 200,
        military: 100,
        unassigned: 200, // Total = 1100, but population = 1000
    };
    
    let result = population_system.process_allocation(planet_id, invalid_allocation, &mut planet_manager, &mut event_bus);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_population_system_handle_event() -> GameResult<()> {
    let mut population_system = PopulationSystem::new();
    
    // Test handling tick completed event
    let tick_event = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(42));
    let result = population_system.handle_event(&tick_event);
    assert!(result.is_ok());
    
    // Test handling worker allocation event
    let allocation_event = GameEvent::PlayerCommand(PlayerCommand::AllocateWorkers {
        planet: 1,
        allocation: WorkerAllocation::default(),
    });
    let result = population_system.handle_event(&allocation_event);
    assert!(result.is_ok());
    
    // Test handling ship arrival event
    let ship_arrival_event = GameEvent::SimulationEvent(SimulationEvent::ShipArrived {
        ship: 1,
        destination: Vector2::default(),
    });
    let result = population_system.handle_event(&ship_arrival_event);
    assert!(result.is_ok());
    
    Ok(())
}

#[test]
fn test_update_method() -> GameResult<()> {
    let mut population_system = PopulationSystem::new();
    let mut event_bus = EventBus::new();
    
    // Test that update method doesn't error
    let result = population_system.update(0.1, &mut event_bus);
    assert!(result.is_ok());
    
    Ok(())
}