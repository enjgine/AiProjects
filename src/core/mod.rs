// src/core/mod.rs
pub mod events;
pub mod types;

// Re-export commonly used types
pub use events::{EventBus, GameEvent, SystemId, PlayerCommand, SimulationEvent, StateChange};
pub use types::*;
use std::collections::VecDeque;

// Import managers and systems
use crate::managers::{PlanetManager, ShipManager, FactionManager};
use crate::systems::{TimeManager, ResourceSystem, PopulationSystem, ConstructionSystem, PhysicsEngine, CombatResolver};
use crate::ui::UIRenderer;

pub struct GameState {
    pub event_bus: EventBus,
    pub planet_manager: PlanetManager,
    pub ship_manager: ShipManager,
    pub faction_manager: FactionManager,
    pub time_manager: TimeManager,
    pub resource_system: ResourceSystem,
    pub population_system: PopulationSystem,
    pub construction_system: ConstructionSystem,
    pub physics_engine: PhysicsEngine,
    pub combat_resolver: CombatResolver,
    pub ui_renderer: UIRenderer,
}

impl GameState {
    pub fn new() -> GameResult<Self> {
        let mut event_bus = EventBus::new();
        
        // Register system subscriptions
        event_bus.subscribe(SystemId::PlanetManager, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::PlanetManager, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::PlanetManager, events::EventType::StateChanged);
        event_bus.subscribe(SystemId::ShipManager, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::PhysicsEngine, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::PhysicsEngine, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::TimeManager, events::EventType::PlayerCommand);
        
        Ok(Self {
            event_bus,
            planet_manager: PlanetManager::new(),
            ship_manager: ShipManager::new(),
            faction_manager: FactionManager::new(),
            time_manager: TimeManager::new(),
            resource_system: ResourceSystem::new(),
            population_system: PopulationSystem::new(),
            construction_system: ConstructionSystem::new(),
            physics_engine: PhysicsEngine::new(),
            combat_resolver: CombatResolver::new(),
            ui_renderer: UIRenderer::new(),
        })
    }
    
    pub fn fixed_update(&mut self, delta: f32) -> GameResult<()> {
        // Strict update order per architecture
        self.ui_renderer.process_input(&mut self.event_bus)?;
        self.physics_engine.update(delta, &mut self.event_bus)?;
        self.resource_system.update(delta, &mut self.event_bus)?;
        self.population_system.update(delta, &mut self.event_bus)?;
        self.construction_system.update(delta, &mut self.event_bus)?;
        self.combat_resolver.update(delta, &mut self.event_bus)?;
        self.time_manager.update(delta, &mut self.event_bus)?;
        
        // Process all queued events - temporarily move event_bus to avoid borrow conflicts
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        event_bus.process_events(self)?;
        self.event_bus = event_bus;
        
        Ok(())
    }
    
    pub fn render(&mut self, interpolation: f32) -> GameResult<()> {
        // Temporarily move ui_renderer to avoid borrow conflicts
        let mut ui_renderer = std::mem::replace(&mut self.ui_renderer, UIRenderer::new());
        ui_renderer.render(self, interpolation)?;
        self.ui_renderer = ui_renderer;
        Ok(())
    }
}

// System trait definition
pub trait GameSystem {
    fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>;
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>;
}