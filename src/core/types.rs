// src/core/types.rs
use std::fmt;
use std::ops::{Add, Sub, AddAssign, SubAssign};

// Core type aliases
pub type GameResult<T> = Result<T, GameError>;
pub type PlanetId = u32;
pub type ShipId = u32;
pub type FactionId = u8;
pub type PlayerId = u8;

// Error handling
#[derive(Debug, Clone)]
pub enum GameError {
    InvalidOperation(String),
    InsufficientResources { required: ResourceBundle, available: ResourceBundle },
    InvalidTarget(String),
    SystemError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            GameError::InsufficientResources { required, available } => {
                write!(f, "Insufficient resources. Required: {:?}, Available: {:?}", required, available)
            }
            GameError::InvalidTarget(msg) => write!(f, "Invalid target: {}", msg),
            GameError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for GameError {}

// Resource system
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ResourceBundle {
    pub minerals: i32,
    pub food: i32,
    pub energy: i32,
    pub alloys: i32,
    pub components: i32,
    pub fuel: i32,
}

impl ResourceBundle {
    pub fn validate_non_negative(&self) -> GameResult<()> {
        if self.minerals < 0 || self.food < 0 || self.energy < 0 
           || self.alloys < 0 || self.components < 0 || self.fuel < 0 {
            return Err(GameError::InvalidOperation("Resources cannot be negative".into()));
        }
        Ok(())
    }
    
    pub fn can_afford(&self, cost: &ResourceBundle) -> bool {
        self.minerals >= cost.minerals &&
        self.food >= cost.food &&
        self.energy >= cost.energy &&
        self.alloys >= cost.alloys &&
        self.components >= cost.components &&
        self.fuel >= cost.fuel
    }
    
    pub fn subtract(&mut self, cost: &ResourceBundle) -> GameResult<()> {
        if !self.can_afford(cost) {
            return Err(GameError::InsufficientResources {
                required: *cost,
                available: *self,
            });
        }
        self.minerals -= cost.minerals;
        self.food -= cost.food;
        self.energy -= cost.energy;
        self.alloys -= cost.alloys;
        self.components -= cost.components;
        self.fuel -= cost.fuel;
        Ok(())
    }
    
    pub fn add(&mut self, resources: &ResourceBundle) -> GameResult<()> {
        self.minerals = self.minerals.saturating_add(resources.minerals);
        self.food = self.food.saturating_add(resources.food);
        self.energy = self.energy.saturating_add(resources.energy);
        self.alloys = self.alloys.saturating_add(resources.alloys);
        self.components = self.components.saturating_add(resources.components);
        self.fuel = self.fuel.saturating_add(resources.fuel);
        Ok(())
    }
    
    pub fn total(&self) -> i64 {
        self.minerals as i64 + self.food as i64 + self.energy as i64 
        + self.alloys as i64 + self.components as i64 + self.fuel as i64
    }
}

impl Add for ResourceBundle {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self {
            minerals: self.minerals + other.minerals,
            food: self.food + other.food,
            energy: self.energy + other.energy,
            alloys: self.alloys + other.alloys,
            components: self.components + other.components,
            fuel: self.fuel + other.fuel,
        }
    }
}

impl Sub for ResourceBundle {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self {
            minerals: self.minerals - other.minerals,
            food: self.food - other.food,
            energy: self.energy - other.energy,
            alloys: self.alloys - other.alloys,
            components: self.components - other.components,
            fuel: self.fuel - other.fuel,
        }
    }
}

impl AddAssign for ResourceBundle {
    fn add_assign(&mut self, other: Self) {
        self.minerals += other.minerals;
        self.food += other.food;
        self.energy += other.energy;
        self.alloys += other.alloys;
        self.components += other.components;
        self.fuel += other.fuel;
    }
}

impl SubAssign for ResourceBundle {
    fn sub_assign(&mut self, other: Self) {
        self.minerals -= other.minerals;
        self.food -= other.food;
        self.energy -= other.energy;
        self.alloys -= other.alloys;
        self.components -= other.components;
        self.fuel -= other.fuel;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    Minerals,
    Food,
    Energy,
    Alloys,
    Components,
    Fuel,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceStorage {
    pub current: ResourceBundle,
    pub capacity: ResourceBundle,
}

impl ResourceStorage {
    pub fn available_space(&self) -> ResourceBundle {
        ResourceBundle {
            minerals: self.capacity.minerals - self.current.minerals,
            food: self.capacity.food - self.current.food,
            energy: self.capacity.energy - self.current.energy,
            alloys: self.capacity.alloys - self.current.alloys,
            components: self.capacity.components - self.current.components,
            fuel: self.capacity.fuel - self.current.fuel,
        }
    }
    
    pub fn can_store(&self, resources: &ResourceBundle) -> bool {
        let space = self.available_space();
        space.can_afford(resources)
    }
    
    pub fn validate(&self) -> GameResult<()> {
        self.current.validate_non_negative()?;
        self.capacity.validate_non_negative()?;
        
        if !self.capacity.can_afford(&self.current) {
            return Err(GameError::InvalidOperation("Current resources exceed capacity".into()));
        }
        Ok(())
    }
}

// Population system
#[derive(Debug, Clone, Default)]
pub struct Demographics {
    pub total: i32,
    pub growth_rate: f32,
    pub allocation: WorkerAllocation,
}

#[derive(Debug, Clone, Default)]
pub struct WorkerAllocation {
    pub agriculture: i32,
    pub mining: i32,
    pub industry: i32,
    pub research: i32,
    pub military: i32,
    pub unassigned: i32,
}

impl WorkerAllocation {
    pub fn validate(&self, total: i32) -> GameResult<()> {
        if self.agriculture < 0 || self.mining < 0 || self.industry < 0 
           || self.research < 0 || self.military < 0 || self.unassigned < 0 {
            return Err(GameError::InvalidOperation("Worker allocations cannot be negative".into()));
        }
        
        let sum = self.agriculture + self.mining + self.industry 
                + self.research + self.military + self.unassigned;
        if sum != total {
            return Err(GameError::InvalidOperation(
                format!("Worker allocation {} doesn't match total {}", sum, total)
            ));
        }
        Ok(())
    }
}

// Buildings and construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Mine,
    Farm,
    PowerPlant,
    Factory,
    ResearchLab,
    Spaceport,
    DefensePlatform,
    StorageFacility,
    Habitat,
}

#[derive(Debug, Clone)]
pub struct Building {
    pub building_type: BuildingType,
    pub tier: u8,
    pub operational: bool,
}

// Ships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShipClass {
    Scout,
    Transport,
    Warship,
    Colony,
}

#[derive(Debug, Clone)]
pub struct Ship {
    pub id: ShipId,
    pub ship_class: ShipClass,
    pub position: Vector2,
    pub trajectory: Option<Trajectory>,
    pub cargo: CargoHold,
    pub fuel: f32,
    pub owner: FactionId,
}

impl Ship {
    pub fn validate(&self) -> GameResult<()> {
        if self.fuel < 0.0 {
            return Err(GameError::InvalidOperation("Ship fuel cannot be negative".into()));
        }
        self.cargo.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CargoHold {
    pub resources: ResourceBundle,
    pub population: i32,
    pub capacity: i32,
}

impl CargoHold {
    pub fn current_load(&self) -> i32 {
        self.resources.total() as i32 + self.population
    }
    
    pub fn available_space(&self) -> i32 {
        self.capacity - self.current_load()
    }
    
    pub fn can_load(&self, additional_resources: &ResourceBundle, additional_population: i32) -> bool {
        let additional_load = additional_resources.total() as i32 + additional_population;
        self.available_space() >= additional_load
    }
    
    pub fn validate(&self) -> GameResult<()> {
        if self.capacity < 0 {
            return Err(GameError::InvalidOperation("Cargo capacity cannot be negative".into()));
        }
        if self.population < 0 {
            return Err(GameError::InvalidOperation("Population cannot be negative".into()));
        }
        if self.current_load() > self.capacity {
            return Err(GameError::InvalidOperation("Cargo exceeds capacity".into()));
        }
        Ok(())
    }
}

// Physics
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Default for Vector2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn distance_to(&self, other: &Vector2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    pub fn normalize(&self) -> Vector2 {
        let mag = self.magnitude();
        if mag == 0.0 {
            *self
        } else {
            Vector2 { x: self.x / mag, y: self.y / mag }
        }
    }
    
    pub fn dot(&self, other: &Vector2) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OrbitalElements {
    pub semi_major_axis: f32,  // AU
    pub period: f32,            // timesteps
    pub phase: f32,             // radians
}

impl Default for OrbitalElements {
    fn default() -> Self {
        Self {
            semi_major_axis: 5.0,
            period: 365.0,
            phase: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Trajectory {
    pub origin: Vector2,
    pub destination: Vector2,
    pub departure_time: u64,
    pub arrival_time: u64,
    pub fuel_cost: f32,
}

// Planets
#[derive(Debug, Clone)]
pub struct Planet {
    pub id: PlanetId,
    pub position: OrbitalElements,
    pub resources: ResourceStorage,
    pub population: Demographics,
    pub developments: Vec<Building>,
    pub controller: Option<FactionId>,
}

// Factions
#[derive(Debug, Clone)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub is_player: bool,
    pub ai_type: AIPersonality,
    pub score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AIPersonality {
    Aggressive,
    Balanced,
    Economic,
}

// Combat
#[derive(Debug, Clone)]
pub struct CombatOutcome {
    pub winner: FactionId,
    pub attacker_losses: Vec<ShipId>,
    pub defender_losses: Vec<ShipId>,
}

// Victory conditions
#[derive(Debug, Clone, Copy)]
pub enum VictoryType {
    Economic,
    Population,
    Military,
    Timeout,
}

// Game modes for menu/game state management
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    MainMenu,
    InGame,
}