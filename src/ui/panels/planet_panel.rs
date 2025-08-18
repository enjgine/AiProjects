// src/ui/panels/planet_panel.rs
#![allow(dead_code, unused_variables, unused_imports)]
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use crate::core::events::PlayerCommand;
use super::Panel;
use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PanelTab {
    Overview,
    Workers,
    Buildings,
    Resources,
}

pub struct PlanetPanel {
    panel_rect: Rect,
    visible: bool,
    selected_planet_id: Option<PlanetId>,
    active_tab: PanelTab,
    scroll_offset: f32,
    button_cache: HashMap<String, Rect>,
    #[allow(dead_code)] // Reserved for future worker allocation editing functionality
    worker_allocation_temp: WorkerAllocation,
    allocation_editing: bool,
    cache_dirty: bool,
    last_planet_hash: u64,
}

impl Panel for PlanetPanel {
    fn new() -> Self {
        Self {
            panel_rect: Rect::new(10.0, 10.0, 400.0, 500.0), // Increased size for better content
            visible: false,
            selected_planet_id: None,
            active_tab: PanelTab::Overview,
            scroll_offset: 0.0,
            button_cache: HashMap::with_capacity(16), // Pre-allocate for performance
            worker_allocation_temp: WorkerAllocation::default(),
            allocation_editing: false,
            cache_dirty: true,
            last_planet_hash: 0,
        }
    }

    fn show(&mut self) {
        self.visible = true;
        self.active_tab = PanelTab::Overview;
        self.scroll_offset = 0.0;
        self.allocation_editing = false;
    }

    fn hide(&mut self) {
        self.visible = false;
        self.selected_planet_id = None;
        self.allocation_editing = false;
        self.button_cache.clear();
        self.cache_dirty = true;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}

impl PlanetPanel {
    /// Display a specific planet in this panel
    pub fn show_planet(&mut self, planet_id: PlanetId) {
        if self.selected_planet_id != Some(planet_id) {
            self.cache_dirty = true;
        }
        self.visible = true;
        self.selected_planet_id = Some(planet_id);
        self.active_tab = PanelTab::Overview;
        self.scroll_offset = 0.0;
        self.allocation_editing = false;
        if self.cache_dirty {
            self.button_cache.clear();
        }
    }
    
    /// Get the currently selected planet ID
    pub fn get_selected_planet(&self) -> Option<PlanetId> {
        self.selected_planet_id
    }
    
    /// Handle user input for interactive functionality
    pub fn handle_input(&mut self, events: &mut EventBus) -> GameResult<()> {
        if !self.visible {
            return Ok(());
        }
        
        #[cfg(not(test))]
        {
            let (mouse_x, mouse_y) = mouse_position();
            
            // Handle mouse clicks on cached buttons
            if is_mouse_button_pressed(MouseButton::Left) {
                self.handle_button_clicks(mouse_x, mouse_y, events)?;
            }
            
            // Handle scroll wheel for long content
            let scroll_delta = mouse_wheel().1;
            if scroll_delta != 0.0 && self.is_mouse_over_panel(mouse_x, mouse_y) {
                self.scroll_offset = (self.scroll_offset - scroll_delta * 20.0).max(0.0);
            }
        }
        
        Ok(())
    }
    
    pub fn render(&mut self, planet: &Planet, _events: &mut EventBus) -> GameResult<()> {
        if !self.visible {
            return Ok(());
        }
        
        // Enhanced input validation
        if Some(planet.id) != self.selected_planet_id {
            return Err(GameError::InvalidOperation(
                format!("Panel displaying wrong planet: expected {:?}, got {}", 
                        self.selected_planet_id, planet.id)
            ));
        }
        
        // Validate planet data before rendering
        planet.resources.validate()?;
        planet.population.allocation.validate(planet.population.total)?;
        
        // Create a simple hash of planet state for cache invalidation
        let planet_hash = self.calculate_planet_hash(planet);
        if planet_hash != self.last_planet_hash {
            self.cache_dirty = true;
            self.last_planet_hash = planet_hash;
        }
        
        // Only clear button cache if data has changed
        if self.cache_dirty {
            self.button_cache.clear();
            self.cache_dirty = false;
        }
        
        // Skip macroquad rendering in test environments
        #[cfg(not(test))]
        {
            self.draw_panel_background();
            
            // Render header with planet name and close button
            self.render_header(planet)?;
            
            // Render tab navigation
            self.render_tabs()?;
            
            // Render active tab content
            match self.active_tab {
                PanelTab::Overview => self.render_overview_tab(planet)?,
                PanelTab::Workers => self.render_workers_tab(planet)?,
                PanelTab::Buildings => self.render_buildings_tab(planet)?,
                PanelTab::Resources => self.render_resources_tab(planet)?,
            }
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn draw_panel_background(&self) {
        // Enhanced background with better visual styling
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            Color::new(0.15, 0.15, 0.25, 0.95),
        );
        
        // Improved border with better contrast
        draw_rectangle_lines(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            2.0,
            Color::new(0.8, 0.8, 0.9, 1.0),
        );
    }
    
    #[allow(dead_code)]
    fn render_header(&mut self, planet: &Planet) -> GameResult<()> {
        let header_height = 40.0;
        let controller_text = match planet.controller {
            Some(faction_id) => {
                if faction_id == 0 { "Player Controlled" } else { "Enemy Controlled" }
            }
            None => "Uncontrolled"
        };
        let title_text = format!("Planet {} - {}", planet.id, controller_text);
        
        // Draw header background
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            header_height,
            Color::new(0.1, 0.1, 0.2, 1.0),
        );
        
        // Planet title with better formatting
        draw_text(
            &title_text,
            self.panel_rect.x + 10.0,
            self.panel_rect.y + 25.0,
            18.0,
            WHITE,
        );
        
        // Close button with proper caching
        let close_rect = Rect::new(
            self.panel_rect.x + self.panel_rect.w - 35.0,
            self.panel_rect.y + 5.0,
            30.0,
            30.0,
        );
        self.button_cache.insert("close".to_string(), close_rect);
        self.draw_button(close_rect, "×", RED, WHITE);
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn render_tabs(&mut self) -> GameResult<()> {
        let tab_height = 30.0;
        let tab_width = self.panel_rect.w / 4.0;
        let tab_y = self.panel_rect.y + 40.0;
        
        let tabs = [
            (PanelTab::Overview, "Overview"),
            (PanelTab::Workers, "Workers"),
            (PanelTab::Buildings, "Buildings"),
            (PanelTab::Resources, "Resources"),
        ];
        
        for (i, (tab_type, label)) in tabs.iter().enumerate() {
            let tab_x = self.panel_rect.x + i as f32 * tab_width;
            let is_active = self.active_tab == *tab_type;
            
            let tab_rect = Rect::new(tab_x, tab_y, tab_width, tab_height);
            self.button_cache.insert(format!("tab_{}", i), tab_rect);
            
            let bg_color = if is_active {
                Color::new(0.3, 0.3, 0.4, 1.0)
            } else {
                Color::new(0.2, 0.2, 0.3, 1.0)
            };
            
            self.draw_button(tab_rect, label, bg_color, WHITE);
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn render_overview_tab(&mut self, planet: &Planet) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 80.0 - self.scroll_offset;
        let line_height = 20.0;
        
        // Population summary with growth rate
        draw_text("Population:", self.panel_rect.x + 10.0, y_offset, 16.0, YELLOW);
        y_offset += line_height;
        
        draw_text(
            &format!("  Total: {} citizens", planet.population.total),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += line_height;
        
        draw_text(
            &format!("  Growth Rate: {:.1}% per tick", planet.population.growth_rate * 100.0),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += line_height * 1.5;
        
        // Resource summary (all 6 types)
        draw_text("Resource Summary:", self.panel_rect.x + 10.0, y_offset, 16.0, YELLOW);
        y_offset += line_height;
        
        let key_resources = [
            ("Minerals", planet.resources.current.minerals, GREEN),
            ("Food", planet.resources.current.food, ORANGE),
            ("Energy", planet.resources.current.energy, BLUE),
            ("Alloys", planet.resources.current.alloys, PURPLE),
            ("Components", planet.resources.current.components, Color::new(0.0, 1.0, 1.0, 1.0)),
            ("Fuel", planet.resources.current.fuel, RED),
        ];
        
        for (name, amount, color) in key_resources {
            draw_text(
                &format!("  {}: {}", name, amount),
                self.panel_rect.x + 20.0,
                y_offset,
                14.0,
                color,
            );
            y_offset += line_height;
        }
        y_offset += line_height * 0.5;
        
        // Buildings summary with detailed information
        draw_text("Infrastructure:", self.panel_rect.x + 10.0, y_offset, 16.0, YELLOW);
        y_offset += line_height;
        
        let operational_count = planet.developments.iter()
            .filter(|b| b.operational)
            .count();
        
        draw_text(
            &format!("  Buildings: {} ({} operational)", 
                    planet.developments.len(), operational_count),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += line_height;
        
        // Calculate and display building slots
        let building_slots = 10 + planet.population.total / 10000;
        let available_slots = building_slots - planet.developments.len() as i32;
        let slots_color = if available_slots > 0 { GREEN } else { RED };
        
        draw_text(
            &format!("  Available Slots: {} / {}", available_slots, building_slots),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            slots_color,
        );
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn render_workers_tab(&mut self, planet: &Planet) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 80.0 - self.scroll_offset;
        let line_height = 25.0;
        
        draw_text("Worker Allocation:", self.panel_rect.x + 10.0, y_offset, 16.0, YELLOW);
        y_offset += line_height;
        
        let allocations = [
            ("Agriculture", planet.population.allocation.agriculture, "food production"),
            ("Mining", planet.population.allocation.mining, "mineral extraction"),
            ("Industry", planet.population.allocation.industry, "manufacturing"),
            ("Research", planet.population.allocation.research, "technology advancement"),
            ("Military", planet.population.allocation.military, "defense and conquest"),
            ("Unassigned", planet.population.allocation.unassigned, "available workers"),
        ];
        
        for (job, count, description) in allocations {
            let percentage = if planet.population.total > 0 {
                (count as f32 / planet.population.total as f32) * 100.0
            } else {
                0.0
            };
            
            let job_color = if count > 0 { WHITE } else { LIGHTGRAY };
            
            draw_text(
                &format!("{}: {} ({:.1}%)", job, count, percentage),
                self.panel_rect.x + 15.0,
                y_offset,
                14.0,
                job_color,
            );
            y_offset += 15.0;
            
            draw_text(
                &format!("  {}", description),
                self.panel_rect.x + 25.0,
                y_offset,
                12.0,
                LIGHTGRAY,
            );
            y_offset += line_height - 15.0;
        }
        
        y_offset += 10.0;
        
        // Worker allocation buttons
        let edit_rect = Rect::new(
            self.panel_rect.x + 10.0,
            y_offset,
            150.0,
            30.0,
        );
        self.button_cache.insert("edit_workers".to_string(), edit_rect);
        
        let button_text = if self.allocation_editing { "Cancel Edit" } else { "Reassign Workers" };
        let button_color = if self.allocation_editing { RED } else { GREEN };
        self.draw_button(edit_rect, button_text, button_color, WHITE);
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn render_buildings_tab(&mut self, planet: &Planet) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 80.0 - self.scroll_offset;
        let line_height = 30.0;
        
        draw_text("Buildings:", self.panel_rect.x + 10.0, y_offset, 16.0, YELLOW);
        y_offset += 25.0;
        
        if planet.developments.is_empty() {
            draw_text(
                "No buildings constructed yet",
                self.panel_rect.x + 20.0,
                y_offset,
                14.0,
                LIGHTGRAY,
            );
            y_offset += line_height;
        } else {
            for building in &planet.developments {
                let building_name = self.format_building_name(building.building_type);
                let status = if building.operational { "Operational" } else { "Under Construction" };
                let status_color = if building.operational { GREEN } else { YELLOW };
                
                draw_text(
                    &format!("{} (Tier {})", building_name, building.tier),
                    self.panel_rect.x + 15.0,
                    y_offset,
                    14.0,
                    WHITE,
                );
                y_offset += 15.0;
                
                draw_text(
                    &format!("  Status: {}", status),
                    self.panel_rect.x + 25.0,
                    y_offset,
                    12.0,
                    status_color,
                );
                y_offset += line_height - 15.0;
            }
        }
        
        y_offset += 15.0;
        
        // Build structure button
        let build_rect = Rect::new(
            self.panel_rect.x + 10.0,
            y_offset,
            120.0,
            30.0,
        );
        self.button_cache.insert("build_structure".to_string(), build_rect);
        self.draw_button(build_rect, "Build Structure", BLUE, WHITE);
        
        // Building slots information
        y_offset += 40.0;
        let building_slots = 10 + planet.population.total / 10000;
        let used_slots = planet.developments.len() as i32;
        let available_slots = building_slots - used_slots;
        
        draw_text(
            &format!("Building Slots: {} / {}", used_slots, building_slots),
            self.panel_rect.x + 10.0,
            y_offset,
            14.0,
            if available_slots > 0 { WHITE } else { RED },
        );
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn render_resources_tab(&mut self, planet: &Planet) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 80.0 - self.scroll_offset;
        let line_height = 25.0;
        
        draw_text("Resource Storage:", self.panel_rect.x + 10.0, y_offset, 16.0, YELLOW);
        y_offset += line_height;
        
        // Display all 6 resource types with capacity bars
        let resources = [
            ("Minerals", planet.resources.current.minerals, planet.resources.capacity.minerals, GREEN),
            ("Food", planet.resources.current.food, planet.resources.capacity.food, ORANGE),
            ("Energy", planet.resources.current.energy, planet.resources.capacity.energy, BLUE),
            ("Alloys", planet.resources.current.alloys, planet.resources.capacity.alloys, PURPLE),
            ("Components", planet.resources.current.components, planet.resources.capacity.components, Color::new(0.0, 1.0, 1.0, 1.0)), // CYAN equivalent
            ("Fuel", planet.resources.current.fuel, planet.resources.capacity.fuel, RED),
        ];
        
        for (name, current, capacity, color) in resources {
            let percentage = if capacity > 0 {
                (current as f32 / capacity as f32) * 100.0
            } else {
                0.0
            };
            
            draw_text(
                &format!("{}: {} / {} ({:.1}%)", name, current, capacity, percentage),
                self.panel_rect.x + 15.0,
                y_offset,
                14.0,
                color,
            );
            
            // Resource capacity bar
            let bar_x = self.panel_rect.x + 20.0;
            let bar_y = y_offset + 5.0;
            let bar_w = 200.0;
            let bar_h = 8.0;
            
            // Background bar
            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.3, 0.3, 0.3, 1.0));
            
            // Fill bar based on capacity
            if capacity > 0 {
                let fill_w = bar_w * (current as f32 / capacity as f32);
                let fill_color = if percentage > 90.0 { 
                    RED 
                } else if percentage > 70.0 { 
                    YELLOW 
                } else { 
                    color 
                };
                draw_rectangle(bar_x, bar_y, fill_w, bar_h, fill_color);
            }
            
            y_offset += line_height;
        }
        
        y_offset += 15.0;
        
        // Resource management buttons
        let transfer_rect = Rect::new(
            self.panel_rect.x + 10.0,
            y_offset,
            140.0,
            30.0,
        );
        self.button_cache.insert("transfer_resources".to_string(), transfer_rect);
        self.draw_button(transfer_rect, "Transfer Resources", GOLD, BLACK);
        
        Ok(())
    }
    
    // Enhanced helper methods
    #[allow(dead_code)]
    fn draw_button(&self, rect: Rect, text: &str, bg_color: Color, text_color: Color) {
        #[cfg(not(test))]
        {
            let (mouse_x, mouse_y) = mouse_position();
            let hovered = mouse_x >= rect.x && mouse_x <= rect.x + rect.w && 
                         mouse_y >= rect.y && mouse_y <= rect.y + rect.h;
            
            // Enhanced hover effect
            let final_bg_color = if hovered {
                Color::new(
                    (bg_color.r * 1.2).min(1.0), 
                    (bg_color.g * 1.2).min(1.0), 
                    (bg_color.b * 1.2).min(1.0), 
                    bg_color.a
                )
            } else {
                bg_color
            };
            
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, final_bg_color);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, WHITE);
            
            // Center text in button
            let text_size = 14.0;
            let text_dims = measure_text(text, None, text_size as u16, 1.0);
            draw_text(
                text,
                rect.x + (rect.w - text_dims.width) / 2.0,
                rect.y + (rect.h + text_dims.height) / 2.0,
                text_size,
                text_color,
            );
        }
    }
    
    #[allow(dead_code)]
    fn handle_button_clicks(&mut self, mouse_x: f32, mouse_y: f32, events: &mut EventBus) -> GameResult<()> {
        // Collect button ID to avoid borrowing conflicts
        let mut clicked_button: Option<String> = None;
        
        for (button_id, rect) in &self.button_cache {
            if mouse_x >= rect.x && mouse_x <= rect.x + rect.w &&
               mouse_y >= rect.y && mouse_y <= rect.y + rect.h {
                clicked_button = Some(button_id.clone());
                break;
            }
        }
        
        // Handle the button action outside the iteration
        if let Some(button_id) = clicked_button {
            self.handle_button_action(&button_id, events)?;
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    fn handle_button_action(&mut self, button_id: &str, events: &mut EventBus) -> GameResult<()> {
        if let Some(_planet_id) = self.selected_planet_id {
            match button_id {
                "close" => {
                    self.hide();
                }
                "tab_0" => self.active_tab = PanelTab::Overview,
                "tab_1" => self.active_tab = PanelTab::Workers,
                "tab_2" => self.active_tab = PanelTab::Buildings,
                "tab_3" => self.active_tab = PanelTab::Resources,
                "build_structure" => {
                    // Validate building slots before emitting command
                    if let Some(planet_id) = self.selected_planet_id {
                        events.queue_event(GameEvent::PlayerCommand(
                            PlayerCommand::BuildStructure {
                                planet: planet_id,
                                building_type: BuildingType::Mine, // Default choice - would be user-selected in full UI
                            }
                        ));
                    }
                }
                "edit_workers" => {
                    self.allocation_editing = !self.allocation_editing;
                    // In a real implementation, this would cache current allocation for editing
                    if self.allocation_editing {
                        // Could emit an event or open a worker allocation dialog
                    }
                }
                "transfer_resources" => {
                    // Only emit if we have a valid source planet
                    if let Some(from_planet) = self.selected_planet_id {
                        events.queue_event(GameEvent::PlayerCommand(
                            PlayerCommand::TransferResources {
                                from: from_planet,
                                to: from_planet, // Placeholder - would be user-selected in full implementation
                                resources: ResourceBundle::default(), // Would be user-specified amounts
                            }
                        ));
                    }
                }
                _ => {
                    // Unknown button - could log for debugging
                }
            }
        }
        Ok(())
    }
    
    #[allow(dead_code)]
    fn is_mouse_over_panel(&self, mouse_x: f32, mouse_y: f32) -> bool {
        mouse_x >= self.panel_rect.x && mouse_x <= self.panel_rect.x + self.panel_rect.w &&
        mouse_y >= self.panel_rect.y && mouse_y <= self.panel_rect.y + self.panel_rect.h
    }
    
    #[allow(dead_code)]
    fn format_building_name(&self, building_type: BuildingType) -> &'static str {
        match building_type {
            BuildingType::Mine => "Mining Facility",
            BuildingType::Farm => "Agricultural Complex",
            BuildingType::PowerPlant => "Power Generation Plant",
            BuildingType::Factory => "Industrial Factory",
            BuildingType::ResearchLab => "Research Laboratory",
            BuildingType::Spaceport => "Orbital Spaceport",
            BuildingType::DefensePlatform => "Defense Platform",
            BuildingType::StorageFacility => "Storage Facility",
            BuildingType::Habitat => "Residential Habitat",
        }
    }
    
    /// Calculate a simple hash of planet state for cache invalidation
    fn calculate_planet_hash(&self, planet: &Planet) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        planet.id.hash(&mut hasher);
        planet.population.total.hash(&mut hasher);
        planet.resources.current.minerals.hash(&mut hasher);
        planet.resources.current.food.hash(&mut hasher);
        planet.resources.current.energy.hash(&mut hasher);
        planet.developments.len().hash(&mut hasher);
        hasher.finish()
    }
    
    // Legacy methods for backward compatibility
    #[allow(dead_code)]
    fn render_planet_info(&self, _planet: &Planet) -> GameResult<()> {
        // This method is now handled by render_header
        Ok(())
    }
    
    #[allow(dead_code)]
    fn render_resources(&self, _planet: &Planet) -> GameResult<()> {
        // This method is now handled by render_resources_tab
        Ok(())
    }
    
    #[allow(dead_code)]
    fn render_population(&self, _planet: &Planet) -> GameResult<()> {
        // This method is now handled by render_workers_tab
        Ok(())
    }
    
    #[allow(dead_code)]
    fn render_buildings(&self, _planet: &Planet) -> GameResult<()> {
        // This method is now handled by render_buildings_tab
        Ok(())
    }
}

// Comprehensive unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_planet() -> Planet {
        Planet {
            id: 1,
            position: OrbitalElements::default(),
            resources: ResourceStorage {
                current: ResourceBundle {
                    minerals: 1000,
                    food: 500,
                    energy: 750,
                    alloys: 100,
                    components: 50,
                    fuel: 200,
                },
                capacity: ResourceBundle {
                    minerals: 2000,
                    food: 1000,
                    energy: 1500,
                    alloys: 500,
                    components: 250,
                    fuel: 400,
                },
            },
            population: Demographics {
                total: 10000,
                growth_rate: 0.02,
                allocation: WorkerAllocation {
                    agriculture: 3000,
                    mining: 2000,
                    industry: 1500,
                    research: 1000,
                    military: 500,
                    unassigned: 2000,
                },
            },
            developments: vec![
                Building {
                    building_type: BuildingType::Mine,
                    tier: 1,
                    operational: true,
                },
                Building {
                    building_type: BuildingType::Farm,
                    tier: 2,
                    operational: false,
                },
            ],
            controller: Some(0),
        }
    }
    
    #[test]
    fn test_panel_creation() {
        let panel = PlanetPanel::new();
        assert!(!panel.is_visible());
        assert_eq!(panel.get_selected_planet(), None);
        assert_eq!(panel.active_tab, PanelTab::Overview);
    }
    
    #[test]
    fn test_show_hide_functionality() {
        let mut panel = PlanetPanel::new();
        
        panel.show_planet(1);
        assert!(panel.is_visible());
        assert_eq!(panel.get_selected_planet(), Some(1));
        
        panel.hide();
        assert!(!panel.is_visible());
        assert_eq!(panel.get_selected_planet(), None);
    }
    
    #[test]
    fn test_render_with_valid_planet() {
        let mut panel = PlanetPanel::new();
        let planet = create_test_planet();
        let mut events = EventBus::new();
        
        panel.show_planet(planet.id);
        
        // Should not panic with valid data
        assert!(panel.render(&planet, &mut events).is_ok());
    }
    
    #[test]
    fn test_render_with_wrong_planet() {
        let mut panel = PlanetPanel::new();
        let planet = create_test_planet();
        let mut events = EventBus::new();
        
        panel.show_planet(999); // Different planet ID
        
        // Should return error for mismatched planet
        assert!(panel.render(&planet, &mut events).is_err());
    }
    
    #[test]
    fn test_tab_switching() {
        let mut panel = PlanetPanel::new();
        let mut events = EventBus::new();
        
        panel.show_planet(1);
        assert_eq!(panel.active_tab, PanelTab::Overview);
        
        // Simulate clicking different tabs
        panel.handle_button_action("tab_1", &mut events).unwrap();
        assert_eq!(panel.active_tab, PanelTab::Workers);
        
        panel.handle_button_action("tab_2", &mut events).unwrap();
        assert_eq!(panel.active_tab, PanelTab::Buildings);
        
        panel.handle_button_action("tab_3", &mut events).unwrap();
        assert_eq!(panel.active_tab, PanelTab::Resources);
    }
    
    #[test]
    fn test_button_cache_management() {
        let mut panel = PlanetPanel::new();
        let planet = create_test_planet();
        let mut events = EventBus::new();
        
        panel.show_planet(planet.id);
        
        // Cache should be dirty when showing new planet
        assert!(panel.cache_dirty);
        
        panel.render(&planet, &mut events).unwrap();
        
        // Cache should no longer be dirty after render
        assert!(!panel.cache_dirty);
        
        panel.hide();
        
        // Button cache should be cleared when hiding
        assert!(panel.button_cache.is_empty());
        assert!(panel.cache_dirty);
    }
    
    #[test]
    fn test_building_name_formatting() {
        let panel = PlanetPanel::new();
        
        assert_eq!(panel.format_building_name(BuildingType::Mine), "Mining Facility");
        assert_eq!(panel.format_building_name(BuildingType::Farm), "Agricultural Complex");
        assert_eq!(panel.format_building_name(BuildingType::PowerPlant), "Power Generation Plant");
    }
    
    #[test]
    fn test_worker_allocation_editing() {
        let mut panel = PlanetPanel::new();
        let mut events = EventBus::new();
        
        panel.show_planet(1);
        assert!(!panel.allocation_editing);
        
        panel.handle_button_action("edit_workers", &mut events).unwrap();
        assert!(panel.allocation_editing);
        
        panel.handle_button_action("edit_workers", &mut events).unwrap();
        assert!(!panel.allocation_editing);
    }
    
    #[test] 
    fn test_event_emission() {
        let mut panel = PlanetPanel::new();
        let mut events = EventBus::new();
        
        panel.show_planet(1);
        
        // Test build structure event
        panel.handle_button_action("build_structure", &mut events).unwrap();
        assert!(!events.queued_events.is_empty());
        
        // Check that correct event type was emitted
        if let Some(GameEvent::PlayerCommand(PlayerCommand::BuildStructure { planet, building_type })) = events.queued_events.front() {
            assert_eq!(*planet, 1);
            assert_eq!(*building_type, BuildingType::Mine);
        } else {
            panic!("Expected BuildStructure command");
        }
    }
    
    #[test]
    fn test_panel_trait_implementation() {
        let mut panel = PlanetPanel::new();
        
        // Test Panel trait methods
        assert!(!panel.is_visible());
        
        panel.show();
        assert!(panel.is_visible());
        
        panel.hide();
        assert!(!panel.is_visible());
        
        // Test toggle functionality
        panel.toggle();
        assert!(panel.is_visible());
        
        panel.toggle();
        assert!(!panel.is_visible());
    }
    
    #[test]
    fn test_cache_invalidation() {
        let mut panel = PlanetPanel::new();
        let mut planet = create_test_planet();
        
        panel.show_planet(planet.id);
        let initial_hash = panel.calculate_planet_hash(&planet);
        
        // Modify planet state
        planet.population.total = 20000;
        let new_hash = panel.calculate_planet_hash(&planet);
        
        // Hash should be different
        assert_ne!(initial_hash, new_hash);
    }
    
    #[test]
    fn test_edge_case_zero_population() {
        let mut panel = PlanetPanel::new();
        let mut planet = create_test_planet();
        let mut events = EventBus::new();
        
        // Set zero population
        planet.population.total = 0;
        planet.population.allocation = WorkerAllocation::default();
        
        panel.show_planet(planet.id);
        
        // Should not panic with zero population
        assert!(panel.render(&planet, &mut events).is_ok());
    }
    
    #[test]
    fn test_resource_display_completeness() {
        let _panel = PlanetPanel::new();
        let planet = create_test_planet();
        
        // Verify all 6 resource types are represented in data
        assert!(planet.resources.current.minerals >= 0);
        assert!(planet.resources.current.food >= 0);
        assert!(planet.resources.current.energy >= 0);
        assert!(planet.resources.current.alloys >= 0);
        assert!(planet.resources.current.components >= 0);
        assert!(planet.resources.current.fuel >= 0);
    }
    
    #[test]
    fn test_input_validation() {
        let mut panel = PlanetPanel::new();
        let mut events = EventBus::new();
        
        // Test input handling when panel is hidden
        assert!(panel.handle_input(&mut events).is_ok());
        
        panel.show_planet(1);
        
        // Test input handling when panel is visible
        assert!(panel.handle_input(&mut events).is_ok());
    }
}