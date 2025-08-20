// src/core/events.rs
use super::types::*;
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
    // Menu-related commands
    NewGame,
    ExitGame,
    BackToMenu,
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
    pub update_order: Vec<SystemId>,
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
    
    
    pub fn clear(&mut self) {
        self.queued_events.clear();
    }
    
    /// Processes queued events through GameState - used for testing.
    /// This allows tests to manually trigger event processing.
    #[cfg(test)]
    pub fn process_events(&mut self, game_state: &mut crate::core::GameState) -> crate::core::types::GameResult<()> {
        game_state.process_queued_events_for_test()
    }
}