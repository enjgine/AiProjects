// src/ui_v2/adapters/faction_adapter.rs
//! Adapter for Faction entities

use super::{EntityAdapter, format_number};
use crate::core::types::Faction;
use crate::core::events::PlayerCommand;
use macroquad::prelude::Color;

/// Adapter for displaying Faction entities in UI
pub struct FactionAdapter {
    show_detailed_stats: bool,
    show_territory_info: bool,
}

impl FactionAdapter {
    pub fn new() -> Self {
        Self {
            show_detailed_stats: true,
            show_territory_info: true,
        }
    }

    pub fn simple() -> Self {
        Self {
            show_detailed_stats: false,
            show_territory_info: false,
        }
    }

    pub fn with_detailed_stats(mut self, show: bool) -> Self {
        self.show_detailed_stats = show;
        self
    }

    pub fn with_territory_info(mut self, show: bool) -> Self {
        self.show_territory_info = show;
        self
    }
}

impl EntityAdapter<Faction> for FactionAdapter {
    fn get_display_fields(&self, faction: &Faction) -> Vec<(String, String)> {
        let mut fields = Vec::new();

        // Basic info
        fields.push(("ID".to_string(), faction.id.to_string()));
        fields.push(("Name".to_string(), faction.name.clone()));
        // Color info - would need to be computed or stored separately
        fields.push(("AI Type".to_string(), format!("{:?}", faction.ai_type)));

        // AI Status
        let ai_status = if !faction.is_player { "AI Controlled" } else { "Player Controlled" };
        fields.push(("Control".to_string(), ai_status.to_string()));

        if self.show_detailed_stats {
            // Territory information would go here
            // This would require additional game state information
            fields.push(("Planets".to_string(), "N/A".to_string())); // Would need planet count
            fields.push(("Ships".to_string(), "N/A".to_string()));   // Would need ship count
            fields.push(("Total Population".to_string(), "N/A".to_string())); // Would need sum of planet populations
        }

        if self.show_territory_info {
            // Territory expansion and influence would go here
            // This would require additional game mechanics
            fields.push(("Territory Size".to_string(), "N/A".to_string()));
            fields.push(("Border Worlds".to_string(), "N/A".to_string()));
        }

        fields
    }

    fn get_actions(&self, faction: &Faction) -> Vec<(String, PlayerCommand)> {
        let mut actions = Vec::new();

        // Basic faction actions
        actions.push(("View Details".to_string(), PlayerCommand::ShowFaction(faction.id)));
        
        // Only show diplomatic/admin actions for AI factions when player is viewing
        if !faction.is_player {
            actions.push(("Diplomacy".to_string(), PlayerCommand::OpenDiplomacy(faction.id)));
            actions.push(("Trade Agreement".to_string(), PlayerCommand::ProposeTradeAgreement(faction.id)));
        }

        // Universal actions
        actions.push(("View Territory".to_string(), PlayerCommand::ShowFactionTerritory(faction.id)));
        actions.push(("Intelligence Report".to_string(), PlayerCommand::ShowIntelligenceReport(faction.id)));

        // Close action
        actions.push(("Close".to_string(), PlayerCommand::CloseFactionPanel));

        actions
    }

    fn format_field(&self, field_name: &str, faction: &Faction) -> String {
        match field_name {
            "name" => faction.name.clone(),
            "ai_type" => format!("{:?}", faction.ai_type),
            "control" => if !faction.is_player { "AI" } else { "Player" }.to_string(),
            "id" => faction.id.to_string(),
            _ => "N/A".to_string(),
        }
    }

    fn get_summary(&self, faction: &Faction) -> String {
        let control_type = if !faction.is_player { "AI" } else { "Player" };
        format!("{} ({}) - {}", faction.name, control_type, faction.id)
    }

    fn get_icon(&self, faction: &Faction) -> Option<String> {
        let icon = if !faction.is_player {
            match faction.id % 4 {
                0 => "🤖", // Robot face
                1 => "👾", // Alien
                2 => "🛸", // UFO
                _ => "🏛️", // Classical building
            }
        } else {
            "👤" // User icon for player
        };
        Some(icon.to_string())
    }

    fn get_status_color(&self, faction: &Faction) -> Option<Color> {
        // Generate color based on faction properties
        let color = if faction.is_player {
            Color::new(0.3, 0.8, 0.3, 1.0) // Green for player
        } else {
            // Generate different colors for AI based on ID and AI type
            match faction.ai_type {
                crate::core::types::AIPersonality::Aggressive => Color::new(0.8, 0.3, 0.3, 1.0), // Red
                crate::core::types::AIPersonality::Economic => Color::new(0.3, 0.3, 0.8, 1.0),   // Blue
                crate::core::types::AIPersonality::Balanced => Color::new(0.8, 0.8, 0.3, 1.0),   // Yellow
            }
        };
        
        Some(color)
    }

    fn is_highlighted(&self, faction: &Faction) -> bool {
        // Highlight player-controlled factions
        faction.is_player
    }
}

impl Default for FactionAdapter {
    fn default() -> Self {
        Self::new()
    }
}