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
    UIRenderer,
}

pub struct EventBus {
    pub queued_events: VecDeque<GameEvent>,
    pub subscribers: HashMap<SystemId, Vec<EventType>>,
    pub event_history: VecDeque<GameEvent>,
    history_limit: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            queued_events: VecDeque::new(),
            subscribers: HashMap::new(),
            event_history: VecDeque::new(),
            history_limit: 100,
        }
    }
    
    pub fn subscribe(&mut self, system: SystemId, event_type: EventType) {
        self.subscribers
            .entry(system)
            .or_insert_with(Vec::new)
            .push(event_type);
    }
    
    pub fn queue_event(&mut self, event: GameEvent) {
        self.queued_events.push_back(event.clone());
        
        // Maintain history for debugging
        self.event_history.push_back(event);
        if self.event_history.len() > self.history_limit {
            self.event_history.pop_front();
        }
    }
    
    pub fn process_events(&mut self, state: &mut super::GameState) -> GameResult<()> {
        while let Some(event) = self.queued_events.pop_front() {
            let event_type = match &event {
                GameEvent::PlayerCommand(_) => EventType::PlayerCommand,
                GameEvent::SimulationEvent(_) => EventType::SimulationEvent,
                GameEvent::StateChanged(_) => EventType::StateChanged,
            };
            
            // Route to subscribed systems
            for (system, subscriptions) in &self.subscribers {
                if subscriptions.contains(&event_type) {
                    match system {
                        SystemId::TimeManager => {
                            state.time_manager.handle_event(&event)?;
                        }
                        SystemId::PlanetManager => {
                            state.planet_manager.handle_event(&event)?;
                        }
                        SystemId::PhysicsEngine => {
                            state.physics_engine.handle_event(&event)?;
                        }
                        _ => {
                            // Other systems will be handled when implemented
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.queued_events.clear();
    }
}