// src/core/mod.rs
pub mod events;
pub mod types;

// Re-export commonly used types
pub use events::{EventBus, GameEvent, SystemId, PlayerCommand, SimulationEvent, StateChange};
pub use types::*;

// Import managers and systems
use crate::managers::{PlanetManager, ShipManager, FactionManager};
use crate::systems::{TimeManager, ResourceSystem, PopulationSystem, ConstructionSystem, PhysicsEngine, CombatResolver, SaveSystem};
use crate::ui::UIRenderer;

pub struct GameState {
    pub event_bus: EventBus,
    pub planet_manager: PlanetManager,
    pub ship_manager: ShipManager,
    pub faction_manager: FactionManager,
    pub time_manager: TimeManager,
    pub resource_system: ResourceSystem,
    pub population_system: PopulationSystem,
    pub construction_system: ConstructionSystem,
    pub physics_engine: PhysicsEngine,
    pub combat_resolver: CombatResolver,
    pub save_system: SaveSystem,
    pub ui_renderer: UIRenderer,
}

impl GameState {
    pub fn new() -> GameResult<Self> {
        let mut event_bus = EventBus::new();
        
        // Register system subscriptions
        event_bus.subscribe(SystemId::PlanetManager, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::PlanetManager, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::PlanetManager, events::EventType::StateChanged);
        event_bus.subscribe(SystemId::ShipManager, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::PhysicsEngine, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::PhysicsEngine, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::TimeManager, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::ResourceSystem, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::ResourceSystem, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::PopulationSystem, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::PopulationSystem, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::CombatResolver, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::CombatResolver, events::EventType::SimulationEvent);
        event_bus.subscribe(SystemId::SaveSystem, events::EventType::PlayerCommand);
        event_bus.subscribe(SystemId::UIRenderer, events::EventType::StateChanged);
        event_bus.subscribe(SystemId::UIRenderer, events::EventType::PlayerCommand);
        
        let mut state = Self {
            event_bus,
            planet_manager: PlanetManager::new(),
            ship_manager: ShipManager::new(),
            faction_manager: FactionManager::new(),
            time_manager: TimeManager::new(),
            resource_system: ResourceSystem::new(),
            population_system: PopulationSystem::new(),
            construction_system: ConstructionSystem::new(),
            physics_engine: PhysicsEngine::new(),
            combat_resolver: CombatResolver::new(),
            save_system: SaveSystem::new(),
            ui_renderer: UIRenderer::new(),
        };
        
        // Initialize with some basic content for testing
        state.initialize_demo_content()?;
        
        Ok(state)
    }
    
    pub fn fixed_update(&mut self, delta: f32) -> GameResult<()> {
        // Process input first
        self.ui_renderer.process_input(&mut self.event_bus)?;
        
        // Update systems in strict order per architecture
        self.physics_engine.update(delta, &mut self.event_bus)?;
        self.resource_system.update(delta, &mut self.event_bus)?;
        self.population_system.update(delta, &mut self.event_bus)?;
        self.construction_system.update(delta, &mut self.event_bus)?;
        self.combat_resolver.update(delta, &mut self.event_bus)?;
        self.time_manager.update(delta, &mut self.event_bus)?;
        
        // Process all queued events after system updates
        self.process_queued_events()?;
        
        Ok(())
    }
    
    fn process_queued_events(&mut self) -> GameResult<()> {
        // Process events while maintaining architectural boundaries
        let events_to_process: Vec<GameEvent> = self.event_bus.queued_events.drain(..).collect();
        
        for event in events_to_process {
            self.route_event_to_systems(event)?;
        }
        
        Ok(())
    }
    
    fn route_event_to_systems(&mut self, event: GameEvent) -> GameResult<()> {
        let event_type = match &event {
            GameEvent::PlayerCommand(_) => events::EventType::PlayerCommand,
            GameEvent::SimulationEvent(_) => events::EventType::SimulationEvent,
            GameEvent::StateChanged(_) => events::EventType::StateChanged,
        };
        
        // First collect all systems that need to handle this event
        let mut systems_to_notify = Vec::new();
        
        // Add systems in update order
        for &system_id in &self.event_bus.update_order {
            if let Some(subscriptions) = self.event_bus.subscribers.get(&system_id) {
                if subscriptions.contains(&event_type) {
                    systems_to_notify.push(system_id);
                }
            }
        }
        
        // Add managers (no specific order)
        for (&system_id, subscriptions) in &self.event_bus.subscribers {
            if !self.event_bus.update_order.contains(&system_id) && subscriptions.contains(&event_type) {
                systems_to_notify.push(system_id);
            }
        }
        
        // Now notify all systems
        for system_id in systems_to_notify {
            self.handle_system_event(system_id, &event)?;
        }
        
        Ok(())
    }
    
    /// Process per-tick simulation updates (resource production, population growth)
    fn process_tick_events(&mut self, tick: u64) -> GameResult<()> {
        // Get planet list for processing (clone to avoid borrow conflicts)
        let planet_ids: Vec<PlanetId> = self.planet_manager.get_all_planet_ids();
        
        // Process each planet's resource production and population growth
        for planet_id in planet_ids {
            // Check if planet is owned before processing
            let has_controller = {
                let planet = self.planet_manager.get_planet(planet_id)?;
                planet.controller.is_some()
            };
            
            if has_controller {
                // Calculate net resource production/consumption
                let net_production = {
                    let planet = self.planet_manager.get_planet(planet_id)?;
                    self.resource_system.calculate_planet_production(planet)?
                };
                
                // Split into positive production and negative consumption
                let mut actual_production = ResourceBundle::default();
                let mut consumption = ResourceBundle::default();
                
                // Separate production from consumption
                actual_production.minerals = net_production.minerals.max(0);
                actual_production.food = net_production.food.max(0);
                actual_production.energy = net_production.energy.max(0);
                actual_production.alloys = net_production.alloys.max(0);
                actual_production.components = net_production.components.max(0);
                actual_production.fuel = net_production.fuel.max(0);
                
                consumption.minerals = (-net_production.minerals).max(0);
                consumption.food = (-net_production.food).max(0);
                consumption.energy = (-net_production.energy).max(0);
                consumption.alloys = (-net_production.alloys).max(0);
                consumption.components = (-net_production.components).max(0);
                consumption.fuel = (-net_production.fuel).max(0);
                
                // Check if we can afford consumption before applying changes
                let can_afford_consumption = {
                    let planet = self.planet_manager.get_planet(planet_id)?;
                    let mut test_resources = planet.resources.current.clone();
                    test_resources.add(&actual_production).is_ok() && 
                    test_resources.can_afford(&consumption)
                };
                
                // Calculate how much production can actually be stored before modifying
                let available_space = {
                    let planet = self.planet_manager.get_planet(planet_id)?;
                    planet.resources.available_space()
                };
                
                let mut capped_production = ResourceBundle::default();
                // Cap production to available storage space
                capped_production.minerals = actual_production.minerals.min(available_space.minerals);
                capped_production.food = actual_production.food.min(available_space.food);
                capped_production.energy = actual_production.energy.min(available_space.energy);
                capped_production.alloys = actual_production.alloys.min(available_space.alloys);
                capped_production.components = actual_production.components.min(available_space.components);
                capped_production.fuel = actual_production.fuel.min(available_space.fuel);
                
                // Apply production and consumption safely with capacity limits
                self.planet_manager.modify_planet(planet_id, |planet| {
                    // Add the capped production (this should never exceed capacity)
                    planet.resources.current.add(&capped_production)?;
                    
                    // Only subtract consumption if we can afford it
                    if can_afford_consumption {
                        planet.resources.current.subtract(&consumption)?;
                    }
                    // If we can't afford consumption, buildings might shut down
                    // but we don't crash the game
                    
                    Ok(())
                })?;
                
                // Calculate net change for event tracking (use actual added resources)
                let mut net_change = capped_production;
                if can_afford_consumption {
                    // Subtract consumption from the change we're reporting
                    net_change.minerals -= consumption.minerals;
                    net_change.food -= consumption.food;
                    net_change.energy -= consumption.energy;
                    net_change.alloys -= consumption.alloys;
                    net_change.components -= consumption.components;
                    net_change.fuel -= consumption.fuel;
                }
                
                // Emit ResourcesProduced event for tracking (net change)
                self.event_bus.queue_event(GameEvent::SimulationEvent(
                    crate::core::events::SimulationEvent::ResourcesProduced {
                        planet: planet_id,
                        resources: net_change
                    }
                ));
                
                // Process population growth (every 10 ticks for performance)
                if tick % 10 == 0 {
                    // Get fresh planet data after resource update
                    let (population, food_available) = {
                        let updated_planet = self.planet_manager.get_planet(planet_id)?;
                        (updated_planet.population.total, updated_planet.resources.current.food)
                    };
                    
                    // Process population growth using existing method
                    self.population_system.process_planet_growth(
                        planet_id, 
                        population, 
                        food_available,
                        &mut self.event_bus
                    )?;
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_system_event(&mut self, system_id: SystemId, event: &GameEvent) -> GameResult<()> {
        // Handle tick processing centrally before routing to systems
        if let GameEvent::SimulationEvent(sim_event) = event {
            if let crate::core::events::SimulationEvent::TickCompleted(tick) = sim_event {
                self.process_tick_events(*tick)?;
            }
        }
        
        match system_id {
            SystemId::TimeManager => self.time_manager.handle_event(event),
            SystemId::PlanetManager => self.planet_manager.handle_event(event),
            SystemId::ShipManager => self.ship_manager.handle_event(event),
            SystemId::FactionManager => self.faction_manager.handle_event(event),
            SystemId::PhysicsEngine => self.physics_engine.handle_event(event),
            SystemId::ResourceSystem => self.resource_system.handle_event(event),
            SystemId::PopulationSystem => self.population_system.handle_event(event),
            SystemId::ConstructionSystem => self.construction_system.handle_event(event),
            SystemId::CombatResolver => self.combat_resolver.handle_event(event),
            SystemId::SaveSystem => {
                // Handle SaveSystem events specially since they need full GameState access
                if let GameEvent::PlayerCommand(cmd) = event {
                    match cmd {
                        PlayerCommand::SaveGame => self.handle_save_game_command(),
                        PlayerCommand::LoadGame => self.handle_load_game_command(),
                        _ => self.save_system.handle_event(event),
                    }
                } else {
                    self.save_system.handle_event(event)
                }
            },
            SystemId::UIRenderer => self.ui_renderer.handle_event(event),
        }
    }

    fn handle_save_game_command(&mut self) -> GameResult<()> {
        // Extract the save system temporarily to avoid borrow conflicts
        let save_system = std::mem::replace(&mut self.save_system, SaveSystem::new());
        let result = save_system.save_game(self);
        self.save_system = save_system;
        result
    }

    fn handle_load_game_command(&mut self) -> GameResult<()> {
        // Extract the save system temporarily to avoid borrow conflicts
        let save_system = std::mem::replace(&mut self.save_system, SaveSystem::new());
        let save_data = save_system.load_game()?;
        self.save_system = save_system;
        
        // Apply the loaded data to the game state
        self.time_manager.set_tick(save_data.tick)?;
        self.planet_manager.load_planets(save_data.planets)?;
        self.ship_manager.load_ships(save_data.ships)?;
        self.faction_manager.load_factions(save_data.factions)?;
        
        Ok(())
    }
    
    pub fn queue_event(&mut self, event: GameEvent) {
        self.event_bus.queue_event(event);
    }
    
    pub fn get_current_tick(&self) -> u64 {
        self.time_manager.get_current_tick()
    }
    
    pub fn save_game(&mut self) -> GameResult<()> {
        self.queue_event(GameEvent::PlayerCommand(PlayerCommand::SaveGame));
        Ok(())
    }
    
    pub fn load_game(&mut self) -> GameResult<()> {
        self.queue_event(GameEvent::PlayerCommand(PlayerCommand::LoadGame));
        Ok(())
    }
    
    pub fn render(&mut self, interpolation: f32) -> GameResult<()> {
        // Create a temporary collection for events generated during rendering
        let mut render_events = Vec::new();
        
        // Temporarily extract ui_renderer to avoid borrow conflicts
        let mut ui_renderer = std::mem::replace(&mut self.ui_renderer, UIRenderer::new());
        let result = ui_renderer.render_with_events(self, interpolation, &mut render_events);
        self.ui_renderer = ui_renderer;
        
        // Queue any events that were generated during rendering
        for event in render_events {
            self.event_bus.queue_event(event);
        }
        
        result
    }
    
    /// Processes queued events manually - used for testing.
    /// This allows tests to trigger event processing without running fixed_update.
    /// Available in both unit tests and integration tests.
    pub fn process_queued_events_for_test(&mut self) -> GameResult<()> {
        self.process_queued_events()
    }
    
    /// Initialize some demo content for testing and initial gameplay
    fn initialize_demo_content(&mut self) -> GameResult<()> {
        // Create a few test planets with basic properties
        let planet1 = self.planet_manager.create_planet(
            OrbitalElements {
                semi_major_axis: 1.0,
                period: 365.0,
                phase: 0.0,
            },
            Some(0), // Player faction
        )?;
        
        let planet2 = self.planet_manager.create_planet(
            OrbitalElements {
                semi_major_axis: 1.5,
                period: 500.0,
                phase: 1.57, // 90 degrees offset
            },
            None, // Neutral
        )?;
        
        let planet3 = self.planet_manager.create_planet(
            OrbitalElements {
                semi_major_axis: 0.7,
                period: 200.0,
                phase: 3.14, // 180 degrees offset
            },
            None, // Neutral
        )?;
        
        // Add some basic resources to the player planet
        self.planet_manager.add_resources(planet1, ResourceBundle {
            minerals: 500,
            food: 300,
            energy: 200,
            alloys: 50,
            components: 25,
            fuel: 100,
        })?;
        
        // Add some population to the player planet
        self.planet_manager.update_population(planet1, 1000)?;
        
        // Create a test ship for the player
        let ship1 = self.ship_manager.create_ship(
            ShipClass::Scout,
            Vector2::new(50.0, 50.0), // Starting position
            0, // Player owner
        )?;
        
        println!("Demo content initialized: {} planets and {} ship created with IDs: planets {}, {}, {} and ship {}", 
                 3, 1, planet1, planet2, planet3, ship1);
        Ok(())
    }
}

// System trait definition
pub trait GameSystem {
    fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()>;
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()>;
}