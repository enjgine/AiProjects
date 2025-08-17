# Stellar Dominion File Structure Guide

## Directory Layout
```
stellar-dominion/
├── Cargo.toml                    # Dependencies (DO NOT MODIFY)
├── src/
│   ├── main.rs                   # Entry point (DO NOT MODIFY)
│   ├── lib.rs                    # Module exports
│   │
│   ├── core/                     # CORE ARCHITECTURE (DO NOT MODIFY)
│   │   ├── mod.rs                # GameState, EventBus ownership
│   │   ├── events.rs             # Event definitions
│   │   └── types.rs              # Shared types (Planet, Ship, etc.)
│   │
│   ├── managers/                 # DATA OWNERS (Implement here)
│   │   ├── mod.rs                # Export all managers
│   │   ├── planet_manager.rs    # → PlanetManager implementation
│   │   ├── ship_manager.rs      # → ShipManager implementation
│   │   └── faction_manager.rs   # → FactionManager implementation
│   │
│   ├── systems/                  # SIMULATION LOGIC (Implement here)
│   │   ├── mod.rs                # Export all systems
│   │   ├── time_manager.rs      # → TimeManager implementation
│   │   ├── physics_engine.rs    # → PhysicsEngine implementation
│   │   ├── resource_system.rs   # → ResourceSystem implementation
│   │   ├── population_system.rs # → PopulationSystem implementation
│   │   ├── construction.rs      # → ConstructionSystem implementation
│   │   ├── combat_resolver.rs   # → CombatResolver implementation
│   │   └── save_system.rs       # → SaveSystem implementation
│   │
│   └── ui/                       # RENDERING (Implement here)
│       ├── mod.rs                # Export UI components
│       ├── renderer.rs           # → UIRenderer implementation
│       ├── input_handler.rs     # → Input to PlayerCommand conversion
│       └── panels/               # → UI panel implementations
│           ├── planet_panel.rs  
│           ├── ship_panel.rs    
│           └── resource_panel.rs
│
└── tests/
    ├── architecture_invariants.rs # Architecture validation (DO NOT MODIFY)
    ├── integration_tests.rs      # System integration tests
    └── systems/                  # Unit tests per system
        ├── physics_test.rs       # → Add your system tests here
        ├── resources_test.rs     
        └── ...
```

## Implementation Rules by Directory

### ❌ DO NOT MODIFY - `/src/core/`
Core architecture files. These define the EventBus, base types, and event definitions. All systems must use these types without modification.

### ✅ IMPLEMENT - `/src/managers/`
**Who implements:** PlanetManager, ShipManager, FactionManager specialists

**What goes here:**
- Data ownership structures (Vec<Planet>, Vec<Ship>)
- CRUD methods returning `GameResult<T>`
- State validation logic
- ID generation and indexing

**Example structure:**
```rust
// planet_manager.rs
pub struct PlanetManager {
    planets: Vec<Planet>,        // Owns the data
    next_id: PlanetId,
    planet_index: HashMap<PlanetId, usize>,
}

impl PlanetManager {
    pub fn add_resources(&mut self, id: PlanetId, resources: ResourceBundle) -> GameResult<()>
    pub fn get_planet(&self, id: PlanetId) -> GameResult<&Planet>
    pub fn update_population(&mut self, id: PlanetId, amount: i32) -> GameResult<()>
}
```

### ✅ IMPLEMENT - `/src/systems/`
**Who implements:** System specialists per the 10 prompts

**What goes here:**
- Simulation logic and calculations
- Event subscription handlers
- Update tick processing
- NO data ownership (reference managers only)

**Example structure:**
```rust
// resource_system.rs
pub struct ResourceSystem {
    production_rates: HashMap<BuildingType, ResourceBundle>,
    // NO Vec<Planet> here - that's in PlanetManager
}

impl ResourceSystem {
    pub fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>
}
```

### ✅ IMPLEMENT - `/src/ui/`
**Who implements:** UIRenderer specialist

**What goes here:**
- Immediate mode rendering with macroquad
- Input processing → PlayerCommand conversion
- Visual representation only
- NO game logic or state mutation

**Example structure:**
```rust
// renderer.rs
pub struct UIRenderer {
    selected_planet: Option<PlanetId>,  // UI state only
    camera_position: Vector2,
}

impl UIRenderer {
    pub fn render(&mut self, state: &GameState, interpolation: f32) -> GameResult<()>
    pub fn process_input(&mut self, events: &mut EventBus) -> GameResult<()>
}
```

## File Creation Checklist

When implementing your system:

1. **Create your file** in the correct directory:
   - Manager? → `/src/managers/your_manager.rs`
   - System? → `/src/systems/your_system.rs`
   - UI? → `/src/ui/your_component.rs`

2. **Add module export** to the directory's `mod.rs`:
   ```rust
   // In /src/systems/mod.rs
   pub mod your_system;
   pub use your_system::YourSystem;
   ```

3. **Import only from core**:
   ```rust
   use crate::core::{GameResult, GameEvent, EventBus};
   use crate::core::types::*;
   ```

4. **Create test file**:
   - Unit tests: `/tests/systems/your_system_test.rs`
   - Use `#[cfg(test)]` for inline tests

5. **Wire into GameState** (only if creating manager):
   ```rust
   // In /src/core/mod.rs GameState::new()
   your_system: YourSystem::new(),
   ```

## Import Hierarchy

```
↓ Can import from ↓         ✗ Cannot import from ✗

main.rs
  ↓ core, managers, systems, ui

systems/*
  ↓ core                    ✗ other systems, ui

managers/*  
  ↓ core                    ✗ systems, other managers, ui

ui/*
  ↓ core                    ✗ systems, managers (read via GameState ref only)

core/*
  ↓ std only                ✗ any game modules
```

## Quick Reference

| Task | Location | Imports | Owns Data | Emits Events |
|------|----------|---------|-----------|--------------|
| Store planets | `/src/managers/planet_manager.rs` | `core::*` | YES | NO |
| Calculate orbits | `/src/systems/physics_engine.rs` | `core::*` | NO | YES |
| Render UI | `/src/ui/renderer.rs` | `core::*` | NO | YES (Commands) |
| Define events | `/src/core/events.rs` | Already done | - | - |
| Game loop | `/src/main.rs` | Already done | - | - |

## Integration Points

Each system connects at exactly these points:

1. **Event subscription** in `GameState::new()`
2. **Update call** in `GameState::fixed_update()` 
3. **Manager access** through `&mut self` references in update
4. **Event emission** through `events.queue_event()`

No other connection points allowed.