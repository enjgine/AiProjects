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
│   │   └── types.rs                    # Shared types (Planet, Ship, etc.) + SaveError
│   │
│   ├── managers/                       # DATA OWNERS (Implemented)
│   │   ├── mod.rs                      # Export all managers
│   │   ├── planet_manager.rs           # PlanetManager implementation
│   │   ├── ship_manager.rs             # ShipManager implementation
│   │   ├── faction_manager.rs          # FactionManager implementation
│   │   └── entity_manager.rs           # Entity management utilities
│   │
│   ├── systems/                        # SIMULATION LOGIC (Enhanced)
│   │   ├── mod.rs                      # Export all systems
│   │   ├── time_manager.rs             # TimeManager implementation
│   │   ├── physics_engine.rs           # PhysicsEngine implementation
│   │   ├── resource_system.rs          # ResourceSystem implementation
│   │   ├── population_system.rs        # PopulationSystem implementation
│   │   ├── construction.rs             # ConstructionSystem implementation
│   │   ├── combat_resolver.rs          # CombatResolver implementation
│   │   ├── save_system.rs              # SaveSystem (simplified JSON-based, 242 lines)
│   │   └── game_initializer.rs         # GameInitializer for configurable new games
│   │
│   └── ui_v2/                          # MODERN UI SYSTEM (Component-Based)
│       ├── mod.rs                      # UI v2 exports and public API
│       ├── core/                       # UI Infrastructure
│       │   ├── mod.rs                  # Core exports
│       │   ├── ui_system.rs            # Main UISystem coordinator
│       │   ├── view_controller.rs      # View management system
│       │   ├── input_controller.rs     # Input handling system
│       │   └── render_context.rs       # Rendering context and theming
│       ├── components/                 # Reusable UI Components
│       │   ├── mod.rs                  # Component exports
│       │   ├── base_component.rs       # UIComponent trait and base functionality
│       │   ├── container.rs            # Panel and container components
│       │   ├── interactive.rs          # Button, Dropdown, TextInput components
│       │   ├── display.rs              # Text, Image, Progress display components
│       │   └── layout.rs               # Layout system and positioning
│       ├── views/                      # Specialized View Components
│       │   ├── mod.rs                  # View exports
│       │   ├── base_view.rs            # View trait and base functionality
│       │   ├── entity_view.rs          # Generic entity display view
│       │   ├── data_view.rs            # Data visualization view
│       │   └── dialog_view.rs          # Modal dialog view
│       ├── adapters/                   # Entity-to-UI Data Adapters
│       │   ├── mod.rs                  # Adapter exports
│       │   ├── entity_adapter.rs       # Generic entity adapter trait
│       │   ├── planet_adapter.rs       # Planet display adapter
│       │   ├── ship_adapter.rs         # Ship display adapter
│       │   └── faction_adapter.rs      # Faction display adapter
│       ├── panels/                     # Production Panel Implementations
│       │   ├── mod.rs                  # Panel exports
│       │   ├── planet_panel_migrated.rs # Planet management panel
│       │   ├── ship_panel_migrated.rs  # Ship management panel
│       │   └── resource_panel_migrated.rs # Resource display panel
│       └── examples/                   # Migration Examples and Demos
│           ├── mod.rs                  # Example exports
│           ├── migration_demo.rs       # UI migration demonstration
│           └── planet_panel_v2.rs      # Example new-style panel
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

### User Interface v2 (`src/ui_v2/`) - MODERN COMPONENT SYSTEM

#### Core Infrastructure (`src/ui_v2/core/`)

##### `ui_system.rs` - Main UI Coordinator
- `UISystem` - Modern component-based UI coordinator
  - `pub fn new() -> Self`
  - `pub fn update(&mut self, delta_time: f32) -> Vec<PlayerCommand>` - Process input and update all views
  - `pub fn render(&mut self)` - Render all active components
  - `pub fn set_enabled(&mut self, enabled: bool)` - Enable/disable UI system
  - `pub fn add_view(&mut self, view: Box<dyn View>)` - Add new view to system
  - `pub fn remove_view(&mut self, view_type: &str)` - Remove view by type
  - `fn create_render_context(&self) -> RenderContext` - Create rendering context with theme
  - `fn update_scale_factor(&mut self)` - Update UI scaling for different screen sizes

##### `view_controller.rs` - View Management System
- `ViewController` - Manages all active views and their lifecycle
  - `pub fn new() -> Self`
  - `pub fn add_view(&mut self, view: Box<dyn View>) -> ViewResult<()>`
  - `pub fn remove_view(&mut self, view_type: &str) -> ViewResult<()>`
  - `pub fn handle_input(&mut self, input: &InputEvent) -> ViewResult<Option<PlayerCommand>>`
  - `pub fn render_all(&mut self, context: &RenderContext) -> ViewResult<()>`
  - `pub fn update_all(&mut self, delta_time: f32) -> ViewResult<()>`
  - `fn handle_view_event(&mut self, event: ViewEvent) -> ViewResult<()>`

##### `input_controller.rs` - Input Handling System
- `InputController` - Processes all input events and converts to UI commands
  - `pub fn new() -> Self`
  - `pub fn process_input(&mut self, delta_time: f32) -> Vec<InputEvent>`
  - `pub fn generate_ui_commands(&mut self, events: &[InputEvent]) -> Vec<PlayerCommand>`
  - `fn handle_keyboard_input(&mut self) -> Vec<InputEvent>`
  - `fn handle_mouse_input(&mut self) -> Vec<InputEvent>`
  - `fn convert_input_to_command(&self, input: &InputEvent) -> Option<PlayerCommand>`

##### `render_context.rs` - Rendering Context and Theming
- `RenderContext` - Provides theming and rendering utilities
  - `pub fn new() -> Self`
  - `pub fn with_theme(theme: Theme) -> Self`
  - `pub font_size: f32` - Base font size
  - `pub theme: Theme` - Current UI theme
  - `pub screen_bounds: Rect` - Screen dimensions
- `Theme` - UI color theme and styling
  - `pub background_color: Color`
  - `pub text_color: Color`
  - `pub primary_color: Color`
  - `pub secondary_color: Color`
  - `pub accent_color: Color`

#### Component Library (`src/ui_v2/components/`)

##### `base_component.rs` - Foundation Component System
- `UIComponent` - Core trait for all UI components
  - `fn render(&mut self, data: &T, context: &RenderContext) -> ComponentResult` - Render component with data
  - `fn handle_input(&mut self, input: &InputEvent) -> ComponentResult` - Process input events
  - `fn update(&mut self, delta_time: f32) -> ComponentResult` - Update component state
  - `fn get_bounds(&self) -> Rect` - Get component screen bounds
- `BaseComponent` - Common component functionality
  - Layout management, event handling, state tracking
- `ComponentState` - Component lifecycle states (Active, Disabled, Hidden)
- `ComponentResult` - Result type for component operations

##### `container.rs` - Container Components
- `Panel` - Basic container component
  - `pub fn new(title: String) -> Self`
  - `pub fn with_layout(mut self, layout: Layout) -> Self`
  - `pub fn collapsible(mut self, collapsible: bool) -> Self`
  - Layout-based positioning and automatic background rendering
- `ScrollView` - Scrollable content container
  - Automatic scrollbar rendering and content clipping
- `TabContainer` - Tabbed interface container
  - Tab management and content switching

##### `interactive.rs` - Interactive Components
- `Button` - Clickable button component
  - `pub fn new(text: String) -> Self`
  - `pub fn with_click_command(mut self, command: PlayerCommand) -> Self`
  - `pub fn set_click_command(&mut self, command: PlayerCommand)`
  - Hover states, click detection, and command emission
- `Dropdown<T>` - Generic dropdown selection component
  - `pub fn new(placeholder: String) -> Self`
  - `pub fn set_items(&mut self, items: Vec<T>)`
  - `pub fn get_selected(&self) -> Option<&T>`
  - Type-safe item selection and display
- `TextInput` - Text input field component
  - Text editing, validation, and submission handling

##### `display.rs` - Display Components
- `Text` - Text rendering component
  - `pub fn new(text: String) -> Self`
  - `pub fn with_color(mut self, color: Color) -> Self`
  - `pub fn with_font_size(mut self, size: f32) -> Self`
  - Static and dynamic text display with styling
- `ProgressBar` - Progress indication component
  - Value display with customizable styling and labels
- `Image` - Image display component
  - Texture loading and scaling support

##### `layout.rs` - Layout System
- `Layout` - Position and size management
  - `pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self`
  - `pub fn center_in(parent: Rect) -> Self`
  - `pub fn get_rect(&self) -> Rect`
  - Automatic positioning and responsive design support
- `LayoutDirection` - Flex-style layout directions (Row, Column)
- `Alignment` - Component alignment options (Start, Center, End)

#### View System (`src/ui_v2/views/`)

##### `base_view.rs` - View Foundation
- `View` - Core trait for all views
  - `fn render(&mut self, context: &RenderContext) -> ComponentResult`
  - `fn handle_input(&mut self, input: &InputEvent) -> ComponentResult`
  - `fn update(&mut self, delta_time: f32) -> ComponentResult`
  - `fn update_data(&mut self, data: ViewData) -> ComponentResult`
  - `fn is_visible(&self) -> bool`
  - `fn set_visible(&mut self, visible: bool)`
  - `fn get_view_type(&self) -> &'static str`

##### `entity_view.rs` - Entity Display View
- `EntityView<T>` - Generic view for displaying entities
  - `pub fn new(title: String, adapter: Box<dyn EntityAdapter<T>>) -> Self`
  - `pub fn set_entity(&mut self, entity: T)`
  - Type-safe entity display using adapter pattern
  - Automatic field rendering and action generation

##### `data_view.rs` - Data Visualization View  
- `DataView<T>` - Generic data visualization component
  - `pub fn new() -> Self`
  - `pub fn set_data(&mut self, data: Vec<T>)`
  - `pub fn with_renderer(mut self, renderer: Box<dyn DataRenderer<T>>) -> Self`
  - Flexible data presentation with custom renderers

##### `dialog_view.rs` - Modal Dialog View
- `DialogView` - Modal dialog system
  - `pub fn new(title: String) -> Self`
  - `pub fn add_field(&mut self, field: DialogField)`
  - `pub fn show(&mut self)`
  - `pub fn close(&mut self)`
  - Form-based input collection with validation

#### Entity Adapters (`src/ui_v2/adapters/`)

##### `entity_adapter.rs` - Adapter Pattern Foundation
- `EntityAdapter<T>` - Core adapter trait
  - `fn get_display_fields(&self, entity: &T) -> Vec<(String, String)>`
  - `fn get_actions(&self, entity: &T) -> Vec<(String, PlayerCommand)>`
  - `fn format_field(&self, field_name: &str, entity: &T) -> String`
  - `fn get_summary(&self, entity: &T) -> String`
  - `fn get_icon(&self, entity: &T) -> Option<String>`
  - `fn get_status_color(&self, entity: &T) -> Option<Color>`
  - `fn is_highlighted(&self, entity: &T) -> bool`

##### `planet_adapter.rs` - Planet Entity Adapter
- `PlanetAdapter` - Converts Planet data to UI display format
  - `pub fn new() -> Self`
  - `pub fn simple() -> Self` - Minimal display mode
  - `pub fn with_detailed_resources(mut self, show: bool) -> Self`
  - `pub fn with_development_slots(mut self, show: bool) -> Self`
  - Configurable detail levels and resource display options

##### `ship_adapter.rs` - Ship Entity Adapter
- `ShipAdapter` - Converts Ship data to UI display format
  - `pub fn new() -> Self`
  - `pub fn simple() -> Self` - Minimal display mode
  - `pub fn with_cargo_details(mut self, show: bool) -> Self`
  - `pub fn with_movement_history(mut self, show: bool) -> Self`
  - Ship status, cargo, and movement information display

#### Production Panels (`src/ui_v2/panels/`)

##### `planet_panel_migrated.rs` - Modern Planet Panel
- `PlanetPanelMigrated` - Component-based planet management
  - **Benefits**: ~75% code reduction from old implementation
  - **Features**: Autonomous entity selection, tabbed interface, real-time updates
  - **Architecture**: Uses EntityView + PlanetAdapter + ListView components
  - Resource management, development planning, population control

##### `ship_panel_migrated.rs` - Modern Ship Panel  
- `ShipPanelMigrated` - Component-based ship management
  - **Benefits**: ~53% code reduction from old implementation
  - **Features**: Ship selector dropdown, cargo management, movement controls
  - **Architecture**: Uses EntityView + ShipAdapter + Dropdown components
  - Ship status, cargo operations, fleet management

##### `resource_panel_migrated.rs` - Modern Resource Panel
- `ResourcePanelMigrated` - Empire-wide resource visualization
  - **Features**: Real-time resource tracking, empire totals, performance monitoring
  - **Architecture**: Uses DataView + custom resource renderers
  - Resource production analysis and economic planning

## UI v2 Architecture Benefits

### Component-Based Design
- **Reusable Components**: Button, Panel, Dropdown, ListView shared across all panels
- **Type-Safe Adapters**: EntityAdapter pattern ensures data consistency
- **Separation of Concerns**: UI logic separated from game data through adapters
- **Automatic Layout**: Layout system handles positioning and responsiveness

### Code Reduction Achievements
- **Planet Panel**: ~75% reduction (1,615 → ~400 lines)
- **Ship Panel**: ~53% reduction (753 → ~350 lines)  
- **Resource Panel**: Modern data visualization with performance optimization
- **Overall**: Massive reduction in UI maintenance overhead

### Modern Patterns
- **MVC Architecture**: Models (Entities) + Views (UI) + Controllers (Adapters)
- **Event-Driven**: All interactions generate PlayerCommand events
- **Composable**: Components can be combined into complex interfaces
- **Testable**: Individual components can be unit tested in isolation

## Migration Examples (`src/ui_v2/examples/`)

##### `migration_demo.rs` - Full Migration Demonstration
- `MigrationDemo` - Shows before/after comparison of UI systems
  - Side-by-side rendering of old vs new UI approaches
  - Performance comparison and code complexity metrics
  - Live demonstration of component composition benefits

##### `planet_panel_v2.rs` - New-Style Panel Example
- `PlanetPanelV2` - Example of modern panel implementation
  - Clean component composition without legacy dependencies
  - Demonstrates adapter pattern integration
  - Shows modern event handling and data binding

## Implementation Rules by Directory

### ❌ DO NOT MODIFY - `/src/core/`
Core architecture files defining EventBus, base types, and event definitions.

### ✅ IMPLEMENTED - `/src/managers/`
Data ownership structures with CRUD methods returning `GameResult<T>`.

### ✅ IMPLEMENTED - `/src/systems/`
Simulation logic processing events through EventBus without data ownership.

### ✅ IMPLEMENTED - `/src/ui_v2/`
Modern component-based UI system with reusable components and adapter patterns.

## Integration Points

Each system connects through:
1. **Event subscription** in `GameState::new()`
2. **Update call** in `GameState::fixed_update()`
3. **Manager access** through `&mut self` references
4. **Event emission** through `events.queue_event()`

## Import Hierarchy

```
main.rs → core, managers, systems, ui_v2
systems/* → core only
managers/* → core only
ui_v2/* → core only (reads via GameState ref)
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
- All managers (Planet, Ship, Faction, Entity)
- All systems (Resource, Population, Construction, Time, Physics, Combat, Save, GameInitializer)
- **Modern UI v2 System**: Component-based architecture with reusable components
- **Production Panels**: Planet, Ship, and Resource panels migrated to ui_v2
- **Adapter Pattern**: Type-safe entity-to-UI data conversion
- **Layout System**: Automatic positioning and responsive design
- Comprehensive test suite (55+ tests passing)

**🎯 UI V2 SYSTEM FEATURES:**
- **Component Library**: Reusable Button, Panel, Dropdown, ListView, TextInput components
- **View System**: EntityView, DataView, DialogView for different display patterns  
- **Entity Adapters**: PlanetAdapter, ShipAdapter, FactionAdapter for data presentation
- **Modern Panels**: Migrated panels with 50-75% code reduction
- **Theme System**: Consistent styling and color management
- **Input Handling**: Centralized input processing with event generation
- **Performance**: Optimized rendering with layout caching and efficient updates

**🎯 PERFORMANCE TARGETS:**
- 30+ FPS with 20 planets, 100 ships
- <10ms per system per tick
- Deterministic save/load compatibility

This structure represents the complete, operational codebase with all major systems implemented, tested, and enhanced with comprehensive save/load functionality and configurable game creation.

## August 2025 Optimization Updates

**Phase 1 Codebase Compression - COMPLETED:**
- **Removed**: `src/core/asset_types.rs` (817 lines) - unused complex asset system
- **Simplified**: `save_system.rs` from 1,257 → 242 lines (removed binary chunks, asset management)
- **Updated**: `core/types.rs` with SaveError, `save_load_dialog.rs` with SaveInfo structure  
- **Results**: Total codebase reduced from 15,910 → 14,078 lines (-11.5% reduction)
- **Quality**: All 25 architecture tests still passing, maintaining architectural compliance

**Phase 2 UI System Migration - COMPLETED:**
- **Removed**: Old UI system (~8,000+ lines across 17+ files)
- **Added**: Modern ui_v2 system with component-based architecture
- **Created**: Component library (Panel, Button, Dropdown, ListView, EntityView)
- **Created**: Adapter pattern for type-safe entity-to-UI conversion
- **Migrated**: All production panels to ui_v2 with 50-75% code reduction

**Phase 3 Module Consolidation - COMPLETED:**
- **Added**: `src/managers/entity_manager.rs` (297 lines) - consolidated manager patterns
- **Added**: UI v2 infrastructure with reusable components
- **Consolidated**: Monolithic UI → modular component system
- **Infrastructure**: Component-based design enables rapid UI development

**UI v2 Migration Results:**
- **Old UI System**: ~8,000+ lines of monolithic UI code (removed)
- **New ui_v2 System**: ~3,000 lines of reusable components and infrastructure
- **Panel Reduction**: Planet Panel (1,615 → ~400 lines), Ship Panel (753 → ~350 lines)
- **Architecture**: Event-driven, component-based, adapter pattern
- **Maintainability**: Massive reduction in UI maintenance overhead
- **Quality**: Modern patterns with better separation of concerns