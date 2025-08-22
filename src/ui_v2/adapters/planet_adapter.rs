// src/ui_v2/adapters/planet_adapter.rs
//! Adapter for Planet entities

use super::{EntityAdapter, format_number, format_resource};
use crate::core::types::Planet;
use crate::core::events::PlayerCommand;
use macroquad::prelude::Color;

/// Adapter for displaying Planet entities in UI
pub struct PlanetAdapter {
    show_detailed_resources: bool,
    show_development_slots: bool,
}

impl PlanetAdapter {
    pub fn new() -> Self {
        Self {
            show_detailed_resources: true,
            show_development_slots: true,
        }
    }

    pub fn simple() -> Self {
        Self {
            show_detailed_resources: false,
            show_development_slots: false,
        }
    }

    pub fn with_detailed_resources(mut self, show: bool) -> Self {
        self.show_detailed_resources = show;
        self
    }

    pub fn with_development_slots(mut self, show: bool) -> Self {
        self.show_development_slots = show;
        self
    }
}

impl EntityAdapter<Planet> for PlanetAdapter {
    fn get_display_fields(&self, planet: &Planet) -> Vec<(String, String)> {
        let mut fields = Vec::new();

        // Basic info
        fields.push(("ID".to_string(), planet.id.to_string()));
        fields.push(("Faction".to_string(), 
            planet.controller.map_or("None".to_string(), |id| id.to_string())));
        fields.push(("Orbit".to_string(), format!("Axis: {:.1} AU", planet.position.semi_major_axis)));

        // Population
        fields.push(("Population".to_string(), format_number(planet.population.total)));
        fields.push(("Growth Rate".to_string(), format!("{:.2}", planet.population.growth_rate)));

        // Resources
        if self.show_detailed_resources {
            fields.push(("Energy".to_string(), format_resource(
                planet.resources.current.energy, 
                planet.resources.capacity.energy
            )));
            fields.push(("Minerals".to_string(), format_resource(
                planet.resources.current.minerals, 
                planet.resources.capacity.minerals
            )));
            fields.push(("Food".to_string(), format_resource(
                planet.resources.current.food, 
                planet.resources.capacity.food
            )));
            fields.push(("Alloys".to_string(), format_resource(
                planet.resources.current.alloys, 
                planet.resources.capacity.alloys
            )));
            fields.push(("Components".to_string(), format_resource(
                planet.resources.current.components, 
                planet.resources.capacity.components
            )));
        } else {
            fields.push(("Energy".to_string(), format_number(planet.resources.current.energy)));
            fields.push(("Minerals".to_string(), format_number(planet.resources.current.minerals)));
            fields.push(("Food".to_string(), format_number(planet.resources.current.food)));
        }

        // Development
        if self.show_development_slots {
            let used_slots = planet.developments.len();
            // Calculate building slots: 10 + population/10000 (per architecture rules)
            let total_slots = 10 + planet.population.total / 10000;
            fields.push(("Development Slots".to_string(), format!("{} / {}", used_slots, total_slots)));
            
            // List developments
            if !planet.developments.is_empty() {
                let dev_list = planet.developments.iter()
                    .map(|d| format!("{:?} (Tier {})", d.building_type, d.tier))
                    .collect::<Vec<_>>()
                    .join(", ");
                fields.push(("Developments".to_string(), dev_list));
            }
        }

        fields
    }

    fn get_actions(&self, planet: &Planet) -> Vec<(String, PlayerCommand)> {
        let mut actions = Vec::new();

        // Always available actions
        actions.push(("View Details".to_string(), PlayerCommand::ShowPlanet(planet.id)));
        actions.push(("Manage Resources".to_string(), PlayerCommand::ShowResourcePanel));

        // Conditional actions based on planet state
        let total_slots = 10 + planet.population.total / 10000;
        if total_slots > planet.developments.len() as i32 {
            actions.push(("Build Development".to_string(), PlayerCommand::BuildDevelopment(planet.id, "Infrastructure".to_string())));
        }

        if planet.resources.current.energy > 100 {
            actions.push(("Build Ship".to_string(), PlayerCommand::BuildShip(planet.id, "Scout".to_string())));
        }

        // Close panel action
        actions.push(("Close".to_string(), PlayerCommand::ClosePlanetPanel));

        actions
    }

    fn format_field(&self, field_name: &str, planet: &Planet) -> String {
        match field_name {
            "population" => format_number(planet.population.total),
            "energy" => format_number(planet.resources.current.energy),
            "minerals" => format_number(planet.resources.current.minerals),
            "food" => format_number(planet.resources.current.food),
            "alloys" => format_number(planet.resources.current.alloys),
            "components" => format_number(planet.resources.current.components),
            "position" => format!("Orbit: {:.1} AU", planet.position.semi_major_axis),
            "developments" => planet.developments.len().to_string(),
            _ => "N/A".to_string(),
        }
    }

    fn get_summary(&self, planet: &Planet) -> String {
        format!("Planet {} - Pop: {} - Energy: {}", 
            planet.id,
            format_number(planet.population.total),
            format_number(planet.resources.current.energy)
        )
    }

    fn get_icon(&self, planet: &Planet) -> Option<String> {
        // Different icons based on planet development level
        let development_count = planet.developments.len();
        let icon = match development_count {
            0..=2 => "🌍", // Basic planet
            3..=5 => "🏙️", // Developed planet
            _ => "🌆",      // Advanced planet
        };
        Some(icon.to_string())
    }

    fn get_status_color(&self, planet: &Planet) -> Option<Color> {
        // Color coding based on resource levels
        let energy_ratio = if planet.resources.capacity.energy > 0 {
            planet.resources.current.energy as f32 / planet.resources.capacity.energy as f32
        } else {
            1.0
        };

        let color = if energy_ratio < 0.25 {
            Color::new(1.0, 0.3, 0.3, 1.0) // Red - low energy
        } else if energy_ratio < 0.5 {
            Color::new(1.0, 0.8, 0.3, 1.0) // Yellow - medium energy
        } else {
            Color::new(0.3, 1.0, 0.3, 1.0) // Green - good energy
        };

        Some(color)
    }

    fn is_highlighted(&self, planet: &Planet) -> bool {
        // Highlight planets with available building slots or low resources
        let total_slots = 10 + planet.population.total / 10000;
        let has_available_slots = total_slots > planet.developments.len() as i32;
        let low_energy = planet.resources.capacity.energy > 0 && 
            (planet.resources.current.energy as f32 / planet.resources.capacity.energy as f32) < 0.3;
        
        has_available_slots || low_energy
    }
}

impl Default for PlanetAdapter {
    fn default() -> Self {
        Self::new()
    }
}