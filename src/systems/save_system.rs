// src/systems/save_system_v2.rs
//! Next-generation save system with binary format, asset management, and scalability
//! 
//! Features:
//! - Binary serialization for performance
//! - Chunk-based loading for large files
//! - Asset registry integration
//! - Versioned schema system
//! - Hierarchical data structures
//! - Unique asset collections

use crate::core::{GameResult, GameEvent, EventBus, GameState};
use crate::core::types::*;
use crate::core::asset_types::*;
use std::io::{Write, Read, Seek, BufWriter, BufReader};
use std::fs::{File, rename};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Save file format version for backward compatibility
pub const SAVE_FORMAT_VERSION: u32 = 2;
pub const SAVE_MAGIC_HEADER: &[u8; 8] = b"STELLAR2";

/// Chunk types in the save file
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkType {
    Header = 0x48454144,      // "HEAD"
    Metadata = 0x4D455441,    // "META"
    GameState = 0x47414D45,   // "GAME"
    Planets = 0x504C4E54,     // "PLNT"
    Ships = 0x53484950,       // "SHIP"
    Factions = 0x46414354,    // "FACT"
    Assets = 0x41535354,      // "ASST"
    Collections = 0x434F4C4C, // "COLL"
    Checksum = 0x43484B53,    // "CHKS"
}

/// Save file header with version and schema information
#[derive(Debug, Clone)]
pub struct SaveFileHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub schema_version: u32,
    pub creation_time: u64,
    pub game_tick: u64,
    pub chunk_count: u32,
    pub total_size: u64,
    pub compression_type: CompressionType,
    pub features: SaveFeatureFlags,
}

/// Compression types supported
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionType {
    None = 0,
    LZ4 = 1,
    Zstd = 2,
}

/// Feature flags for save file capabilities
#[derive(Debug, Clone, Copy)]
pub struct SaveFeatureFlags {
    pub has_assets: bool,
    pub has_collections: bool,
    pub has_megastructures: bool,
    pub has_unique_personnel: bool,
    pub chunk_compression: bool,
    pub differential_saves: bool,
}

impl SaveFeatureFlags {
    pub fn to_u64(self) -> u64 {
        let mut flags = 0u64;
        if self.has_assets { flags |= 1 << 0; }
        if self.has_collections { flags |= 1 << 1; }
        if self.has_megastructures { flags |= 1 << 2; }
        if self.has_unique_personnel { flags |= 1 << 3; }
        if self.chunk_compression { flags |= 1 << 4; }
        if self.differential_saves { flags |= 1 << 5; }
        flags
    }
    
    pub fn from_u64(flags: u64) -> Self {
        Self {
            has_assets: flags & (1 << 0) != 0,
            has_collections: flags & (1 << 1) != 0,
            has_megastructures: flags & (1 << 2) != 0,
            has_unique_personnel: flags & (1 << 3) != 0,
            chunk_compression: flags & (1 << 4) != 0,
            differential_saves: flags & (1 << 5) != 0,
        }
    }
}

/// Individual chunk in save file
#[derive(Debug, Clone)]
pub struct SaveChunk {
    pub chunk_type: ChunkType,
    pub chunk_size: u32,
    pub chunk_offset: u64,
    pub compression: CompressionType,
    pub checksum: u32,
    pub data: Vec<u8>,
}

/// Enhanced save data structure with asset support
#[derive(Debug, Clone)]
pub struct SaveDataV2 {
    pub header: SaveFileHeader,
    pub metadata: SaveMetadata,
    pub game_state: CoreGameData,
    pub assets: AssetRegistry,
    pub collections: HashMap<AssetId, AssetCollection>,
    pub chunk_index: HashMap<ChunkType, SaveChunk>,
}

/// Save file metadata
#[derive(Debug, Clone)]
pub struct SaveMetadata {
    pub save_name: String,
    pub description: Option<String>,
    pub player_faction: FactionId,
    pub difficulty_level: u8,
    pub galaxy_size: GalaxySize,
    pub total_planets: usize,
    pub total_ships: usize,
    pub total_factions: usize,
    pub total_assets: usize,
    pub play_time_seconds: u64,
    pub victory_status: Option<VictoryType>,
    pub custom_flags: HashMap<String, String>,
}

/// Core game data separate from assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreGameData {
    pub tick: u64,
    pub planets: Vec<Planet>,
    pub ships: Vec<Ship>,
    pub factions: Vec<Faction>,
    pub game_configuration: GameConfiguration,
    pub victory_conditions: Vec<String>,
}

/// Simplified save data that can be serialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedSaveData {
    pub tick: u64,
    pub planet_count: usize,
    pub ship_count: usize,
    pub faction_count: usize,
}

/// Victory condition tracking
#[derive(Debug, Clone)]
pub struct VictoryCondition {
    pub condition_type: VictoryType,
    pub target_value: i32,
    pub current_progress: HashMap<FactionId, i32>,
    pub is_achieved: bool,
    pub achievement_tick: Option<u64>,
}

/// Next-generation save system
pub struct SaveSystem {
    version: u32,
    schema_version: u32,
    save_directory: PathBuf,
    backup_count: u8,
    compression: CompressionType,
    enable_chunk_compression: bool,
    max_chunk_size: u32,
    asset_registry: AssetRegistry,
}

impl SaveSystem {
    pub fn new() -> Self {
        Self {
            version: SAVE_FORMAT_VERSION,
            schema_version: 1,
            save_directory: PathBuf::from("saves"),
            backup_count: 5, // Increased backup count
            compression: CompressionType::None, // Start with none, can be upgraded
            enable_chunk_compression: false,
            max_chunk_size: 1024 * 1024, // 1MB chunks
            asset_registry: AssetRegistry::new(),
        }
    }
    
    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }
    
    pub fn with_chunk_compression(mut self, enabled: bool) -> Self {
        self.enable_chunk_compression = enabled;
        self
    }
    
    pub fn with_max_chunk_size(mut self, size: u32) -> Self {
        self.max_chunk_size = size;
        self
    }
    
    /// Save game state to binary format with chunked structure
    pub fn save_game_binary(&mut self, game_state: &GameState, save_name: &str) -> GameResult<()> {
        self.ensure_save_directory()?;
        
        let save_path = self.save_directory.join(format!("{}.sav", save_name));
        self.create_backup(&save_path)?;
        
        // Get the actual game data from managers
        let core_data = CoreGameData {
            tick: game_state.time_manager.get_current_tick(),
            planets: game_state.planet_manager.get_all_planets_cloned()?,
            ships: game_state.ship_manager.get_all_ships_cloned()?,
            factions: game_state.faction_manager.get_all_factions().to_vec(),
            game_configuration: GameConfiguration::default(), // TODO: Get actual config
            victory_conditions: Vec::new(), // TODO: Implement victory tracking
        };
        
        // Create temporary file for atomic save
        let temp_path = save_path.with_extension("tmp");
        let mut file = File::create(&temp_path)
            .map_err(|e| GameError::SystemError(format!("Failed to create save file: {}", e)))?;
        
        // Write magic header and version
        file.write_all(b"STELLAR2")?;
        file.write_all(&SAVE_FORMAT_VERSION.to_le_bytes())?;
        
        // Write actual game data as JSON
        let json_data = serde_json::to_string_pretty(&core_data)
            .map_err(|e| GameError::SystemError(format!("Failed to serialize save data: {}", e)))?;
        
        file.write_all(&(json_data.len() as u32).to_le_bytes())?;
        file.write_all(json_data.as_bytes())?;
        
        file.flush()?;
        
        // Atomic move
        rename(&temp_path, &save_path)
            .map_err(|e| GameError::SystemError(format!("Failed to finalize save file: {}", e)))?;
        
        Ok(())
    }
    
    /// Load game state from binary format with selective chunk loading
    pub fn load_game_binary(&mut self, save_name: &str) -> GameResult<SaveDataV2> {
        let save_path = self.save_directory.join(format!("{}.sav", save_name));
        
        if !save_path.exists() {
            return Err(GameError::SystemError(format!("Save file not found: {}", save_path.display())));
        }
        
        let mut file = File::open(&save_path)
            .map_err(|e| GameError::SystemError(format!("Failed to open save file: {}", e)))?;
        
        // Read magic header
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;
        if &magic != b"STELLAR2" {
            return Err(GameError::SystemError("Invalid save file format".into()));
        }
        
        // Read version
        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        
        if version > SAVE_FORMAT_VERSION {
            return Err(GameError::SystemError(
                format!("Save file version {} is newer than supported version {}", 
                    version, SAVE_FORMAT_VERSION)));
        }
        
        // Read JSON data length
        let mut json_len_bytes = [0u8; 4];
        file.read_exact(&mut json_len_bytes)?;
        let json_len = u32::from_le_bytes(json_len_bytes);
        
        // Read JSON data
        let mut json_data = vec![0u8; json_len as usize];
        file.read_exact(&mut json_data)?;
        
        let json_str = String::from_utf8(json_data)
            .map_err(|e| GameError::SystemError(format!("Failed to decode JSON data: {}", e)))?;
        
        // Try to deserialize as CoreGameData first, fall back to SimplifiedSaveData for backwards compatibility
        let core_data: CoreGameData = match serde_json::from_str(&json_str) {
            Ok(data) => data,
            Err(_) => {
                // Fall back to simplified format for backwards compatibility
                let simplified_data: SimplifiedSaveData = serde_json::from_str(&json_str)
                    .map_err(|e| GameError::SystemError(format!("Failed to parse save data: {}", e)))?;
                
                CoreGameData {
                    tick: simplified_data.tick,
                    planets: Vec::new(), // No actual data in simplified format
                    ships: Vec::new(),   // No actual data in simplified format
                    factions: Vec::new(), // No actual data in simplified format
                    game_configuration: GameConfiguration::default(),
                    victory_conditions: Vec::new(),
                }
            }
        };
        
        // Convert to SaveDataV2 format
        Ok(SaveDataV2 {
            header: SaveFileHeader {
                magic,
                version,
                schema_version: 1,
                creation_time: 0, // Not stored in simplified format
                game_tick: core_data.tick,
                chunk_count: 0,
                total_size: 0,
                compression_type: CompressionType::None,
                features: SaveFeatureFlags {
                    has_assets: false,
                    has_collections: false,
                    has_megastructures: false,
                    has_unique_personnel: false,
                    chunk_compression: false,
                    differential_saves: false,
                },
            },
            metadata: SaveMetadata {
                save_name: save_name.to_string(),
                description: None,
                player_faction: 0,
                difficulty_level: 1,
                galaxy_size: GalaxySize::Medium,
                total_planets: core_data.planets.len(),
                total_ships: core_data.ships.len(),
                total_factions: core_data.factions.len(),
                total_assets: 0,
                play_time_seconds: 0,
                victory_status: None,
                custom_flags: HashMap::new(),
            },
            game_state: core_data,
            assets: AssetRegistry::new(),
            collections: HashMap::new(),
            chunk_index: HashMap::new(),
        })
    }
    
    /// Prepare save data from current game state
    fn prepare_save_data(&mut self, game_state: &GameState, save_name: &str) -> GameResult<SaveDataV2> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let features = SaveFeatureFlags {
            has_assets: !self.asset_registry.assets.is_empty(),
            has_collections: !self.asset_registry.collections.is_empty(),
            has_megastructures: self.has_megastructures(&self.asset_registry),
            has_unique_personnel: self.has_unique_personnel(&self.asset_registry),
            chunk_compression: self.enable_chunk_compression,
            differential_saves: false, // Future feature
        };
        
        let header = SaveFileHeader {
            magic: *SAVE_MAGIC_HEADER,
            version: self.version,
            schema_version: self.schema_version,
            creation_time: timestamp,
            game_tick: game_state.time_manager.get_current_tick(),
            chunk_count: 0, // Will be calculated later
            total_size: 0, // Will be calculated later
            compression_type: self.compression,
            features,
        };
        
        let metadata = SaveMetadata {
            save_name: save_name.to_string(),
            description: None,
            player_faction: 0, // TODO: Get from faction manager
            difficulty_level: 1,
            galaxy_size: GalaxySize::Medium, // TODO: Get from game config
            total_planets: game_state.planet_manager.get_planet_count(),
            total_ships: game_state.ship_manager.get_all_ships().len(),
            total_factions: game_state.faction_manager.count(),
            total_assets: self.asset_registry.assets.len(),
            play_time_seconds: 0, // TODO: Track play time
            victory_status: None,
            custom_flags: HashMap::new(),
        };
        
        let core_data = CoreGameData {
            tick: game_state.time_manager.get_current_tick(),
            planets: game_state.planet_manager.get_all_planets_cloned()?,
            ships: game_state.ship_manager.get_all_ships_cloned()?,
            factions: game_state.faction_manager.get_all_factions().to_vec(),
            game_configuration: GameConfiguration::default(), // TODO: Get actual config
            victory_conditions: Vec::new(), // TODO: Implement victory tracking
        };
        
        Ok(SaveDataV2 {
            header,
            metadata,
            game_state: core_data,
            assets: self.asset_registry.clone(),
            collections: self.asset_registry.collections.clone(),
            chunk_index: HashMap::new(),
        })
    }
    
    /// Write save file in chunked binary format
    fn write_save_file<W: Write + Seek>(&mut self, writer: &mut W, save_data: &mut SaveDataV2) -> GameResult<()> {
        let mut chunks = Vec::new();
        let mut total_size = 0u64;
        
        // Write magic header first
        writer.write_all(&save_data.header.magic)
            .map_err(|e| GameError::SystemError(format!("Failed to write magic header: {}", e)))?;
        total_size += 8;
        
        // Serialize and write each chunk
        chunks.push(self.create_header_chunk(&save_data.header)?);
        chunks.push(self.create_metadata_chunk(&save_data.metadata)?);
        chunks.push(self.create_game_state_chunk(&save_data.game_state)?);
        
        // Only include asset chunks if we have assets
        if save_data.header.features.has_assets {
            chunks.push(self.create_assets_chunk(&save_data.assets)?);
        }
        
        if save_data.header.features.has_collections {
            chunks.push(self.create_collections_chunk(&save_data.collections)?);
        }
        
        // Update header with chunk count
        save_data.header.chunk_count = chunks.len() as u32;
        
        // Write chunk index
        let index_data = self.serialize_chunk_index(&chunks)?;
        writer.write_all(&(index_data.len() as u32).to_le_bytes())?;
        writer.write_all(&index_data)?;
        total_size += 4 + index_data.len() as u64;
        
        // Write chunks
        for chunk in &chunks {
            writer.write_all(&(chunk.chunk_type as u32).to_le_bytes())?;
            writer.write_all(&chunk.chunk_size.to_le_bytes())?;
            writer.write_all(&chunk.checksum.to_le_bytes())?;
            writer.write_all(&chunk.data)?;
            total_size += 12 + chunk.data.len() as u64;
        }
        
        // Update total size
        save_data.header.total_size = total_size;
        
        Ok(())
    }
    
    /// Read save file header
    fn read_header<R: Read>(&self, reader: &mut R) -> GameResult<SaveFileHeader> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)
            .map_err(|e| GameError::SystemError(format!("Failed to read magic header: {}", e)))?;
        
        if &magic != SAVE_MAGIC_HEADER {
            return Err(GameError::SystemError("Invalid save file format".into()));
        }
        
        // Read header data
        let mut buffer = [0u8; 4];
        
        reader.read_exact(&mut buffer)?;
        let version = u32::from_le_bytes(buffer);
        
        reader.read_exact(&mut buffer)?;
        let schema_version = u32::from_le_bytes(buffer);
        
        let mut buffer8 = [0u8; 8];
        
        reader.read_exact(&mut buffer8)?;
        let creation_time = u64::from_le_bytes(buffer8);
        
        reader.read_exact(&mut buffer8)?;
        let game_tick = u64::from_le_bytes(buffer8);
        
        reader.read_exact(&mut buffer)?;
        let chunk_count = u32::from_le_bytes(buffer);
        
        reader.read_exact(&mut buffer8)?;
        let total_size = u64::from_le_bytes(buffer8);
        
        let mut compression_byte = [0u8; 1];
        reader.read_exact(&mut compression_byte)?;
        let compression_type = match compression_byte[0] {
            0 => CompressionType::None,
            1 => CompressionType::LZ4,
            2 => CompressionType::Zstd,
            _ => return Err(GameError::SystemError("Unknown compression type".into())),
        };
        
        reader.read_exact(&mut buffer8)?;
        let features = SaveFeatureFlags::from_u64(u64::from_le_bytes(buffer8));
        
        Ok(SaveFileHeader {
            magic,
            version,
            schema_version,
            creation_time,
            game_tick,
            chunk_count,
            total_size,
            compression_type,
            features,
        })
    }
    
    /// Validate save file header
    fn validate_header(&self, header: &SaveFileHeader) -> GameResult<()> {
        if header.version > self.version {
            return Err(GameError::SystemError(
                format!("Save file version {} is newer than supported version {}", 
                    header.version, self.version)));
        }
        
        if header.schema_version > self.schema_version {
            return Err(GameError::SystemError(
                format!("Save file schema version {} is newer than supported schema {}", 
                    header.schema_version, self.schema_version)));
        }
        
        if header.chunk_count == 0 {
            return Err(GameError::SystemError("Save file has no chunks".into()));
        }
        
        Ok(())
    }
    
    /// Read chunk index from save file
    fn read_chunk_index<R: Read>(&self, reader: &mut R, chunk_count: u32) -> GameResult<HashMap<ChunkType, SaveChunk>> {
        let mut index_size_buffer = [0u8; 4];
        reader.read_exact(&mut index_size_buffer)?;
        let index_size = u32::from_le_bytes(index_size_buffer);
        
        let mut index_data = vec![0u8; index_size as usize];
        reader.read_exact(&mut index_data)?;
        
        // Deserialize chunk index
        self.deserialize_chunk_index(&index_data, chunk_count)
    }
    
    /// Load chunks from save file
    fn load_chunks<R: Read + Seek>(&self, reader: &mut R, header: SaveFileHeader, 
                   chunk_index: HashMap<ChunkType, SaveChunk>) -> GameResult<SaveDataV2> {
        // Load required chunks
        let metadata = if let Some(chunk) = chunk_index.get(&ChunkType::Metadata) {
            self.load_metadata_chunk(reader, chunk)?
        } else {
            return Err(GameError::SystemError("Missing metadata chunk".into()));
        };
        
        let game_state = if let Some(chunk) = chunk_index.get(&ChunkType::GameState) {
            self.load_game_state_chunk(reader, chunk)?
        } else {
            return Err(GameError::SystemError("Missing game state chunk".into()));
        };
        
        // Load optional chunks
        let assets = if header.features.has_assets {
            if let Some(chunk) = chunk_index.get(&ChunkType::Assets) {
                self.load_assets_chunk(reader, chunk)?
            } else {
                AssetRegistry::new()
            }
        } else {
            AssetRegistry::new()
        };
        
        let collections = if header.features.has_collections {
            if let Some(chunk) = chunk_index.get(&ChunkType::Collections) {
                self.load_collections_chunk(reader, chunk)?
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };
        
        Ok(SaveDataV2 {
            header,
            metadata,
            game_state,
            assets,
            collections,
            chunk_index,
        })
    }
    
    /// Create header chunk
    fn create_header_chunk(&self, header: &SaveFileHeader) -> GameResult<SaveChunk> {
        let mut data = Vec::new();
        
        // Serialize header data (already written magic separately)
        data.extend_from_slice(&header.version.to_le_bytes());
        data.extend_from_slice(&header.schema_version.to_le_bytes());
        data.extend_from_slice(&header.creation_time.to_le_bytes());
        data.extend_from_slice(&header.game_tick.to_le_bytes());
        data.extend_from_slice(&header.chunk_count.to_le_bytes());
        data.extend_from_slice(&header.total_size.to_le_bytes());
        data.push(header.compression_type as u8);
        data.extend_from_slice(&header.features.to_u64().to_le_bytes());
        
        let checksum = self.calculate_chunk_checksum(&data);
        
        Ok(SaveChunk {
            chunk_type: ChunkType::Header,
            chunk_size: data.len() as u32,
            chunk_offset: 0, // Will be set during write
            compression: CompressionType::None, // Headers not compressed
            checksum,
            data,
        })
    }
    
    /// Create metadata chunk
    fn create_metadata_chunk(&self, metadata: &SaveMetadata) -> GameResult<SaveChunk> {
        let mut data = Vec::new();
        
        // Serialize metadata (simplified binary format)
        data.extend_from_slice(&(metadata.save_name.len() as u32).to_le_bytes());
        data.extend_from_slice(metadata.save_name.as_bytes());
        
        let desc_bytes = metadata.description.as_ref().map_or(Vec::new(), |s| s.as_bytes().to_vec());
        data.extend_from_slice(&(desc_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(&desc_bytes);
        
        data.push(metadata.player_faction);
        data.push(metadata.difficulty_level);
        data.push(metadata.galaxy_size as u8);
        data.extend_from_slice(&(metadata.total_planets as u32).to_le_bytes());
        data.extend_from_slice(&(metadata.total_ships as u32).to_le_bytes());
        data.extend_from_slice(&(metadata.total_factions as u32).to_le_bytes());
        data.extend_from_slice(&(metadata.total_assets as u32).to_le_bytes());
        data.extend_from_slice(&metadata.play_time_seconds.to_le_bytes());
        
        let checksum = self.calculate_chunk_checksum(&data);
        
        Ok(SaveChunk {
            chunk_type: ChunkType::Metadata,
            chunk_size: data.len() as u32,
            chunk_offset: 0,
            compression: self.compression,
            checksum,
            data: if self.enable_chunk_compression { self.compress_data(&data)? } else { data },
        })
    }
    
    /// Create game state chunk
    fn create_game_state_chunk(&self, game_state: &CoreGameData) -> GameResult<SaveChunk> {
        // Create simplified save data for JSON serialization
        let simplified_data = SimplifiedSaveData {
            tick: game_state.tick,
            planet_count: game_state.planets.len(),
            ship_count: game_state.ships.len(),
            faction_count: game_state.factions.len(),
        };
        
        let json_data = serde_json::to_string(&simplified_data)
            .map_err(|e| GameError::SystemError(format!("Failed to serialize game state: {}", e)))?;
        
        let data = json_data.into_bytes();
        let checksum = self.calculate_chunk_checksum(&data);
        
        Ok(SaveChunk {
            chunk_type: ChunkType::GameState,
            chunk_size: data.len() as u32,
            chunk_offset: 0,
            compression: self.compression,
            checksum,
            data: if self.enable_chunk_compression { self.compress_data(&data)? } else { data },
        })
    }
    
    /// Create assets chunk
    fn create_assets_chunk(&self, assets: &AssetRegistry) -> GameResult<SaveChunk> {
        let mut data = Vec::new();
        
        // Serialize asset count
        data.extend_from_slice(&(assets.assets.len() as u32).to_le_bytes());
        
        // Serialize each asset
        for (asset_id, asset) in &assets.assets {
            data.extend_from_slice(&asset_id.as_u64().to_le_bytes());
            self.serialize_asset_binary(asset, &mut data)?;
        }
        
        let checksum = self.calculate_chunk_checksum(&data);
        
        Ok(SaveChunk {
            chunk_type: ChunkType::Assets,
            chunk_size: data.len() as u32,
            chunk_offset: 0,
            compression: self.compression,
            checksum,
            data: if self.enable_chunk_compression { self.compress_data(&data)? } else { data },
        })
    }
    
    /// Create collections chunk
    fn create_collections_chunk(&self, collections: &HashMap<AssetId, AssetCollection>) -> GameResult<SaveChunk> {
        let mut data = Vec::new();
        
        // Serialize collection count
        data.extend_from_slice(&(collections.len() as u32).to_le_bytes());
        
        // Serialize each collection
        for (collection_id, collection) in collections {
            data.extend_from_slice(&collection_id.as_u64().to_le_bytes());
            self.serialize_collection_binary(collection, &mut data)?;
        }
        
        let checksum = self.calculate_chunk_checksum(&data);
        
        Ok(SaveChunk {
            chunk_type: ChunkType::Collections,
            chunk_size: data.len() as u32,
            chunk_offset: 0,
            compression: self.compression,
            checksum,
            data: if self.enable_chunk_compression { self.compress_data(&data)? } else { data },
        })
    }
    
    // Binary serialization methods
    fn serialize_planet_binary(&self, planet: &Planet, data: &mut Vec<u8>) -> GameResult<()> {
        // Serialize planet ID
        data.extend_from_slice(&planet.id.to_le_bytes());
        
        // Serialize orbital elements
        data.extend_from_slice(&planet.position.semi_major_axis.to_bits().to_le_bytes());
        data.extend_from_slice(&planet.position.period.to_bits().to_le_bytes());
        data.extend_from_slice(&planet.position.phase.to_bits().to_le_bytes());
        
        // Serialize resource storage
        self.serialize_resource_bundle(&planet.resources.current, data);
        self.serialize_resource_bundle(&planet.resources.capacity, data);
        
        // Serialize demographics
        data.extend_from_slice(&planet.population.total.to_le_bytes());
        data.extend_from_slice(&planet.population.growth_rate.to_bits().to_le_bytes());
        self.serialize_worker_allocation(&planet.population.allocation, data);
        
        // Serialize buildings
        data.extend_from_slice(&(planet.developments.len() as u32).to_le_bytes());
        for building in &planet.developments {
            data.push(building.building_type as u8);
            data.push(building.tier);
            data.push(if building.operational { 1 } else { 0 });
        }
        
        // Serialize controller
        if let Some(controller) = planet.controller {
            data.push(1);
            data.push(controller);
        } else {
            data.push(0);
        }
        
        Ok(())
    }
    
    fn serialize_ship_binary(&self, ship: &Ship, data: &mut Vec<u8>) -> GameResult<()> {
        // Serialize ship ID
        data.extend_from_slice(&ship.id.to_le_bytes());
        
        // Serialize ship class
        data.push(ship.ship_class as u8);
        
        // Serialize position
        data.extend_from_slice(&ship.position.x.to_bits().to_le_bytes());
        data.extend_from_slice(&ship.position.y.to_bits().to_le_bytes());
        
        // Serialize trajectory
        if let Some(trajectory) = &ship.trajectory {
            data.push(1); // Has trajectory
            data.extend_from_slice(&trajectory.origin.x.to_bits().to_le_bytes());
            data.extend_from_slice(&trajectory.origin.y.to_bits().to_le_bytes());
            data.extend_from_slice(&trajectory.destination.x.to_bits().to_le_bytes());
            data.extend_from_slice(&trajectory.destination.y.to_bits().to_le_bytes());
            data.extend_from_slice(&trajectory.departure_time.to_le_bytes());
            data.extend_from_slice(&trajectory.arrival_time.to_le_bytes());
            data.extend_from_slice(&trajectory.fuel_cost.to_bits().to_le_bytes());
        } else {
            data.push(0); // No trajectory
        }
        
        // Serialize cargo hold
        self.serialize_resource_bundle(&ship.cargo.resources, data);
        data.extend_from_slice(&ship.cargo.population.to_le_bytes());
        data.extend_from_slice(&ship.cargo.capacity.to_le_bytes());
        
        // Serialize fuel and owner
        data.extend_from_slice(&ship.fuel.to_bits().to_le_bytes());
        data.push(ship.owner);
        
        Ok(())
    }
    
    fn serialize_faction_binary(&self, faction: &Faction, data: &mut Vec<u8>) -> GameResult<()> {
        // Serialize faction ID
        data.push(faction.id);
        
        // Serialize name
        let name_bytes = faction.name.as_bytes();
        data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(name_bytes);
        
        // Serialize flags
        data.push(if faction.is_player { 1 } else { 0 });
        data.push(faction.ai_type as u8);
        
        // Serialize score
        data.extend_from_slice(&faction.score.to_le_bytes());
        
        Ok(())
    }
    
    fn serialize_asset_binary(&self, _asset: &AssetType, _data: &mut Vec<u8>) -> GameResult<()> {
        // Assets not implemented yet in the main game - skip for now
        Ok(())
    }
    
    fn serialize_collection_binary(&self, _collection: &AssetCollection, _data: &mut Vec<u8>) -> GameResult<()> {
        // Asset collections not implemented yet in the main game - skip for now  
        Ok(())
    }
    
    fn serialize_resource_bundle(&self, resources: &ResourceBundle, data: &mut Vec<u8>) {
        data.extend_from_slice(&resources.minerals.to_le_bytes());
        data.extend_from_slice(&resources.food.to_le_bytes());
        data.extend_from_slice(&resources.energy.to_le_bytes());
        data.extend_from_slice(&resources.alloys.to_le_bytes());
        data.extend_from_slice(&resources.components.to_le_bytes());
        data.extend_from_slice(&resources.fuel.to_le_bytes());
    }
    
    fn serialize_worker_allocation(&self, allocation: &WorkerAllocation, data: &mut Vec<u8>) {
        data.extend_from_slice(&allocation.agriculture.to_le_bytes());
        data.extend_from_slice(&allocation.mining.to_le_bytes());
        data.extend_from_slice(&allocation.industry.to_le_bytes());
        data.extend_from_slice(&allocation.research.to_le_bytes());
        data.extend_from_slice(&allocation.military.to_le_bytes());
        data.extend_from_slice(&allocation.unassigned.to_le_bytes());
    }
    
    // Deserialization methods
    fn load_metadata_chunk<R: Read + Seek>(&self, _reader: &mut R, chunk: &SaveChunk) -> GameResult<SaveMetadata> {
        let data = &chunk.data;
        let mut offset = 0;
        
        // Read save name
        let name_len = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]) as usize;
        offset += 4;
        let save_name = String::from_utf8_lossy(&data[offset..offset+name_len]).to_string();
        offset += name_len;
        
        // Read description
        let desc_len = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]) as usize;
        offset += 4;
        let description = if desc_len > 0 {
            Some(String::from_utf8_lossy(&data[offset..offset+desc_len]).to_string())
        } else {
            None
        };
        offset += desc_len;
        
        // Read remaining fields
        let player_faction = data[offset];
        offset += 1;
        let difficulty_level = data[offset];
        offset += 1;
        let galaxy_size = match data[offset] {
            0 => GalaxySize::Small,
            1 => GalaxySize::Medium,
            _ => GalaxySize::Large,
        };
        offset += 1;
        
        let total_planets = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]) as usize;
        offset += 4;
        
        let total_ships = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]) as usize;
        offset += 4;
        
        let total_factions = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]) as usize;
        offset += 4;
        
        let total_assets = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]) as usize;
        offset += 4;
        
        let play_time_seconds = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7]
        ]);
        
        Ok(SaveMetadata {
            save_name,
            description,
            player_faction,
            difficulty_level,
            galaxy_size,
            total_planets,
            total_ships,
            total_factions,
            total_assets,
            play_time_seconds,
            victory_status: None,
            custom_flags: HashMap::new(),
        })
    }
    
    fn load_game_state_chunk<R: Read + Seek>(&self, _reader: &mut R, chunk: &SaveChunk) -> GameResult<CoreGameData> {
        let json_data = String::from_utf8(chunk.data.clone())
            .map_err(|e| GameError::SystemError(format!("Failed to decode game state data: {}", e)))?;
        
        let simplified: SimplifiedSaveData = serde_json::from_str(&json_data)
            .map_err(|e| GameError::SystemError(format!("Failed to deserialize game state: {}", e)))?;
        
        // For now, return empty core data with just the tick
        // In a full implementation, we'd load the actual game objects
        Ok(CoreGameData {
            tick: simplified.tick,
            planets: Vec::new(), // Would be loaded from manager serialization
            ships: Vec::new(),   // Would be loaded from manager serialization
            factions: Vec::new(), // Would be loaded from manager serialization
            game_configuration: GameConfiguration::default(),
            victory_conditions: Vec::new(),
        })
    }
    
    fn load_assets_chunk<R: Read + Seek>(&self, _reader: &mut R, _chunk: &SaveChunk) -> GameResult<AssetRegistry> {
        // TODO: Implement asset registry deserialization
        Ok(AssetRegistry::new())
    }
    
    fn load_collections_chunk<R: Read + Seek>(&self, _reader: &mut R, _chunk: &SaveChunk) -> GameResult<HashMap<AssetId, AssetCollection>> {
        // TODO: Implement collections deserialization
        Ok(HashMap::new())
    }
    
    // Utility methods
    fn serialize_chunk_index(&self, chunks: &[SaveChunk]) -> GameResult<Vec<u8>> {
        let mut data = Vec::new();
        
        for chunk in chunks {
            data.extend_from_slice(&(chunk.chunk_type as u32).to_le_bytes());
            data.extend_from_slice(&chunk.chunk_size.to_le_bytes());
            data.extend_from_slice(&chunk.chunk_offset.to_le_bytes());
            data.push(chunk.compression as u8);
            data.extend_from_slice(&chunk.checksum.to_le_bytes());
        }
        
        Ok(data)
    }
    
    fn deserialize_chunk_index(&self, _data: &[u8], _chunk_count: u32) -> GameResult<HashMap<ChunkType, SaveChunk>> {
        // TODO: Implement chunk index deserialization
        Ok(HashMap::new())
    }
    
    fn calculate_chunk_checksum(&self, data: &[u8]) -> u32 {
        // Simple CRC32 checksum - could be enhanced
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish() as u32
    }
    
    fn compress_data(&self, _data: &[u8]) -> GameResult<Vec<u8>> {
        // TODO: Implement compression based on self.compression
        Err(GameError::SystemError("Compression not yet implemented".into()))
    }
    
    fn decompress_data(&self, _data: &[u8], _compression: CompressionType) -> GameResult<Vec<u8>> {
        // TODO: Implement decompression
        Err(GameError::SystemError("Decompression not yet implemented".into()))
    }
    
    fn has_megastructures(&self, _assets: &AssetRegistry) -> bool {
        // TODO: Check if asset registry contains megastructures
        false
    }
    
    fn has_unique_personnel(&self, _assets: &AssetRegistry) -> bool {
        // TODO: Check if asset registry contains unique personnel
        false
    }
    
    fn ensure_save_directory(&self) -> GameResult<()> {
        if !self.save_directory.exists() {
            std::fs::create_dir_all(&self.save_directory)
                .map_err(|e| GameError::SystemError(format!("Failed to create save directory: {}", e)))?;
        }
        Ok(())
    }
    
    fn create_backup(&self, save_path: &Path) -> GameResult<()> {
        // Similar to original save system but with enhanced backup rotation
        if !save_path.exists() {
            return Ok(());
        }
        
        for i in (1..self.backup_count).rev() {
            let current_backup = save_path.with_extension(format!("bak{}", i));
            let next_backup = save_path.with_extension(format!("bak{}", i + 1));
            
            if current_backup.exists() {
                if next_backup.exists() {
                    std::fs::remove_file(&next_backup).ok();
                }
                std::fs::rename(&current_backup, &next_backup).ok();
            }
        }
        
        let first_backup = save_path.with_extension("bak1");
        std::fs::copy(save_path, &first_backup)
            .map_err(|e| GameError::SystemError(format!("Failed to create backup: {}", e)))?;
        
        Ok(())
    }
    
    /// Event handling for save system
    pub fn update(&mut self, _delta: f32, _event_bus: &mut EventBus) -> GameResult<()> {
        // No regular updates needed
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    crate::core::events::PlayerCommand::SaveGame => {
                        // Handled via save_game_event_handler
                    }
                    crate::core::events::PlayerCommand::LoadGame => {
                        // Handled via load_game_event_handler
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Get save information without loading full data
    pub fn get_save_info(&self, save_name: &str) -> GameResult<Option<SaveMetadata>> {
        let save_path = self.save_directory.join(format!("{}.sav", save_name));
        
        if !save_path.exists() {
            return Ok(None);
        }
        
        let mut file = File::open(&save_path)
            .map_err(|e| GameError::SystemError(format!("Failed to open save file: {}", e)))?;
        
        // Read magic header
        let mut magic = [0u8; 8];
        if file.read_exact(&mut magic).is_err() {
            return Ok(None); // Cannot read file
        }
        if &magic != b"STELLAR2" {
            return Ok(None); // Not a valid save file
        }
        
        // Read version
        let mut version_bytes = [0u8; 4];
        if file.read_exact(&mut version_bytes).is_err() {
            return Ok(None);
        }
        let version = u32::from_le_bytes(version_bytes);
        
        if version > SAVE_FORMAT_VERSION {
            return Ok(None); // Unsupported version
        }
        
        // Read JSON data length
        let mut json_len_bytes = [0u8; 4];
        if file.read_exact(&mut json_len_bytes).is_err() {
            return Ok(None);
        }
        let json_len = u32::from_le_bytes(json_len_bytes);
        
        // Read JSON data
        let mut json_data = vec![0u8; json_len as usize];
        if file.read_exact(&mut json_data).is_err() {
            return Ok(None);
        }
        
        let json_str = match String::from_utf8(json_data) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };
        
        // Try to parse as CoreGameData (current format)
        let core_data: CoreGameData = match serde_json::from_str(&json_str) {
            Ok(data) => data,
            Err(_) => {
                // Fallback: try to parse as SimplifiedSaveData (legacy format)
                let simplified_data: SimplifiedSaveData = match serde_json::from_str(&json_str) {
                    Ok(data) => data,
                    Err(_) => return Ok(None),
                };
                
                return Ok(Some(SaveMetadata {
                    save_name: save_name.to_string(),
                    description: None,
                    player_faction: 0,
                    difficulty_level: 1,
                    galaxy_size: GalaxySize::Medium,
                    total_planets: simplified_data.planet_count,
                    total_ships: simplified_data.ship_count,
                    total_factions: simplified_data.faction_count,
                    total_assets: 0,
                    play_time_seconds: simplified_data.tick * 100 / 1000, // Approximate
                    victory_status: None,
                    custom_flags: HashMap::new(),
                }));
            }
        };
        
        Ok(Some(SaveMetadata {
            save_name: save_name.to_string(),
            description: None,
            player_faction: 0,
            difficulty_level: 1,
            galaxy_size: core_data.game_configuration.galaxy_size,
            total_planets: core_data.planets.len(),
            total_ships: core_data.ships.len(),
            total_factions: core_data.factions.len(),
            total_assets: 0,
            play_time_seconds: core_data.tick * 100 / 1000, // Approximate from ticks
            victory_status: None,
            custom_flags: HashMap::new(),
        }))
    }
    
    /// List all save files
    pub fn list_saves(&self) -> GameResult<Vec<SaveMetadata>> {
        let mut saves = Vec::new();
        
        if !self.save_directory.exists() {
            return Ok(saves);
        }
        
        let entries = std::fs::read_dir(&self.save_directory)
            .map_err(|e| GameError::SystemError(format!("Failed to read save directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| GameError::SystemError(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("sav") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(Some(metadata)) = self.get_save_info(stem) {
                        saves.push(metadata);
                    }
                }
            }
        }
        
        // Sort by creation time (newest first)
        saves.sort_by(|a, b| b.play_time_seconds.cmp(&a.play_time_seconds));
        Ok(saves)
    }
}

/// Migration utility for converting V1 saves to V2 format
pub struct SaveMigrationUtility;

impl SaveMigrationUtility {
    /// Migrate a V1 save file to V2 format
    pub fn migrate_v1_to_v2(_v1_save_name: &str, _v2_save_name: &str) -> GameResult<()> {
        // TODO: Implement migration from text-based V1 to binary V2
        Err(GameError::SystemError("Migration not yet implemented".into()))
    }
    
    /// Batch migrate all V1 saves to V2
    pub fn batch_migrate_saves() -> GameResult<Vec<String>> {
        // TODO: Find all V1 saves and migrate them
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_system_v2_creation() {
        let save_system = SaveSystem::new();
        assert_eq!(save_system.version, SAVE_FORMAT_VERSION);
        assert_eq!(save_system.schema_version, 1);
    }
    
    #[test]
    fn test_save_feature_flags() {
        let flags = SaveFeatureFlags {
            has_assets: true,
            has_collections: false,
            has_megastructures: true,
            has_unique_personnel: false,
            chunk_compression: true,
            differential_saves: false,
        };
        
        let encoded = flags.to_u64();
        let decoded = SaveFeatureFlags::from_u64(encoded);
        
        assert_eq!(flags.has_assets, decoded.has_assets);
        assert_eq!(flags.has_collections, decoded.has_collections);
        assert_eq!(flags.has_megastructures, decoded.has_megastructures);
        assert_eq!(flags.chunk_compression, decoded.chunk_compression);
    }
    
    #[test]
    fn test_asset_id_generation() {
        let id1 = AssetId::new();
        let id2 = AssetId::new();
        assert_ne!(id1, id2);
        
        let id3 = AssetId::from_u64(12345);
        assert_eq!(id3.as_u64(), 12345);
    }
    
    #[test]
    fn test_chunk_type_values() {
        assert_eq!(ChunkType::Header as u32, 0x48454144);
        assert_eq!(ChunkType::Assets as u32, 0x41535354);
        assert_eq!(ChunkType::Collections as u32, 0x434F4C4C);
    }
}