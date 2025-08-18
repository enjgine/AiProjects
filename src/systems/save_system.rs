// src/systems/save_system.rs
use crate::core::{GameResult, GameEvent, EventBus, GameState};
use crate::core::types::*;
use std::fs::{read_to_string, write, rename, copy};
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! parse_field {
    ($line:expr, $prefix:expr, $type:ty) => {
        $line.strip_prefix($prefix)
            .ok_or_else(|| GameError::SystemError(format!("Invalid format: {}", $prefix)))?
            .parse::<$type>()
            .map_err(|_| GameError::SystemError(format!("Parse error: {}", $prefix)))?
    };
}

macro_rules! write_line {
    ($buffer:expr, $($arg:tt)*) => {
        writeln!($buffer, $($arg)*)
            .map_err(|e| GameError::SystemError(format!("Serialization error: {}", e)))?
    };
}

macro_rules! parse_csv {
    ($line:expr, $prefix:expr, $count:expr, $type:ty) => {{
        let line_str = parse_field!($line, $prefix, String);
        let parts: Vec<&str> = line_str.split(',').collect();
        if parts.len() != $count {
            return Err(GameError::SystemError(format!("Invalid data format: {}", $prefix)));
        }
        parts.into_iter()
            .map(|p| p.parse::<$type>())
            .collect::<Result<Vec<$type>, _>>()
            .map_err(|_| GameError::SystemError(format!("Parse error: {}", $prefix)))?
    }};
}

#[derive(Debug, Clone)]
pub struct SaveData {
    pub version: u32,
    pub tick: u64,
    pub planets: Vec<Planet>,
    pub ships: Vec<Ship>,
    pub factions: Vec<Faction>,
    pub checksum: u64,
    pub save_timestamp: u64,
}

impl SaveData {
    pub fn serialize(&self) -> GameResult<String> {
        let mut buffer = Vec::with_capacity(8192);
        
        write_line!(buffer, "STELLAR_SAVE_V{}", self.version);
        write_line!(buffer, "TIMESTAMP:{}", self.save_timestamp);
        write_line!(buffer, "TICK:{}", self.tick);
        
        write_line!(buffer, "PLANETS:{}", self.planets.len());
        for planet in &self.planets {
            self.serialize_planet_to_buffer(planet, &mut buffer)?;
        }
        
        write_line!(buffer, "SHIPS:{}", self.ships.len());
        for ship in &self.ships {
            self.serialize_ship_to_buffer(ship, &mut buffer)?;
        }
        
        write_line!(buffer, "FACTIONS:{}", self.factions.len());
        for faction in &self.factions {
            self.serialize_faction_to_buffer(faction, &mut buffer)?;
        }
        
        write_line!(buffer, "CHECKSUM:{}", self.checksum);
        
        String::from_utf8(buffer)
            .map_err(|e| GameError::SystemError(format!("UTF-8 conversion error: {}", e)))
    }
    
    pub fn deserialize(data: &str) -> GameResult<Self> {
        let lines: Vec<&str> = data.lines().collect();
        let mut line_idx = 0;
        
        let version = parse_field!(lines.get(line_idx).unwrap_or(&""), "STELLAR_SAVE_V", u32);
        if version > 1 {
            return Err(GameError::SystemError(format!("Unsupported version: {}", version)));
        }
        line_idx += 1;
        
        let save_timestamp = if lines.get(line_idx).map_or(false, |l| l.starts_with("TIMESTAMP:")) {
            line_idx += 1;
            parse_field!(lines[line_idx - 1], "TIMESTAMP:", u64)
        } else {
            0
        };
        
        let tick = parse_field!(lines.get(line_idx).unwrap_or(&""), "TICK:", u64);
        line_idx += 1;
        
        let planet_count = parse_field!(lines.get(line_idx).unwrap_or(&""), "PLANETS:", usize);
        line_idx += 1;
        
        let mut planets = Vec::with_capacity(planet_count);
        for _ in 0..planet_count {
            let (planet, consumed) = Self::deserialize_planet(&lines[line_idx..])?;
            planets.push(planet);
            line_idx += consumed;
        }
        
        let ship_count = parse_field!(lines.get(line_idx).unwrap_or(&""), "SHIPS:", usize);
        line_idx += 1;
        
        let mut ships = Vec::with_capacity(ship_count);
        for _ in 0..ship_count {
            let (ship, consumed) = Self::deserialize_ship(&lines[line_idx..])?;
            ships.push(ship);
            line_idx += consumed;
        }
        
        let faction_count = parse_field!(lines.get(line_idx).unwrap_or(&""), "FACTIONS:", usize);
        line_idx += 1;
        
        let mut factions = Vec::with_capacity(faction_count);
        for _ in 0..faction_count {
            let (faction, consumed) = Self::deserialize_faction(&lines[line_idx..])?;
            factions.push(faction);
            line_idx += consumed;
        }
        
        let checksum = parse_field!(lines.get(line_idx).unwrap_or(&""), "CHECKSUM:", u64);
        
        Ok(SaveData {
            version,
            tick,
            planets,
            ships,
            factions,
            checksum,
            save_timestamp,
        })
    }
    
    fn serialize_planet_to_buffer(&self, planet: &Planet, buffer: &mut Vec<u8>) -> GameResult<()> {
        write_line!(buffer, "P_ID:{}", planet.id);
        write_line!(buffer, "P_ORBIT:{},{},{}", 
            planet.position.semi_major_axis, planet.position.period, planet.position.phase);
        
        let res = &planet.resources.current;
        write_line!(buffer, "P_RES:{},{},{},{},{},{}", 
            res.minerals, res.food, res.energy, res.alloys, res.components, res.fuel);
        
        let cap = &planet.resources.capacity;
        write_line!(buffer, "P_CAP:{},{},{},{},{},{}", 
            cap.minerals, cap.food, cap.energy, cap.alloys, cap.components, cap.fuel);
        
        let pop = &planet.population;
        write_line!(buffer, "P_POP:{},{}", pop.total, pop.growth_rate);
        write_line!(buffer, "P_WORK:{},{},{},{},{},{}", 
            pop.allocation.agriculture, pop.allocation.mining, pop.allocation.industry,
            pop.allocation.research, pop.allocation.military, pop.allocation.unassigned);
        
        write_line!(buffer, "P_BLDG:{}", planet.developments.len());
        for building in &planet.developments {
            let building_type_id = building.building_type as u8;
            write_line!(buffer, "B:{},{},{}", building_type_id, building.tier, building.operational as u8);
        }
        
        let controller_id = planet.controller.map(|id| id as i32).unwrap_or(-1);
        write_line!(buffer, "P_CTRL:{}", controller_id);
        
        Ok(())
    }
    
    fn deserialize_planet(lines: &[&str]) -> GameResult<(Planet, usize)> {
        let mut line_idx = 0;
        
        let id = parse_field!(lines.get(line_idx).unwrap_or(&""), "P_ID:", PlanetId);
        line_idx += 1;
        
        let orbit_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "P_ORBIT:", 3, f32);
        let position = OrbitalElements {
            semi_major_axis: orbit_data[0],
            period: orbit_data[1],
            phase: orbit_data[2],
        };
        line_idx += 1;
        
        let res_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "P_RES:", 6, i32);
        let current = ResourceBundle {
            minerals: res_data[0], food: res_data[1], energy: res_data[2],
            alloys: res_data[3], components: res_data[4], fuel: res_data[5],
        };
        line_idx += 1;
        
        let cap_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "P_CAP:", 6, i32);
        let capacity = ResourceBundle {
            minerals: cap_data[0], food: cap_data[1], energy: cap_data[2],
            alloys: cap_data[3], components: cap_data[4], fuel: cap_data[5],
        };
        line_idx += 1;
        
        let resources = ResourceStorage { current, capacity };
        
        let pop_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "P_POP:", 2, String);
        let total = pop_data[0].parse::<i32>().unwrap();
        let growth_rate = pop_data[1].parse::<f32>().unwrap();
        line_idx += 1;
        
        let work_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "P_WORK:", 6, i32);
        let allocation = WorkerAllocation {
            agriculture: work_data[0], mining: work_data[1], industry: work_data[2],
            research: work_data[3], military: work_data[4], unassigned: work_data[5],
        };
        line_idx += 1;
        
        let population = Demographics { total, growth_rate, allocation };
        
        let building_count = parse_field!(lines.get(line_idx).unwrap_or(&""), "P_BLDG:", usize);
        line_idx += 1;
        
        let mut developments = Vec::with_capacity(building_count);
        for _ in 0..building_count {
            let building_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "B:", 3, String);
            let building_type_id: u8 = building_data[0].parse().unwrap();
            let building_type = match building_type_id {
                0 => BuildingType::Mine, 1 => BuildingType::Farm, 2 => BuildingType::PowerPlant,
                3 => BuildingType::Factory, 4 => BuildingType::ResearchLab, 5 => BuildingType::Spaceport,
                6 => BuildingType::DefensePlatform, 7 => BuildingType::StorageFacility, 8 => BuildingType::Habitat,
                _ => return Err(GameError::SystemError("Unknown building type".into())),
            };
            let tier = building_data[1].parse().unwrap();
            let operational = building_data[2].parse::<u8>().unwrap() != 0;
            
            developments.push(Building { building_type, tier, operational });
            line_idx += 1;
        }
        
        let controller_id = parse_field!(lines.get(line_idx).unwrap_or(&""), "P_CTRL:", i32);
        let controller = if controller_id >= 0 { Some(controller_id as FactionId) } else { None };
        line_idx += 1;
        
        Ok((Planet {
            id, position, resources, population, developments, controller,
        }, line_idx))
    }
    
    fn serialize_ship_to_buffer(&self, ship: &Ship, buffer: &mut Vec<u8>) -> GameResult<()> {
        write_line!(buffer, "S_ID:{}", ship.id);
        write_line!(buffer, "S_CLASS:{}", ship.ship_class as u8);
        write_line!(buffer, "S_POS:{},{}", ship.position.x, ship.position.y);
        
        match &ship.trajectory {
            Some(traj) => {
                write_line!(buffer, "S_TRAJ:{},{},{},{},{},{}", 
                    traj.origin.x, traj.origin.y, traj.destination.x, traj.destination.y,
                    traj.departure_time, traj.arrival_time);
                write_line!(buffer, "S_FUEL_COST:{}", traj.fuel_cost);
            }
            None => write_line!(buffer, "S_TRAJ:NONE"),
        }
        
        let cargo = &ship.cargo;
        write_line!(buffer, "S_CARGO_RES:{},{},{},{},{},{}", 
            cargo.resources.minerals, cargo.resources.food, cargo.resources.energy,
            cargo.resources.alloys, cargo.resources.components, cargo.resources.fuel);
        write_line!(buffer, "S_CARGO_POP:{},{}", cargo.population, cargo.capacity);
        write_line!(buffer, "S_FUEL:{}", ship.fuel);
        write_line!(buffer, "S_OWNER:{}", ship.owner);
        
        Ok(())
    }
    
    fn deserialize_ship(lines: &[&str]) -> GameResult<(Ship, usize)> {
        let mut line_idx = 0;
        
        let id = parse_field!(lines.get(line_idx).unwrap_or(&""), "S_ID:", ShipId);
        line_idx += 1;
        
        let class_id = parse_field!(lines.get(line_idx).unwrap_or(&""), "S_CLASS:", u8);
        let ship_class = match class_id {
            0 => ShipClass::Scout, 1 => ShipClass::Transport,
            2 => ShipClass::Warship, 3 => ShipClass::Colony,
            _ => return Err(GameError::SystemError("Unknown ship class".into())),
        };
        line_idx += 1;
        
        let pos_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "S_POS:", 2, f32);
        let position = Vector2 { x: pos_data[0], y: pos_data[1] };
        line_idx += 1;
        
        let trajectory = if lines.get(line_idx).unwrap_or(&"") == &"S_TRAJ:NONE" {
            None
        } else {
            let traj_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "S_TRAJ:", 6, String);
            line_idx += 1;
            let fuel_cost = parse_field!(lines.get(line_idx).unwrap_or(&""), "S_FUEL_COST:", f32);
            
            Some(Trajectory {
                origin: Vector2 { x: traj_data[0].parse().unwrap(), y: traj_data[1].parse().unwrap() },
                destination: Vector2 { x: traj_data[2].parse().unwrap(), y: traj_data[3].parse().unwrap() },
                departure_time: traj_data[4].parse().unwrap(),
                arrival_time: traj_data[5].parse().unwrap(),
                fuel_cost,
            })
        };
        line_idx += 1;
        
        let cargo_res_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "S_CARGO_RES:", 6, i32);
        let cargo_resources = ResourceBundle {
            minerals: cargo_res_data[0], food: cargo_res_data[1], energy: cargo_res_data[2],
            alloys: cargo_res_data[3], components: cargo_res_data[4], fuel: cargo_res_data[5],
        };
        line_idx += 1;
        
        let cargo_pop_data = parse_csv!(lines.get(line_idx).unwrap_or(&""), "S_CARGO_POP:", 2, i32);
        let cargo = CargoHold {
            resources: cargo_resources,
            population: cargo_pop_data[0],
            capacity: cargo_pop_data[1],
        };
        line_idx += 1;
        
        let fuel = parse_field!(lines.get(line_idx).unwrap_or(&""), "S_FUEL:", f32);
        line_idx += 1;
        
        let owner = parse_field!(lines.get(line_idx).unwrap_or(&""), "S_OWNER:", FactionId);
        line_idx += 1;
        
        Ok((Ship { id, ship_class, position, trajectory, cargo, fuel, owner }, line_idx))
    }
    
    fn serialize_faction_to_buffer(&self, faction: &Faction, buffer: &mut Vec<u8>) -> GameResult<()> {
        write_line!(buffer, "F_ID:{}", faction.id);
        write_line!(buffer, "F_NAME:{}", faction.name);
        write_line!(buffer, "F_PLAYER:{}", faction.is_player as u8);
        write_line!(buffer, "F_AI:{}", faction.ai_type as u8);
        write_line!(buffer, "F_SCORE:{}", faction.score);
        Ok(())
    }
    
    fn deserialize_faction(lines: &[&str]) -> GameResult<(Faction, usize)> {
        let mut line_idx = 0;
        
        let id = parse_field!(lines.get(line_idx).unwrap_or(&""), "F_ID:", FactionId);
        line_idx += 1;
        
        let name = parse_field!(lines.get(line_idx).unwrap_or(&""), "F_NAME:", String);
        line_idx += 1;
        
        let is_player = parse_field!(lines.get(line_idx).unwrap_or(&""), "F_PLAYER:", u8) != 0;
        line_idx += 1;
        
        let ai_type_id = parse_field!(lines.get(line_idx).unwrap_or(&""), "F_AI:", u8);
        let ai_type = match ai_type_id {
            0 => AIPersonality::Aggressive, 1 => AIPersonality::Balanced, 2 => AIPersonality::Economic,
            _ => return Err(GameError::SystemError("Unknown AI personality".into())),
        };
        line_idx += 1;
        
        let score = parse_field!(lines.get(line_idx).unwrap_or(&""), "F_SCORE:", i32);
        line_idx += 1;
        
        Ok((Faction { id, name, is_player, ai_type, score }, line_idx))
    }
}

pub struct SaveSystem {
    version: u32,
    compression: bool,
    save_directory: PathBuf,
    backup_count: u8,
}

impl SaveSystem {
    pub fn new() -> Self {
        Self {
            version: 1,
            compression: false,
            save_directory: PathBuf::from("saves"),
            backup_count: 3,
        }
    }
    
    pub fn with_save_directory(mut self, path: PathBuf) -> Self {
        self.save_directory = path;
        self
    }
    
    pub fn with_backup_count(mut self, count: u8) -> Self {
        self.backup_count = count;
        self
    }
    
    pub fn update(&mut self, _delta: f32, _event_bus: &mut EventBus) -> GameResult<()> {
        // No regular updates needed for save system
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    crate::core::events::PlayerCommand::SaveGame => {
                        // SaveGame handling will be done by the GameState directly
                        // because SaveSystem doesn't have access to the full game state
                    }
                    crate::core::events::PlayerCommand::LoadGame => {
                        // LoadGame handling will be done by the GameState directly
                        // because SaveSystem doesn't have access to the full game state
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    pub fn save_game(&self, game_state: &GameState) -> GameResult<()> {
        self.save_game_to_slot(game_state, "quicksave")
    }
    
    pub fn save_game_to_slot(&self, game_state: &GameState, slot_name: &str) -> GameResult<()> {
        // Ensure save directory exists
        if !self.save_directory.exists() {
            std::fs::create_dir_all(&self.save_directory)
                .map_err(|e| GameError::SystemError(format!("Failed to create save directory: {}", e)))?;
        }
        
        // Validate slot name for security
        if slot_name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
            return Err(GameError::InvalidOperation("Invalid characters in save slot name".into()));
        }
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create SaveData from current game state
        let mut save_data = SaveData {
            version: self.version,
            tick: game_state.time_manager.get_current_tick(),
            planets: game_state.planet_manager.get_all_planets_cloned()?,
            ships: game_state.ship_manager.get_all_ships_cloned()?,
            factions: game_state.faction_manager.get_all_factions().to_vec(),
            checksum: 0, // Will be calculated
            save_timestamp: timestamp,
        };
        
        // Calculate checksum before setting it
        save_data.checksum = self.calculate_checksum(&save_data)?;
        
        // Serialize to string
        let serialized = save_data.serialize()?;
        
        // Create backup before saving
        let save_path = self.save_directory.join(format!("{}.sav", slot_name));
        self.create_backup(&save_path)?;
        
        // Atomic save: write to temporary file first
        let temp_path = save_path.with_extension("tmp");
        write(&temp_path, &serialized)
            .map_err(|e| GameError::SystemError(format!("Failed to write temporary save file: {}", e)))?;
        
        // Atomic move from temporary to final location
        rename(&temp_path, &save_path)
            .map_err(|e| GameError::SystemError(format!("Failed to finalize save file: {}", e)))?;
        
        Ok(())
    }
    
    pub fn load_game(&self) -> GameResult<SaveData> {
        self.load_game_from_slot("quicksave")
    }
    
    pub fn load_game_from_slot(&self, slot_name: &str) -> GameResult<SaveData> {
        // Validate slot name for security
        if slot_name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
            return Err(GameError::InvalidOperation("Invalid characters in save slot name".into()));
        }
        
        let save_path = self.save_directory.join(format!("{}.sav", slot_name));
        
        // Check if file exists
        if !save_path.exists() {
            return Err(GameError::SystemError(format!("Save file not found: {}", save_path.display())));
        }
        
        // Read file
        let serialized = read_to_string(&save_path)
            .map_err(|e| GameError::SystemError(format!("Failed to read save file '{}': {}", save_path.display(), e)))?;
        
        // Deserialize
        let save_data = match SaveData::deserialize(&serialized) {
            Ok(data) => data,
            Err(e) => {
                // Try to recover from backup if main save is corrupted
                match self.try_recover_from_backup(&save_path) {
                    Ok(recovered_data) => {
                        eprintln!("Warning: Main save was corrupted, recovered from backup");
                        recovered_data
                    }
                    Err(_) => return Err(e), // Return original error if recovery fails
                }
            }
        };
        
        // Validate save
        self.validate_save(&save_data)?;
        
        Ok(save_data)
    }
    
    fn create_backup(&self, save_path: &Path) -> GameResult<()> {
        if !save_path.exists() {
            return Ok(()); // No file to backup
        }
        
        // Rotate existing backups
        for i in (1..self.backup_count).rev() {
            let current_backup = save_path.with_extension(format!("bak{}", i));
            let next_backup = save_path.with_extension(format!("bak{}", i + 1));
            
            if current_backup.exists() {
                if next_backup.exists() {
                    std::fs::remove_file(&next_backup).ok(); // Ignore errors for cleanup
                }
                rename(&current_backup, &next_backup).ok(); // Ignore errors for cleanup
            }
        }
        
        // Create new backup
        let first_backup = save_path.with_extension("bak1");
        copy(save_path, &first_backup)
            .map_err(|e| GameError::SystemError(format!("Failed to create backup: {}", e)))?;
        
        Ok(())
    }
    
    fn try_recover_from_backup(&self, save_path: &Path) -> GameResult<SaveData> {
        for i in 1..=self.backup_count {
            let backup_path = save_path.with_extension(format!("bak{}", i));
            if backup_path.exists() {
                match read_to_string(&backup_path) {
                    Ok(serialized) => {
                        if let Ok(save_data) = SaveData::deserialize(&serialized) {
                            if self.validate_save(&save_data).is_ok() {
                                return Ok(save_data);
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        Err(GameError::SystemError("No valid backup found".into()))
    }
    
    fn calculate_checksum(&self, save_data: &SaveData) -> GameResult<u64> {
        let mut hasher = DefaultHasher::new();
        
        // Hash version and tick
        save_data.version.hash(&mut hasher);
        save_data.tick.hash(&mut hasher);
        save_data.save_timestamp.hash(&mut hasher);
        
        // Hash planets count and comprehensive data
        save_data.planets.len().hash(&mut hasher);
        for planet in &save_data.planets {
            planet.id.hash(&mut hasher);
            planet.position.semi_major_axis.to_bits().hash(&mut hasher);
            planet.position.period.to_bits().hash(&mut hasher);
            planet.position.phase.to_bits().hash(&mut hasher);
            
            // Hash all resource values
            planet.resources.current.minerals.hash(&mut hasher);
            planet.resources.current.food.hash(&mut hasher);
            planet.resources.current.energy.hash(&mut hasher);
            planet.resources.current.alloys.hash(&mut hasher);
            planet.resources.current.components.hash(&mut hasher);
            planet.resources.current.fuel.hash(&mut hasher);
            
            planet.population.total.hash(&mut hasher);
            planet.population.growth_rate.to_bits().hash(&mut hasher);
            
            // Hash worker allocation
            planet.population.allocation.agriculture.hash(&mut hasher);
            planet.population.allocation.mining.hash(&mut hasher);
            planet.population.allocation.industry.hash(&mut hasher);
            planet.population.allocation.research.hash(&mut hasher);
            planet.population.allocation.military.hash(&mut hasher);
            planet.population.allocation.unassigned.hash(&mut hasher);
            
            // Hash buildings
            planet.developments.len().hash(&mut hasher);
            for building in &planet.developments {
                std::mem::discriminant(&building.building_type).hash(&mut hasher);
                building.tier.hash(&mut hasher);
                building.operational.hash(&mut hasher);
            }
            
            planet.controller.hash(&mut hasher);
        }
        
        // Hash ships count and comprehensive data
        save_data.ships.len().hash(&mut hasher);
        for ship in &save_data.ships {
            ship.id.hash(&mut hasher);
            std::mem::discriminant(&ship.ship_class).hash(&mut hasher);
            ship.position.x.to_bits().hash(&mut hasher);
            ship.position.y.to_bits().hash(&mut hasher);
            
            // Hash trajectory if present
            match &ship.trajectory {
                Some(traj) => {
                    true.hash(&mut hasher);
                    traj.origin.x.to_bits().hash(&mut hasher);
                    traj.origin.y.to_bits().hash(&mut hasher);
                    traj.destination.x.to_bits().hash(&mut hasher);
                    traj.destination.y.to_bits().hash(&mut hasher);
                    traj.departure_time.hash(&mut hasher);
                    traj.arrival_time.hash(&mut hasher);
                    traj.fuel_cost.to_bits().hash(&mut hasher);
                }
                None => {
                    false.hash(&mut hasher);
                }
            }
            
            // Hash cargo
            ship.cargo.resources.minerals.hash(&mut hasher);
            ship.cargo.resources.food.hash(&mut hasher);
            ship.cargo.resources.energy.hash(&mut hasher);
            ship.cargo.resources.alloys.hash(&mut hasher);
            ship.cargo.resources.components.hash(&mut hasher);
            ship.cargo.resources.fuel.hash(&mut hasher);
            ship.cargo.population.hash(&mut hasher);
            ship.cargo.capacity.hash(&mut hasher);
            
            ship.fuel.to_bits().hash(&mut hasher);
            ship.owner.hash(&mut hasher);
        }
        
        // Hash factions count and comprehensive data
        save_data.factions.len().hash(&mut hasher);
        for faction in &save_data.factions {
            faction.id.hash(&mut hasher);
            faction.name.hash(&mut hasher);
            faction.is_player.hash(&mut hasher);
            std::mem::discriminant(&faction.ai_type).hash(&mut hasher);
            faction.score.hash(&mut hasher);
        }
        
        Ok(hasher.finish())
    }
    
    fn validate_save(&self, save_data: &SaveData) -> GameResult<()> {
        // Check version compatibility
        if save_data.version > self.version {
            return Err(GameError::SystemError(
                format!("Save file version {} is newer than supported version {}", 
                    save_data.version, self.version)));
        }
        
        // Check if checksum matches
        let mut save_data_copy = save_data.clone();
        let original_checksum = save_data_copy.checksum;
        save_data_copy.checksum = 0;
        
        let calculated_checksum = self.calculate_checksum(&save_data_copy)?;
        
        if calculated_checksum != original_checksum {
            return Err(GameError::SystemError(
                format!("Save file checksum mismatch - file may be corrupted. Expected: {}, Got: {}", 
                    original_checksum, calculated_checksum)));
        }
        
        // Validate game state constraints
        self.validate_game_constraints(save_data)?;
        
        // Validate referential integrity
        self.validate_referential_integrity(save_data)?;
        
        Ok(())
    }
    
    fn validate_game_constraints(&self, save_data: &SaveData) -> GameResult<()> {
        // Validate planets
        for (idx, planet) in save_data.planets.iter().enumerate() {
            planet.resources.current.validate_non_negative()
                .map_err(|e| GameError::SystemError(format!("Planet {} has invalid resources: {}", idx, e)))?;
            
            planet.resources.capacity.validate_non_negative()
                .map_err(|e| GameError::SystemError(format!("Planet {} has invalid capacity: {}", idx, e)))?;
            
            // Check that current <= capacity
            let current = &planet.resources.current;
            let capacity = &planet.resources.capacity;
            if current.minerals > capacity.minerals || current.food > capacity.food ||
               current.energy > capacity.energy || current.alloys > capacity.alloys ||
               current.components > capacity.components || current.fuel > capacity.fuel {
                return Err(GameError::SystemError(
                    format!("Planet {} has resources exceeding capacity", idx)));
            }
            
            planet.population.allocation.validate(planet.population.total)
                .map_err(|e| GameError::SystemError(format!("Planet {} has invalid worker allocation: {}", idx, e)))?;
            
            // Validate orbital elements
            if planet.position.semi_major_axis <= 0.0 || planet.position.period <= 0.0 {
                return Err(GameError::SystemError(
                    format!("Planet {} has invalid orbital elements", idx)));
            }
            
            // Validate population
            if planet.population.total < 0 {
                return Err(GameError::SystemError(
                    format!("Planet {} has negative population", idx)));
            }
        }
        
        // Validate ships
        for (idx, ship) in save_data.ships.iter().enumerate() {
            ship.cargo.resources.validate_non_negative()
                .map_err(|e| GameError::SystemError(format!("Ship {} has invalid cargo: {}", idx, e)))?;
            
            if ship.cargo.population < 0 || ship.cargo.capacity < 0 {
                return Err(GameError::SystemError(
                    format!("Ship {} has invalid cargo data: population={}, capacity={}", 
                        idx, ship.cargo.population, ship.cargo.capacity)));
            }
            
            if ship.cargo.population > ship.cargo.capacity {
                return Err(GameError::SystemError(
                    format!("Ship {} has population exceeding cargo capacity", idx)));
            }
            
            if ship.fuel < 0.0 || !ship.fuel.is_finite() {
                return Err(GameError::SystemError(
                    format!("Ship {} has invalid fuel: {}", idx, ship.fuel)));
            }
            
            // Validate trajectory if present
            if let Some(traj) = &ship.trajectory {
                if traj.departure_time >= traj.arrival_time {
                    return Err(GameError::SystemError(
                        format!("Ship {} has invalid trajectory times", idx)));
                }
                
                if traj.fuel_cost < 0.0 || !traj.fuel_cost.is_finite() {
                    return Err(GameError::SystemError(
                        format!("Ship {} has invalid trajectory fuel cost", idx)));
                }
            }
        }
        
        // Validate factions
        for (idx, faction) in save_data.factions.iter().enumerate() {
            if faction.name.is_empty() {
                return Err(GameError::SystemError(
                    format!("Faction {} has empty name", idx)));
            }
            
            if faction.name.len() > 100 {
                return Err(GameError::SystemError(
                    format!("Faction {} name too long", idx)));
            }
        }
        
        Ok(())
    }
    
    fn validate_referential_integrity(&self, save_data: &SaveData) -> GameResult<()> {
        let faction_ids: HashSet<FactionId> = save_data.factions.iter().map(|f| f.id).collect();
        let planet_ids: HashSet<PlanetId> = save_data.planets.iter().map(|p| p.id).collect();
        let ship_ids: HashSet<ShipId> = save_data.ships.iter().map(|s| s.id).collect();
        
        // Check for duplicate IDs
        if faction_ids.len() != save_data.factions.len() {
            return Err(GameError::SystemError("Duplicate faction IDs found".into()));
        }
        if planet_ids.len() != save_data.planets.len() {
            return Err(GameError::SystemError("Duplicate planet IDs found".into()));
        }
        if ship_ids.len() != save_data.ships.len() {
            return Err(GameError::SystemError("Duplicate ship IDs found".into()));
        }
        
        // Validate references
        for planet in &save_data.planets {
            if let Some(controller) = planet.controller {
                if !faction_ids.contains(&controller) {
                    return Err(GameError::SystemError(
                        format!("Planet {} controlled by non-existent faction {}", planet.id, controller)));
                }
            }
        }
        
        for ship in &save_data.ships {
            if !faction_ids.contains(&ship.owner) {
                return Err(GameError::SystemError(
                    format!("Ship {} owned by non-existent faction {}", ship.id, ship.owner)));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_data_serialization() {
        // Create test data
        let test_planet = Planet {
            id: 1,
            position: OrbitalElements {
                semi_major_axis: 5.0,
                period: 365.0,
                phase: 0.0,
            },
            resources: ResourceStorage {
                current: ResourceBundle {
                    minerals: 100,
                    food: 50,
                    energy: 75,
                    alloys: 25,
                    components: 10,
                    fuel: 200,
                },
                capacity: ResourceBundle {
                    minerals: 1000,
                    food: 500,
                    energy: 750,
                    alloys: 250,
                    components: 100,
                    fuel: 2000,
                },
            },
            population: Demographics {
                total: 1000,
                growth_rate: 0.02,
                allocation: WorkerAllocation {
                    agriculture: 200,
                    mining: 300,
                    industry: 250,
                    research: 100,
                    military: 50,
                    unassigned: 100,
                },
            },
            developments: vec![
                Building {
                    building_type: BuildingType::Mine,
                    tier: 1,
                    operational: true,
                },
                Building {
                    building_type: BuildingType::Farm,
                    tier: 2,
                    operational: true,
                },
            ],
            controller: Some(0),
        };

        let test_ship = Ship {
            id: 2,
            ship_class: ShipClass::Scout,
            position: Vector2 { x: 10.0, y: 20.0 },
            trajectory: Some(Trajectory {
                origin: Vector2 { x: 10.0, y: 20.0 },
                destination: Vector2 { x: 30.0, y: 40.0 },
                departure_time: 100,
                arrival_time: 150,
                fuel_cost: 25.0,
            }),
            cargo: CargoHold {
                resources: ResourceBundle {
                    minerals: 10,
                    food: 5,
                    energy: 0,
                    alloys: 0,
                    components: 0,
                    fuel: 0,
                },
                population: 0,
                capacity: 50,
            },
            fuel: 85.0,
            owner: 0,
        };

        let test_faction = Faction {
            id: 0,
            name: "Test Empire".to_string(),
            is_player: true,
            ai_type: AIPersonality::Balanced,
            score: 1000,
        };

        let save_data = SaveData {
            version: 1,
            tick: 42,
            planets: vec![test_planet],
            ships: vec![test_ship],
            factions: vec![test_faction],
            checksum: 12345,
            save_timestamp: 1234567890,
        };

        // Test serialization
        let serialized = save_data.serialize().unwrap();
        assert!(serialized.contains("STELLAR_SAVE_V1"));
        assert!(serialized.contains("TICK:42"));
        assert!(serialized.contains("PLANETS:1"));
        assert!(serialized.contains("SHIPS:1"));
        assert!(serialized.contains("FACTIONS:1"));
        assert!(serialized.contains("CHECKSUM:12345"));

        // Test deserialization
        let deserialized = SaveData::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.tick, 42);
        assert_eq!(deserialized.planets.len(), 1);
        assert_eq!(deserialized.ships.len(), 1);
        assert_eq!(deserialized.factions.len(), 1);
        assert_eq!(deserialized.checksum, 12345);

        // Verify planet data
        let planet = &deserialized.planets[0];
        assert_eq!(planet.id, 1);
        assert_eq!(planet.position.semi_major_axis, 5.0);
        assert_eq!(planet.resources.current.minerals, 100);
        assert_eq!(planet.population.total, 1000);
        assert_eq!(planet.developments.len(), 2);
        assert_eq!(planet.controller, Some(0));

        // Verify ship data
        let ship = &deserialized.ships[0];
        assert_eq!(ship.id, 2);
        assert_eq!(ship.ship_class, ShipClass::Scout);
        assert_eq!(ship.position.x, 10.0);
        assert!(ship.trajectory.is_some());
        assert_eq!(ship.fuel, 85.0);

        // Verify faction data
        let faction = &deserialized.factions[0];
        assert_eq!(faction.id, 0);
        assert_eq!(faction.name, "Test Empire");
        assert!(faction.is_player);
        assert_eq!(faction.ai_type, AIPersonality::Balanced);
        assert_eq!(faction.score, 1000);
    }

    #[test]
    fn test_checksum_calculation() {
        let save_system = SaveSystem::new();
        
        let save_data = SaveData {
            version: 1,
            tick: 100,
            planets: vec![],
            ships: vec![],
            factions: vec![],
            checksum: 0,
            save_timestamp: 0,
        };

        let checksum = save_system.calculate_checksum(&save_data).unwrap();
        assert!(checksum > 0);

        // Same data should produce same checksum
        let checksum2 = save_system.calculate_checksum(&save_data).unwrap();
        assert_eq!(checksum, checksum2);

        // Different data should produce different checksum
        let save_data2 = SaveData {
            version: 1,
            tick: 101,
            planets: vec![],
            ships: vec![],
            factions: vec![],
            checksum: 0,
            save_timestamp: 0,
        };
        let checksum3 = save_system.calculate_checksum(&save_data2).unwrap();
        assert_ne!(checksum, checksum3);
    }

    #[test]
    fn test_save_validation() {
        let save_system = SaveSystem::new();

        // Valid save should pass validation
        let valid_save = SaveData {
            version: 1,
            tick: 50,
            planets: vec![Planet {
                id: 0,
                position: OrbitalElements::default(),
                resources: ResourceStorage {
                    current: ResourceBundle {
                        minerals: 100,
                        food: 50,
                        energy: 25,
                        alloys: 10,
                        components: 5,
                        fuel: 75,
                    },
                    capacity: ResourceBundle {
                        minerals: 1000,
                        food: 500,
                        energy: 250,
                        alloys: 100,
                        components: 50,
                        fuel: 750,
                    },
                },
                population: Demographics {
                    total: 100,
                    growth_rate: 0.02,
                    allocation: WorkerAllocation {
                        agriculture: 20,
                        mining: 30,
                        industry: 25,
                        research: 10,
                        military: 5,
                        unassigned: 10,
                    },
                },
                developments: vec![],
                controller: None,
            }],
            ships: vec![Ship {
                id: 0,
                ship_class: ShipClass::Scout,
                position: Vector2 { x: 0.0, y: 0.0 },
                trajectory: None,
                cargo: CargoHold {
                    resources: ResourceBundle::default(),
                    population: 0,
                    capacity: 10,
                },
                fuel: 100.0,
                owner: 0,
            }],
            factions: vec![Faction {
                id: 0,
                name: "Test Faction".to_string(),
                is_player: true,
                ai_type: AIPersonality::Balanced,
                score: 100,
            }],
            checksum: 0,
            save_timestamp: 0,
        };

        let checksum = save_system.calculate_checksum(&valid_save).unwrap();
        let valid_save_with_checksum = SaveData {
            checksum,
            ..valid_save
        };

        assert!(save_system.validate_save(&valid_save_with_checksum).is_ok());

        // Invalid checksum should fail validation
        let invalid_checksum_save = SaveData {
            checksum: checksum + 1,
            ..valid_save_with_checksum
        };
        assert!(save_system.validate_save(&invalid_checksum_save).is_err());
    }

    #[test]
    fn test_save_validation_negative_resources() {
        let save_system = SaveSystem::new();

        // Save with negative resources should fail validation
        let invalid_save = SaveData {
            version: 1,
            tick: 50,
            planets: vec![Planet {
                id: 0,
                position: OrbitalElements::default(),
                resources: ResourceStorage {
                    current: ResourceBundle {
                        minerals: -100, // Negative resource
                        food: 50,
                        energy: 25,
                        alloys: 10,
                        components: 5,
                        fuel: 75,
                    },
                    capacity: ResourceBundle {
                        minerals: 1000,
                        food: 500,
                        energy: 250,
                        alloys: 100,
                        components: 50,
                        fuel: 750,
                    },
                },
                population: Demographics {
                    total: 100,
                    growth_rate: 0.02,
                    allocation: WorkerAllocation {
                        agriculture: 20,
                        mining: 30,
                        industry: 25,
                        research: 10,
                        military: 5,
                        unassigned: 10,
                    },
                },
                developments: vec![],
                controller: None,
            }],
            ships: vec![],
            factions: vec![],
            checksum: 0,
            save_timestamp: 0,
        };

        let checksum = save_system.calculate_checksum(&invalid_save).unwrap();
        let invalid_save_with_checksum = SaveData {
            checksum,
            ..invalid_save
        };

        assert!(save_system.validate_save(&invalid_save_with_checksum).is_err());
    }

    #[test]
    fn test_save_validation_worker_allocation() {
        let save_system = SaveSystem::new();

        // Save with invalid worker allocation should fail validation
        let invalid_save = SaveData {
            version: 1,
            tick: 50,
            planets: vec![Planet {
                id: 0,
                position: OrbitalElements::default(),
                resources: ResourceStorage::default(),
                population: Demographics {
                    total: 100,
                    growth_rate: 0.02,
                    allocation: WorkerAllocation {
                        agriculture: 20,
                        mining: 30,
                        industry: 25,
                        research: 10,
                        military: 5,
                        unassigned: 15, // Total: 105, but population is 100
                    },
                },
                developments: vec![],
                controller: None,
            }],
            ships: vec![],
            factions: vec![],
            checksum: 0,
            save_timestamp: 0,
        };

        let checksum = save_system.calculate_checksum(&invalid_save).unwrap();
        let invalid_save_with_checksum = SaveData {
            checksum,
            ..invalid_save
        };

        assert!(save_system.validate_save(&invalid_save_with_checksum).is_err());
    }

    #[test]
    fn test_ship_trajectory_serialization() {
        // Test ship with no trajectory
        let ship_no_traj = Ship {
            id: 0,
            ship_class: ShipClass::Transport,
            position: Vector2 { x: 5.0, y: 10.0 },
            trajectory: None,
            cargo: CargoHold::default(),
            fuel: 50.0,
            owner: 1,
        };

        let save_data = SaveData {
            version: 1,
            tick: 1,
            planets: vec![],
            ships: vec![ship_no_traj],
            factions: vec![],
            checksum: 0,
            save_timestamp: 0,
        };

        let serialized = save_data.serialize().unwrap();
        assert!(serialized.contains("S_TRAJ:NONE"));

        let deserialized = SaveData::deserialize(&serialized).unwrap();
        assert!(deserialized.ships[0].trajectory.is_none());

        // Test ship with trajectory
        let ship_with_traj = Ship {
            id: 1,
            ship_class: ShipClass::Warship,
            position: Vector2 { x: 0.0, y: 0.0 },
            trajectory: Some(Trajectory {
                origin: Vector2 { x: 0.0, y: 0.0 },
                destination: Vector2 { x: 100.0, y: 200.0 },
                departure_time: 10,
                arrival_time: 50,
                fuel_cost: 75.5,
            }),
            cargo: CargoHold::default(),
            fuel: 80.0,
            owner: 2,
        };

        let save_data2 = SaveData {
            version: 1,
            tick: 2,
            planets: vec![],
            ships: vec![ship_with_traj],
            factions: vec![],
            checksum: 0,
            save_timestamp: 0,
        };

        let serialized2 = save_data2.serialize().unwrap();
        assert!(serialized2.contains("S_TRAJ:0,0,100,200,10,50"));
        assert!(serialized2.contains("S_FUEL_COST:75.5"));

        let deserialized2 = SaveData::deserialize(&serialized2).unwrap();
        let traj = deserialized2.ships[0].trajectory.as_ref().unwrap();
        assert_eq!(traj.destination.x, 100.0);
        assert_eq!(traj.destination.y, 200.0);
        assert_eq!(traj.fuel_cost, 75.5);
    }

    #[test]
    fn test_invalid_save_format() {
        // Test invalid version
        assert!(SaveData::deserialize("INVALID_VERSION\n").is_err());

        // Test missing tick
        assert!(SaveData::deserialize("STELLAR_SAVE_V1\n").is_err());

        // Test invalid tick format
        assert!(SaveData::deserialize("STELLAR_SAVE_V1\nTICK:invalid\n").is_err());

        // Test missing planet data
        assert!(SaveData::deserialize("STELLAR_SAVE_V1\nTICK:100\n").is_err());
    }

    #[test]
    fn test_save_system_event_handling() {
        let mut save_system = SaveSystem::new();

        // Test SaveGame event
        let save_event = GameEvent::PlayerCommand(crate::core::events::PlayerCommand::SaveGame);
        assert!(save_system.handle_event(&save_event).is_ok());

        // Test LoadGame event
        let load_event = GameEvent::PlayerCommand(crate::core::events::PlayerCommand::LoadGame);
        assert!(save_system.handle_event(&load_event).is_ok());

        // Test other events are ignored
        let other_event = GameEvent::PlayerCommand(crate::core::events::PlayerCommand::PauseGame(true));
        assert!(save_system.handle_event(&other_event).is_ok());
    }
}