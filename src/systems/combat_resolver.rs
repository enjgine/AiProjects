// src/systems/combat_resolver.rs
use crate::core::{GameResult, GameError, GameEvent, EventBus, GameSystem};
use crate::core::types::*;
use crate::core::events::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Battle {
    pub attacker: ShipId,
    pub defender: Option<ShipId>, // None for planetary invasions
    pub location: Vector2,
    pub start_tick: u64,
    pub planet_id: Option<PlanetId>, // For planetary invasions
    pub attacker_faction: FactionId,
    pub defender_faction: FactionId,
}

pub struct CombatResolver {
    active_battles: Vec<Battle>,
    combat_modifiers: HashMap<FactionId, f32>,
    current_tick: u64,
    pending_battle_results: Vec<CombatOutcome>,
}

impl CombatResolver {
    pub fn new() -> Self {
        Self {
            active_battles: Vec::new(),
            combat_modifiers: HashMap::new(),
            current_tick: 0,
            pending_battle_results: Vec::new(),
        }
    }
    
    pub fn update(&mut self, _delta: f32, event_bus: &mut EventBus) -> GameResult<()> {
        // Emit any pending combat results from previous tick
        self.emit_pending_results(event_bus)?;
        
        // Process battles that are ready to resolve (1 tick delay for deterministic combat)
        let mut battles_to_resolve = Vec::new();
        self.active_battles.retain(|battle| {
            if self.current_tick > battle.start_tick {
                battles_to_resolve.push(battle.clone());
                false // Remove from active battles
            } else {
                true // Keep in active battles
            }
        });
        
        for battle in battles_to_resolve {
            self.resolve_battle(&battle)?;
        }
        
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    PlayerCommand::AttackTarget { attacker, target } => {
                        // Validate ship IDs are different
                        if attacker == target {
                            return Err(GameError::InvalidOperation(
                                "Ship cannot attack itself".into()
                            ));
                        }
                        // Note: Ship validation will be done by the system that has access to ShipManager
                        self.initiate_ship_combat(*attacker, *target)?
                    }
                    PlayerCommand::ColonizePlanet { ship, planet } => {
                        // Check if planet is hostile and initiate invasion if necessary
                        self.check_planetary_invasion(*ship, *planet)?
                    }
                    _ => {}
                }
            }
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    SimulationEvent::TickCompleted(tick) => {
                        self.current_tick = *tick;
                    }
                    SimulationEvent::ShipArrived { ship, destination } => {
                        self.check_for_automatic_combat(*ship, *destination)?
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn initiate_ship_combat(&mut self, attacker: ShipId, defender: ShipId) -> GameResult<()> {
        // Validate input parameters
        if attacker == defender {
            return Err(GameError::InvalidOperation("Ship cannot attack itself".into()));
        }
        
        // Check if either ship is already in combat
        if self.is_ship_in_combat(attacker) {
            return Err(GameError::InvalidOperation(
                format!("Ship {} is already in combat", attacker)
            ));
        }
        if self.is_ship_in_combat(defender) {
            return Err(GameError::InvalidOperation(
                format!("Ship {} is already in combat", defender)
            ));
        }
        
        // Create battle with proper initialization
        let battle = Battle {
            attacker,
            defender: Some(defender),
            location: Vector2 { x: 0.0, y: 0.0 }, // Position will be determined by GameState
            start_tick: self.current_tick,
            planet_id: None,
            attacker_faction: 0, // Will be set by GameState which has access to ship data
            defender_faction: 0, // Will be set by GameState which has access to ship data
        };
        
        self.active_battles.push(battle);
        Ok(())
    }
    
    fn check_for_automatic_combat(&mut self, ship_id: ShipId, location: Vector2) -> GameResult<()> {
        // Validate input parameters
        if !location.x.is_finite() || !location.y.is_finite() {
            return Err(GameError::InvalidOperation(
                "Invalid location coordinates for combat check".into()
            ));
        }
        
        // Check if ship is already in combat
        if self.is_ship_in_combat(ship_id) {
            return Ok(()); // Already fighting, don't start another battle
        }
        
        // Combat detection requires access to ShipManager and PlanetManager
        // This will be handled by the GameState which can access both managers
        // For now, just log that a ship has arrived
        
        Ok(())
    }
    
    fn check_planetary_invasion(&mut self, ship_id: ShipId, planet_id: PlanetId) -> GameResult<()> {
        // Check if ship is already in combat
        if self.is_ship_in_combat(ship_id) {
            return Err(GameError::InvalidOperation(
                format!("Ship {} is already in combat", ship_id)
            ));
        }
        
        // Planetary invasion logic will be handled by GameState
        // which has access to both ShipManager and PlanetManager
        // to determine if the planet is hostile
        
        Ok(())
    }
    
    fn resolve_battle(&mut self, battle: &Battle) -> GameResult<()> {
        // Calculate combat outcome using deterministic rules
        let outcome = if let Some(defender_ship) = battle.defender {
            // Ship vs Ship combat
            self.resolve_ship_combat(battle, defender_ship)?
        } else {
            // Planetary invasion - requires different logic
            self.resolve_planetary_combat(battle)?
        };
        
        // Store result to be emitted next update cycle
        self.pending_battle_results.push(outcome);
        
        Ok(())
    }
    
    fn resolve_ship_combat(&self, battle: &Battle, defender_ship: ShipId) -> GameResult<CombatOutcome> {
        // For now, use simple deterministic combat based on ship IDs
        // Real implementation would use ship classes and faction modifiers
        // This would be called by GameState which has access to ship data
        
        let attacker_wins = battle.attacker < defender_ship; // Deterministic but arbitrary
        
        let (attacker_losses, defender_losses) = if attacker_wins {
            (Vec::new(), vec![defender_ship])
        } else {
            (vec![battle.attacker], Vec::new())
        };
        
        let winner = if attacker_wins {
            battle.attacker_faction
        } else {
            battle.defender_faction
        };
        
        Ok(CombatOutcome {
            winner,
            attacker_losses,
            defender_losses,
        })
    }
    
    fn resolve_planetary_combat(&self, battle: &Battle) -> GameResult<CombatOutcome> {
        // Planetary invasion logic
        // For now, make planetary defenses strong
        let planet_wins = true; // Simplified logic
        
        let (attacker_losses, defender_losses) = if planet_wins {
            (vec![battle.attacker], Vec::new())
        } else {
            (Vec::new(), Vec::new()) // No ships destroyed in successful invasion
        };
        
        let winner = if planet_wins {
            battle.defender_faction
        } else {
            battle.attacker_faction
        };
        
        Ok(CombatOutcome {
            winner,
            attacker_losses,
            defender_losses,
        })
    }
    
    fn emit_pending_results(&mut self, event_bus: &mut EventBus) -> GameResult<()> {
        for outcome in self.pending_battle_results.drain(..) {
            event_bus.queue_event(GameEvent::SimulationEvent(
                SimulationEvent::CombatResolved {
                    attacker: 0, // Will be properly set by GameState
                    defender: 0, // Will be properly set by GameState  
                    outcome,
                }
            ));
        }
        Ok(())
    }
    
    /// Check if a ship is currently engaged in combat
    pub fn is_ship_in_combat(&self, ship_id: ShipId) -> bool {
        self.active_battles.iter().any(|battle| {
            battle.attacker == ship_id || 
            battle.defender.map_or(false, |defender| defender == ship_id)
        })
    }
    
    /// Calculate combat strength for a ship based on its class
    pub fn calculate_ship_strength(&self, ship_class: ShipClass) -> f32 {
        match ship_class {
            ShipClass::Scout => 1.0,
            ShipClass::Transport => 0.5,
            ShipClass::Warship => 5.0,
            ShipClass::Colony => 0.1,
        }
    }
    
    /// Calculate total combat strength for a fleet
    fn calculate_fleet_strength(&self, ships: &[ShipClass], faction_modifier: f32) -> f32 {
        let base_strength: f32 = ships.iter()
            .map(|ship_class| self.calculate_ship_strength(*ship_class))
            .sum();
        
        base_strength * faction_modifier
    }
    
    /// Determine combat outcome based on strength comparison
    fn determine_outcome(&self, attacker_strength: f32, defender_strength: f32, is_planetary: bool) -> (bool, f32, f32) {
        let effective_defender_strength = if is_planetary {
            defender_strength * 2.0 // Planetary defense multiplies by 2
        } else {
            defender_strength
        };
        
        let attacker_wins = attacker_strength >= effective_defender_strength * 1.5;
        
        let (attacker_losses, defender_losses) = if attacker_wins {
            (0.3, 0.5) // Attacker 30%, Defender 50% losses
        } else {
            (0.5, 0.3) // Attacker 50%, Defender 30% losses
        };
        
        (attacker_wins, attacker_losses, defender_losses)
    }
    
    /// Set combat modifier for a faction
    pub fn set_combat_modifier(&mut self, faction: FactionId, modifier: f32) {
        self.combat_modifiers.insert(faction, modifier);
    }
    
    /// Get combat modifier for a faction (defaults to 1.0)
    pub fn get_combat_modifier(&self, faction: FactionId) -> f32 {
        self.combat_modifiers.get(&faction).copied().unwrap_or(1.0)
    }
    
    /// Initiate planetary invasion
    pub fn initiate_planetary_invasion(&mut self, attacker: ShipId, planet: PlanetId, location: Vector2, attacker_faction: FactionId, defender_faction: FactionId) -> GameResult<()> {
        // Validate input parameters
        if !location.x.is_finite() || !location.y.is_finite() {
            return Err(GameError::InvalidOperation(
                "Invalid location coordinates for planetary invasion".into()
            ));
        }
        
        // Check if ship is already in combat
        if self.is_ship_in_combat(attacker) {
            return Err(GameError::InvalidOperation(
                format!("Ship {} is already in combat", attacker)
            ));
        }
        
        let battle = Battle {
            attacker,
            defender: None, // No defending ship for planetary invasion
            location,
            start_tick: self.current_tick,
            planet_id: Some(planet),
            attacker_faction,
            defender_faction,
        };
        
        self.active_battles.push(battle);
        Ok(())
    }
    
    /// Check if there are any active battles
    pub fn has_active_battles(&self) -> bool {
        !self.active_battles.is_empty()
    }
    
    /// Get number of active battles
    pub fn get_active_battle_count(&self) -> usize {
        self.active_battles.len()
    }
}

impl GameSystem for CombatResolver {
    fn update(&mut self, delta: f32, events: &mut EventBus) -> GameResult<()> {
        self.update(delta, events)
    }
    
    fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        self.handle_event(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_combat_resolver() {
        let resolver = CombatResolver::new();
        assert_eq!(resolver.get_active_battle_count(), 0);
        assert!(!resolver.has_active_battles());
    }

    #[test]
    fn test_ship_strength_calculation() {
        let resolver = CombatResolver::new();
        
        assert_eq!(resolver.calculate_ship_strength(ShipClass::Scout), 1.0);
        assert_eq!(resolver.calculate_ship_strength(ShipClass::Transport), 0.5);
        assert_eq!(resolver.calculate_ship_strength(ShipClass::Warship), 5.0);
        assert_eq!(resolver.calculate_ship_strength(ShipClass::Colony), 0.1);
    }

    #[test]
    fn test_fleet_strength_calculation() {
        let resolver = CombatResolver::new();
        let fleet = vec![ShipClass::Warship, ShipClass::Scout, ShipClass::Transport];
        
        // Base strength: 5.0 + 1.0 + 0.5 = 6.5
        let expected_strength = 6.5 * 1.2; // With 1.2 modifier
        assert_eq!(resolver.calculate_fleet_strength(&fleet, 1.2), expected_strength);
    }

    #[test]
    fn test_combat_outcome_determination() {
        let resolver = CombatResolver::new();
        
        // Test attacker victory in space
        let (attacker_wins, attacker_losses, defender_losses) = 
            resolver.determine_outcome(15.0, 10.0, false);
        assert!(attacker_wins); // 15.0 >= 10.0 * 1.5
        assert_eq!(attacker_losses, 0.3);
        assert_eq!(defender_losses, 0.5);
        
        // Test defender victory in space
        let (attacker_wins, attacker_losses, defender_losses) = 
            resolver.determine_outcome(10.0, 10.0, false);
        assert!(!attacker_wins); // 10.0 < 10.0 * 1.5
        assert_eq!(attacker_losses, 0.5);
        assert_eq!(defender_losses, 0.3);
        
        // Test planetary defense bonus
        let (attacker_wins, _, _) = 
            resolver.determine_outcome(15.0, 10.0, true);
        assert!(!attacker_wins); // 15.0 < (10.0 * 2.0) * 1.5 = 30.0
    }

    #[test]
    fn test_combat_modifier() {
        let mut resolver = CombatResolver::new();
        
        // Default modifier should be 1.0
        assert_eq!(resolver.get_combat_modifier(1), 1.0);
        
        // Set and get custom modifier
        resolver.set_combat_modifier(1, 1.5);
        assert_eq!(resolver.get_combat_modifier(1), 1.5);
    }

    #[test]
    fn test_initiate_combat() {
        let mut resolver = CombatResolver::new();
        
        assert_eq!(resolver.get_active_battle_count(), 0);
        
        resolver.initiate_ship_combat(1, 2).unwrap();
        assert_eq!(resolver.get_active_battle_count(), 1);
        assert!(resolver.has_active_battles());
        
        // Test validation - ship cannot attack itself
        assert!(resolver.initiate_ship_combat(3, 3).is_err());
        
        // Test ship already in combat
        assert!(resolver.initiate_ship_combat(4, 1).is_err()); // Ship 1 already fighting
    }

    #[test]
    fn test_planetary_invasion() {
        let mut resolver = CombatResolver::new();
        
        let location = Vector2 { x: 100.0, y: 200.0 };
        resolver.initiate_planetary_invasion(1, 5, location, 1, 2).unwrap();
        
        assert_eq!(resolver.get_active_battle_count(), 1);
        assert!(resolver.has_active_battles());
        
        // Test validation - invalid coordinates
        let invalid_location = Vector2 { x: f32::NAN, y: 200.0 };
        assert!(resolver.initiate_planetary_invasion(2, 6, invalid_location, 1, 2).is_err());
        
        // Test ship already in combat
        assert!(resolver.initiate_planetary_invasion(1, 7, location, 1, 2).is_err());
    }

    #[test]
    fn test_resolve_battles_clears_active() {
        let mut resolver = CombatResolver::new();
        let mut event_bus = EventBus::new();
        
        // Set current tick to 5 to test battle resolution timing
        resolver.current_tick = 5;
        
        // Add a battle that started at tick 4 (should be resolved)
        resolver.initiate_ship_combat(1, 2).unwrap();
        // Manually set start_tick to test resolution timing
        if let Some(battle) = resolver.active_battles.get_mut(0) {
            battle.start_tick = 4;
        }
        
        assert_eq!(resolver.get_active_battle_count(), 1);
        
        // First update resolves battles and stores results
        resolver.update(0.1, &mut event_bus).unwrap();
        assert_eq!(resolver.get_active_battle_count(), 0);
        
        // Events are emitted on the next update cycle for proper timing
        // So we need to call update again to emit the stored results
        resolver.update(0.1, &mut event_bus).unwrap();
        
        // Should have emitted a combat resolved event
        assert!(!event_bus.queued_events.is_empty());
        
        // Verify it's the correct event type
        if let Some(GameEvent::SimulationEvent(SimulationEvent::CombatResolved { attacker: _, defender: _, outcome })) = event_bus.queued_events.front() {
            // Verify outcome has expected structure
            assert!(!outcome.attacker_losses.is_empty() || !outcome.defender_losses.is_empty());
        } else {
            panic!("Expected CombatResolved event");
        }
    }
    
    #[test]
    fn test_battle_timing() {
        let mut resolver = CombatResolver::new();
        resolver.current_tick = 10;
        
        // Add a battle
        resolver.initiate_ship_combat(1, 2).unwrap();
        
        // Battle should start at current tick
        assert_eq!(resolver.active_battles[0].start_tick, 10);
        
        // Battle should not resolve immediately (needs tick to advance)
        let mut event_bus = EventBus::new();
        resolver.update(0.1, &mut event_bus).unwrap();
        assert_eq!(resolver.get_active_battle_count(), 1);
        
        // Advance tick and try again
        resolver.current_tick = 11;
        resolver.update(0.1, &mut event_bus).unwrap();
        assert_eq!(resolver.get_active_battle_count(), 0);
    }
    
    #[test]
    fn test_ship_in_combat_detection() {
        let mut resolver = CombatResolver::new();
        
        // Initially no ships in combat
        assert!(!resolver.is_ship_in_combat(1));
        assert!(!resolver.is_ship_in_combat(2));
        
        // Start combat between ships 1 and 2
        resolver.initiate_ship_combat(1, 2).unwrap();
        
        // Both ships should now be in combat
        assert!(resolver.is_ship_in_combat(1));
        assert!(resolver.is_ship_in_combat(2));
        
        // Other ships should not be in combat
        assert!(!resolver.is_ship_in_combat(3));
    }
}