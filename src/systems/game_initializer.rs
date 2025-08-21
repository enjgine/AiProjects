use crate::core::types::*;
use crate::managers::*;
use std::f32::consts::PI;

/// GameInitializer handles creating new games with configurable parameters
pub struct GameInitializer {
    configuration: GameConfiguration,
}

impl GameInitializer {
    pub fn new(config: GameConfiguration) -> Self {
        Self {
            configuration: config,
        }
    }

    /// Initialize a new game with the configured parameters
    pub fn initialize_game(
        &self,
        planet_manager: &mut PlanetManager,
        ship_manager: &mut ShipManager,
        faction_manager: &mut FactionManager,
    ) -> GameResult<()> {
        // Clear existing data
        *planet_manager = PlanetManager::new();
        *ship_manager = ShipManager::new();
        *faction_manager = FactionManager::new();

        // Create factions
        self.create_factions(faction_manager)?;

        // Create planets based on configuration
        self.create_planets(planet_manager)?;

        // Create starting ships
        self.create_starting_ships(ship_manager, planet_manager)?;

        // Apply starting resources and population
        self.apply_starting_conditions(planet_manager)?;

        Ok(())
    }

    fn create_factions(&self, faction_manager: &mut FactionManager) -> GameResult<()> {
        // Create player faction
        faction_manager.create_faction("Player Empire".to_string(), true, AIPersonality::Balanced)?;

        // Create AI factions based on configuration
        let ai_names = [
            "Stellar Federation", "Cosmic Republic", "Galactic Union", "Star Alliance",
            "Void Collective", "Nebula Empire", "Solar Dynasty", "Astral Dominion"
        ];

        for i in 0..self.configuration.ai_opponents {
            let name = ai_names.get(i).unwrap_or(&"AI Empire").to_string();
            let personality = match i % 3 {
                0 => AIPersonality::Aggressive,
                1 => AIPersonality::Economic,
                _ => AIPersonality::Balanced,
            };
            faction_manager.create_faction(name, false, personality)?;
        }

        Ok(())
    }

    fn create_planets(&self, planet_manager: &mut PlanetManager) -> GameResult<Vec<PlanetId>> {
        let mut planet_ids = Vec::new();

        // Calculate planet positions in a spiral or grid pattern
        for i in 0..self.configuration.planet_count {
            let orbital_elements = self.generate_orbital_elements(i);
            
            // Assign controller based on planet index
            let controller = if i == 0 {
                Some(0) // Player faction always gets first planet
            } else if i < 1 + self.configuration.ai_opponents {
                Some(i as u8) // AI factions get next planets
            } else {
                None // Remaining planets are neutral
            };

            let planet_id = planet_manager.create_planet(orbital_elements, controller)?;
            planet_ids.push(planet_id);
        }

        Ok(planet_ids)
    }

    fn generate_orbital_elements(&self, index: usize) -> OrbitalElements {
        // Create varied orbital parameters for interesting gameplay
        let base_radius = match self.configuration.galaxy_size {
            GalaxySize::Small => 1.0 + (index as f32 * 0.8),
            GalaxySize::Medium => 1.0 + (index as f32 * 0.6),
            GalaxySize::Large => 1.0 + (index as f32 * 0.4),
        };

        // Add some deterministic variation for variety
        let radius_variation = ((index as f32 * 0.618) % 1.0 - 0.5) * 0.6; // Golden ratio for pseudo-randomness
        let semi_major_axis = (base_radius + radius_variation).max(0.5);

        // Calculate period based on Kepler's laws (roughly)
        let period = (semi_major_axis.powf(1.5) * 100.0).max(50.0);

        // Distribute phases evenly with some deterministic variation
        let base_phase = (index as f32 * 2.0 * PI) / self.configuration.planet_count as f32;
        let phase_variation = ((index as f32 * 2.718) % 1.0 - 0.5) * 1.0; // e for pseudo-randomness
        let phase = base_phase + phase_variation;

        OrbitalElements {
            semi_major_axis,
            period,
            phase,
        }
    }

    fn create_starting_ships(
        &self,
        ship_manager: &mut ShipManager,
        planet_manager: &PlanetManager,
    ) -> GameResult<()> {
        let planets = planet_manager.get_all_planets_cloned()?;

        // Create starting ships for player and AI factions
        for planet in &planets {
            if let Some(faction_id) = planet.controller {
                // Calculate starting position near the planet using macroquad Vec2 for calculations
                let planet_pos = macroquad::math::Vec2::new(
                    planet.position.semi_major_axis * planet.position.phase.cos(),
                    planet.position.semi_major_axis * planet.position.phase.sin(),
                );
                
                // Create initial ships based on faction type
                let ship_class = if faction_id == 0 {
                    ShipClass::Scout // Player starts with scout
                } else {
                    ShipClass::Warship // AI starts with warships
                };

                // Convert Vec2 to Vector2
                let position = Vector2 { x: planet_pos.x, y: planet_pos.y };
                ship_manager.create_ship(ship_class, position, faction_id)?;
            }
        }

        Ok(())
    }

    fn apply_starting_conditions(&self, planet_manager: &mut PlanetManager) -> GameResult<()> {
        let planets = planet_manager.get_all_planets_cloned()?;

        for planet in planets {
            if planet.controller.is_some() {
                // Apply starting resources (only to controlled planets)
                planet_manager.add_resources(planet.id, self.configuration.starting_resources.clone())?;
                
                // Apply starting population
                planet_manager.update_population(planet.id, self.configuration.starting_population)?;
                
                // Set up basic worker allocation (ensure minimum 10% unassigned)
                let total_pop = self.configuration.starting_population;
                let min_unassigned = total_pop / 10; // 10% minimum unassigned
                let assignable = total_pop - min_unassigned; // 900 workers to assign
                
                let agriculture = assignable / 4;    // 225
                let mining = assignable / 4;         // 225  
                let industry = assignable / 6;       // 150
                let research = assignable / 6;       // 150
                let military = assignable / 10;      // 90
                let assigned_total = agriculture + mining + industry + research + military;
                let unassigned = total_pop - assigned_total; // Ensures minimum requirement
                
                let allocation = WorkerAllocation {
                    agriculture,
                    mining,
                    industry,
                    research,
                    military,
                    unassigned,
                };
                
                planet_manager.set_worker_allocation(planet.id, allocation)?;
            }
        }

        Ok(())
    }

    /// Get the current configuration
    pub fn get_configuration(&self) -> &GameConfiguration {
        &self.configuration
    }

    /// Update the configuration
    pub fn set_configuration(&mut self, config: GameConfiguration) {
        self.configuration = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configuration() {
        let config = GameConfiguration::default();
        assert_eq!(config.planet_count, 3);
        assert_eq!(config.starting_population, 1000);
        assert_eq!(config.ai_opponents, 1);
    }

    #[test]
    fn test_galaxy_size_ranges() {
        assert_eq!(GalaxySize::Small.planet_range(), (5, 10));
        assert_eq!(GalaxySize::Medium.planet_range(), (10, 20));
        assert_eq!(GalaxySize::Large.planet_range(), (20, 50));
    }

    #[test]
    fn test_game_initializer_creation() {
        let config = GameConfiguration::default();
        let initializer = GameInitializer::new(config.clone());
        assert_eq!(initializer.get_configuration().planet_count, config.planet_count);
    }

    #[test]
    fn test_faction_creation() {
        let config = GameConfiguration {
            ai_opponents: 2,
            ..Default::default()
        };
        let initializer = GameInitializer::new(config);
        let mut faction_manager = FactionManager::new();

        initializer.create_factions(&mut faction_manager).unwrap();
        
        // Should have player + 2 AI factions = 3 total
        let factions = faction_manager.get_all_factions();
        assert_eq!(factions.len(), 3);
    }

    #[test] 
    fn test_planet_creation() {
        let config = GameConfiguration {
            planet_count: 5,
            ai_opponents: 2,
            ..Default::default()
        };
        let initializer = GameInitializer::new(config);
        let mut planet_manager = PlanetManager::new();

        let planet_ids = initializer.create_planets(&mut planet_manager).unwrap();
        assert_eq!(planet_ids.len(), 5);

        // Check planet ownership
        let planets = planet_manager.get_all_planets_cloned().unwrap();
        assert!(planets[0].controller == Some(0)); // Player planet
        assert!(planets[1].controller == Some(1)); // AI planet 1
        assert!(planets[2].controller == Some(2)); // AI planet 2
        assert!(planets[3].controller.is_none()); // Neutral
        assert!(planets[4].controller.is_none()); // Neutral
    }
}