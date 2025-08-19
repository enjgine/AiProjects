# Test Coverage Expansion Recommendations

Based on comprehensive analysis of all test files, here are prioritized recommendations for expanding test coverage to more rigorously validate the Stellar Dominion codebase:

## Priority 1: Architecture Enforcement (tests/architecture_invariants.rs)

- Add system update order validation tests (UI→Physics→Resources→Population→Construction→Combat→Time)
- Implement compile-time verification that all public system methods return GameResult<T>
- Add tests to verify systems cannot directly access manager fields, only through methods
- Test event routing respects EventBus subscriptions
- Verify event processing is atomic within each tick
- Add fixed timestep determinism validation (same inputs = identical outputs)
- Test resource overflow protection for i32 boundaries
- Implement comprehensive simulation cycle testing with multiple systems

## Priority 2: Cross-System Integration (tests/integration_tests.rs)

- Add complete resource production cycle testing (TickCompleted → ResourceSystem → ResourcesProduced → PlanetManager)
- Implement ship colonization workflow tests with failure scenarios (distance, ownership, population)
- Test construction chain integration with resource validation and timing
- Add combat resolution flow testing with damage calculations and cleanup
- Verify EventBus ordering with conflicting commands
- Test cross-manager consistency during complex multi-system operations
- Add realistic game scenario integration beyond simple CRUD operations

## Priority 3: Physics Engine Rigor (tests/physics_engine_test.rs)

- Add ship trajectory interpolation and arrival detection tests
- Implement transfer window calculation and planetary alignment validation
- Test fuel cost calculations with varying ship parameters
- Add collision detection for overlapping trajectories
- Test performance with maximum ship/planet counts
- Add orbital mechanics edge cases (extreme periods, phase offsets)
- Implement concurrent ship arrivals and departures testing
- Test orbital position updates affecting active trajectories

## Priority 4: Planet Management Completeness (tests/planet_manager_test.rs)

- Add resource subtraction edge cases (insufficient funds, exact amounts)
- Implement building management tests (types, counts, constraints)
- Test event system integration (GameSystem implementation)
- Add storage capacity upgrade validation
- Test complex state validation with multiple constraint violations
- Implement multi-planet operations and faction filtering
- Test worker reallocation on population changes
- Validate building slot formula accuracy (10 + population/10000)

## Priority 5: Save System Robustness (tests/save_system_integration.rs)

- Add version compatibility testing for save file migrations
- Implement corruption recovery scenarios (truncated files, checksum mismatches)
- Test large-scale data handling (1000+ entities)
- Add edge case validation (Unicode names, i32::MAX resources, invalid trajectories)
- Test concurrent access and file locking scenarios
- Implement atomic save operation interruption testing
- Add filesystem permission error handling
- Test performance degradation with complex game states

## Priority 6: Temporal Consistency (tests/time_manager_integration.rs)

- Add system update order enforcement validation
- Implement tick event processing sequence testing
- Test cross-system temporal consistency across multiple ticks
- Add delta spike handling for large time jumps
- Test event queue overflow protection under high load
- Implement save/load temporal consistency validation
- Add sustained operation stress testing
- Test accumulated_time reset to prevent drift

## Implementation Strategy

1. **Start with Architecture Enforcement** - These tests catch fundamental violations that break the EventBus pattern
2. **Focus on Integration Testing** - Validate complete workflows before diving into component details  
3. **Add Edge Case Coverage** - Test boundary conditions and error paths thoroughly
4. **Performance and Stress Testing** - Ensure the system scales to realistic game sizes
5. **Robustness Testing** - Validate error recovery and data integrity scenarios

Each test category builds upon the previous ones, creating a comprehensive validation suite that ensures the Stellar Dominion architecture remains intact as the codebase evolves.