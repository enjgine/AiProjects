// src/core/types.rs
use std::fmt;

// Core type aliases
pub type GameResult<T> = Result<T, GameError>;
pub type PlanetId = u32;
pub type ShipId = u32;
pub type FactionId = u8;
pub type PlayerId = u8;

// Error handling
#[derive(Debug)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Default)]
pub struct CargoHold {
    pub resources: ResourceBundle,
    pub population: i32,
    pub capacity: i32,
}

// Physics
#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Default for Vector2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
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