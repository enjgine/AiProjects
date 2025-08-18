# Stellar Dominion Project - Sonnet 4 Token Cost Analysis

## Overview
This analysis calculates the estimated Sonnet 4 token cost for each file in the project using the approximation: **1 token ≈ 4 characters**.

---

## Root Configuration Files
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `Cargo.toml` | 186 | 47 |
| `Cargo.lock` | 8,929 | 2,232 |
| **Subtotal** | **9,115** | **2,279** |

---

## Documentation Files
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `CLAUDE.md` | 3,728 | 932 |
| `readme.md` | 16 | 4 |
| `integration_guide.md` | 6,876 | 1,719 |
| `structure.md` | 6,284 | 1,571 |
| `system_implement_prompts.md` | 10,632 | 2,658 |
| `claude_recommendations.md` | 5,123 | 1,281 |
| **Subtotal** | **32,659** | **8,165** |

---

## Source Code Files

### Core Module (`src/core/`)
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `src/lib.rs` | 107 | 27 |
| `src/main.rs` | 1,133 | 283 |
| `src/core/mod.rs` | 6,083 | 1,521 |
| `src/core/events.rs` | 5,231 | 1,308 |
| `src/core/types.rs` | 6,522 | 1,631 |
| **Subtotal** | **19,076** | **4,770** |

### Managers Module (`src/managers/`)
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `src/managers/mod.rs` | 206 | 52 |
| `src/managers/faction_manager.rs` | 1,887 | 472 |
| `src/managers/planet_manager.rs` | 7,491 | 1,873 |
| `src/managers/ship_manager.rs` | 10,415 | 2,604 |
| **Subtotal** | **19,999** | **5,001** |

### Systems Module (`src/systems/`)
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `src/systems/mod.rs` | 397 | 99 |
| `src/systems/combat_resolver.rs` | 7,236 | 1,809 |
| `src/systems/construction.rs` | 2,693 | 673 |
| `src/systems/physics_engine.rs` | 5,406 | 1,352 |
| `src/systems/population_system.rs` | 5,373 | 1,343 |
| `src/systems/resource_system.rs` | 8,635 | 2,159 |
| `src/systems/save_system.rs` | 29,524 | 7,381 |
| `src/systems/time_manager.rs` | 1,735 | 434 |
| **Subtotal** | **60,999** | **15,250** |

### UI Module (`src/ui/`)
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `src/ui/mod.rs` | 158 | 40 |
| `src/ui/input_handler.rs` | 2,801 | 700 |
| `src/ui/renderer.rs` | 19,694 | 4,924 |
| `src/ui/panels/mod.rs` | 194 | 49 |
| `src/ui/panels/planet_panel.rs` | 3,829 | 957 |
| `src/ui/panels/resource_panel.rs` | 4,128 | 1,032 |
| `src/ui/panels/ship_panel.rs` | 3,903 | 976 |
| **Subtotal** | **34,707** | **8,678** |

**Total Source Code:** | **134,781** | **33,699** |

---

## Test Files

### Main Test Files (`tests/`)
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `tests/architecture_invariants.rs` | 5,006 | 1,252 |
| `tests/integration_tests.rs` | 6,041 | 1,510 |
| `tests/physics_engine_test.rs` | 2,950 | 738 |
| `tests/save_system_integration.rs` | 3,324 | 831 |
| `tests/time_manager_integration.rs` | 4,184 | 1,046 |
| **Subtotal** | **21,505** | **5,377** |

### System Test Files (`tests/systems/`)
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `tests/systems/physics_test.rs` | 3,093 | 773 |
| `tests/systems/population_test.rs` | 4,430 | 1,108 |
| `tests/systems/resources_test.rs` | 10,821 | 2,705 |
| `tests/systems/time_manager_test.rs` | 4,523 | 1,131 |
| `tests/systems/ui_renderer_test.rs` | 6,507 | 1,627 |
| **Subtotal** | **29,374** | **7,344** |

**Total Test Files:** | **50,879** | **12,721** |

---

## Data Files
| File | Characters | Estimated Tokens |
|------|------------|------------------|
| `stellar_dominion_save.txt` | 1,264 | 316 |
| `libtest_planet_manager.rmeta` | 1 | 1 |
| **Subtotal** | **1,265** | **317** |

---

## Project Summary

### By Directory
| Directory | Characters | Estimated Tokens | Percentage |
|-----------|------------|------------------|------------|
| **Root Configuration** | 9,115 | 2,279 | 4.0% |
| **Documentation** | 32,659 | 8,165 | 14.4% |
| **Source Code** | 134,781 | 33,699 | 59.4% |
| **Test Files** | 50,879 | 12,721 | 22.4% |
| **Data Files** | 1,265 | 317 | 0.6% |

### By Source Module
| Module | Characters | Estimated Tokens | Percentage of Source |
|--------|------------|------------------|----------------------|
| **Core** | 19,076 | 4,770 | 14.2% |
| **Managers** | 19,999 | 5,001 | 14.8% |
| **Systems** | 60,999 | 15,250 | 45.2% |
| **UI** | 34,707 | 8,678 | 25.8% |

### Total Project Cost
| Metric | Value |
|--------|-------|
| **Total Characters** | **228,699** |
| **Total Estimated Tokens** | **57,181** |
| **Total Files Analyzed** | **42** |

---

## Top 10 Most Expensive Files
| Rank | File | Tokens | % of Total |
|------|------|---------|------------|
| 1 | `src/systems/save_system.rs` | 7,381 | 12.9% |
| 2 | `src/ui/renderer.rs` | 4,924 | 8.6% |
| 3 | `tests/systems/resources_test.rs` | 2,705 | 4.7% |
| 4 | `system_implement_prompts.md` | 2,658 | 4.6% |
| 5 | `src/managers/ship_manager.rs` | 2,604 | 4.6% |
| 6 | `Cargo.lock` | 2,232 | 3.9% |
| 7 | `src/systems/resource_system.rs` | 2,159 | 3.8% |
| 8 | `src/managers/planet_manager.rs` | 1,873 | 3.3% |
| 9 | `src/systems/combat_resolver.rs` | 1,809 | 3.2% |
| 10 | `integration_guide.md` | 1,719 | 3.0% |

---

## Analysis Notes

1. **Largest Component**: The Systems module contains the most complex logic, representing 45.2% of the source code tokens.

2. **Save System Dominance**: The save system alone accounts for 12.9% of the total project tokens, indicating complex serialization logic.

3. **Test Coverage**: Test files represent 22.4% of the total project, showing good test coverage.

4. **Documentation Heavy**: Documentation files account for 14.4% of tokens, indicating well-documented project structure.

5. **Build Dependencies**: Cargo.lock represents significant token cost due to extensive dependency tree for the macroquad graphics library.

*Generated on 2025-08-18 - Token calculations based on Sonnet 4 approximation of 1 token per 4 characters*