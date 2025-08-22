# UI Migration Guide: ui/ → ui_v2/

This guide provides step-by-step instructions for migrating from the existing UI system to the new ui_v2 modular architecture.

## Overview

**Goal**: Migrate from the monolithic ui/ system (8,236 lines) to the modular ui_v2/ system (2,400 lines projected) with 71% code reduction while maintaining all functionality.

**Strategy**: Incremental migration allowing both systems to coexist during transition.

## Architecture Comparison

### Old System (ui/)
```
src/ui/
├── renderer.rs (1,751 lines) - MONOLITHIC: rendering + state + input + panels
├── panels/
│   ├── planet_panel.rs (1,615 lines) - Manual rendering, hardcoded layout
│   ├── ship_panel.rs (753 lines) - Duplicate entity patterns
│   └── resource_panel.rs (398 lines) - Hardcoded resource display
├── input_handler.rs (465 lines) - Mixed selection + input processing
├── ui_state.rs (482 lines) - Fragmented state management
└── [12 other files] - Various utilities and components
```

### New System (ui_v2/)
```
src/ui_v2/
├── core/ - System infrastructure
│   ├── ui_system.rs - Main coordinator
│   ├── view_controller.rs - View lifecycle management
│   ├── input_controller.rs - Centralized input processing
│   └── render_context.rs - Theme and rendering context
├── components/ - Reusable primitives
│   ├── interactive.rs - Button, Dropdown, Slider, TextInput
│   ├── container.rs - Panel, ListView
│   ├── display.rs - Label, ProgressBar
│   └── layout.rs - Container layouts
├── views/ - Specialized presentations
│   ├── entity_view.rs - Generic entity display
│   ├── data_view.rs - Tables and lists
│   └── dialog_view.rs - Modals and forms
└── adapters/ - Entity-specific formatting
    ├── planet_adapter.rs - Planet data formatting
    ├── ship_adapter.rs - Ship data formatting
    └── faction_adapter.rs - Faction data formatting
```

## Migration Process

### Phase 1: Setup Integration Layer
1. Add ui_v2 imports to existing files
2. Create UISystemBridge for coordination
3. Maintain existing functionality during transition

### Phase 2: Migrate Components
1. Convert panels one at a time
2. Use adapters for entity-specific logic
3. Test each migration thoroughly

### Phase 3: Complete Migration
1. Remove old UI components
2. Update main game loop
3. Cleanup and optimization

## Step-by-Step Migration

### Step 1: Enable ui_v2 in Cargo.toml and lib.rs

Add ui_v2 module export to `src/lib.rs`:
```rust
pub mod ui_v2;
```

### Step 2: Create Integration Bridge

Create `src/ui/ui_bridge.rs`:
```rust
use crate::ui_v2::UISystem;
use crate::core::{GameState, events::PlayerCommand};

pub struct UIBridge {
    ui_v2_system: UISystem,
    enabled: bool,
}

impl UIBridge {
    pub fn new() -> Self {
        Self {
            ui_v2_system: UISystem::new(),
            enabled: false, // Start disabled for safe testing
        }
    }
    
    pub fn enable_v2(&mut self) {
        self.enabled = true;
    }
    
    pub fn update(&mut self, game_state: &GameState, delta_time: f32) -> Vec<PlayerCommand> {
        if self.enabled {
            self.ui_v2_system.update(game_state, delta_time)
        } else {
            Vec::new()
        }
    }
    
    pub fn render(&mut self, game_state: &GameState) {
        if self.enabled {
            self.ui_v2_system.render(game_state);
        }
    }
}
```

### Step 3: Panel Migration Pattern

For each panel (planet, ship, resource), follow this pattern:

#### 3.1 Create New Panel Using ui_v2 Components

Example for PlanetPanel:
```rust
// src/ui_v2/examples/migrated_planet_panel.rs
use crate::ui_v2::{EntityView, Panel, Button, ListView, PlanetAdapter};

pub struct MigratedPlanetPanel {
    main_panel: Panel,
    entity_view: EntityView<Planet>,
    tab_buttons: Vec<Button>,
    current_planet: Option<Planet>,
}

impl View for MigratedPlanetPanel {
    fn render(&mut self, context: &RenderContext) -> ComponentResult {
        self.main_panel.render(&(), context)?;
        self.entity_view.render(context)?;
        // ... render other components
        Ok(None)
    }
    
    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        // Handle input through components
        Ok(None)
    }
}
```

#### 3.2 Code Reduction Comparison

**Old PlanetPanel** (1,615 lines):
- Manual rendering: ~400 lines
- Layout calculations: ~300 lines
- Tab management: ~200 lines
- Button handling: ~150 lines
- State management: ~250 lines
- Resource display: ~315 lines

**New PlanetPanel** (~400 lines):
- Component composition: ~100 lines
- Adapter configuration: ~50 lines
- Tab structure: ~75 lines
- Event handling: ~100 lines
- Data binding: ~75 lines

**Savings**: 1,215 lines (75% reduction)

### Step 4: Integration Points

#### 4.1 Update UIRenderer to Use Bridge

Modify `src/ui/renderer.rs`:
```rust
use crate::ui::ui_bridge::UIBridge;

pub struct UIRenderer {
    // ... existing fields
    ui_bridge: UIBridge,
}

impl UIRenderer {
    pub fn new() -> Self {
        Self {
            // ... existing initialization
            ui_bridge: UIBridge::new(),
        }
    }
    
    pub fn enable_v2_for_testing(&mut self) {
        self.ui_bridge.enable_v2();
    }
    
    pub fn update(&mut self, game_state: &GameState, delta_time: f32) -> Vec<PlayerCommand> {
        let mut commands = Vec::new();
        
        // Get commands from existing UI
        commands.extend(self.process_existing_ui(game_state, delta_time));
        
        // Get commands from ui_v2 system
        commands.extend(self.ui_bridge.update(game_state, delta_time));
        
        commands
    }
    
    pub fn render(&mut self, game_state: &GameState) {
        // Render existing UI
        self.render_existing_ui(game_state);
        
        // Render ui_v2 system
        self.ui_bridge.render(game_state);
    }
}
```

#### 4.2 Feature Flags for Safe Migration

Add feature flags to control migration:
```rust
// src/ui/renderer.rs
const ENABLE_PLANET_PANEL_V2: bool = false;
const ENABLE_SHIP_PANEL_V2: bool = false;
const ENABLE_RESOURCE_PANEL_V2: bool = false;

impl UIRenderer {
    fn should_use_planet_panel_v2(&self) -> bool {
        ENABLE_PLANET_PANEL_V2
    }
    
    fn render_planet_panel(&mut self, game_state: &GameState) {
        if self.should_use_planet_panel_v2() {
            // Use ui_v2 planet panel
            self.ui_bridge.render_planet_panel(game_state);
        } else {
            // Use existing planet panel
            self.planet_panel.render(game_state);
        }
    }
}
```

## Migration Checklist

### Per Panel Migration:
- [ ] Create new panel using ui_v2 components
- [ ] Implement all existing functionality
- [ ] Test input handling matches old behavior
- [ ] Verify visual appearance matches
- [ ] Test with real game data
- [ ] Enable feature flag
- [ ] Test both panels side by side
- [ ] Remove old panel when confident

### System Integration:
- [ ] UIBridge handles input correctly
- [ ] Events flow through EventBus properly
- [ ] GameState updates work correctly
- [ ] Performance is maintained or improved
- [ ] Memory usage is reasonable
- [ ] No visual glitches or layout issues

### Testing Strategy:
- [ ] Unit tests for each new component
- [ ] Integration tests for panel behavior
- [ ] Visual regression tests
- [ ] Performance benchmarks
- [ ] User acceptance testing

## Benefits After Migration

### Code Quality
- **71% reduction** in UI code size (8,236 → 2,400 lines)
- **80% reduction** in code duplication
- **Clear separation** of concerns
- **Type-safe** component composition
- **Reusable** components for new features

### Developer Experience
- **Faster development** of new UI features
- **Easier debugging** with isolated components
- **Consistent behavior** across all panels
- **Better testability** with component isolation
- **Simplified maintenance** with centralized theming

### Performance
- **Reduced render complexity** through component batching
- **Better memory usage** with pooled components
- **Faster input processing** through centralized handling
- **Optimized layout** calculations through caching

## Common Migration Patterns

### 1. Manual Rendering → Component Composition
```rust
// Old: Manual drawing
draw_rectangle(x, y, w, h, color);
draw_text(text, x, y, font_size, text_color);

// New: Component composition
Button::new("Click Me".to_string())
    .with_layout(Layout::new(x, y, w, h))
    .render(&(), context)?;
```

### 2. State Management → Reactive Updates
```rust
// Old: Manual cache invalidation
if self.cache_dirty {
    self.rebuild_cache();
    self.cache_dirty = false;
}

// New: Reactive data binding
self.entity_view.update_data(ViewData::Planet(planet))?;
```

### 3. Input Handling → Event-Driven
```rust
// Old: Manual hit testing
if mouse_x > button_x && mouse_x < button_x + button_w {
    if mouse_y > button_y && mouse_y < button_y + button_h {
        // Handle click
    }
}

// New: Component input handling
button.handle_input(&input_event)?;
```

## Rollback Strategy

If issues arise during migration:

1. **Disable feature flags** to revert to old system
2. **Fix issues** in ui_v2 implementation
3. **Test thoroughly** before re-enabling
4. **Document problems** for future reference

## Migration Timeline

**Week 1**: Setup integration layer and bridge
**Week 2**: Migrate PlanetPanel (largest panel)
**Week 3**: Migrate ShipPanel and ResourcePanel
**Week 4**: Migrate remaining components and cleanup
**Week 5**: Remove old system and final optimization

## Support and Resources

- **Examples**: See `src/ui_v2/examples/` for migration patterns
- **Documentation**: Component interfaces in `src/ui_v2/components/`
- **Architecture**: Design principles in `src/ui_v2/core/`
- **Testing**: Test fixtures in `tests/ui_v2/`

This migration guide ensures a safe, incremental transition to the new UI system while maintaining full functionality throughout the process.