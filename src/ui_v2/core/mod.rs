// src/ui_v2/core/mod.rs
//! Core UI system infrastructure
//! 
//! This module provides the foundational types and systems for the new UI:
//! - RenderContext: Rendering state and utilities
//! - ComponentResult: Standardized component results
//! - UISystem: Main coordinator for all UI operations
//! - Event types and routing

pub mod ui_system;
pub mod view_controller;
pub mod input_controller;
pub mod render_context;

pub use ui_system::{UISystem, UIMetrics, UISystemBuilder};
pub use view_controller::ViewController;
pub use input_controller::InputController;
pub use render_context::{RenderContext, Theme, ComponentResult, ComponentError};

// Types are defined below - no need for re-export

use crate::core::types::*;
use macroquad::prelude::*;
use std::collections::HashMap;

/// Unique identifier for UI views
pub type ViewId = u32;

/// Unique identifier for UI components
pub type ComponentId = u32;

/// Input events processed by UI components
#[derive(Debug, Clone)]
pub enum InputEvent {
    MouseMove { x: f32, y: f32 },
    MouseClick { x: f32, y: f32, button: MouseButton },
    MouseRelease { x: f32, y: f32, button: MouseButton },
    KeyPress { key: KeyCode },
    KeyRelease { key: KeyCode },
    Scroll { x: f32, y: f32, delta: f32 },
}

/// Events for view lifecycle management
#[derive(Debug, Clone)]
pub enum ViewEvent {
    ShowEntity { entity_type: EntityType, id: u32 },
    UpdateData { view_type: String, data: ViewData },
    ShowView { view_type: String },
    HideView { view_type: String },
    CloseView { view_id: ViewId },
    ToggleView { view_type: ViewType },
    RefreshView { view_id: ViewId },
}

/// Types of entities that can be displayed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Planet,
    Ship,
    Faction,
    Resource,
}

/// Types of views that can be created
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewType {
    PlanetPanel,
    ShipPanel,
    MainMenu,
    GameOptions,
    SaveLoad,
    ResourcePanel,
}

/// Generic data container for views
#[derive(Debug, Clone)]
pub enum ViewData {
    Planet(Planet),
    Ship(Ship),
    Faction(Faction),
    ResourceBundle(ResourceBundle),
    Text(String),
    Custom(HashMap<String, String>),
}

/// Layout information for positioning components
#[derive(Debug, Clone)]
pub struct Layout {
    pub position: Vec2,
    pub size: Vec2,
    pub padding: f32,
    pub margin: f32,
    pub anchor: Anchor,
}

/// Anchor points for component positioning
#[derive(Debug, Clone, Copy)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    TopCenter,
    BottomCenter,
    LeftCenter,
    RightCenter,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            size: Vec2::new(100.0, 100.0),
            padding: 5.0,
            margin: 5.0,
            anchor: Anchor::TopLeft,
        }
    }
}

impl Layout {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
            ..Default::default()
        }
    }

    pub fn get_rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        let rect = self.get_rect();
        point.x >= rect.x && point.x <= rect.x + rect.w &&
        point.y >= rect.y && point.y <= rect.y + rect.h
    }
}