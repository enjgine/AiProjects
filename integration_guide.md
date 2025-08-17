# Stellar Dominion Integration Guide

## Architecture Overview

The game operates on a strict EventBus architecture where systems communicate exclusively through messages. This ensures complete decoupling and enables parallel development by multiple AI agents.

## Core Flow

```
User Input → UIRenderer → PlayerCommand Event
                ↓
            EventBus
                ↓
    System Subscriptions Process Events
                ↓
    Systems Emit New Events → EventBus
                ↓
        State Changes Propagate
```

## Integration Protocol

### Phase 1: Individual System Implementation
Each AI agent implements their assigned system independently:
1. Create system struct with required fields
2. Implement update() method processing tick events
3. Implement handle_event() for subscribed events
4. Write unit tests validating behavior

### Phase 2: Event Wiring
Connect systems through EventBus subscriptions:
```rust
// In GameState::new()
event_bus.subscribe(SystemId::ResourceSystem, EventType::SimulationEvent);
event_bus.subscribe(SystemId::PopulationSystem, EventType::SimulationEvent);
```

### Phase 3: Manager Integration
Systems access data through manager methods:
```rust
// ResourceSystem reads from PlanetManager
let planet = planet_manager.get_planet(id)?;
let workers = planet.population.allocation.mining;

// But modifies through methods
planet_manager.add_resources(id, produced)?;
```

### Phase 4: Validation
Run architecture tests to verify:
- No direct system references
- All operations return GameResult
- Events flow correctly
- State remains consistent

## Key Integration Points

### TimeManager → All Systems
- Emits `TickCompleted` driving simulation
- All systems subscribe to this for updates
- Tick counter ensures deterministic replay

### PlanetManager ← Multiple Systems
- ResourceSystem: Modifies resources via add/remove methods
- PopulationSystem: Updates population/allocation
- ConstructionSystem: Adds buildings when complete
- CombatResolver: Changes ownership

### ShipManager ← Multiple Systems
- PhysicsEngine: Updates positions
- ConstructionSystem: Creates new ships
- CombatResolver: Destroys ships in battle

### EventBus Message Patterns

**Command → Action → Notification:**
```rust
PlayerCommand::BuildStructure
    → ConstructionSystem processes
    → SimulationEvent::ConstructionCompleted
    → StateChange::PlanetUpdated
```

**Tick → Update → State Change:**
```rust
SimulationEvent::TickCompleted
    → ResourceSystem calculates production
    → SimulationEvent::ResourcesProduced
    → PlanetManager updates storage
```

## Testing Strategy

### Unit Tests (Per System)
- Input validation
- Event generation
- State transitions
- Error conditions

### Integration Tests
- Event flow between systems
- Manager state consistency
- Full update cycle
- Victory condition detection

### Architecture Tests
- No circular dependencies
- Event-only communication
- Deterministic operations
- Performance constraints

## Common Pitfalls to Avoid

1. **Direct System Access**
   ```rust
   // WRONG
   self.planet_manager.planets[0].resources.minerals += 100;
   
   // CORRECT
   self.planet_manager.add_resources(planet_id, resources)?;
   ```

2. **Skipping Events**
   ```rust
   // WRONG
   ship.position = new_position;
   
   // CORRECT
   event_bus.queue_event(GameEvent::SimulationEvent(
       SimulationEvent::ShipArrived { ship: id, destination: pos }
   ));
   ```

3. **Floating Resources**
   ```rust
   // WRONG
   let minerals: f32 = 100.5;
   
   // CORRECT
   let minerals: i32 = 100;
   ```

4. **Mutable Exposure**
   ```rust
   // WRONG
   pub fn get_planet_mut(&mut self) -> &mut Planet
   
   // CORRECT
   pub fn modify_planet<F>(&mut self, id: PlanetId, f: F) -> GameResult<()>
       where F: FnOnce(&mut Planet) -> GameResult<()>
   ```

## Performance Guidelines

- Pre-allocate collections: `Vec::with_capacity()`
- Use indices not HashMaps for primary storage
- Cache orbital calculations between ticks
- Batch event processing per frame
- Profile with 100 planets, 500 ships target

## Compilation Verification

After implementing each system:
```bash
cargo build --release
cargo test
cargo test --test architecture_invariants
```

## Module Structure

```
src/
├── main.rs           # Entry point, game loop
├── core/
│   ├── mod.rs       # GameState, managers
│   ├── events.rs    # Event definitions
│   └── types.rs     # Shared types
├── systems/
│   ├── mod.rs       # System trait
│   ├── physics.rs   # PhysicsEngine impl
│   ├── resources.rs # ResourceSystem impl
│   └── ...          # Other systems
├── entities/
│   ├── planet.rs    # Planet struct
│   ├── ship.rs      # Ship struct
│   └── faction.rs   # Faction struct
└── ui/
    └── renderer.rs  # UIRenderer impl
```

## Final Integration Checklist

- [ ] All systems compile independently
- [ ] EventBus subscriptions registered
- [ ] Manager methods return GameResult
- [ ] No direct field access between systems
- [ ] Tests pass for each system
- [ ] Integration tests pass
- [ ] 60 FPS with target entity counts
- [ ] Save/load maintains exact state
- [ ] Victory conditions detected
- [ ] UI generates only PlayerCommands

This architecture ensures clean boundaries enabling parallel development while maintaining system coherence through the EventBus.