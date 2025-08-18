// src/core/mod.rs
pub mod events;
pub mod types;

// Re-export commonly used types
pub use events::{EventBus, GameEvent, SystemId, PlayerCommand, SimulationEvent, StateChange};
pub use types::*;
use std::collections::VecDeque;

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
    
    pub fn process_tick_production(&mut self) -> GameResult<()> {
        // Handle resource production during tick processing
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        self.resource_system.process_production(&mut self.planet_manager, &mut event_bus)?;
        self.event_bus = event_bus;
        Ok(())
    }
    
    pub fn process_resource_transfer(&mut self, from: PlanetId, to: PlanetId, resources: ResourceBundle) -> GameResult<()> {
        // Handle resource transfers
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        self.resource_system.process_transfer(from, to, resources, &mut self.planet_manager, &mut event_bus)?;
        self.event_bus = event_bus;
        Ok(())
    }
    
    pub fn process_ship_cargo_loading(&mut self, ship_id: ShipId, planet_id: PlanetId, resources: ResourceBundle) -> GameResult<()> {
        // Handle ship cargo loading
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        self.resource_system.process_ship_loading(ship_id, planet_id, resources, 
                                                 &mut self.planet_manager, 
                                                 &mut self.ship_manager, 
                                                 &mut event_bus)?;
        self.event_bus = event_bus;
        Ok(())
    }
    
    pub fn process_ship_cargo_unloading(&mut self, ship_id: ShipId, planet_id: PlanetId) -> GameResult<()> {
        // Handle ship cargo unloading
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        self.resource_system.process_ship_unloading(ship_id, planet_id,
                                                   &mut self.planet_manager,
                                                   &mut self.ship_manager,
                                                   &mut event_bus)?;
        self.event_bus = event_bus;
        Ok(())
    }
    
    pub fn process_population_growth(&mut self, tick: u64) -> GameResult<()> {
        // Handle population growth during tick processing
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        self.population_system.process_growth(tick, &mut self.planet_manager, &mut event_bus)?;
        self.event_bus = event_bus;
        Ok(())
    }
    
    pub fn process_worker_allocation(&mut self, planet_id: PlanetId, allocation: WorkerAllocation) -> GameResult<()> {
        // Handle worker allocation changes
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        self.population_system.process_allocation(planet_id, allocation, &mut self.planet_manager, &mut event_bus)?;
        self.event_bus = event_bus;
        Ok(())
    }
    
    pub fn process_population_migration(&mut self, ship_id: ShipId) -> GameResult<()> {
        // Handle population migration when transport ships arrive
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        self.population_system.process_migration(ship_id, &mut self.planet_manager, &mut self.ship_manager, &mut event_bus)?;
        self.event_bus = event_bus;
        Ok(())
    }
    
    pub fn process_combat_resolution(&mut self, attacker: ShipId, defender: ShipId) -> GameResult<()> {
        // Handle detailed combat resolution with access to ships and planets
        let mut event_bus = std::mem::replace(&mut self.event_bus, EventBus::new());
        
        // Get ship information for combat calculations
        let attacker_ship = self.ship_manager.get_ship(attacker)?;
        let defender_ship = self.ship_manager.get_ship(defender)?;
        
        // Calculate combat strengths
        let attacker_strength = self.combat_resolver.get_combat_modifier(attacker_ship.owner) * 
            match attacker_ship.ship_class {
                ShipClass::Scout => 1.0,
                ShipClass::Transport => 0.5,
                ShipClass::Warship => 5.0,
                ShipClass::Colony => 0.1,
            };
            
        let defender_strength = self.combat_resolver.get_combat_modifier(defender_ship.owner) * 
            match defender_ship.ship_class {
                ShipClass::Scout => 1.0,
                ShipClass::Transport => 0.5,
                ShipClass::Warship => 5.0,
                ShipClass::Colony => 0.1,
            };
        
        // Determine outcome using CombatResolver logic
        let attacker_wins = attacker_strength >= defender_strength * 1.5;
        
        let mut attacker_losses = Vec::new();
        let mut defender_losses = Vec::new();
        
        if attacker_wins {
            // Attacker wins: 30% attacker losses, 50% defender losses
            // For simplicity, destroy the defender ship
            defender_losses.push(defender);
        } else {
            // Defender wins: 50% attacker losses, 30% defender losses
            // For simplicity, destroy the attacker ship
            attacker_losses.push(attacker);
        }
        
        let outcome = CombatOutcome {
            winner: if attacker_wins { attacker_ship.owner } else { defender_ship.owner },
            attacker_losses,
            defender_losses,
        };
        
        // Emit the combat resolved event
        event_bus.queue_event(GameEvent::SimulationEvent(
            SimulationEvent::CombatResolved {
                attacker,
                defender,
                outcome,
            }
        ));
        
        self.event_bus = event_bus;
        Ok(())
    }
    
    pub fn process_save_game(&mut self) -> GameResult<()> {
        // Save the current game state
        self.save_system.save_game(self)?;
        Ok(())
    }
    
    pub fn process_load_game(&mut self) -> GameResult<()> {
        // Load game state from save file
        let save_data = self.save_system.load_game()?;
        
        // Restore the complete game state
        self.time_manager.set_tick(save_data.tick);
        self.planet_manager.load_planets(save_data.planets)?;
        self.ship_manager.load_ships(save_data.ships)?;
        self.faction_manager.load_factions(save_data.factions)?;
        
        // Emit GameLoaded event
        self.event_bus.queue_event(GameEvent::StateChanged(StateChange::GameLoaded));
        
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