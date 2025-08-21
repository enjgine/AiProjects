# Stellar Dominion File Structure & Function Map

## Complete Directory Layout

```
stellar-dominion/
├── Cargo.toml                          # Dependencies (DO NOT MODIFY)
├── src/
│   ├── main.rs                         # Entry point (DO NOT MODIFY)
│   ├── lib.rs                          # Module exports
│   │
│   ├── core/                           # CORE ARCHITECTURE (Enhanced)
│   │   ├── mod.rs                      # GameState, EventBus ownership
│   │   ├── events.rs                   # Event definitions
│   │   ├── types.rs                    # Shared types (Planet, Ship, etc.)
│   │   └── asset_types.rs              # Asset system types (50+ asset types)
│   │
│   ├── managers/                       # DATA OWNERS (Implemented)
│   │   ├── mod.rs                      # Export all managers
│   │   ├── planet_manager.rs           # PlanetManager implementation
│   │   ├── ship_manager.rs             # ShipManager implementation
│   │   └── faction_manager.rs          # FactionManager implementation
│   │
│   ├── systems/                        # SIMULATION LOGIC (Enhanced)
│   │   ├── mod.rs                      # Export all systems
│   │   ├── time_manager.rs             # TimeManager implementation
│   │   ├── physics_engine.rs           # PhysicsEngine implementation
│   │   ├── resource_system.rs          # ResourceSystem implementation
│   │   ├── population_system.rs        # PopulationSystem implementation
│   │   ├── construction.rs             # ConstructionSystem implementation
│   │   ├── combat_resolver.rs          # CombatResolver implementation
│   │   ├── save_system.rs              # SaveSystem (binary, scalable, asset-aware)
│   │   └── game_initializer.rs         # GameInitializer for configurable new games
│   │
│   └── ui/                             # RENDERING (Implemented)
│       ├── mod.rs                      # Export UI components
│       ├── renderer.rs                 # UIRenderer implementation
│       ├── input_handler.rs            # Input to PlayerCommand conversion
│       ├── camera.rs                   # Camera system
│       ├── toolbar.rs                  # Toolbar UI component
│       ├── list_menus.rs               # List menu components
│       ├── ui_state.rs                 # UI state management
│       ├── start_menu.rs               # StartMenu component for main menu
│       ├── save_load_dialog.rs         # Save/Load dialog system
│       └── panels/                     # UI panel implementations
│           ├── mod.rs                  # Panel exports
│           ├── planet_panel.rs         # Planet information panel
│           ├── ship_panel.rs           # Ship information panel
│           └── resource_panel.rs       # Resource display panel
│
└── tests/
    ├── architecture_invariants.rs      # Architecture validation (DO NOT MODIFY)
    ├── dialog_state_test.rs            # Dialog state management, race condition, and event handling tests
    ├── integration_tests.rs            # System integration tests
    ├── phase2_integration_test.rs      # Phase 2 integration validation
    ├── physics_engine_test.rs          # Physics engine unit tests
    ├── planet_manager_test.rs          # Planet manager unit tests
    ├── save_system_test.rs             # Save system comprehensive tests
    ├── time_manager_integration.rs     # Time manager integration tests
    └── systems/                        # Unit tests per system
        ├── physics_test.rs             # Physics system tests
        ├── population_test.rs          # Population system tests
        ├── resources_test.rs           # Resource system tests
        ├── time_manager_test.rs        # Time manager tests
        └── ui_renderer_test.rs         # UI renderer tests
```

## Module & Function Map

### Core Architecture (`src/core/`) - DO NOT MODIFY

#### `mod.rs` - Game State & EventBus
- `GameState` - Central game state container
  - Contains StartMenu component, SaveLoadDialog, and current_mode field for mode switching
  - Contains GameInitializer for configurable new game creation
  - `pub fn new() -> GameResult<Self>` - Initializes in MainMenu mode
  - `pub fn fixed_update(&mut self, delta: f32) -> GameResult<()>` - Handles both menu and game updates
  - `pub fn queue_event(&mut self, event: GameEvent)`
  - `pub fn get_current_tick(&self) -> u64`
  - `pub fn save_game(&mut self) -> GameResult<()>`
  - `pub fn load_game(&mut self) -> GameResult<()>`
  - `pub fn render(&mut self, interpolation: f32) -> GameResult<()>` - Mode-aware rendering
  - `pub fn process_queued_events_for_test(&mut self) -> GameResult<()>`
  - Named save/load support with dialog integration
- `EventBus` - Event routing system
  - `pub fn new() -> Self`
  - `pub fn subscribe(&mut self, system: SystemId, event_type: EventType)`
  - `pub fn queue_event(&mut self, event: GameEvent)`
  - `pub fn clear(&mut self)`
- `GameSystem` trait - Common system interface
  - `fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>`
  - `fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`

#### `events.rs` - Event Definitions
- `GameEvent` - Top-level event enum
  - `PlayerCommand(PlayerCommand)`
  - `SimulationEvent(SimulationEvent)`
  - `StateChanged(StateChange)`
- `PlayerCommand` - User input events
  - `SelectPlanet(PlanetId)`
  - `SelectShip(ShipId)`
  - `BuildStructure { planet: PlanetId, building_type: BuildingType }`
  - `MoveShip { ship: ShipId, target: Vector2 }`
  - `TransferResources { from: PlanetId, to: PlanetId, resources: ResourceBundle }`
  - `AllocateWorkers { planet: PlanetId, allocation: WorkerAllocation }`
  - `ConstructShip { planet: PlanetId, ship_class: ShipClass }`
  - `AttackTarget { attacker: ShipId, target: ShipId }`
  - `ColonizePlanet { ship: ShipId, planet: PlanetId }`
  - `LoadShipCargo { ship: ShipId, planet: PlanetId, resources: ResourceBundle }`
  - `UnloadShipCargo { ship: ShipId, planet: PlanetId }`
  - `SetGameSpeed(f32)`
  - `PauseGame(bool)`
  - `SaveGame`
  - `SaveGameAs(String)` - Save with custom name
  - `LoadGame`
  - `LoadGameFrom(String)` - Load specific named save
  - `NewGame` - Start a new game (menu command)
  - `NewGameNamed(String)` - Create new game with custom name
  - `ExitGame` - Exit the application (menu command)
  - `BackToMenu` - Return to main menu from in-game
  - `GameOptions` - Cycle through game configuration presets
- `SimulationEvent` - System-generated events
  - `TickCompleted(u64)`
  - `ResourcesProduced { planet: PlanetId, resources: ResourceBundle }`
  - `PopulationGrowth { planet: PlanetId, amount: i32 }`
  - `ConstructionCompleted { planet: PlanetId, building: BuildingType }`
  - `ShipCompleted { planet: PlanetId, ship: ShipId }`
  - `ShipArrived { ship: ShipId, destination: Vector2 }`
  - `CombatResolved { attacker: ShipId, defender: ShipId, outcome: CombatOutcome }`
  - `PlanetConquered { planet: PlanetId, new_owner: FactionId }`
  - `ResourceShortage { planet: PlanetId, resource: ResourceType }`
  - `TransferWindowOpen { from: PlanetId, to: PlanetId }`
- `StateChange` - State mutation events
  - `PlanetUpdated(PlanetId)`
  - `ShipUpdated(ShipId)`
  - `FactionUpdated(FactionId)`
  - `VictoryConditionMet(VictoryType)`
  - `GameOver(FactionId)`
  - `GameLoaded`

#### `types.rs` - Shared Types
- `ResourceBundle` - Resource container
  - `pub fn validate_non_negative(&self) -> GameResult<()>`
  - `pub fn can_afford(&self, cost: &ResourceBundle) -> bool`
  - `pub fn subtract(&mut self, cost: &ResourceBundle) -> GameResult<()>`
  - `pub fn add(&mut self, resources: &ResourceBundle) -> GameResult<()>`
  - `pub fn total(&self) -> i64`
- `ResourceStorage` - Storage with capacity limits
  - `pub fn available_space(&self) -> ResourceBundle`
  - `pub fn can_store(&self, resources: &ResourceBundle) -> bool`
  - `pub fn validate(&self) -> GameResult<()>`
- `WorkerAllocation` - Population job assignments
  - `pub fn validate(&self, total: i32) -> GameResult<()>`
- `Vector2` - 2D position vector
  - `pub fn new(x: f32, y: f32) -> Self`
  - `pub fn distance_to(&self, other: &Vector2) -> f32`
  - `pub fn magnitude(&self) -> f32`
  - `pub fn normalize(&self) -> Vector2`
  - `pub fn dot(&self, other: &Vector2) -> f32`
- `GameError` - Error handling enum
- `GameMode` - Game state enum (MainMenu, InGame)
- `GameConfiguration` - New game configuration settings
  - `planet_count: usize` - Number of planets to create
  - `starting_resources: ResourceBundle` - Initial resources for player
  - `starting_population: i32` - Initial population on player planet
  - `galaxy_size: GalaxySize` - Galaxy size preset
  - `ai_opponents: usize` - Number of AI factions
- `GalaxySize` - Galaxy size presets (Small, Medium, Large)
  - `pub fn planet_range(&self) -> (usize, usize)` - Get planet count range
- `Planet`, `Ship`, `Faction` - Core entity structures
- Type aliases: `PlanetId`, `ShipId`, `FactionId`, `GameResult<T>`

### Data Managers (`src/managers/`) - IMPLEMENTED

#### `planet_manager.rs` - Planet Data Management
- `PlanetManager` - Main manager struct
  - `pub fn new() -> Self`
  - `pub fn create_planet(&mut self, position: OrbitalElements, controller: Option<FactionId>) -> GameResult<PlanetId>`
  - `pub fn get_planet(&self, id: PlanetId) -> GameResult<&Planet>`
  - `pub fn get_all_planets(&self) -> &Vec<Planet>`
  - `pub fn get_planet_count(&self) -> usize`
  - `pub fn get_all_planet_ids(&self) -> Vec<PlanetId>`
  - `pub fn get_all_planets_cloned(&self) -> GameResult<Vec<Planet>>`
  - `pub fn modify_planet<F>(&mut self, id: PlanetId, modifier: F) -> GameResult<()>`
  - `pub fn validate_all_planets(&self) -> GameResult<()>`
  - `pub fn get_planets_by_faction(&self, faction: FactionId) -> Vec<&Planet>`
  - `pub fn add_resources(&mut self, id: PlanetId, resources: ResourceBundle) -> GameResult<()>`
  - `pub fn remove_resources(&mut self, id: PlanetId, resources: ResourceBundle) -> GameResult<()>`
  - `pub fn update_population(&mut self, id: PlanetId, amount: i32) -> GameResult<()>`
  - `pub fn set_worker_allocation(&mut self, id: PlanetId, allocation: WorkerAllocation) -> GameResult<()>`
  - `pub fn add_building(&mut self, id: PlanetId, building_type: BuildingType) -> GameResult<()>`
  - `pub fn get_building_count(&self, id: PlanetId, building_type: BuildingType) -> GameResult<usize>`
  - `pub fn get_available_building_slots(&self, id: PlanetId) -> GameResult<usize>`
  - `pub fn change_controller(&mut self, id: PlanetId, new_controller: Option<FactionId>) -> GameResult<()>`
  - `pub fn upgrade_storage(&mut self, id: PlanetId, additional_capacity: ResourceBundle) -> GameResult<()>`
  - `pub fn load_planets(&mut self, planets: Vec<Planet>) -> GameResult<()>`

#### `ship_manager.rs` - Ship Data Management
- `ShipManager` - Main manager struct
  - `pub fn new() -> Self`
  - `pub fn create_ship(&mut self, ship_class: ShipClass, position: Vector2, owner: FactionId) -> GameResult<ShipId>`
  - `pub fn get_ship(&self, id: ShipId) -> GameResult<&Ship>`
  - `pub fn update_position(&mut self, id: ShipId, position: Vector2) -> GameResult<()>`
  - `pub fn destroy_ship(&mut self, id: ShipId) -> GameResult<()>`
  - `pub fn load_cargo(&mut self, ship_id: ShipId, resources: ResourceBundle) -> GameResult<()>`
  - `pub fn unload_cargo(&mut self, ship_id: ShipId) -> GameResult<ResourceBundle>`
  - `pub fn get_cargo_capacity(&self, ship_id: ShipId) -> GameResult<i32>`
  - `pub fn get_cargo_contents(&self, ship_id: ShipId) -> GameResult<&ResourceBundle>`
  - `pub fn set_trajectory(&mut self, ship_id: ShipId, trajectory: Trajectory) -> GameResult<()>`
  - `pub fn consume_fuel(&mut self, ship_id: ShipId, amount: f32) -> GameResult<()>`
  - `pub fn get_ships_at_planet(&self, planet_position: Vector2, radius: f32) -> GameResult<Vec<ShipId>>`
  - `pub fn get_all_ships(&self) -> &Vec<Ship>`
  - `pub fn get_all_ships_cloned(&self) -> GameResult<Vec<Ship>>`
  - `pub fn calculate_fuel_cost(&self, ship_id: ShipId, distance: f32) -> GameResult<f32>`
  - `pub fn get_ships_by_owner(&self, owner: FactionId) -> Vec<ShipId>`
  - `pub fn get_ships_by_class(&self, ship_class: ShipClass) -> Vec<ShipId>`
  - `pub fn load_ships(&mut self, ships: Vec<Ship>) -> GameResult<()>`

#### `faction_manager.rs` - Faction Data Management
- `FactionManager` - Main manager struct
  - `pub fn new() -> Self`
  - `pub fn create_faction(&mut self, name: String, is_player: bool, ai_type: AIPersonality) -> GameResult<FactionId>`
  - `pub fn get_faction(&self, id: FactionId) -> GameResult<&Faction>`
  - `pub fn update_score(&mut self, id: FactionId, score: i32) -> GameResult<()>`
  - `pub fn add_score(&mut self, id: FactionId, points: i32) -> GameResult<()>`
  - `pub fn get_all_factions(&self) -> &[Faction]`
  - `pub fn count(&self) -> usize`
  - `pub fn find_by_name(&self, name: &str) -> Option<&Faction>`
  - `pub fn get_player_faction(&self) -> Option<&Faction>`
  - `pub fn load_factions(&mut self, factions: Vec<Faction>) -> GameResult<()>`

### Simulation Systems (`src/systems/`) - IMPLEMENTED

#### `time_manager.rs` - Time & Tick Management
- `TimeManager` - Main system struct
  - `pub fn new() -> Self`
  - `pub fn update(&mut self, delta: f32, event_bus: &mut EventBus) -> GameResult<()>`
  - `pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`
  - `pub fn get_current_tick(&self) -> u64`
  - `pub fn get_tick(&self) -> u64`
  - `pub fn set_tick(&mut self, tick: u64) -> GameResult<()>`
  - `pub fn set_speed_multiplier(&mut self, speed: f32) -> GameResult<()>`
  - `pub fn get_speed_multiplier(&self) -> f32`
  - `pub fn is_paused(&self) -> bool`
  - `pub fn get_tick_duration(&self) -> f64`
  - `pub fn get_game_time_seconds(&self) -> f64`
  - `pub fn validate(&self) -> GameResult<()>`

#### `physics_engine.rs` - Movement & Physics
- `PhysicsEngine` - Main system struct
  - `pub fn new() -> Self`
  - `fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>`
  - `fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`
  - `pub fn calculate_orbital_position(&self, elements: &OrbitalElements, time: f64) -> Vector2`
  - `pub fn calculate_ship_movement(&self, ship: &Ship, delta: f32) -> Vector2`
  - `pub fn validate_trajectory(&self, trajectory: &Trajectory) -> GameResult<()>`
  - `pub fn calculate_arrival_time(&self, ship: &Ship, target: Vector2) -> f32`
  - Physics calculations and interpolation

#### `resource_system.rs` - Resource Production
- `ResourceSystem` - Main system struct
  - `pub fn new() -> Self`
  - `fn update(&mut self, delta: f32, event_bus: &mut EventBus) -> GameResult<()>`
  - `fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`
  - `pub fn calculate_planet_production(&self, planet: &Planet) -> GameResult<ResourceBundle>`
  - `pub fn process_production(&mut self, planets: &[Planet], event_bus: &mut EventBus) -> GameResult<()>`
  - `pub fn validate_transfer(&self, source: &Planet, destination: &Planet, requested: ResourceBundle) -> GameResult<ResourceBundle>`
  - `pub fn get_consumption_for_planet(&self, planet_id: PlanetId) -> Option<&ResourceBundle>`
  - `pub fn validate_cargo_loading(&self, ship: &Ship, planet: &Planet, requested: ResourceBundle, current_tick: u64) -> GameResult<ResourceBundle>`
  - `pub fn validate_cargo_unloading(&self, ship: &Ship, planet: &Planet, current_tick: u64) -> GameResult<ResourceBundle>`

#### `population_system.rs` - Population Management
- `PopulationSystem` - Main system struct
  - `pub fn new() -> Self`
  - `pub fn update(&mut self, delta: f32, event_bus: &mut EventBus) -> GameResult<()>`
  - `fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`
  - `pub fn process_planet_growth(&mut self, planet_id: PlanetId, population: i32, food_available: i32, event_bus: &mut EventBus) -> GameResult<()>`
  - `pub fn get_growth_rate(&self, planet_id: PlanetId) -> Option<f32>`
  - `pub fn pending_migrations(&self) -> usize`
- `MigrationOrder` - Population transfer tracking
  - Migration between planets via ships

#### `construction.rs` - Building & Ship Construction
- `ConstructionSystem` - Main system struct
  - `pub fn new() -> Self`
  - `fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>`
  - `fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`
  - `pub fn start_building_construction(&mut self, planet_id: PlanetId, building_type: BuildingType, current_tick: u64) -> GameResult<()>`
  - `pub fn start_ship_construction(&mut self, planet_id: PlanetId, ship_class: ShipClass, current_tick: u64) -> GameResult<()>`
  - `pub fn get_construction_queue_length(&self, planet_id: PlanetId) -> usize`
  - `pub fn get_estimated_completion_time(&self, planet_id: PlanetId, project_index: usize) -> Option<u64>`
  - Construction queues and resource validation

#### `combat_resolver.rs` - Combat System
- `CombatResolver` - Main system struct
  - `pub fn new() -> Self`
  - `fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>`
  - `fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`
  - `pub fn initiate_combat(&mut self, attacker: ShipId, target: ShipId, current_tick: u64) -> GameResult<()>`
  - `pub fn resolve_combat(&mut self, combat_id: CombatId, attacker: &Ship, defender: &Ship) -> CombatOutcome`
  - `pub fn calculate_damage(&self, attacker: &Ship, defender: &Ship) -> i32`
  - `pub fn get_active_combats(&self) -> &[Combat]`
  - Combat mechanics and ship destruction

#### `save_system.rs` - Save/Load Operations
- `SaveSystem` - Main system struct
  - `pub fn new() -> Self`
  - `fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>`
  - `fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`
  - `pub fn save_game(&self, state: &GameState) -> GameResult<()>`
  - `pub fn save_game_to_slot(&self, state: &GameState, slot_name: &str) -> GameResult<()>`
  - `pub fn load_game(&self) -> GameResult<SaveData>`
  - `pub fn load_game_from_slot(&self, slot_name: &str) -> GameResult<SaveData>`
  - `pub fn list_saves(&self) -> GameResult<Vec<SaveInfo>>` - List all available saves
  - `pub fn save_exists(&self, slot_name: &str) -> bool`
  - `pub fn delete_save(&self, slot_name: &str) -> GameResult<()>`
  - `pub fn validate_save_integrity(&self, save_data: &SaveData) -> GameResult<()>`
  - Deterministic state preservation and validation
- `SaveInfo` - Save file metadata
  - `name: String` - Save file name
  - `timestamp: u64` - Save creation time
  - `tick: u64` - Game tick when saved
  - `planets: usize` - Number of planets
  - `ships: usize` - Number of ships
  - `factions: usize` - Number of factions

#### `game_initializer.rs` - Configurable Game Creation
- `GameInitializer` - New game creation system
  - `pub fn new(config: GameConfiguration) -> Self`
  - `pub fn initialize_game(&self, planet_manager: &mut PlanetManager, ship_manager: &mut ShipManager, faction_manager: &mut FactionManager) -> GameResult<()>`
  - `pub fn get_configuration(&self) -> &GameConfiguration`
  - `pub fn set_configuration(&mut self, config: GameConfiguration)`
  - Creates configurable new games with custom planet counts, resources, AI opponents
  - Deterministic planet placement with varied orbital elements
  - Faction setup with different AI personalities
  - Starting ship placement and resource allocation
  - Proper worker allocation with validation compliance

### User Interface (`src/ui/`) - IMPLEMENTED

#### `renderer.rs` - Main UI Renderer
- `UIRenderer` - Main renderer struct with centralized panel management
  - `pub fn new() -> Self`
  - `pub fn get_selected_planet(&self) -> Option<PlanetId>`
  - `pub fn get_selected_ship(&self) -> Option<ShipId>`
  - `pub fn set_selected_ship(&mut self, ship_id: Option<ShipId>)`
  - `pub fn set_selected_planet(&mut self, planet_id: Option<PlanetId>)`
  - `pub fn get_zoom_level(&self) -> f32`
  - `pub fn get_ui_scale(&self) -> f32`
  - `pub fn is_paused(&self) -> bool`
  - `pub fn is_planet_panel_open(&self) -> bool`
  - `pub fn is_ship_panel_open(&self) -> bool`
  - `pub fn render_with_events(&mut self, state: &GameState, interpolation: f32, events: &mut Vec<GameEvent>) -> GameResult<()>`
  - `pub fn process_input(&mut self, events: &mut EventBus) -> GameResult<()>`
  - `pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>`
  - `fn handle_toolbar_interactions(&mut self, state: &GameState, events: &mut Vec<GameEvent>) -> GameResult<()>` - **ENHANCED**: Now accepts GameState to check content availability
  - `fn render_disabled_toolbar_button(&mut self, x: f32, y: f32, w: f32, h: f32, text: &str)` - **NEW**: Renders grayed-out, non-clickable buttons
  - `fn sync_panel_states(&mut self)` - **NEW**: Centralized panel state synchronization system
  - `fn update_panel_positions(&mut self)` - **NEW**: Dynamic panel positioning based on screen size

**Panel State Management Architecture:**
- **Three-Tier Panel Control System**: UIRenderer.ui_context + Panel.visible + centralized synchronization
- **Primary Control**: `ui_context.planet_panel_open`/`ship_panel_open` flags determine panel visibility intent
- **Panel Internal State**: Individual panels maintain `visible` field controlled by `show()`/`hide()` methods  
- **Synchronization**: `sync_panel_states()` called every frame to coordinate all three systems
- **Automatic Cleanup**: Panels close and flags reset when selections are cleared
- **Non-Breaking**: All existing Panel trait methods preserved

**Dynamic Panel Positioning System:**
- **Planet Panel**: Top-left corner at `(10.0, 50.0)` - below toolbar with 50px offset
- **Ship Panel**: Bottom-right corner at `(screen_width - 280 - 10, screen_height - 400 - 10)` - with 10px margins
- **Screen Responsive**: Panel positions recalculate every frame based on current screen dimensions

**Panel Visibility Architecture (August 2025 Fix):**
- **Single Point of Control**: UIRenderer validates visibility conditions before calling panel render methods
- **Renderer Logic**: Checks `ui_context.planet_panel_open && selected_planet.is_some()` before rendering
- **Panel Render Methods**: Removed internal `if !self.visible { return Ok(()); }` guards to eliminate dual checking
- **Centralized Synchronization**: `sync_panel_states()` method coordinates all three panel state systems
- **Clean Implementation**: No redundant visibility checks between renderer and panel internal logic
- **Position Methods**: Both panels have `set_position(x, y)` methods for dynamic positioning
- **Integration**: Position updates integrated into main synchronization system for consistency

#### `input_handler.rs` - Input Processing
- `InputHandler` - Main input struct
  - `pub fn new() -> Self`
  - `pub fn process_input(&mut self, camera: &mut Camera, ui_state: &mut UIState) -> Vec<GameEvent>`
  - `pub fn handle_mouse_input(&mut self, camera: &Camera, ui_state: &UIState) -> Vec<GameEvent>`
  - `pub fn handle_keyboard_input(&mut self) -> Vec<GameEvent>`
  - `pub fn world_to_screen(&self, world_pos: Vector2, camera: &Camera) -> Vector2`
  - `pub fn screen_to_world(&self, screen_pos: Vector2, camera: &Camera) -> Vector2`
  - Input validation and command generation

#### `camera.rs` - Camera System
- `Camera` - Camera state struct
  - `pub fn new() -> Self`
  - `pub fn update(&mut self, delta: f32)`
  - `pub fn screen_to_world(&self, screen_pos: Vector2) -> Vector2`
  - `pub fn world_to_screen(&self, world_pos: Vector2) -> Vector2`
  - `pub fn zoom(&mut self, zoom_delta: f32, zoom_center: Vector2)`
  - `pub fn pan(&mut self, delta: Vector2)`
  - `pub fn set_position(&mut self, position: Vector2)`
  - `pub fn get_position(&self) -> Vector2`
  - `pub fn get_zoom(&self) -> f32`
  - `pub fn get_viewport_bounds(&self) -> (Vector2, Vector2)`
  - Viewport and coordinate transformation management

#### `toolbar.rs` - Toolbar Component
- `Toolbar` - Toolbar state
  - `pub fn new() -> Self`
  - `pub fn render(&mut self, ui_state: &UIState, state: &GameState) -> Vec<GameEvent>`
  - `pub fn handle_button_click(&mut self, button_id: &str) -> Option<GameEvent>`
  - `pub fn update_resource_display(&mut self, resources: &ResourceBundle)`
  - `pub fn set_game_speed(&mut self, speed: f32)`
  - `pub fn toggle_pause(&mut self) -> bool`
  - Resource display and game control interface

#### `start_menu.rs` - Start Menu Component
- `StartMenu` - Main menu state and rendering
  - `pub fn new() -> Self`
  - `pub fn render(&mut self, game_config: Option<&GameConfiguration>) -> GameResult<Vec<GameEvent>>`
  - `pub fn update_save_status(&mut self, save_exists: bool)`
  - `pub fn refresh_save_status(&mut self)` - **NEW**: Dynamically refresh save detection using SaveSystem
  - `pub fn process_input(&mut self) -> GameResult<Vec<GameEvent>>`
  - `fn check_for_saves() -> bool` - **ENHANCED**: Uses SaveSystem.list_saves() instead of hardcoded file checking
  - Main menu with New Game, Load Game, Game Options, and Exit options


#### `save_load_dialog.rs` - Save/Load Dialog System
- `SaveLoadDialog` - Modal dialog for save/load operations
  - `pub fn new() -> Self`
  - `pub fn show_new_game_dialog(&mut self)`
  - `pub fn show_save_dialog(&mut self)`
  - `pub fn show_load_dialog(&mut self, saves: Vec<SaveInfo>)`
  - `pub fn is_active(&self) -> bool`
  - `pub fn close(&mut self)`
  - `pub fn handle_input(&mut self) -> GameResult<Vec<GameEvent>>`
  - `pub fn render(&mut self) -> GameResult<()>`
  - Text input for game/save naming with validation
- `DialogType` - Dialog mode enum (NewGame, SaveGame, LoadGame)

#### `list_menus.rs` - Menu Components
- Menu rendering functions
  - `pub fn render_build_menu(selected_planet: PlanetId, state: &GameState) -> Option<GameEvent>`
  - `pub fn render_ship_construction_menu(planet_id: PlanetId, state: &GameState) -> Option<GameEvent>`
  - `pub fn render_resource_transfer_menu(from_planet: PlanetId, state: &GameState) -> Option<GameEvent>`
  - `pub fn render_worker_allocation_menu(planet_id: PlanetId, state: &GameState) -> Option<GameEvent>`
  - `pub fn handle_menu_selection(menu_type: MenuType, selection: usize) -> Option<GameEvent>`
  - Dynamic construction and management menus

#### `ui_state.rs` - UI State Management
- `UIState` - UI state container
  - `pub fn new() -> Self`
  - `pub fn select_planet(&mut self, planet_id: Option<PlanetId>)`
  - `pub fn select_ship(&mut self, ship_id: Option<ShipId>)`
  - `pub fn get_selected_planet(&self) -> Option<PlanetId>`
  - `pub fn get_selected_ship(&self) -> Option<ShipId>`
  - `pub fn toggle_planet_panel(&mut self)`
  - `pub fn toggle_ship_panel(&mut self)`
  - `pub fn is_planet_panel_open(&self) -> bool`
  - `pub fn is_ship_panel_open(&self) -> bool`
  - `pub fn set_active_tab(&mut self, tab: TabType)`
  - `pub fn get_active_tab(&self) -> TabType`
  - UI state and panel management coordination

#### `panels/planet_panel.rs` - Planet Information
- `PlanetPanel` - Panel state with dynamic positioning
  - `pub fn new() -> Self`
  - `pub fn set_position(&mut self, x: f32, y: f32)` - **NEW**: Dynamic positioning support
  - `pub fn render(&mut self, planet_id: PlanetId, state: &GameState, ui_state: &mut UIState) -> Vec<GameEvent>`
  - `pub fn render_overview_tab(&mut self, planet: &Planet) -> Vec<GameEvent>`
  - `pub fn render_buildings_tab(&mut self, planet: &Planet, state: &GameState) -> Vec<GameEvent>`
  - `pub fn render_population_tab(&mut self, planet: &Planet) -> Vec<GameEvent>`
  - `pub fn render_construction_tab(&mut self, planet: &Planet, state: &GameState) -> Vec<GameEvent>`
  - `pub fn handle_tab_change(&mut self, new_tab: PlanetTab)`
  - **Position**: Top-left corner at `(10, 50)` below toolbar
  - Comprehensive planet information and management interface

#### `panels/ship_panel.rs` - Ship Information
- `ShipPanel` - Panel state with dynamic positioning
  - `pub fn new() -> Self`
  - `pub fn set_position(&mut self, x: f32, y: f32)` - **NEW**: Dynamic positioning support
  - `pub fn render(&mut self, ship_id: ShipId, state: &GameState, ui_state: &mut UIState) -> Vec<GameEvent>`
  - `pub fn render_ship_details(&mut self, ship: &Ship) -> Vec<GameEvent>`
  - `pub fn render_cargo_display(&mut self, ship: &Ship, state: &GameState) -> Vec<GameEvent>`
  - `pub fn render_movement_controls(&mut self, ship: &Ship, state: &GameState) -> Vec<GameEvent>`
  - `pub fn render_combat_status(&mut self, ship: &Ship, state: &GameState) -> Vec<GameEvent>`
  - **Position**: Bottom-right corner with screen-responsive calculations
  - Ship status, cargo, and command interface

#### `resource_display.rs` - Resource Display Module  
- `ResourceDisplay` - Modular resource rendering component
  - `pub fn new() -> Self`
  - `pub fn render_horizontal_bar(&mut self, state: &GameState) -> GameResult<()>` - Renders horizontal resource bar
  - `pub fn render_side_panel(&mut self, state: &GameState, mouse_over_ui: &mut bool) -> GameResult<()>` - Renders side resource panel
  - `fn update_240_tick_tracking(&mut self, state: &GameState) -> GameResult<()>` - Resource change tracking
  - `fn get_cached_empire_resources(&mut self, state: &GameState) -> GameResult<ResourceBundle>` - Performance caching
  - `fn calculate_empire_resources(&self, state: &GameState) -> GameResult<ResourceBundle>` - Resource calculation
  - `fn calculate_empire_population(&self, state: &GameState) -> i32` - Population calculation
  - Resource visualization with change indicators and performance optimization

#### `drawing_utils.rs` - Drawing Utilities Module (Phase 3 Optimization)
- `DrawingUtils` - Centralized drawing utilities for game objects
  - `pub fn draw_orbit(planet: &Planet, zoom_level: f32) -> GameResult<()>` - Draws orbital paths
  - `pub fn draw_planet_indicators(screen_pos: Vector2, planet: &Planet, zoom_level: f32) -> GameResult<()>` - Planet visual indicators
  - `pub fn draw_ship_shape(screen_pos: Vector2, size: f32, color: Color, ship_class: ShipClass) -> GameResult<()>` - Ship rendering by class
  - `pub fn draw_trajectory_line(screen_pos: Vector2, trajectory: &Trajectory, color: Color) -> GameResult<()>` - Movement trajectories
  - `pub fn screen_to_world(screen_pos: Vector2, camera_position: Vector2, zoom_level: f32) -> Vector2` - Coordinate conversion
  - `pub fn world_to_screen(world_pos: Vector2, camera_position: Vector2, zoom_level: f32) -> Vector2` - Coordinate conversion
  - Optimized rendering with zoom-level scaling and screen validation

#### `panels/resource_panel.rs` - Resource Display
- `ResourcePanel` - Panel state
  - `pub fn new() -> Self`
  - `pub fn render(&mut self, resources: &ResourceBundle, storage: &ResourceStorage) -> Vec<GameEvent>`
  - `pub fn render_resource_bars(&mut self, current: &ResourceBundle, capacity: &ResourceBundle)`
  - `pub fn render_production_info(&mut self, production: &ResourceBundle, consumption: &ResourceBundle)`
  - `pub fn render_transfer_controls(&mut self, planet_id: PlanetId, state: &GameState) -> Vec<GameEvent>`
  - `pub fn calculate_bar_width(&self, current: i32, max: i32) -> f32`
  - Resource visualization and transfer interface

## Implementation Rules by Directory

### ❌ DO NOT MODIFY - `/src/core/`
Core architecture files defining EventBus, base types, and event definitions.

### ✅ IMPLEMENTED - `/src/managers/`
Data ownership structures with CRUD methods returning `GameResult<T>`.

### ✅ IMPLEMENTED - `/src/systems/`
Simulation logic processing events through EventBus without data ownership.

### ✅ IMPLEMENTED - `/src/ui/`
Immediate mode rendering with input processing generating PlayerCommand events only.

## Integration Points

Each system connects through:
1. **Event subscription** in `GameState::new()`
2. **Update call** in `GameState::fixed_update()`
3. **Manager access** through `&mut self` references
4. **Event emission** through `events.queue_event()`

## Import Hierarchy

```
main.rs → core, managers, systems, ui
systems/* → core only
managers/* → core only
ui/* → core only (reads via GameState ref)
core/* → std only
```

## Testing Structure

### Integration Tests
- `architecture_invariants.rs` - Architecture compliance validation
- `integration_tests.rs` - Full system integration tests
- `phase2_integration_test.rs` - Phase 2 validation
- `save_system_integration.rs` - Save/load integration
- `time_manager_integration.rs` - Time system integration

### Unit Tests
- `physics_engine_test.rs` - Physics engine validation
- `planet_manager_test.rs` - Planet manager validation
- `systems/*.rs` - Individual system unit tests

## Current Implementation Status

**✅ COMPLETE & OPERATIONAL:**
- All core architecture and EventBus
- All managers (Planet, Ship, Faction)
- All systems (Resource, Population, Construction, Time, Physics, Combat, Save, GameInitializer)
- Complete UI rendering with interactive panels
- Save/Load dialog system with named saves
- Main menu with configurable game options
- Start menu with game configuration cycling
- Comprehensive test suite (55+ tests passing)

**🎯 NEW FEATURES ADDED:**
- **Named Save System**: Save and load games with custom names
- **Save Browser**: Interactive list of available saves with metadata
- **Game Configuration**: Configurable new games with planet count, resources, AI opponents
- **Galaxy Size Presets**: Small/Medium/Large configurations
- **Game Naming**: Custom names for new games with auto-save
- **Modal Dialogs**: Full-featured save/load/new game dialog system
- **Enhanced Menu**: Game options cycling and configuration display

**🎯 PERFORMANCE TARGETS:**
- 30+ FPS with 20 planets, 100 ships
- <10ms per system per tick
- Deterministic save/load compatibility

This structure represents the complete, operational codebase with all major systems implemented, tested, and enhanced with comprehensive save/load functionality and configurable game creation.