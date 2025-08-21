// src/core/asset_types.rs
//! Enhanced asset system for scalable save/load operations
//! Supports unique assets, collections, and hierarchical data structures

use crate::core::types::{PlanetId, ShipId, FactionId, Vector2, ResourceBundle, GameResult, GameError};
use std::collections::{HashMap, BTreeSet};
use std::hash::Hash;

/// Unique identifier for any asset in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId(pub u64);

impl AssetId {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        AssetId(timestamp)
    }

    pub fn from_u64(id: u64) -> Self {
        AssetId(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Asset rarity classifications for unique items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetRarity {
    Common,
    Uncommon, 
    Rare,
    Epic,
    Legendary,
    Artifact,  // Unique one-of-a-kind items
}

/// Categories of assets that can exist in the game world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetCategory {
    PlanetaryStructure,
    ShipEquipment,
    ShipModule,
    FactionArtifact,
    SpaceStructure,
    Resource,
    Technology,
    Personnel,
}

/// Base trait for all assets in the game
pub trait Asset {
    fn asset_id(&self) -> AssetId;
    fn asset_type(&self) -> AssetType;
    fn category(&self) -> AssetCategory;
    fn rarity(&self) -> AssetRarity;
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn validate(&self) -> GameResult<()>;
}

/// Comprehensive asset type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    // Planetary assets
    PlanetaryFacility(PlanetaryFacility),
    PlanetaryDefense(PlanetaryDefense),
    PlanetaryWonder(PlanetaryWonder),
    
    // Ship assets
    ShipWeapon(ShipWeapon),
    ShipArmor(ShipArmor),
    ShipEngine(ShipEngine),
    ShipComputer(ShipComputer),
    ShipSpecialModule(ShipSpecialModule),
    
    // Space structures
    SpaceStation(SpaceStation),
    StarBase(StarBase),
    Megastructure(Megastructure),
    
    // Faction artifacts
    RelicArtifact(RelicArtifact),
    TechnologyBlueprint(TechnologyBlueprint),
    UniquePersonnel(UniquePersonnel),
    
    // Resource deposits and special materials
    ResourceDeposit(ResourceDeposit),
    ExoticMaterial(ExoticMaterial),
}

/// Planetary facility assets that can be built on planets
#[derive(Debug, Clone, PartialEq)]
pub struct PlanetaryFacility {
    pub asset_id: AssetId,
    pub facility_type: PlanetaryFacilityType,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub resource_bonus: ResourceBundle,
    pub population_capacity: i32,
    pub power_requirement: i32,
    pub maintenance_cost: ResourceBundle,
    pub special_effects: Vec<SpecialEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanetaryFacilityType {
    AdvancedMine,
    HydroponicFarm,
    FusionReactor,
    NanoFactory,
    QuantumLab,
    MegaSpaceport,
    PlanetaryShield,
    ArtificialHabitat,
    TerraformingStation,
}

/// Planetary defense systems
#[derive(Debug, Clone, PartialEq)]
pub struct PlanetaryDefense {
    pub asset_id: AssetId,
    pub defense_type: PlanetaryDefenseType,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub defense_rating: i32,
    pub range: f32,
    pub power_requirement: i32,
    pub special_abilities: Vec<DefenseAbility>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanetaryDefenseType {
    IonCannon,
    ParticleBeam,
    MissileArray,
    PlanetaryShield,
    GraviticMine,
}

/// Unique planetary wonders
#[derive(Debug, Clone, PartialEq)]
pub struct PlanetaryWonder {
    pub asset_id: AssetId,
    pub wonder_type: PlanetaryWonderType,
    pub name: String,
    pub description: String,
    pub completion_requirements: ResourceBundle,
    pub time_to_build: u64,
    pub global_effects: Vec<GlobalEffect>,
    pub victory_points: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanetaryWonderType {
    RingWorld,
    DysonSphere,
    QuantumComputer,
    GalacticLibrary,
    AscensionGate,
}

/// Ship weapon systems
#[derive(Debug, Clone, PartialEq)]
pub struct ShipWeapon {
    pub asset_id: AssetId,
    pub weapon_type: WeaponType,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub damage: i32,
    pub range: f32,
    pub accuracy: f32,
    pub power_requirement: i32,
    pub special_properties: Vec<WeaponProperty>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    KineticCannon,
    LaserBeam,
    PlasmaLauncher,
    MissileSystem,
    DisruptorRay,
    GraviticTorpedo,
}

/// Ship armor systems
#[derive(Debug, Clone, PartialEq)]
pub struct ShipArmor {
    pub asset_id: AssetId,
    pub armor_type: ArmorType,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub protection: i32,
    pub weight: f32,
    pub special_resistances: Vec<DamageResistance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArmorType {
    CompositePlating,
    EnergyShield,
    AdaptiveArmor,
    QuantumBarrier,
    NeutroniumHull,
}

/// Ship engine systems
#[derive(Debug, Clone, PartialEq)]
pub struct ShipEngine {
    pub asset_id: AssetId,
    pub engine_type: EngineType,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub thrust: f32,
    pub efficiency: f32,
    pub fuel_consumption: f32,
    pub special_capabilities: Vec<EngineCapability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineType {
    ChemicalRocket,
    IonDrive,
    FusionTorch,
    WarpDrive,
    JumpDrive,
    QuantumTunnel,
}

/// Ship computer systems
#[derive(Debug, Clone, PartialEq)]
pub struct ShipComputer {
    pub asset_id: AssetId,
    pub computer_type: ComputerType,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub processing_power: i32,
    pub targeting_bonus: f32,
    pub evasion_bonus: f32,
    pub special_functions: Vec<ComputerFunction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComputerType {
    BasicAI,
    TacticalComputer,
    QuantumProcessor,
    SentientAI,
    PrecursorCore,
}

/// Special ship modules
#[derive(Debug, Clone, PartialEq)]
pub struct ShipSpecialModule {
    pub asset_id: AssetId,
    pub module_type: SpecialModuleType,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub power_requirement: i32,
    pub special_abilities: Vec<ModuleAbility>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialModuleType {
    SensorArray,
    CloakingDevice,
    ShieldRecharger,
    CargoExpansion,
    ScienceLab,
    PlanetCracker,
}

/// Space stations
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceStation {
    pub asset_id: AssetId,
    pub station_type: SpaceStationType,
    pub position: Vector2,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub construction_cost: ResourceBundle,
    pub maintenance_cost: ResourceBundle,
    pub capabilities: Vec<StationCapability>,
    pub docking_capacity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpaceStationType {
    TradingPost,
    RefuelStation,
    RepairYard,
    ResearchStation,
    MilitaryOutpost,
    DeepSpaceFortress,
}

/// Star bases for system control
#[derive(Debug, Clone, PartialEq)]
pub struct StarBase {
    pub asset_id: AssetId,
    pub base_type: StarBaseType,
    pub position: Vector2,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub defense_rating: i32,
    pub fleet_capacity: u8,
    pub command_range: f32,
    pub special_systems: Vec<BaseSystem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StarBaseType {
    CommandCenter,
    Fortress,
    Shipyard,
    SensorGrid,
    ResourceProcessor,
}

/// Megastructures
#[derive(Debug, Clone, PartialEq)]
pub struct Megastructure {
    pub asset_id: AssetId,
    pub structure_type: MegastructureType,
    pub position: Vector2,
    pub construction_progress: f32,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: String,
    pub construction_cost: ResourceBundle,
    pub maintenance_cost: ResourceBundle,
    pub effects: Vec<GlobalEffect>,
    pub completion_time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MegastructureType {
    RingWorld,
    DysonSphere,
    AldersonDisk,
    MatrioshkaBrain,
    StellarEngine,
}

/// Faction artifacts and relics
#[derive(Debug, Clone, PartialEq)]
pub struct RelicArtifact {
    pub asset_id: AssetId,
    pub artifact_type: ArtifactType,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: String,
    pub discovery_location: Option<PlanetId>,
    pub activation_cost: ResourceBundle,
    pub powers: Vec<ArtifactPower>,
    pub restrictions: Vec<ArtifactRestriction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactType {
    PrecursorRelic,
    AncientTechnology,
    PsionicCrystal,
    TemporalDevice,
    DimensionalKey,
}

/// Technology blueprints
#[derive(Debug, Clone, PartialEq)]
pub struct TechnologyBlueprint {
    pub asset_id: AssetId,
    pub tech_type: TechnologyType,
    pub tier: u8,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: String,
    pub research_cost: ResourceBundle,
    pub prerequisites: Vec<TechnologyType>,
    pub unlocks: Vec<UnlockableAsset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TechnologyType {
    Physics,
    Engineering,
    Biology,
    Computing,
    Psionics,
    Precursor,
}

/// Unique personnel assets
#[derive(Debug, Clone, PartialEq)]
pub struct UniquePersonnel {
    pub asset_id: AssetId,
    pub personnel_type: PersonnelType,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: String,
    pub assigned_to: Option<AssignmentLocation>,
    pub skills: Vec<PersonnelSkill>,
    pub traits: Vec<PersonnelTrait>,
    pub experience_level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonnelType {
    Admiral,
    General,
    Governor,
    Scientist,
    Engineer,
    Diplomat,
    Spy,
}

/// Resource deposits
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceDeposit {
    pub asset_id: AssetId,
    pub deposit_type: DepositType,
    pub location: Vector2,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: Option<String>,
    pub yield_rate: ResourceBundle,
    pub total_reserves: ResourceBundle,
    pub depletion_time: Option<u64>,
    pub extraction_requirements: Vec<ExtractionRequirement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepositType {
    MineralVein,
    GasGiant,
    AsteroidField,
    ExoticMatter,
    DarkEnergy,
    Neutronium,
}

/// Exotic materials
#[derive(Debug, Clone, PartialEq)]
pub struct ExoticMaterial {
    pub asset_id: AssetId,
    pub material_type: ExoticMaterialType,
    pub quantity: i32,
    pub rarity: AssetRarity,
    pub name: String,
    pub description: String,
    pub properties: Vec<MaterialProperty>,
    pub applications: Vec<MaterialApplication>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExoticMaterialType {
    Neutronium,
    DarkMatter,
    QuarkMatter,
    StrangeMatter,
    TimeDistortion,
    PsionicCrystal,
}

/// Location where assets can be assigned
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignmentLocation {
    Planet(PlanetId),
    Ship(ShipId),
    Faction(FactionId),
    Space(Vector2),
}

impl Eq for AssignmentLocation {}

impl std::hash::Hash for AssignmentLocation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            AssignmentLocation::Planet(id) => {
                0u8.hash(state);
                id.hash(state);
            }
            AssignmentLocation::Ship(id) => {
                1u8.hash(state);
                id.hash(state);
            }
            AssignmentLocation::Faction(id) => {
                2u8.hash(state);
                id.hash(state);
            }
            AssignmentLocation::Space(vector) => {
                3u8.hash(state);
                // Hash Vector2 by converting floats to bits for deterministic hashing
                vector.x.to_bits().hash(state);
                vector.y.to_bits().hash(state);
            }
        }
    }
}

/// Asset collection for managing groups of related assets
#[derive(Debug, Clone)]
pub struct AssetCollection {
    pub collection_id: AssetId,
    pub name: String,
    pub description: Option<String>,
    pub assets: BTreeSet<AssetId>,
    pub collection_type: CollectionType,
    pub owner: Option<FactionId>,
    pub location: Option<AssignmentLocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectionType {
    ShipLoadout,
    PlanetaryInfrastructure,
    FactionArtifacts,
    FleetConfiguration,
    TechnologySuite,
    DefenseGrid,
}

/// Comprehensive asset registry for tracking all assets
#[derive(Debug, Clone)]
pub struct AssetRegistry {
    pub assets: HashMap<AssetId, AssetType>,
    pub collections: HashMap<AssetId, AssetCollection>,
    pub location_index: HashMap<AssignmentLocation, BTreeSet<AssetId>>,
    pub category_index: HashMap<AssetCategory, BTreeSet<AssetId>>,
    pub rarity_index: HashMap<AssetRarity, BTreeSet<AssetId>>,
    pub next_asset_id: u64,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            collections: HashMap::new(),
            location_index: HashMap::new(),
            category_index: HashMap::new(),
            rarity_index: HashMap::new(),
            next_asset_id: 1,
        }
    }
    
    pub fn generate_asset_id(&mut self) -> AssetId {
        let id = AssetId::from_u64(self.next_asset_id);
        self.next_asset_id += 1;
        id
    }
    
    pub fn register_asset(&mut self, asset: AssetType) -> GameResult<AssetId> {
        let asset_id = match &asset {
            AssetType::PlanetaryFacility(f) => f.asset_id,
            AssetType::ShipWeapon(w) => w.asset_id,
            // ... handle all asset types
            _ => return Err(GameError::SystemError("Unhandled asset type in registration".into())),
        };
        
        if self.assets.contains_key(&asset_id) {
            return Err(GameError::InvalidOperation(format!("Asset ID {} already exists", asset_id.as_u64())));
        }
        
        // Update indices
        let category = match &asset {
            AssetType::PlanetaryFacility(_) => AssetCategory::PlanetaryStructure,
            AssetType::ShipWeapon(_) => AssetCategory::ShipEquipment,
            // ... handle all categories
            _ => AssetCategory::Resource,
        };
        
        let rarity = match &asset {
            AssetType::PlanetaryFacility(f) => f.rarity,
            AssetType::ShipWeapon(w) => w.rarity,
            // ... handle all asset types
            _ => AssetRarity::Common,
        };
        
        self.category_index.entry(category).or_insert_with(BTreeSet::new).insert(asset_id);
        self.rarity_index.entry(rarity).or_insert_with(BTreeSet::new).insert(asset_id);
        
        self.assets.insert(asset_id, asset);
        Ok(asset_id)
    }
    
    pub fn assign_asset(&mut self, asset_id: AssetId, location: AssignmentLocation) -> GameResult<()> {
        if !self.assets.contains_key(&asset_id) {
            return Err(GameError::InvalidTarget(format!("Asset {} not found", asset_id.as_u64())));
        }
        
        self.location_index.entry(location).or_insert_with(BTreeSet::new).insert(asset_id);
        Ok(())
    }
    
    pub fn get_assets_at_location(&self, location: &AssignmentLocation) -> Vec<AssetId> {
        self.location_index.get(location).map_or_else(Vec::new, |set| set.iter().copied().collect())
    }
    
    pub fn get_assets_by_category(&self, category: &AssetCategory) -> Vec<AssetId> {
        self.category_index.get(category).map_or_else(Vec::new, |set| set.iter().copied().collect())
    }
    
    pub fn get_assets_by_rarity(&self, rarity: &AssetRarity) -> Vec<AssetId> {
        self.rarity_index.get(rarity).map_or_else(Vec::new, |set| set.iter().copied().collect())
    }
    
    pub fn create_collection(&mut self, name: String, collection_type: CollectionType) -> AssetId {
        let collection_id = self.generate_asset_id();
        let collection = AssetCollection {
            collection_id,
            name,
            description: None,
            assets: BTreeSet::new(),
            collection_type,
            owner: None,
            location: None,
        };
        
        self.collections.insert(collection_id, collection);
        collection_id
    }
    
    pub fn add_to_collection(&mut self, collection_id: AssetId, asset_id: AssetId) -> GameResult<()> {
        if let Some(collection) = self.collections.get_mut(&collection_id) {
            collection.assets.insert(asset_id);
            Ok(())
        } else {
            Err(GameError::InvalidTarget(format!("Collection {} not found", collection_id.as_u64())))
        }
    }
}

// Supporting enums and structs for special effects and abilities
#[derive(Debug, Clone, PartialEq)]
pub enum SpecialEffect {
    ResourceBonus(ResourceBundle),
    PopulationGrowth(f32),
    ResearchBonus(f32),
    DefenseBonus(i32),
    ProductionEfficiency(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalEffect {
    FactionResourceBonus(ResourceBundle),
    TechnologyUnlock(TechnologyType),
    VictoryPoints(i32),
    FleetSpeedBonus(f32),
    DiplomacyBonus(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefenseAbility {
    Regeneration,
    EnergyAbsorption,
    Counterbattery,
    AreaDenial,
    ShieldReflection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WeaponProperty {
    ArmorPiercing,
    ShieldPenetrating,
    AreaOfEffect,
    EnergyDrain,
    DisruptorField,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DamageResistance {
    Kinetic(f32),
    Energy(f32),
    Explosive(f32),
    Psionic(f32),
    Temporal(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineCapability {
    FastTravel,
    StealthMode,
    GravityResistance,
    DimensionalPhasing,
    TemporalShift,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComputerFunction {
    AutoTargeting,
    EvasiveManeuvers,
    FleetCoordination,
    TacticalAnalysis,
    PredictiveAlgorithms,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleAbility {
    ExtendedRange,
    EnhancedYield,
    SelfRepair,
    EnergyEfficiency,
    MultiFunctionality,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StationCapability {
    ShipRepair,
    ResourceProcessing,
    Research,
    Manufacturing,
    DefenseCoordination,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseSystem {
    CommandAndControl,
    SensorNetwork,
    WeaponBattery,
    ShieldGenerator,
    HangarBay,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactPower {
    ResourceGeneration(ResourceBundle),
    TechnologyAcceleration,
    FleetEnhancement,
    PlanetaryTransformation,
    DimensionalGateway,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactRestriction {
    SingleUse,
    FactionExclusive,
    LocationBound,
    TechnologyRequired(TechnologyType),
    ResourceCost(ResourceBundle),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnlockableAsset {
    Building(PlanetaryFacilityType),
    ShipComponent(WeaponType),
    SpaceStructure(SpaceStationType),
    SpecialAbility(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PersonnelSkill {
    Leadership(u8),
    Tactics(u8),
    Engineering(u8),
    Diplomacy(u8),
    Espionage(u8),
    Research(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PersonnelTrait {
    Brilliant,
    Cautious,
    Aggressive,
    Diplomatic,
    Innovative,
    Veteran,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionRequirement {
    Technology(TechnologyType),
    Facility(PlanetaryFacilityType),
    Personnel(PersonnelType),
    Resources(ResourceBundle),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialProperty {
    SelfReplicating,
    EnergyConductive,
    Indestructible,
    Phaseshift,
    QuantumEntangled,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialApplication {
    ShipConstruction,
    PlanetaryEngineering,
    WeaponManufacturing,
    DefenseSystem,
    Megastructure,
}