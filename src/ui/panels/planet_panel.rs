// src/ui/panels/planet_panel.rs
use crate::core::{GameResult, GameEvent, EventBus};
use crate::core::types::*;
use macroquad::prelude::*;

pub struct PlanetPanel {
    panel_rect: Rect,
    visible: bool,
}

impl PlanetPanel {
    pub fn new() -> Self {
        Self {
            panel_rect: Rect::new(10.0, 10.0, 300.0, 400.0),
            visible: false,
        }
    }
    
    pub fn render(&mut self, planet: &Planet, events: &mut EventBus) -> GameResult<()> {
        if !self.visible {
            return Ok(());
        }
        
        // Draw panel background
        draw_rectangle(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            Color::new(0.2, 0.2, 0.3, 0.9),
        );
        
        // Draw panel border
        draw_rectangle_lines(
            self.panel_rect.x,
            self.panel_rect.y,
            self.panel_rect.w,
            self.panel_rect.h,
            2.0,
            WHITE,
        );
        
        // Render planet information
        self.render_planet_info(planet)?;
        
        // Render resource information
        self.render_resources(planet)?;
        
        // Render population information
        self.render_population(planet)?;
        
        // Render buildings
        self.render_buildings(planet)?;
        
        Ok(())
    }
    
    pub fn show(&mut self, planet_id: PlanetId) {
        self.visible = true;
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    fn render_planet_info(&self, planet: &Planet) -> GameResult<()> {
        let y_offset = self.panel_rect.y + 20.0;
        
        draw_text(
            &format!("Planet {}", planet.id),
            self.panel_rect.x + 10.0,
            y_offset,
            20.0,
            WHITE,
        );
        
        Ok(())
    }
    
    fn render_resources(&self, planet: &Planet) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 60.0;
        
        draw_text("Resources:", self.panel_rect.x + 10.0, y_offset, 16.0, WHITE);
        y_offset += 25.0;
        
        let resources = &planet.resources.current;
        draw_text(
            &format!("Minerals: {}", resources.minerals),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += 20.0;
        
        draw_text(
            &format!("Food: {}", resources.food),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        y_offset += 20.0;
        
        draw_text(
            &format!("Energy: {}", resources.energy),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        
        Ok(())
    }
    
    fn render_population(&self, planet: &Planet) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 180.0;
        
        draw_text("Population:", self.panel_rect.x + 10.0, y_offset, 16.0, WHITE);
        y_offset += 25.0;
        
        draw_text(
            &format!("Total: {}", planet.population.total),
            self.panel_rect.x + 20.0,
            y_offset,
            14.0,
            WHITE,
        );
        
        Ok(())
    }
    
    fn render_buildings(&self, planet: &Planet) -> GameResult<()> {
        let mut y_offset = self.panel_rect.y + 250.0;
        
        draw_text("Buildings:", self.panel_rect.x + 10.0, y_offset, 16.0, WHITE);
        y_offset += 25.0;
        
        for building in &planet.developments {
            draw_text(
                &format!("{:?} (Tier {})", building.building_type, building.tier),
                self.panel_rect.x + 20.0,
                y_offset,
                14.0,
                WHITE,
            );
            y_offset += 20.0;
        }
        
        Ok(())
    }
}