// src/ui_v2/adapters/entity_adapter.rs
//! Base trait for entity adapters

use crate::core::events::PlayerCommand;

/// Trait for adapting game entities to UI display format
pub trait EntityAdapter<T> {
    /// Get display fields as (label, value) pairs
    fn get_display_fields(&self, entity: &T) -> Vec<(String, String)>;
    
    /// Get available actions for this entity
    fn get_actions(&self, entity: &T) -> Vec<(String, PlayerCommand)>;
    
    /// Format a specific field value
    fn format_field(&self, field_name: &str, entity: &T) -> String;
    
    /// Get entity summary for lists
    fn get_summary(&self, entity: &T) -> String;
    
    /// Get entity icon/symbol
    fn get_icon(&self, entity: &T) -> Option<String> {
        None
    }
    
    /// Get color coding for entity state
    fn get_status_color(&self, entity: &T) -> Option<macroquad::prelude::Color> {
        None
    }
    
    /// Check if entity should be highlighted
    fn is_highlighted(&self, entity: &T) -> bool {
        false
    }
}

/// Helper function to format large numbers
pub fn format_number(value: i32) -> String {
    if value >= 1_000_000 {
        format!("{:.1}M", value as f32 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}K", value as f32 / 1_000.0)
    } else {
        value.to_string()
    }
}

/// Helper function to format percentages
pub fn format_percentage(value: f32) -> String {
    format!("{:.1}%", value * 100.0)
}

/// Helper function to format resource amounts
pub fn format_resource(current: i32, capacity: i32) -> String {
    if capacity > 0 {
        format!("{} / {} ({})", 
            format_number(current), 
            format_number(capacity),
            format_percentage(current as f32 / capacity as f32)
        )
    } else {
        format_number(current)
    }
}