// src/ui_v2/adapters/ship_adapter.rs
//! Adapter for Ship entities

use super::{EntityAdapter, format_number};
use crate::core::types::{Ship, ShipClass, ResourceBundle};
use crate::core::events::PlayerCommand;
use macroquad::prelude::Color;

/// Adapter for displaying Ship entities in UI
pub struct ShipAdapter {
    show_cargo_details: bool,
    show_movement_history: bool,
}

impl ShipAdapter {
    pub fn new() -> Self {
        Self {
            show_cargo_details: true,
            show_movement_history: false,
        }
    }

    pub fn simple() -> Self {
        Self {
            show_cargo_details: false,
            show_movement_history: false,
        }
    }

    pub fn with_cargo_details(mut self, show: bool) -> Self {
        self.show_cargo_details = show;
        self
    }

    pub fn with_movement_history(mut self, show: bool) -> Self {
        self.show_movement_history = show;
        self
    }
}

impl EntityAdapter<Ship> for ShipAdapter {
    fn get_display_fields(&self, ship: &Ship) -> Vec<(String, String)> {
        let mut fields = Vec::new();

        // Basic info
        fields.push(("ID".to_string(), ship.id.to_string()));
        fields.push(("Type".to_string(), format!("{:?}", ship.ship_class)));
        fields.push(("Faction".to_string(), ship.owner.to_string()));
        fields.push(("Position".to_string(), format!("({:.1}, {:.1})", ship.position.x, ship.position.y)));

        // Status based on trajectory
        let status = match &ship.trajectory {
            Some(traj) => format!("Moving to ({:.1}, {:.1})", traj.destination.x, traj.destination.y),
            None => "Idle".to_string(),
        };
        fields.push(("Status".to_string(), status));

        // Fuel instead of speed
        fields.push(("Fuel".to_string(), format!("{:.1}%", ship.fuel)));
        
        if let Some(traj) = &ship.trajectory {
            let distance = ((traj.destination.x - ship.position.x).powi(2) + (traj.destination.y - ship.position.y).powi(2)).sqrt();
            fields.push(("Distance to Target".to_string(), format!("{:.1}", distance)));
            
            // Calculate ETA based on trajectory timing
            let duration = traj.arrival_time.saturating_sub(traj.departure_time);
            fields.push(("ETA".to_string(), format!("{} ticks", duration)));
        }

        // Cargo information
        if self.show_cargo_details {
            let total_cargo = ship.cargo.current_load();
            fields.push(("Total Cargo".to_string(), format_number(total_cargo)));
            
            if total_cargo > 0 {
                fields.push(("Energy".to_string(), format_number(ship.cargo.resources.energy)));
                fields.push(("Minerals".to_string(), format_number(ship.cargo.resources.minerals)));
                fields.push(("Food".to_string(), format_number(ship.cargo.resources.food)));
                fields.push(("Alloys".to_string(), format_number(ship.cargo.resources.alloys)));
                fields.push(("Components".to_string(), format_number(ship.cargo.resources.components)));
                if ship.cargo.population > 0 {
                    fields.push(("Population".to_string(), format_number(ship.cargo.population)));
                }
            }
        } else if ship.cargo.current_load() > 0 {
            fields.push(("Cargo".to_string(), format_number(ship.cargo.current_load())));
        }

        fields
    }

    fn get_actions(&self, ship: &Ship) -> Vec<(String, PlayerCommand)> {
        let mut actions = Vec::new();

        // Always available actions
        actions.push(("View Details".to_string(), PlayerCommand::ShowShip(ship.id)));

        // Movement actions
        if ship.trajectory.is_none() {
            actions.push(("Move Ship".to_string(), PlayerCommand::MoveShip { ship: ship.id, target: ship.position }));
        } else {
            actions.push(("Stop Ship".to_string(), PlayerCommand::StopShip(ship.id)));
        }

        // Cargo actions
        let has_cargo = ship.cargo.current_load() > 0;
        
        if has_cargo {
            actions.push(("Unload Cargo".to_string(), PlayerCommand::UnloadShipCargo { ship: ship.id, planet: 0 }));
        }

        // Ship type specific actions
        match ship.ship_class {
            ShipClass::Scout => {
                actions.push(("Scout Area".to_string(), PlayerCommand::ScoutLocation(ship.position)));
            }
            ShipClass::Transport => {
                actions.push(("Load Cargo".to_string(), PlayerCommand::LoadShipCargo { ship: ship.id, planet: 0, resources: ResourceBundle::default() }));
            }
            ShipClass::Colony => {
                actions.push(("Colonize".to_string(), PlayerCommand::ColonizePlanet { ship: ship.id, planet: 0 }));
            }
            ShipClass::Warship => {
                actions.push(("Attack".to_string(), PlayerCommand::AttackTarget { attacker: ship.id, target: 0 }));
            }
        }

        // Close panel action
        actions.push(("Close".to_string(), PlayerCommand::CloseShipPanel));

        actions
    }

    fn format_field(&self, field_name: &str, ship: &Ship) -> String {
        match field_name {
            "position" => format!("({:.1}, {:.1})", ship.position.x, ship.position.y),
            "ship_class" => format!("{:?}", ship.ship_class),
            "cargo_total" => format_number(ship.cargo.current_load()),
            "energy" => format_number(ship.cargo.resources.energy),
            "minerals" => format_number(ship.cargo.resources.minerals),
            "food" => format_number(ship.cargo.resources.food),
            "alloys" => format_number(ship.cargo.resources.alloys),
            "components" => format_number(ship.cargo.resources.components),
            "population" => format_number(ship.cargo.population),
            "type" => format!("{:?}", ship.ship_class),
            "status" => {
                match &ship.trajectory {
                    Some(traj) => format!("Moving to ({:.1}, {:.1})", traj.destination.x, traj.destination.y),
                    None => "Idle".to_string(),
                }
            },
            _ => "N/A".to_string(),
        }
    }

    fn get_summary(&self, ship: &Ship) -> String {
        let status = match ship.trajectory {
            Some(_) => "Moving",
            None => "Idle",
        };
        
        format!("{:?} {} - {} - ({:.0}, {:.0})", 
            ship.ship_class,
            ship.id,
            status,
            ship.position.x,
            ship.position.y
        )
    }

    fn get_icon(&self, ship: &Ship) -> Option<String> {
        let icon = match ship.ship_class {
            crate::core::types::ShipClass::Scout => "🔍",
            crate::core::types::ShipClass::Transport => "🚛",
            crate::core::types::ShipClass::Colony => "🏗️",
            crate::core::types::ShipClass::Warship => "⚔️",
        };
        Some(icon.to_string())
    }

    fn get_status_color(&self, ship: &Ship) -> Option<Color> {
        let color = match ship.trajectory {
            Some(_) => Color::new(0.3, 0.8, 1.0, 1.0), // Blue - moving
            None => Color::new(0.8, 0.8, 0.8, 1.0),    // Gray - idle
        };
        Some(color)
    }

    fn is_highlighted(&self, ship: &Ship) -> bool {
        // Highlight ships that are idle or have special conditions
        ship.trajectory.is_none() || ship.ship_class == crate::core::types::ShipClass::Colony
    }
}

impl Default for ShipAdapter {
    fn default() -> Self {
        Self::new()
    }
}