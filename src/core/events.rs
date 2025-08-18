// src/core/events.rs
use super::types::*;
use super::GameSystem;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerCommand(PlayerCommand),
    SimulationEvent(SimulationEvent),
    StateChanged(StateChange),
}

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    SelectPlanet(PlanetId),
    SelectShip(ShipId),
    BuildStructure { planet: PlanetId, building_type: BuildingType },
    MoveShip { ship: ShipId, target: Vector2 },
    TransferResources { from: PlanetId, to: PlanetId, resources: ResourceBundle },
    AllocateWorkers { planet: PlanetId, allocation: WorkerAllocation },
    ConstructShip { planet: PlanetId, ship_class: ShipClass },
    AttackTarget { attacker: ShipId, target: ShipId },
    ColonizePlanet { ship: ShipId, planet: PlanetId },
    LoadShipCargo { ship: ShipId, planet: PlanetId, resources: ResourceBundle },
    UnloadShipCargo { ship: ShipId, planet: PlanetId },
    SetGameSpeed(f32),
    PauseGame(bool),
    SaveGame,
    LoadGame,
}

#[derive(Debug, Clone)]
pub enum SimulationEvent {
    TickCompleted(u64),
    ResourcesProduced { planet: PlanetId, resources: ResourceBundle },
    PopulationGrowth { planet: PlanetId, amount: i32 },
    ConstructionCompleted { planet: PlanetId, building: BuildingType },
    ShipCompleted { planet: PlanetId, ship: ShipId },
    ShipArrived { ship: ShipId, destination: Vector2 },
    CombatResolved { attacker: ShipId, defender: ShipId, outcome: CombatOutcome },
    PlanetConquered { planet: PlanetId, new_owner: FactionId },
    ResourceShortage { planet: PlanetId, resource: ResourceType },
    TransferWindowOpen { from: PlanetId, to: PlanetId },
}

#[derive(Debug, Clone)]
pub enum StateChange {
    PlanetUpdated(PlanetId),
    ShipUpdated(ShipId),
    FactionUpdated(FactionId),
    VictoryConditionMet(VictoryType),
    GameOver(FactionId),
    GameLoaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    PlayerCommand,
    SimulationEvent,
    StateChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemId {
    PlanetManager,
    ShipManager,
    FactionManager,
    TimeManager,
    ResourceSystem,
    PopulationSystem,
    ConstructionSystem,
    PhysicsEngine,
    CombatResolver,
    SaveSystem,
    UIRenderer,
}

pub struct EventBus {
    pub queued_events: VecDeque<GameEvent>,
    pub subscribers: HashMap<SystemId, Vec<EventType>>,
    pub event_history: VecDeque<GameEvent>,
    history_limit: usize,
    update_order: Vec<SystemId>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            queued_events: VecDeque::with_capacity(256),
            subscribers: HashMap::with_capacity(16),
            event_history: VecDeque::with_capacity(100),
            history_limit: 100,
            update_order: vec![
                SystemId::UIRenderer,
                SystemId::PhysicsEngine,
                SystemId::ResourceSystem,
                SystemId::PopulationSystem,
                SystemId::ConstructionSystem,
                SystemId::CombatResolver,
                SystemId::TimeManager,
            ],
        }
    }
    
    pub fn subscribe(&mut self, system: SystemId, event_type: EventType) {
        self.subscribers
            .entry(system)
            .or_insert_with(|| Vec::with_capacity(4))
            .push(event_type);
    }
    
    pub fn queue_event(&mut self, event: GameEvent) {
        self.event_history.push_back(event.clone());
        if self.event_history.len() > self.history_limit {
            self.event_history.pop_front();
        }
        
        self.queued_events.push_back(event);
    }
    
    pub fn process_events(&mut self, state: &mut super::GameState) -> GameResult<()> {
        while let Some(event) = self.queued_events.pop_front() {
            let event_type = match &event {
                GameEvent::PlayerCommand(_) => EventType::PlayerCommand,
                GameEvent::SimulationEvent(_) => EventType::SimulationEvent,
                GameEvent::StateChanged(_) => EventType::StateChanged,
            };
            
            // Route to subscribed systems in update order
            for &system_id in &self.update_order {
                if let Some(subscriptions) = self.subscribers.get(&system_id) {
                    if subscriptions.contains(&event_type) {
                        self.route_to_system(system_id, &event, state)?;
                    }
                }
            }
            
            // Handle managers (no specific order required)
            for (&system_id, subscriptions) in &self.subscribers {
                if !self.update_order.contains(&system_id) && subscriptions.contains(&event_type) {
                    self.route_to_system(system_id, &event, state)?;
                }
            }
        }
        Ok(())
    }
    
    fn route_to_system(&self, system_id: SystemId, event: &GameEvent, state: &mut super::GameState) -> GameResult<()> {
        match system_id {
            SystemId::TimeManager => state.time_manager.handle_event(event),
            SystemId::PlanetManager => state.planet_manager.handle_event(event),
            SystemId::ShipManager => state.ship_manager.handle_event(event),
            SystemId::FactionManager => state.faction_manager.handle_event(event),
            SystemId::PhysicsEngine => state.physics_engine.handle_event(event),
            SystemId::ResourceSystem => state.resource_system.handle_event(event),
            SystemId::PopulationSystem => state.population_system.handle_event(event),
            SystemId::ConstructionSystem => state.construction_system.handle_event(event),
            SystemId::CombatResolver => state.combat_resolver.handle_event(event),
            SystemId::SaveSystem => state.save_system.handle_event(event),
            SystemId::UIRenderer => state.ui_renderer.handle_event(event),
        }
    }
    
    pub fn clear(&mut self) {
        self.queued_events.clear();
    }
}