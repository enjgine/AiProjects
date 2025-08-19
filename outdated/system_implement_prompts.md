# Stellar Dominion System Implementation Prompts

## 1. TimeManager Implementation Task

### Responsibility
Manages game tick progression, pause state, and speed control. Emits tick events that drive all simulation systems. Maintains deterministic tick counter for save/load consistency.

### Required Structures
```rust
pub struct TimeManager {
    tick: u64,
    paused: bool,
    speed_multiplier: f32,
    accumulated_time: f32,
    tick_duration: f32, // 0.1 seconds
}
```

### Event Interface
Subscribes to: PlayerCommand::SetGameSpeed, PlayerCommand::PauseGame
Emits: SimulationEvent::TickCompleted(tick)

### Integration Points
- Reads from: None
- Modifies through: Internal state only, broadcasts tick events

### Validation Tests
1. Verify tick events emit exactly once per 0.1s at 1x speed
2. Confirm no tick events when paused
3. Test speed multiplier correctly scales tick rate

### Prohibited
No direct time manipulation outside fixed timestep. No floating-point tick counters. No non-deterministic timing.

---

## 2. PlanetManager Implementation Task

### Responsibility
Owns all planet entities and manages their lifecycle. Validates resource storage limits and development slot availability. Processes planet-targeted commands and state changes.

### Required Structures
```rust
pub struct PlanetManager {
    planets: Vec<Planet>,
    next_id: PlanetId,
    planet_index: HashMap<PlanetId, usize>,
}
```

### Event Interface
Subscribes to: PlayerCommand::BuildStructure, SimulationEvent::ConstructionCompleted, StateChange::PlanetUpdated
Emits: StateChange::PlanetUpdated, SimulationEvent::ResourceShortage

### Integration Points
- Reads from: None (owns planet data)
- Modifies through: add_resources(), remove_resources(), add_building() methods

### Validation Tests
1. Resource operations fail when exceeding storage capacity
2. Building placement respects slot limits (10 + pop/10000)
3. Planet IDs remain unique and sequential

### Prohibited
Direct planet field modification. Resource values below zero. Buildings without available slots.

---

## 3. PhysicsEngine Implementation Task

### Responsibility
Calculates orbital positions each tick and manages ship trajectories. Determines transfer windows between planets and validates movement commands against fuel availability.

### Required Structures
```rust
pub struct PhysicsEngine {
    orbital_cache: Vec<(PlanetId, Vector2)>,
    transfer_windows: HashMap<(PlanetId, PlanetId), u64>,
    trajectories: HashMap<ShipId, Trajectory>,
}
```

### Event Interface
Subscribes to: SimulationEvent::TickCompleted, PlayerCommand::MoveShip
Emits: SimulationEvent::ShipArrived, SimulationEvent::TransferWindowOpen

### Integration Points
- Reads from: PlanetManager (positions), ShipManager (ship data)
- Modifies through: ShipManager::update_position()

### Validation Tests
1. Orbital positions follow circular paths at correct periods
2. Transfer windows occur at predictable intervals
3. Fuel costs scale correctly with/without windows

### Prohibited
Non-circular orbits. Random trajectory variations. Position updates outside tick events.

---

## 4. ResourceSystem Implementation Task

### Responsibility
Processes production chains each tick based on worker allocation. Validates consumption before production. Enforces storage limits and processes resource transfers between planets.

### Required Structures
```rust
pub struct ResourceSystem {
    production_rates: HashMap<BuildingType, ResourceBundle>,
    consumption_tracking: HashMap<PlanetId, ResourceBundle>,
}
```

### Event Interface
Subscribes to: SimulationEvent::TickCompleted, PlayerCommand::TransferResources
Emits: SimulationEvent::ResourcesProduced, SimulationEvent::ResourceShortage

### Integration Points
- Reads from: PlanetManager (workers, buildings)
- Modifies through: PlanetManager::add_resources(), PlanetManager::remove_resources()

### Validation Tests
1. Production = Workers × Efficiency × Timestep
2. Negative resources trigger shortage events
3. Transfers validate source availability before execution

### Prohibited
Floating-point resource values. Production without workers. Storage overflow without events.

---

## 5. PopulationSystem Implementation Task

### Responsibility
Calculates population growth/decline based on food availability. Validates worker allocation changes. Processes migration between planets via transport ships.

### Required Structures
```rust
pub struct PopulationSystem {
    growth_modifiers: HashMap<PlanetId, f32>,
    migration_queue: Vec<MigrationOrder>,
}
```

### Event Interface
Subscribes to: SimulationEvent::TickCompleted, PlayerCommand::AllocateWorkers, SimulationEvent::ShipArrived
Emits: SimulationEvent::PopulationGrowth, StateChange::PlanetUpdated

### Integration Points
- Reads from: PlanetManager (population, food)
- Modifies through: PlanetManager::update_population(), PlanetManager::set_allocation()

### Validation Tests
1. Growth +2%/tick with food surplus >20%
2. Worker allocation sum equals total population
3. Minimum 10% unassigned workers maintained

### Prohibited
Population below zero. Allocation exceeding total. Growth without food surplus.

---

## 6. ConstructionSystem Implementation Task

### Responsibility
Manages build queues for structures and ships. Deducts resources at construction start. Tracks progress and emits completion events.

### Required Structures
```rust
pub struct ConstructionSystem {
    building_queue: HashMap<PlanetId, Vec<ConstructionOrder>>,
    ship_queue: HashMap<PlanetId, Vec<ShipOrder>>,
    construction_costs: HashMap<BuildingType, (ResourceBundle, u64)>,
}
```

### Event Interface
Subscribes to: PlayerCommand::BuildStructure, PlayerCommand::ConstructShip, SimulationEvent::TickCompleted
Emits: SimulationEvent::ConstructionCompleted, SimulationEvent::ShipCompleted

### Integration Points
- Reads from: PlanetManager (resources, spaceport)
- Modifies through: PlanetManager::remove_resources(), ShipManager::create_ship()

### Validation Tests
1. Resources deducted immediately on queue entry
2. Construction time scales with building tier
3. Ships require operational spaceport

### Prohibited
Refunds on cancellation. Parallel construction without multiple spaceports. Progress without resources paid.

---

## 7. ShipManager Implementation Task

### Responsibility
Owns all ship entities and manages movement execution. Tracks fuel consumption and cargo capacity. Validates colonization and transport operations.

### Required Structures
```rust
pub struct ShipManager {
    ships: Vec<Ship>,
    next_id: ShipId,
    ship_index: HashMap<ShipId, usize>,
}
```

### Event Interface
Subscribes to: PlayerCommand::MoveShip, SimulationEvent::ShipCompleted, SimulationEvent::CombatResolved
Emits: StateChange::ShipUpdated, SimulationEvent::ShipArrived

### Integration Points
- Reads from: PhysicsEngine (trajectories)
- Modifies through: Internal ship state, fuel consumption

### Validation Tests
1. Fuel consumption = Speed × Distance / 100
2. Cargo capacity enforced on loading
3. Colony ships consumed on planet colonization

### Prohibited
Movement without fuel. Cargo exceeding capacity. Ships with negative fuel.

---

## 8. CombatResolver Implementation Task

### Responsibility
Resolves fleet battles through deterministic strength comparison. Processes planetary invasions with defender bonuses. Applies losses and conquest outcomes.

### Required Structures
```rust
pub struct CombatResolver {
    active_battles: Vec<Battle>,
    combat_modifiers: HashMap<FactionId, f32>,
}
```

### Event Interface
Subscribes to: PlayerCommand::AttackTarget, SimulationEvent::ShipArrived
Emits: SimulationEvent::CombatResolved, SimulationEvent::PlanetConquered

### Integration Points
- Reads from: ShipManager (combat strength), PlanetManager (defenses)
- Modifies through: ShipManager::destroy_ship(), PlanetManager::change_controller()

### Validation Tests
1. Attacker needs 1.5× strength to win space battle
2. Planetary defense multiplies defender strength by 2
3. Losses: Attacker 30%, Defender 50% on defeat

### Prohibited
Random combat outcomes. Strength modifications mid-battle. Survivors below zero.

---

## 9. UIRenderer Implementation Task

### Responsibility
Renders immediate-mode UI panels showing game state. Converts player input into PlayerCommand events. Never modifies game state directly.

### Required Structures
```rust
pub struct UIRenderer {
    selected_planet: Option<PlanetId>,
    selected_ship: Option<ShipId>,
    camera_position: Vector2,
    ui_scale: f32,
}
```

### Event Interface
Subscribes to: StateChange::PlanetUpdated, StateChange::ShipUpdated
Emits: All PlayerCommand variants based on input

### Integration Points
- Reads from: All managers (display only)
- Modifies through: None (emit commands only)

### Validation Tests
1. Click detection correctly maps to game entities
2. Commands validate before emission
3. No direct state mutation from UI code

### Prohibited
Game logic in render functions. State mutation without events. UI state persisting between frames.

---

## 10. SaveSystem Implementation Task

### Responsibility
Serializes complete game state deterministically. Restores state maintaining exact tick synchronization. Validates save file integrity before loading.

### Required Structures
```rust
pub struct SaveSystem {
    version: u32,
    compression: bool,
}

pub struct SaveData {
    tick: u64,
    planets: Vec<Planet>,
    ships: Vec<Ship>,
    factions: Vec<Faction>,
    checksum: u64,
}
```

### Event Interface
Subscribes to: PlayerCommand::SaveGame, PlayerCommand::LoadGame
Emits: StateChange::GameLoaded

### Integration Points
- Reads from: All managers (serialization)
- Modifies through: Complete state replacement on load

### Validation Tests
1. Save/load cycle produces identical state
2. Checksum detects corrupted saves
3. Version mismatch handled gracefully

### Prohibited
Non-deterministic serialization order. Partial state saves. Loading without validation.