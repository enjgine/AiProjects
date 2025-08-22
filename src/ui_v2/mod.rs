// src/ui_v2/mod.rs
//! UI Version 2: Modular, Component-Based Architecture
//! 
//! This is the new UI system designed to replace the existing UI with:
//! - Generic, reusable components
//! - Clear separation of concerns (Component-View-Controller)
//! - Event-driven architecture
//! - Minimal code duplication
//! 
//! Architecture:
//! - core/: Fundamental UI infrastructure (renderer, controllers, state)
//! - components/: Reusable UI primitives (buttons, panels, lists)
//! - views/: Specialized presentations (entity view, data view, dialog view)
//! - adapters/: Entity-specific data formatting (planet, ship, resource)

pub mod core;
pub mod components;
pub mod views;
pub mod adapters;
pub mod panels;

// Re-export main public interfaces
pub use core::{
    UISystem,
    ViewController,
    InputController,
    RenderContext,
    Layout,
    ComponentResult,
    ComponentError,
    InputEvent,
    ViewEvent,
    ViewData,
    ViewId,
    ViewType,
};

pub use components::{
    UIComponent,
    Button,
    Dropdown,
    Panel,
    ListView,
};

pub use views::{
    View,
    BaseView,
    EntityView,
    DataView,
    DialogView,
};

pub use adapters::{
    EntityAdapter,
    PlanetAdapter,
    ShipAdapter,
    FactionAdapter,
};

pub use panels::{
    PlanetPanelMigrated,
    ShipPanelMigrated,
    ResourcePanelMigrated,
};

// Version and compatibility info
pub const UI_VERSION: &str = "2.0.0";
pub const COMPATIBILITY_NOTE: &str = "This UI system replaces ui/ module";