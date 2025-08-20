# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
Read integration_guide.md and structure.md. Read and adhere to claude_working_memory.md.

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

# Quality checks (run before committing)
cargo clippy -- -D warnings
cargo fmt --check
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

### Manager Pattern (src/managers/)
- Own data collections (Vec<Planet>, Vec<Ship>)
- Provide CRUD methods returning `GameResult<T>`
- No direct field access from other systems
- Validate all operations before state changes

### System Pattern (src/systems/) 
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
- All new systems must have unit tests (>95% coverage for business logic)
- Integration tests verify event flow
- Property-based tests for deterministic behavior
- Performance tests for entity count targets
- No Arc/Mutex allowed (single-threaded design)

## Current Implementation Status

**Complete & Operational:**
- Core architecture and EventBus
- All managers (Planet, Ship, Faction)
- All core systems (Resource, Population, Construction, Time, Physics, Combat)
- Save/Load system with deterministic validation
- UI rendering with interactive panels
- Comprehensive test suite (55+ tests)
- Architecture validation and compliance

**Current Focus:**
- Performance optimization and scaling
- Advanced testing patterns
- Production readiness features

## File Organization

```
src/
├── main.rs           # Game loop, fixed timestep (DO NOT MODIFY)
├── core/             # Architecture (DO NOT MODIFY)
│   ├── mod.rs       # GameState, managers
│   ├── events.rs    # Event definitions  
│   └── types.rs     # Shared types
├── managers/         # Data owners (IMPLEMENTED)
├── systems/          # Simulation logic (IMPLEMENTED)
└── ui/              # Rendering (IMPLEMENTED)
```

## Implementation Guidance

When modifying or extending systems:
1. Reference `integration_guide_renew.md` for comprehensive patterns
2. Use only `core::*` imports
3. Return `GameResult<T>` from all operations
4. Emit events for state changes
5. Run architecture tests to verify compliance
6. Follow code review checklist in integration guide
7. Maintain deterministic behavior for save/load compatibility

## Key Integration Points

- TimeManager emits `TickCompleted` events driving all systems
- PlanetManager provides resource/population data to systems
- ShipManager handles movement and cargo
- EventBus routes commands to appropriate systems
- UI generates only PlayerCommand events
- SaveSystem ensures deterministic state preservation

All systems must integrate through these defined interfaces only.

## Quality Standards

### Performance Targets
- **Current Phase**: 30+ FPS with 20 planets, 100 ships
- **System Updates**: <10ms per system per tick
- **Memory**: Bounded collections, no memory leaks

### Code Quality
- All manager operations return `GameResult<T>`
- No `unsafe` code without justification
- Comprehensive error handling
- Full rustdoc documentation for public APIs

### Development Workflow
1. Create feature branch from main
2. Implement with tests (use `test_fixtures` for consistency)
3. Run full test suite: `cargo test`
4. Run quality checks: `cargo clippy && cargo fmt`
5. Verify architecture compliance
6. Create PR with detailed description

## Advanced Patterns

For complex implementations, consult `integration_guide_renew.md` for:
- Event aggregation and optimization
- Manager scaling strategies
- Testing patterns (property-based, integration)
- Error handling and recovery
- Performance monitoring

This guide covers essential patterns. See integration guide for comprehensive scaling patterns.