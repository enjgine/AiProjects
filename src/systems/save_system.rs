// src/systems/save_system.rs
use crate::core::{GameResult, GameEvent, EventBus, GameState};
use crate::core::types::*;
use std::fs::{read_to_string, write};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct SaveData {
    pub tick: u64,
    pub planets: Vec<Planet>,
    pub ships: Vec<Ship>,
    pub factions: Vec<Faction>,
    pub checksum: u64,
}

impl SaveData {
    // Simple deterministic serialization to ensure consistency
    pub fn serialize(&self) -> GameResult<String> {
        let mut output = String::new();
        
        // Version header
        output.push_str("STELLAR_SAVE_V1\n");
        
        // Tick
        output.push_str(&format!("TICK:{}\n", self.tick));
        
        // Planets
        output.push_str(&format!("PLANETS:{}\n", self.planets.len()));
        for planet in &self.planets {
            output.push_str(&self.serialize_planet(planet)?);
        }
        
        // Ships  
        output.push_str(&format!("SHIPS:{}\n", self.ships.len()));
        for ship in &self.ships {
            output.push_str(&self.serialize_ship(ship)?);
        }
        
        // Factions
        output.push_str(&format!("FACTIONS:{}\n", self.factions.len()));
        for faction in &self.factions {
            output.push_str(&self.serialize_faction(faction)?);
        }
        
        // Checksum
        output.push_str(&format!("CHECKSUM:{}\n", self.checksum));
        
        Ok(output)
    }
    
    pub fn deserialize(data: &str) -> GameResult<Self> {
        let lines: Vec<&str> = data.lines().collect();
        let mut line_idx = 0;
        
        // Check version
        if line_idx >= lines.len() || lines[line_idx] != "STELLAR_SAVE_V1" {
            return Err(GameError::SystemError("Invalid save file format".into()));
        }
        line_idx += 1;
        
        // Parse tick
        let tick = if line_idx < lines.len() && lines[line_idx].starts_with("TICK:") {
            let tick_str = &lines[line_idx][5..];
            line_idx += 1;
            tick_str.parse::<u64>().map_err(|_| GameError::SystemError("Invalid tick format".into()))?
        } else {
            return Err(GameError::SystemError("Missing tick data".into()));
        };
        
        // Parse planets
        let planet_count = if line_idx < lines.len() && lines[line_idx].starts_with("PLANETS:") {
            let count_str = &lines[line_idx][8..];
            line_idx += 1;
            count_str.parse::<usize>().map_err(|_| GameError::SystemError("Invalid planet count".into()))?
        } else {
            return Err(GameError::SystemError("Missing planet data".into()));
        };
        
        let mut planets = Vec::with_capacity(planet_count);
        for _ in 0..planet_count {
            let (planet, consumed_lines) = Self::deserialize_planet(&lines[line_idx..])?;
            planets.push(planet);
            line_idx += consumed_lines;
        }
        
        // Parse ships
        let ship_count = if line_idx < lines.len() && lines[line_idx].starts_with("SHIPS:") {
            let count_str = &lines[line_idx][6..];
            line_idx += 1;
            count_str.parse::<usize>().map_err(|_| GameError::SystemError("Invalid ship count".into()))?
        } else {
            return Err(GameError::SystemError("Missing ship data".into()));
        };
        
        let mut ships = Vec::with_capacity(ship_count);
        for _ in 0..ship_count {
            let (ship, consumed_lines) = Self::deserialize_ship(&lines[line_idx..])?;
            ships.push(ship);
            line_idx += consumed_lines;
        }
        
        // Parse factions
        let faction_count = if line_idx < lines.len() && lines[line_idx].starts_with("FACTIONS:") {
            let count_str = &lines[line_idx][9..];
            line_idx += 1;
            count_str.parse::<usize>().map_err(|_| GameError::SystemError("Invalid faction count".into()))?
        } else {
            return Err(GameError::SystemError("Missing faction data".into()));
        };
        
        let mut factions = Vec::with_capacity(faction_count);
        for _ in 0..faction_count {
            let (faction, consumed_lines) = Self::deserialize_faction(&lines[line_idx..])?;
            factions.push(faction);
            line_idx += consumed_lines;
        }
        
        // Parse checksum
        let checksum = if line_idx < lines.len() && lines[line_idx].starts_with("CHECKSUM:") {
            let checksum_str = &lines[line_idx][9..];
            checksum_str.parse::<u64>().map_err(|_| GameError::SystemError("Invalid checksum format".into()))?
        } else {
            return Err(GameError::SystemError("Missing checksum".into()));
        };
        
        Ok(SaveData {
            tick,
            planets,
            ships,
            factions,
            checksum,
        })
    }
    
    fn serialize_planet(&self, planet: &Planet) -> GameResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("P_ID:{}\n", planet.id));
        output.push_str(&format!("P_ORBIT:{},{},{}\n", 
            planet.position.semi_major_axis, planet.position.period, planet.position.phase));
        
        // Resources
        let res = &planet.resources.current;
        output.push_str(&format!("P_RES:{},{},{},{},{},{}\n",
            res.minerals, res.food, res.energy, res.alloys, res.components, res.fuel));
        
        let cap = &planet.resources.capacity;
        output.push_str(&format!("P_CAP:{},{},{},{},{},{}\n",
            cap.minerals, cap.food, cap.energy, cap.alloys, cap.components, cap.fuel));
        
        // Population
        let pop = &planet.population;
        output.push_str(&format!("P_POP:{},{}\n", pop.total, pop.growth_rate));
        output.push_str(&format!("P_WORK:{},{},{},{},{},{}\n",
            pop.allocation.agriculture, pop.allocation.mining, pop.allocation.industry,
            pop.allocation.research, pop.allocation.military, pop.allocation.unassigned));
        
        // Buildings
        output.push_str(&format!("P_BLDG:{}\n", planet.developments.len()));
        for building in &planet.developments {
            let building_type_id = match building.building_type {
                BuildingType::Mine => 0,
                BuildingType::Farm => 1,
                BuildingType::PowerPlant => 2,
                BuildingType::Factory => 3,
                BuildingType::ResearchLab => 4,
                BuildingType::Spaceport => 5,
                BuildingType::DefensePlatform => 6,
                BuildingType::StorageFacility => 7,
                BuildingType::Habitat => 8,
            };
            output.push_str(&format!("B:{},{},{}\n", building_type_id, building.tier, building.operational as u8));
        }
        
        // Controller
        let controller_id = match planet.controller {
            Some(id) => id as i32,
            None => -1,
        };
        output.push_str(&format!("P_CTRL:{}\n", controller_id));
        
        Ok(output)
    }
    
    fn deserialize_planet(lines: &[&str]) -> GameResult<(Planet, usize)> {
        let mut line_idx = 0;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("P_ID:") {
            return Err(GameError::SystemError("Invalid planet ID format".into()));
        }
        let id = lines[line_idx][5..].parse::<PlanetId>()
            .map_err(|_| GameError::SystemError("Invalid planet ID".into()))?;
        line_idx += 1;
        
        // Parse orbital elements
        if line_idx >= lines.len() || !lines[line_idx].starts_with("P_ORBIT:") {
            return Err(GameError::SystemError("Invalid planet orbit format".into()));
        }
        let orbit_parts: Vec<&str> = lines[line_idx][8..].split(',').collect();
        if orbit_parts.len() != 3 {
            return Err(GameError::SystemError("Invalid orbit data".into()));
        }
        let position = OrbitalElements {
            semi_major_axis: orbit_parts[0].parse().map_err(|_| GameError::SystemError("Invalid orbit data".into()))?,
            period: orbit_parts[1].parse().map_err(|_| GameError::SystemError("Invalid orbit data".into()))?,
            phase: orbit_parts[2].parse().map_err(|_| GameError::SystemError("Invalid orbit data".into()))?,
        };
        line_idx += 1;
        
        // Parse resources current
        if line_idx >= lines.len() || !lines[line_idx].starts_with("P_RES:") {
            return Err(GameError::SystemError("Invalid planet resources format".into()));
        }
        let res_parts: Vec<&str> = lines[line_idx][6..].split(',').collect();
        if res_parts.len() != 6 {
            return Err(GameError::SystemError("Invalid resource data".into()));
        }
        let current = ResourceBundle {
            minerals: res_parts[0].parse().map_err(|_| GameError::SystemError("Invalid resource data".into()))?,
            food: res_parts[1].parse().map_err(|_| GameError::SystemError("Invalid resource data".into()))?,
            energy: res_parts[2].parse().map_err(|_| GameError::SystemError("Invalid resource data".into()))?,
            alloys: res_parts[3].parse().map_err(|_| GameError::SystemError("Invalid resource data".into()))?,
            components: res_parts[4].parse().map_err(|_| GameError::SystemError("Invalid resource data".into()))?,
            fuel: res_parts[5].parse().map_err(|_| GameError::SystemError("Invalid resource data".into()))?,
        };
        line_idx += 1;
        
        // Parse resources capacity
        if line_idx >= lines.len() || !lines[line_idx].starts_with("P_CAP:") {
            return Err(GameError::SystemError("Invalid planet capacity format".into()));
        }
        let cap_parts: Vec<&str> = lines[line_idx][6..].split(',').collect();
        if cap_parts.len() != 6 {
            return Err(GameError::SystemError("Invalid capacity data".into()));
        }
        let capacity = ResourceBundle {
            minerals: cap_parts[0].parse().map_err(|_| GameError::SystemError("Invalid capacity data".into()))?,
            food: cap_parts[1].parse().map_err(|_| GameError::SystemError("Invalid capacity data".into()))?,
            energy: cap_parts[2].parse().map_err(|_| GameError::SystemError("Invalid capacity data".into()))?,
            alloys: cap_parts[3].parse().map_err(|_| GameError::SystemError("Invalid capacity data".into()))?,
            components: cap_parts[4].parse().map_err(|_| GameError::SystemError("Invalid capacity data".into()))?,
            fuel: cap_parts[5].parse().map_err(|_| GameError::SystemError("Invalid capacity data".into()))?,
        };
        line_idx += 1;
        
        let resources = ResourceStorage { current, capacity };
        
        // Parse population
        if line_idx >= lines.len() || !lines[line_idx].starts_with("P_POP:") {
            return Err(GameError::SystemError("Invalid population format".into()));
        }
        let pop_parts: Vec<&str> = lines[line_idx][6..].split(',').collect();
        if pop_parts.len() != 2 {
            return Err(GameError::SystemError("Invalid population data".into()));
        }
        let total: i32 = pop_parts[0].parse().map_err(|_| GameError::SystemError("Invalid population data".into()))?;
        let growth_rate: f32 = pop_parts[1].parse().map_err(|_| GameError::SystemError("Invalid population data".into()))?;
        line_idx += 1;
        
        // Parse worker allocation
        if line_idx >= lines.len() || !lines[line_idx].starts_with("P_WORK:") {
            return Err(GameError::SystemError("Invalid worker allocation format".into()));
        }
        let work_parts: Vec<&str> = lines[line_idx][7..].split(',').collect();
        if work_parts.len() != 6 {
            return Err(GameError::SystemError("Invalid worker allocation data".into()));
        }
        let allocation = WorkerAllocation {
            agriculture: work_parts[0].parse().map_err(|_| GameError::SystemError("Invalid worker allocation data".into()))?,
            mining: work_parts[1].parse().map_err(|_| GameError::SystemError("Invalid worker allocation data".into()))?,
            industry: work_parts[2].parse().map_err(|_| GameError::SystemError("Invalid worker allocation data".into()))?,
            research: work_parts[3].parse().map_err(|_| GameError::SystemError("Invalid worker allocation data".into()))?,
            military: work_parts[4].parse().map_err(|_| GameError::SystemError("Invalid worker allocation data".into()))?,
            unassigned: work_parts[5].parse().map_err(|_| GameError::SystemError("Invalid worker allocation data".into()))?,
        };
        line_idx += 1;
        
        let population = Demographics { total, growth_rate, allocation };
        
        // Parse buildings
        if line_idx >= lines.len() || !lines[line_idx].starts_with("P_BLDG:") {
            return Err(GameError::SystemError("Invalid buildings format".into()));
        }
        let building_count: usize = lines[line_idx][7..].parse()
            .map_err(|_| GameError::SystemError("Invalid building count".into()))?;
        line_idx += 1;
        
        let mut developments = Vec::with_capacity(building_count);
        for _ in 0..building_count {
            if line_idx >= lines.len() || !lines[line_idx].starts_with("B:") {
                return Err(GameError::SystemError("Invalid building format".into()));
            }
            let building_parts: Vec<&str> = lines[line_idx][2..].split(',').collect();
            if building_parts.len() != 3 {
                return Err(GameError::SystemError("Invalid building data".into()));
            }
            
            let building_type_id: u8 = building_parts[0].parse()
                .map_err(|_| GameError::SystemError("Invalid building type".into()))?;
            let building_type = match building_type_id {
                0 => BuildingType::Mine,
                1 => BuildingType::Farm,
                2 => BuildingType::PowerPlant,
                3 => BuildingType::Factory,
                4 => BuildingType::ResearchLab,
                5 => BuildingType::Spaceport,
                6 => BuildingType::DefensePlatform,
                7 => BuildingType::StorageFacility,
                8 => BuildingType::Habitat,
                _ => return Err(GameError::SystemError("Unknown building type".into())),
            };
            
            let tier: u8 = building_parts[1].parse()
                .map_err(|_| GameError::SystemError("Invalid building tier".into()))?;
            let operational: bool = building_parts[2].parse::<u8>()
                .map_err(|_| GameError::SystemError("Invalid building operational state".into()))? != 0;
            
            developments.push(Building { building_type, tier, operational });
            line_idx += 1;
        }
        
        // Parse controller
        if line_idx >= lines.len() || !lines[line_idx].starts_with("P_CTRL:") {
            return Err(GameError::SystemError("Invalid controller format".into()));
        }
        let controller_id: i32 = lines[line_idx][7..].parse()
            .map_err(|_| GameError::SystemError("Invalid controller ID".into()))?;
        let controller = if controller_id >= 0 { Some(controller_id as FactionId) } else { None };
        line_idx += 1;
        
        Ok((Planet {
            id,
            position,
            resources,
            population,
            developments,
            controller,
        }, line_idx))
    }
    
    fn serialize_ship(&self, ship: &Ship) -> GameResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("S_ID:{}\n", ship.id));
        
        let class_id = match ship.ship_class {
            ShipClass::Scout => 0,
            ShipClass::Transport => 1,
            ShipClass::Warship => 2,
            ShipClass::Colony => 3,
        };
        output.push_str(&format!("S_CLASS:{}\n", class_id));
        
        output.push_str(&format!("S_POS:{},{}\n", ship.position.x, ship.position.y));
        
        // Trajectory (optional)
        match &ship.trajectory {
            Some(traj) => {
                output.push_str(&format!("S_TRAJ:{},{},{},{},{},{}\n",
                    traj.origin.x, traj.origin.y, traj.destination.x, traj.destination.y,
                    traj.departure_time, traj.arrival_time));
                output.push_str(&format!("S_FUEL_COST:{}\n", traj.fuel_cost));
            }
            None => {
                output.push_str("S_TRAJ:NONE\n");
            }
        }
        
        // Cargo
        let cargo = &ship.cargo;
        output.push_str(&format!("S_CARGO_RES:{},{},{},{},{},{}\n",
            cargo.resources.minerals, cargo.resources.food, cargo.resources.energy,
            cargo.resources.alloys, cargo.resources.components, cargo.resources.fuel));
        output.push_str(&format!("S_CARGO_POP:{},{}\n", cargo.population, cargo.capacity));
        
        output.push_str(&format!("S_FUEL:{}\n", ship.fuel));
        output.push_str(&format!("S_OWNER:{}\n", ship.owner));
        
        Ok(output)
    }
    
    fn deserialize_ship(lines: &[&str]) -> GameResult<(Ship, usize)> {
        let mut line_idx = 0;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("S_ID:") {
            return Err(GameError::SystemError("Invalid ship ID format".into()));
        }
        let id = lines[line_idx][5..].parse::<ShipId>()
            .map_err(|_| GameError::SystemError("Invalid ship ID".into()))?;
        line_idx += 1;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("S_CLASS:") {
            return Err(GameError::SystemError("Invalid ship class format".into()));
        }
        let class_id: u8 = lines[line_idx][8..].parse()
            .map_err(|_| GameError::SystemError("Invalid ship class".into()))?;
        let ship_class = match class_id {
            0 => ShipClass::Scout,
            1 => ShipClass::Transport,
            2 => ShipClass::Warship,
            3 => ShipClass::Colony,
            _ => return Err(GameError::SystemError("Unknown ship class".into())),
        };
        line_idx += 1;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("S_POS:") {
            return Err(GameError::SystemError("Invalid ship position format".into()));
        }
        let pos_parts: Vec<&str> = lines[line_idx][6..].split(',').collect();
        if pos_parts.len() != 2 {
            return Err(GameError::SystemError("Invalid position data".into()));
        }
        let position = Vector2 {
            x: pos_parts[0].parse().map_err(|_| GameError::SystemError("Invalid position data".into()))?,
            y: pos_parts[1].parse().map_err(|_| GameError::SystemError("Invalid position data".into()))?,
        };
        line_idx += 1;
        
        // Parse trajectory
        if line_idx >= lines.len() || !lines[line_idx].starts_with("S_TRAJ:") {
            return Err(GameError::SystemError("Invalid ship trajectory format".into()));
        }
        let trajectory = if lines[line_idx] == "S_TRAJ:NONE" {
            None
        } else {
            let traj_parts: Vec<&str> = lines[line_idx][7..].split(',').collect();
            if traj_parts.len() != 6 {
                return Err(GameError::SystemError("Invalid trajectory data".into()));
            }
            
            line_idx += 1;
            if line_idx >= lines.len() || !lines[line_idx].starts_with("S_FUEL_COST:") {
                return Err(GameError::SystemError("Invalid fuel cost format".into()));
            }
            let fuel_cost: f32 = lines[line_idx][12..].parse()
                .map_err(|_| GameError::SystemError("Invalid fuel cost".into()))?;
            
            Some(Trajectory {
                origin: Vector2 {
                    x: traj_parts[0].parse().map_err(|_| GameError::SystemError("Invalid trajectory data".into()))?,
                    y: traj_parts[1].parse().map_err(|_| GameError::SystemError("Invalid trajectory data".into()))?,
                },
                destination: Vector2 {
                    x: traj_parts[2].parse().map_err(|_| GameError::SystemError("Invalid trajectory data".into()))?,
                    y: traj_parts[3].parse().map_err(|_| GameError::SystemError("Invalid trajectory data".into()))?,
                },
                departure_time: traj_parts[4].parse().map_err(|_| GameError::SystemError("Invalid trajectory data".into()))?,
                arrival_time: traj_parts[5].parse().map_err(|_| GameError::SystemError("Invalid trajectory data".into()))?,
                fuel_cost,
            })
        };
        line_idx += 1;
        
        // Parse cargo resources
        if line_idx >= lines.len() || !lines[line_idx].starts_with("S_CARGO_RES:") {
            return Err(GameError::SystemError("Invalid cargo resources format".into()));
        }
        let cargo_res_parts: Vec<&str> = lines[line_idx][12..].split(',').collect();
        if cargo_res_parts.len() != 6 {
            return Err(GameError::SystemError("Invalid cargo resource data".into()));
        }
        let cargo_resources = ResourceBundle {
            minerals: cargo_res_parts[0].parse().map_err(|_| GameError::SystemError("Invalid cargo resource data".into()))?,
            food: cargo_res_parts[1].parse().map_err(|_| GameError::SystemError("Invalid cargo resource data".into()))?,
            energy: cargo_res_parts[2].parse().map_err(|_| GameError::SystemError("Invalid cargo resource data".into()))?,
            alloys: cargo_res_parts[3].parse().map_err(|_| GameError::SystemError("Invalid cargo resource data".into()))?,
            components: cargo_res_parts[4].parse().map_err(|_| GameError::SystemError("Invalid cargo resource data".into()))?,
            fuel: cargo_res_parts[5].parse().map_err(|_| GameError::SystemError("Invalid cargo resource data".into()))?,
        };
        line_idx += 1;
        
        // Parse cargo population and capacity
        if line_idx >= lines.len() || !lines[line_idx].starts_with("S_CARGO_POP:") {
            return Err(GameError::SystemError("Invalid cargo population format".into()));
        }
        let cargo_pop_parts: Vec<&str> = lines[line_idx][12..].split(',').collect();
        if cargo_pop_parts.len() != 2 {
            return Err(GameError::SystemError("Invalid cargo population data".into()));
        }
        let cargo = CargoHold {
            resources: cargo_resources,
            population: cargo_pop_parts[0].parse().map_err(|_| GameError::SystemError("Invalid cargo population data".into()))?,
            capacity: cargo_pop_parts[1].parse().map_err(|_| GameError::SystemError("Invalid cargo population data".into()))?,
        };
        line_idx += 1;
        
        // Parse fuel
        if line_idx >= lines.len() || !lines[line_idx].starts_with("S_FUEL:") {
            return Err(GameError::SystemError("Invalid ship fuel format".into()));
        }
        let fuel: f32 = lines[line_idx][7..].parse()
            .map_err(|_| GameError::SystemError("Invalid fuel data".into()))?;
        line_idx += 1;
        
        // Parse owner
        if line_idx >= lines.len() || !lines[line_idx].starts_with("S_OWNER:") {
            return Err(GameError::SystemError("Invalid ship owner format".into()));
        }
        let owner: FactionId = lines[line_idx][8..].parse()
            .map_err(|_| GameError::SystemError("Invalid owner data".into()))?;
        line_idx += 1;
        
        Ok((Ship {
            id,
            ship_class,
            position,
            trajectory,
            cargo,
            fuel,
            owner,
        }, line_idx))
    }
    
    fn serialize_faction(&self, faction: &Faction) -> GameResult<String> {
        let mut output = String::new();
        
        output.push_str(&format!("F_ID:{}\n", faction.id));
        output.push_str(&format!("F_NAME:{}\n", faction.name));
        output.push_str(&format!("F_PLAYER:{}\n", faction.is_player as u8));
        
        let ai_type_id = match faction.ai_type {
            AIPersonality::Aggressive => 0,
            AIPersonality::Balanced => 1,
            AIPersonality::Economic => 2,
        };
        output.push_str(&format!("F_AI:{}\n", ai_type_id));
        output.push_str(&format!("F_SCORE:{}\n", faction.score));
        
        Ok(output)
    }
    
    fn deserialize_faction(lines: &[&str]) -> GameResult<(Faction, usize)> {
        let mut line_idx = 0;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("F_ID:") {
            return Err(GameError::SystemError("Invalid faction ID format".into()));
        }
        let id = lines[line_idx][5..].parse::<FactionId>()
            .map_err(|_| GameError::SystemError("Invalid faction ID".into()))?;
        line_idx += 1;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("F_NAME:") {
            return Err(GameError::SystemError("Invalid faction name format".into()));
        }
        let name = lines[line_idx][7..].to_string();
        line_idx += 1;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("F_PLAYER:") {
            return Err(GameError::SystemError("Invalid faction player format".into()));
        }
        let is_player = lines[line_idx][9..].parse::<u8>()
            .map_err(|_| GameError::SystemError("Invalid faction player data".into()))? != 0;
        line_idx += 1;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("F_AI:") {
            return Err(GameError::SystemError("Invalid faction AI format".into()));
        }
        let ai_type_id: u8 = lines[line_idx][5..].parse()
            .map_err(|_| GameError::SystemError("Invalid faction AI data".into()))?;
        let ai_type = match ai_type_id {
            0 => AIPersonality::Aggressive,
            1 => AIPersonality::Balanced,
            2 => AIPersonality::Economic,
            _ => return Err(GameError::SystemError("Unknown AI personality".into())),
        };
        line_idx += 1;
        
        if line_idx >= lines.len() || !lines[line_idx].starts_with("F_SCORE:") {
            return Err(GameError::SystemError("Invalid faction score format".into()));
        }
        let score: i32 = lines[line_idx][8..].parse()
            .map_err(|_| GameError::SystemError("Invalid faction score data".into()))?;
        line_idx += 1;
        
        Ok((Faction {
            id,
            name,
            is_player,
            ai_type,
            score,
        }, line_idx))
    }
}

pub struct SaveSystem {
    version: u32,
    compression: bool,
}

impl SaveSystem {
    pub fn new() -> Self {
        Self {
            version: 1,
            compression: false,
        }
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
        // Create SaveData from current game state
        let save_data = SaveData {
            tick: game_state.time_manager.get_current_tick(),
            planets: game_state.planet_manager.get_all_planets_cloned()?,
            ships: game_state.ship_manager.get_all_ships_cloned()?,
            factions: game_state.faction_manager.get_all_factions().to_vec(),
            checksum: 0, // Will be calculated after serialization
        };
        
        // Calculate checksum
        let mut save_data_with_checksum = save_data;
        save_data_with_checksum.checksum = self.calculate_checksum(&save_data_with_checksum)?;
        
        // Serialize to string
        let serialized = save_data_with_checksum.serialize()?;
        
        // Write to file
        let save_path = "stellar_dominion_save.txt";
        write(save_path, serialized)
            .map_err(|e| GameError::SystemError(format!("Failed to write save file: {}", e)))?;
        
        Ok(())
    }
    
    pub fn load_game(&self) -> GameResult<SaveData> {
        let save_path = "stellar_dominion_save.txt";
        
        // Read file
        let serialized = read_to_string(save_path)
            .map_err(|e| GameError::SystemError(format!("Failed to read save file: {}", e)))?;
        
        // Deserialize
        let save_data = SaveData::deserialize(&serialized)?;
        
        // Validate save
        self.validate_save(&save_data)?;
        
        Ok(save_data)
    }
    
    fn calculate_checksum(&self, save_data: &SaveData) -> GameResult<u64> {
        let mut hasher = DefaultHasher::new();
        
        // Hash tick
        save_data.tick.hash(&mut hasher);
        
        // Hash planets count and basic data
        save_data.planets.len().hash(&mut hasher);
        for planet in &save_data.planets {
            planet.id.hash(&mut hasher);
            planet.position.semi_major_axis.to_bits().hash(&mut hasher);
            planet.resources.current.minerals.hash(&mut hasher);
            planet.population.total.hash(&mut hasher);
        }
        
        // Hash ships count and basic data
        save_data.ships.len().hash(&mut hasher);
        for ship in &save_data.ships {
            ship.id.hash(&mut hasher);
            ship.position.x.to_bits().hash(&mut hasher);
            ship.position.y.to_bits().hash(&mut hasher);
            ship.owner.hash(&mut hasher);
        }
        
        // Hash factions count and basic data
        save_data.factions.len().hash(&mut hasher);
        for faction in &save_data.factions {
            faction.id.hash(&mut hasher);
            faction.name.hash(&mut hasher);
            faction.score.hash(&mut hasher);
        }
        
        Ok(hasher.finish())
    }
    
    fn validate_save(&self, save_data: &SaveData) -> GameResult<()> {
        // Check if checksum matches
        let mut save_data_copy = save_data.clone();
        let original_checksum = save_data_copy.checksum;
        save_data_copy.checksum = 0;
        
        let calculated_checksum = self.calculate_checksum(&save_data_copy)?;
        
        if calculated_checksum != original_checksum {
            return Err(GameError::SystemError("Save file checksum mismatch - file may be corrupted".into()));
        }
        
        // Validate resource constraints
        for planet in &save_data.planets {
            planet.resources.current.validate_non_negative()?;
            planet.population.allocation.validate(planet.population.total)?;
        }
        
        // Validate ship data
        for ship in &save_data.ships {
            ship.cargo.resources.validate_non_negative()?;
            if ship.cargo.population < 0 || ship.cargo.capacity < 0 {
                return Err(GameError::SystemError("Invalid ship cargo data".into()));
            }
            if ship.fuel < 0.0 {
                return Err(GameError::SystemError("Invalid ship fuel data".into()));
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
            tick: 42,
            planets: vec![test_planet],
            ships: vec![test_ship],
            factions: vec![test_faction],
            checksum: 12345,
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
            tick: 100,
            planets: vec![],
            ships: vec![],
            factions: vec![],
            checksum: 0,
        };

        let checksum = save_system.calculate_checksum(&save_data).unwrap();
        assert!(checksum > 0);

        // Same data should produce same checksum
        let checksum2 = save_system.calculate_checksum(&save_data).unwrap();
        assert_eq!(checksum, checksum2);

        // Different data should produce different checksum
        let save_data2 = SaveData {
            tick: 101,
            planets: vec![],
            ships: vec![],
            factions: vec![],
            checksum: 0,
        };
        let checksum3 = save_system.calculate_checksum(&save_data2).unwrap();
        assert_ne!(checksum, checksum3);
    }

    #[test]
    fn test_save_validation() {
        let save_system = SaveSystem::new();

        // Valid save should pass validation
        let valid_save = SaveData {
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
            factions: vec![],
            checksum: 0,
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
            tick: 1,
            planets: vec![],
            ships: vec![ship_no_traj],
            factions: vec![],
            checksum: 0,
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
            tick: 2,
            planets: vec![],
            ships: vec![ship_with_traj],
            factions: vec![],
            checksum: 0,
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