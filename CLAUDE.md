# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
Make requests of the user in the form of singular dot points (i.e. "- Install package X", or "- update rust") via claude_recommendations.md
Read integration_guide.md and structure.md

## Project Overview

**Stellar Dominion** - A real-time space empire simulation game built in Rust using macroquad. The project implements a strict EventBus architecture with fixed timestep simulation for deterministic gameplay.

## Build & Test Commands

```bash
# Build project
cargo build --release

# Run tests (including architecture validation)
cargo test
cargo test --test architecture_invariants

# Run the game
cargo run
```

## Core Architecture - DO NOT MODIFY

### EventBus Communication
- All systems communicate exclusively through events
- No direct system-to-system references allowed
- Fixed update order: UI → Physics → Resources → Population → Construction → Combat → Time
- All state mutations return `GameResult<T>`

### Core Modules (src/core/)
- `mod.rs` - GameState, manager ownership, EventBus subscriptions
- `events.rs` - Event definitions (PlayerCommand, SimulationEvent, StateChange)
- `types.rs` - Shared types (Planet, Ship, Resources, etc.)

### Fixed Timestep
- 0.1 second timesteps for deterministic simulation
- Tick counter (u64) drives all systems
- Interpolated rendering for smooth visuals

## Implementation Rules

### Manager Pattern (src/managers/ - Future)
- Own data collections (Vec<Planet>, Vec<Ship>)
- Provide CRUD methods returning `GameResult<T>`
- No direct field access from other systems
- Validate all operations before state changes

### System Pattern (src/systems/ - Future) 
- Subscribe to relevant events via EventBus
- Process logic in `update()` method
- Emit new events, never modify state directly
- Reference managers through GameState only

### Resource Constraints
- All resources are `i32` (no floating point)
- Resources cannot go negative (enforced by `ResourceBundle::subtract()`)
- Worker allocation must equal total population
- Building slots = 10 + population/10000

### Testing Requirements
- Architecture invariants enforced by tests/architecture_invariants.rs
- All new systems must have unit tests
- Integration tests verify event flow
- No Arc/Mutex allowed (single-threaded design)

## Current Implementation Status

**Complete:**
- Core architecture and EventBus
- Basic managers (Planet, Ship, Faction) 
- TimeManager with tick events
- Type system and error handling
- Architecture validation tests
- Main game loop with fixed timestep

**Placeholder Systems (Need Implementation):**
- ResourceSystem
- PopulationSystem  
- ConstructionSystem
- PhysicsEngine
- CombatResolver
- UIRenderer

## File Organization

```
src/
├── main.rs           # Game loop, fixed timestep (DO NOT MODIFY)
├── core/             # Architecture (DO NOT MODIFY)
│   ├── mod.rs       # GameState, managers
│   ├── events.rs    # Event definitions  
│   └── types.rs     # Shared types
├── managers/         # Data owners (Future implementation)
├── systems/          # Simulation logic (Future implementation)
└── ui/              # Rendering (Future implementation)
```

## Implementation Guidance

When implementing systems:
1. Follow exact specifications in `system_implement_prompts.md`
2. Use only `core::*` imports
3. Return `GameResult<T>` from all operations
4. Emit events for state changes
5. Run architecture tests to verify compliance

## Key Integration Points

- TimeManager emits `TickCompleted` events driving all systems
- PlanetManager provides resource/population data to systems
- ShipManager handles movement and cargo
- EventBus routes commands to appropriate systems
- UI generates only PlayerCommand events

All systems must integrate through these defined interfaces only.