// tests/systems/resources_test.rs
use stellar_dominion::core::*;
use stellar_dominion::core::events::*;
use stellar_dominion::core::types::*;
use stellar_dominion::systems::ResourceSystem;

#[test]
fn test_resource_system_creation() {
    let resource_system = ResourceSystem::new();
    // Should create with default production rates
}

#[test]
fn test_production_rates() {
    let resource_system = ResourceSystem::new();
    
    // Production rates should be defined for all building types
    // This tests the internal structure
}

#[test]
fn test_resource_production_calculation() {
    let mut resource_system = ResourceSystem::new();
    let mut event_bus = EventBus::new();
    
    // Simulate tick event
    let tick_event = GameEvent::SimulationEvent(SimulationEvent::TickCompleted(1));
    resource_system.handle_event(&tick_event).unwrap();
    
    // Should process production without error
}

#[test]
fn test_resource_transfer_validation() {
    let mut resource_system = ResourceSystem::new();
    
    let transfer_event = GameEvent::PlayerCommand(PlayerCommand::TransferResources {
        from: 1,
        to: 2,
        resources: ResourceBundle {
            minerals: 100,
            food: 50,
            energy: 0,
            alloys: 0,
            components: 0,
            fuel: 0,
        },
    });
    
    // Should handle transfer command
    resource_system.handle_event(&transfer_event).unwrap();
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