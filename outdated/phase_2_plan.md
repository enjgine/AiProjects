# Phase 2: Menu-Driven UI Refactor Plan

## Overview
Transform Stellar Dominion from a 3D space-focused interface to a menu-driven interface with permanent toolbar and list-based entity management.

## Current State Analysis
- Game currently requires 3D space interaction for entity selection
- Keyboard shortcuts (P, B, S) require pre-selected entities to function
- UI panels only open when entities are selected in 3D space
- No persistent menu or toolbar interface

## Phase 2 Goals
1. **Permanent Toolbar**: Always-visible top toolbar with menu options
2. **List-Based Navigation**: Planets and ships accessible via clickable lists
3. **Panel Independence**: Management panels open without requiring 3D selection
4. **Improved UX**: Intuitive menu-driven workflow for all game features

---

## Implementation Progress

### ✅ COMPLETED: Core Infrastructure (Steps 1-2)
- **Toolbar Component**: ✅ Created `src/ui/toolbar.rs` with permanent top bar
- **List Menu System**: ✅ Created `src/ui/list_menus.rs` with planet/ship dropdowns
- **Integration**: ✅ Added new components to UIRenderer with proper event handling
- **Compilation**: ✅ All code compiles successfully with only minor warnings

### ✅ COMPLETED: Basic Menu Functionality  
- **Clickable Toolbar**: ✅ Planets, Ships, Resources, Build, Settings buttons
- **Dropdown Lists**: ✅ Planet and ship lists with filtering options
- **Auto-close**: ✅ Menus close when clicking outside
- **Event Integration**: ✅ Menu selections emit proper PlayerCommand events

### 🔄 IN PROGRESS: Panel Integration (Step 3-4)
- **Panel Positioning**: ✅ Adjusted resource panel to avoid toolbar overlap
- **Menu-Panel Sync**: ✅ Toolbar state syncs with legacy panel toggles
- **Entity Selection**: ✅ List menu selections properly update UIRenderer state

### ✅ COMPLETED: Enhanced Features & Testing
- **Enhanced List Features**: ✅ Added detailed planet/ship info with resources, population, and cargo
- **Visual Polish**: ✅ Improved toolbar styling with highlights and better feedback
- **Keyboard Shortcuts**: ✅ Added F1/F2 for direct menu access and Tab cycling
- **Testing**: ✅ Complete Phase 2 integration test suite (6 tests passing)
- **Documentation**: ✅ Updated help text with new Phase 2 controls

### 🎉 PHASE 2 IMPLEMENTATION COMPLETE!

**All major goals achieved:**
- ✅ Permanent toolbar with always-visible menu buttons
- ✅ Dropdown planet and ship lists with rich information display
- ✅ Menu-driven interface that doesn't require 3D space selection
- ✅ Enhanced keyboard shortcuts (F1, F2, Tab, R)
- ✅ Comprehensive testing with 6 integration tests
- ✅ Full EventBus architecture compliance maintained

---

## Implementation Plan

### 1. UI Architecture Analysis & Refactoring Points
- **File**: `src/ui/renderer.rs`
  - Current panel system uses `ui_context` with boolean toggles
  - Panels require `selected_planet`/`selected_ship` to display content
  - Need to decouple panel visibility from entity selection

- **File**: `src/ui/input_handler.rs` 
  - Currently handles 3D space clicking for entity selection
  - Need to maintain 3D interaction alongside menu system
  - Add toolbar interaction handling

### 2. New Menu System Design

#### 2.1 Toolbar Component
```rust
pub struct Toolbar {
    pub height: f32,
    pub planets_menu_open: bool,
    pub ships_menu_open: bool,
    pub resources_panel_open: bool,
    pub construction_panel_open: bool,
    pub settings_menu_open: bool,
}
```

#### 2.2 List Menu Components
```rust
pub struct PlanetListMenu {
    pub open: bool,
    pub selected_planet: Option<PlanetId>,
    pub scroll_offset: f32,
    pub filter_owned_only: bool,
}

pub struct ShipListMenu {
    pub open: bool,
    pub selected_ship: Option<ShipId>,
    pub scroll_offset: f32,
    pub filter_owned_only: bool,
    pub group_by_class: bool,
}
```

### 3. Implementation Steps

#### Step 1: Create Toolbar Infrastructure
- Add `Toolbar` struct to UI system
- Implement top-bar rendering with menu buttons
- Add click detection for toolbar buttons
- Reserve screen space for permanent toolbar

#### Step 2: Design List Menu System
- Create `PlanetListMenu` and `ShipListMenu` components
- Implement dropdown/sidebar list rendering
- Add entity filtering and sorting options
- Create clickable list item interface

#### Step 3: Refactor Panel System
- Modify existing panels to work without required selection
- Add "Select from List" buttons to panels when no entity selected
- Maintain current functionality while adding menu-driven access
- Update panel positioning to account for toolbar space

#### Step 4: Integration & Event Handling
- Add toolbar button events to `PlayerCommand` enum
- Update event routing for menu interactions
- Maintain existing 3D selection alongside menu system
- Add keyboard shortcuts for menu navigation

#### Step 5: Enhanced List Functionality
- **Planet List Features**:
  - Show planet name, owner, population, resources summary
  - Filter by ownership (Player/Neutral/Enemy)
  - Sort by distance, population, resource wealth
  - Click to select and open management panel
  
- **Ship List Features**:
  - Show ship name, class, location, status
  - Group by ship class or fleet
  - Filter by ownership and status
  - Click to select and open management panel

### 4. UI Layout Changes

#### New Screen Layout:
```
┌─────────────────────────────────────────────────┐
│ [Planets▼] [Ships▼] [Resources] [Build] [Menu] │ ← Toolbar (always visible)
├─────────────────────────────────────────────────┤
│ ┌─Planet List──┐                                │
│ │ • Planet A   │ 3D Space View                  │
│ │ • Planet B   │ (with orbiting planets/ships)  │
│ │ • Planet C   │                                │
│ └──────────────┘                                │
│                  ┌─Planet Management Panel─────┐ │
│                  │ Planet A Details            │ │
│                  │ Population: 1000            │ │
│                  │ Resources: [...]            │ │
│                  │ Buildings: [...]            │ │
│                  └─────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### 5. Testing Strategy

#### 5.1 Unit Tests
- Test toolbar component rendering and interaction
- Test list menu functionality (filtering, sorting, selection)
- Test panel independence from 3D selection
- Test event routing for menu interactions

#### 5.2 Integration Tests
- Test complete workflow: toolbar → list → panel
- Test interaction between 3D selection and menu system
- Test keyboard shortcuts with new menu system
- Test resource panel accessibility via toolbar

#### 5.3 User Experience Tests
- Verify all game features accessible via menus
- Test intuitive navigation flow
- Validate panel management without 3D interaction required
- Ensure 3D view remains functional alongside menus

### 6. Technical Implementation Details

#### 6.1 File Modifications Required
- `src/ui/renderer.rs`: Major refactor for toolbar and list menus
- `src/ui/input_handler.rs`: Add toolbar click handling
- `src/core/events.rs`: Add menu-specific PlayerCommand variants
- `src/core/mod.rs`: Update event routing for menu commands

#### 6.2 New Files to Create
- `src/ui/toolbar.rs`: Toolbar component implementation
- `src/ui/list_menus.rs`: Planet and ship list menu components
- `src/ui/layout.rs`: Screen layout management with toolbar
- `tests/ui_integration_tests.rs`: UI system integration tests

#### 6.3 Architectural Considerations
- Maintain EventBus architecture for all menu interactions
- Ensure menu state persists across game updates
- Keep 3D interaction as optional enhancement to menu-driven workflow
- Design for future expansion (diplomacy menus, research trees, etc.)

---

## Success Criteria

### Phase 2 Complete When:
1. ✅ Permanent toolbar visible at top of screen
2. ✅ Planets accessible via dropdown list with clickable entries
3. ✅ Ships accessible via dropdown list with clickable entries  
4. ✅ All management panels openable via menu (no 3D selection required)
5. ✅ Resource panel accessible via toolbar button
6. ✅ Construction/building features accessible via toolbar
7. ✅ 3D view remains functional for visual reference
8. ✅ All existing keyboard shortcuts still work
9. ✅ Comprehensive test coverage for new UI system
10. ✅ User can complete all game actions via menu-driven interface

### Timeline Estimate
- **Step 1-2**: Toolbar & List Infrastructure (2-3 implementation sessions)
- **Step 3-4**: Panel Refactor & Integration (2-3 implementation sessions) 
- **Step 5**: Enhanced List Features (1-2 implementation sessions)
- **Testing & Validation**: (1 implementation session)
- **Total**: 6-9 implementation sessions

---

## Implementation Priority
1. **High Priority**: Toolbar creation, basic list menus
2. **Medium Priority**: Panel refactoring, enhanced list features
3. **Low Priority**: Advanced filtering, keyboard navigation, visual polish

This plan transforms Stellar Dominion into a true menu-driven strategy game while maintaining the visual appeal of the 3D space view.