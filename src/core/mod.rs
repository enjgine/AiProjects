// src/core/mod.rs
pub mod events;
pub mod types;

use events::{EventBus, GameEvent, SystemId};
use types::*;
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
        event_bus.subscribe(SystemId::PlanetManager, events::EventType::StateChanged);
        event_bus.subscribe(SystemId::ShipManager, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::PhysicsEngine, events::EventType::SimulationEvent);
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
        
        // Process all queued events
        self.event_bus.process_events(self)?;
        
        Ok(())
    }
    
    pub fn render(&mut self, interpolation: f32) -> GameResult<()> {
        self.ui_renderer.render(self, interpolation)?;
        Ok(())
    }
}

// System trait definition
pub trait GameSystem {
    fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>;
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>;
}