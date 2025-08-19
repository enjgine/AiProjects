// src/core/mod.rs
pub mod events;
pub mod types;

// Re-export commonly used types
pub use events::{EventBus, GameEvent, SystemId, PlayerCommand, SimulationEvent, StateChange};
pub use types::*;

// Import managers and systems
use crate::managers::{PlanetManager, ShipManager, FactionManager};
use crate::systems::{TimeManager, ResourceSystem, PopulationSystem, ConstructionSystem, PhysicsEngine, CombatResolver, SaveSystem};
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
    pub save_system: SaveSystem,
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
        event_bus.subscribe(SystemId::ResourceSystem, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::ResourceSystem, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::PopulationSystem, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::PopulationSystem, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::CombatResolver, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::CombatResolver, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::SaveSystem, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::UIRenderer, events::EventType::StateChanged);
        
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
            save_system: SaveSystem::new(),
            ui_renderer: UIRenderer::new(),
        })
    }
    
    pub fn fixed_update(&mut self, delta: f32) -> GameResult<()> {
        // Process input first
        self.ui_renderer.process_input(&mut self.event_bus)?;
        
        // Update systems in strict order per architecture
        self.physics_engine.update(delta, &mut self.event_bus)?;
        self.resource_system.update(delta, &mut self.event_bus)?;
        self.population_system.update(delta, &mut self.event_bus)?;
        self.construction_system.update(delta, &mut self.event_bus)?;
        self.combat_resolver.update(delta, &mut self.event_bus)?;
        self.time_manager.update(delta, &mut self.event_bus)?;
        
        // Process all queued events after system updates
        self.process_queued_events()?;
        
        Ok(())
    }
    
    fn process_queued_events(&mut self) -> GameResult<()> {
        // Process events while maintaining architectural boundaries
        let events_to_process: Vec<GameEvent> = self.event_bus.queued_events.drain(..).collect();
        
        for event in events_to_process {
            self.route_event_to_systems(event)?;
        }
        
        Ok(())
    }
    
    fn route_event_to_systems(&mut self, event: GameEvent) -> GameResult<()> {
        let event_type = match &event {
            GameEvent::PlayerCommand(_) => events::EventType::PlayerCommand,
            GameEvent::SimulationEvent(_) => events::EventType::SimulationEvent,
            GameEvent::StateChanged(_) => events::EventType::StateChanged,
        };
        
        // First collect all systems that need to handle this event
        let mut systems_to_notify = Vec::new();
        
        // Add systems in update order
        for &system_id in &self.event_bus.update_order {
            if let Some(subscriptions) = self.event_bus.subscribers.get(&system_id) {
                if subscriptions.contains(&event_type) {
                    systems_to_notify.push(system_id);
                }
            }
        }
        
        // Add managers (no specific order)
        for (&system_id, subscriptions) in &self.event_bus.subscribers {
            if !self.event_bus.update_order.contains(&system_id) && subscriptions.contains(&event_type) {
                systems_to_notify.push(system_id);
            }
        }
        
        // Now notify all systems
        for system_id in systems_to_notify {
            self.handle_system_event(system_id, &event)?;
        }
        
        Ok(())
    }
    
    fn handle_system_event(&mut self, system_id: SystemId, event: &GameEvent) -> GameResult<()> {
        match system_id {
            SystemId::TimeManager => self.time_manager.handle_event(event),
            SystemId::PlanetManager => self.planet_manager.handle_event(event),
            SystemId::ShipManager => self.ship_manager.handle_event(event),
            SystemId::FactionManager => self.faction_manager.handle_event(event),
            SystemId::PhysicsEngine => self.physics_engine.handle_event(event),
            SystemId::ResourceSystem => self.resource_system.handle_event(event),
            SystemId::PopulationSystem => self.population_system.handle_event(event),
            SystemId::ConstructionSystem => self.construction_system.handle_event(event),
            SystemId::CombatResolver => self.combat_resolver.handle_event(event),
            SystemId::SaveSystem => {
                // Handle SaveSystem events specially since they need full GameState access
                if let GameEvent::PlayerCommand(cmd) = event {
                    match cmd {
                        PlayerCommand::SaveGame => self.handle_save_game_command(),
                        PlayerCommand::LoadGame => self.handle_load_game_command(),
                        _ => self.save_system.handle_event(event),
                    }
                } else {
                    self.save_system.handle_event(event)
                }
            },
            SystemId::UIRenderer => self.ui_renderer.handle_event(event),
        }
    }

    fn handle_save_game_command(&mut self) -> GameResult<()> {
        // Extract the save system temporarily to avoid borrow conflicts
        let save_system = std::mem::replace(&mut self.save_system, SaveSystem::new());
        let result = save_system.save_game(self);
        self.save_system = save_system;
        result
    }

    fn handle_load_game_command(&mut self) -> GameResult<()> {
        // Extract the save system temporarily to avoid borrow conflicts
        let save_system = std::mem::replace(&mut self.save_system, SaveSystem::new());
        let save_data = save_system.load_game()?;
        self.save_system = save_system;
        
        // Apply the loaded data to the game state
        self.time_manager.set_tick(save_data.tick)?;
        self.planet_manager.load_planets(save_data.planets)?;
        self.ship_manager.load_ships(save_data.ships)?;
        self.faction_manager.load_factions(save_data.factions)?;
        
        Ok(())
    }
    
    pub fn queue_event(&mut self, event: GameEvent) {
        self.event_bus.queue_event(event);
    }
    
    pub fn get_current_tick(&self) -> u64 {
        self.time_manager.get_current_tick()
    }
    
    pub fn save_game(&mut self) -> GameResult<()> {
        self.queue_event(GameEvent::PlayerCommand(PlayerCommand::SaveGame));
        Ok(())
    }
    
    pub fn load_game(&mut self) -> GameResult<()> {
        self.queue_event(GameEvent::PlayerCommand(PlayerCommand::LoadGame));
        Ok(())
    }
    
    pub fn render(&mut self, interpolation: f32) -> GameResult<()> {
        // Temporarily extract ui_renderer to avoid borrow conflicts
        let mut ui_renderer = std::mem::replace(&mut self.ui_renderer, UIRenderer::new());
        let result = ui_renderer.render(self, interpolation);
        self.ui_renderer = ui_renderer;
        result
    }
    
    /// Processes queued events manually - used for testing.
    /// This allows tests to trigger event processing without running fixed_update.
    /// Available in both unit tests and integration tests.
    pub fn process_queued_events_for_test(&mut self) -> GameResult<()> {
        self.process_queued_events()
    }
}

// System trait definition
pub trait GameSystem {
    fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>;
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>;
}