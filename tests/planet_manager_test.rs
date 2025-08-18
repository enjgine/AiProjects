#[cfg(test)]
mod tests {
    use stellar_dominion::core::types::*;
    use stellar_dominion::managers::PlanetManager;

    #[test]
    fn test_planet_creation() {
        let mut manager = PlanetManager::new();
        let orbital_elements = OrbitalElements::default();
        
        let planet_id = manager.create_planet(orbital_elements, None).unwrap();
        assert_eq!(planet_id, 0);
        
        let planet = manager.get_planet(planet_id).unwrap();
        assert_eq!(planet.id, planet_id);
        assert_eq!(planet.controller, None);
    }

    #[test]
    fn test_resource_addition_with_validation() {
        let mut manager = PlanetManager::new();
        let orbital_elements = OrbitalElements::default();
        let planet_id = manager.create_planet(orbital_elements, None).unwrap();
        
        let resources = ResourceBundle {
            minerals: 100,
            food: 50,
            energy: 25,
            alloys: 10,
            components: 5,
            fuel: 20,
        };
        
        // Should succeed with valid resources
        assert!(manager.add_resources(planet_id, resources).is_ok());
        
        let planet = manager.get_planet(planet_id).unwrap();
        assert_eq!(planet.resources.current.minerals, 100);
        assert_eq!(planet.resources.current.food, 50);
    }

    #[test]
    fn test_resource_addition_negative_validation() {
        let mut manager = PlanetManager::new();
        let orbital_elements = OrbitalElements::default();
        let planet_id = manager.create_planet(orbital_elements, None).unwrap();
        
        let negative_resources = ResourceBundle {
            minerals: -100,
            food: 50,
            energy: 25,
            alloys: 10,
            components: 5,
            fuel: 20,
        };
        
        // Should fail with negative resources
        assert!(manager.add_resources(planet_id, negative_resources).is_err());
    }

    #[test]
    fn test_storage_capacity_exceeded() {
        let mut manager = PlanetManager::new();
        let orbital_elements = OrbitalElements::default();
        let planet_id = manager.create_planet(orbital_elements, None).unwrap();
        
        let excessive_resources = ResourceBundle {
            minerals: 20000, // Exceeds default capacity of 10000
            food: 0,
            energy: 0,
            alloys: 0,
            components: 0,
            fuel: 0,
        };
        
        // Should fail due to capacity limit
        assert!(manager.add_resources(planet_id, excessive_resources).is_err());
    }

    #[test]
    fn test_worker_allocation_integer_arithmetic() {
        let mut manager = PlanetManager::new();
        let orbital_elements = OrbitalElements::default();
        let planet_id = manager.create_planet(orbital_elements, None).unwrap();
        
        // Set population to 1000
        manager.update_population(planet_id, 1000).unwrap();
        
        let allocation = WorkerAllocation {
            agriculture: 200,
            mining: 200,
            industry: 200,
            research: 100,
            military: 100,
            unassigned: 200, // This should be >= 100 (10% of 1000)
        };
        
        // Should succeed with proper allocation
        assert!(manager.set_worker_allocation(planet_id, allocation).is_ok());
        
        // Try with insufficient unassigned workers
        let bad_allocation = WorkerAllocation {
            agriculture: 200,
            mining: 200,
            industry: 200,
            research: 200,
            military: 150,
            unassigned: 50, // Less than 100 (10% of 1000)
        };
        
        // Should fail due to insufficient unassigned workers
        assert!(manager.set_worker_allocation(planet_id, bad_allocation).is_err());
    }

    #[test]
    fn test_building_slot_calculation() {
        let mut manager = PlanetManager::new();
        let orbital_elements = OrbitalElements::default();
        let planet_id = manager.create_planet(orbital_elements, None).unwrap();
        
        // Test with 0 population (should have 10 base slots)
        let slots = manager.get_available_building_slots(planet_id).unwrap();
        assert_eq!(slots, 10);
        
        // Add population to get more slots
        manager.update_population(planet_id, 50000).unwrap(); // Should add 5 more slots
        let slots = manager.get_available_building_slots(planet_id).unwrap();
        assert_eq!(slots, 15);
    }

    #[test]
    fn test_population_overflow_protection() {
        let mut manager = PlanetManager::new();
        let orbital_elements = OrbitalElements::default();
        let planet_id = manager.create_planet(orbital_elements, None).unwrap();
        
        // Set population near max
        manager.update_population(planet_id, i32::MAX - 10).unwrap();
        
        // Try to add more population (should be protected by saturating_add)
        let result = manager.update_population(planet_id, 20);
        assert!(result.is_err());
    }
}