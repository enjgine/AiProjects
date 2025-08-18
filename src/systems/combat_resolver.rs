// src/systems/combat_resolver.rs
use crate::core::{GameResult, GameEvent, EventBus, GameSystem};
use crate::core::types::*;
use crate::core::events::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Battle {
    pub attacker: ShipId,
    pub defender: ShipId,
    pub location: Vector2,
    pub start_tick: u64,
    pub planet_id: Option<PlanetId>, // For planetary invasions
}

pub struct CombatResolver {
    active_battles: Vec<Battle>,
    combat_modifiers: HashMap<FactionId, f32>,
}

impl CombatResolver {
    pub fn new() -> Self {
        Self {
            active_battles: Vec::new(),
            combat_modifiers: HashMap::new(),
        }
    }
    
    pub fn update(&mut self, _delta: f32, event_bus: &mut EventBus) -> GameResult<()> {
        // Process any active battles that need resolution
        let battles_to_resolve: Vec<Battle> = self.active_battles.drain(..).collect();
        
        for battle in battles_to_resolve {
            self.resolve_battle(&battle, event_bus)?;
        }
        
        Ok(())
    }
    
    pub fn handle_event(&mut self, event: &GameEvent) -> GameResult<()> {
        match event {
            GameEvent::PlayerCommand(cmd) => {
                match cmd {
                    PlayerCommand::AttackTarget { attacker, target } => {
                        self.initiate_combat(*attacker, *target)?;
                    }
                    _ => {}
                }
            }
            GameEvent::SimulationEvent(sim_event) => {
                match sim_event {
                    SimulationEvent::ShipArrived { ship, destination } => {
                        self.check_for_combat(*ship, *destination)?;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn initiate_combat(&mut self, attacker: ShipId, defender: ShipId) -> GameResult<()> {
        // Start combat between two ships
        let battle = Battle {
            attacker,
            defender,
            location: Vector2 { x: 0.0, y: 0.0 }, // Will be updated with actual position
            start_tick: 0, // Will be updated with current tick
            planet_id: None,
        };
        
        self.active_battles.push(battle);
        Ok(())
    }
    
    fn check_for_combat(&mut self, ship_id: ShipId, location: Vector2) -> GameResult<()> {
        // Check if arriving ship triggers combat
        // This could involve checking for hostile ships at the destination
        // For now, we'll just add it to potential combat scenarios
        // Real implementation would require access to ship manager to check for hostile ships
        Ok(())
    }
    
    fn resolve_battle(&mut self, battle: &Battle, event_bus: &mut EventBus) -> GameResult<()> {
        // Resolve combat with deterministic strength comparison
        // The detailed resolution will be handled by GameState which has access to managers
        // For now, just queue the battle to be resolved by triggering an AttackTarget command
        
        event_bus.queue_event(GameEvent::PlayerCommand(
            PlayerCommand::AttackTarget {
                attacker: battle.attacker,
                target: battle.defender,
            }
        ));
        
        Ok(())
    }
    
    /// Calculate combat strength for a ship based on its class
    fn calculate_ship_strength(&self, ship_class: ShipClass) -> f32 {
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
    pub fn initiate_planetary_invasion(&mut self, attacker: ShipId, planet: PlanetId, location: Vector2) -> GameResult<()> {
        let battle = Battle {
            attacker,
            defender: 0, // Placeholder for planetary defenses
            location,
            start_tick: 0,
            planet_id: Some(planet),
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
        
        resolver.initiate_combat(1, 2).unwrap();
        assert_eq!(resolver.get_active_battle_count(), 1);
        assert!(resolver.has_active_battles());
    }

    #[test]
    fn test_planetary_invasion() {
        let mut resolver = CombatResolver::new();
        
        let location = Vector2 { x: 100.0, y: 200.0 };
        resolver.initiate_planetary_invasion(1, 5, location).unwrap();
        
        assert_eq!(resolver.get_active_battle_count(), 1);
        assert!(resolver.has_active_battles());
    }

    #[test]
    fn test_resolve_battles_clears_active() {
        let mut resolver = CombatResolver::new();
        let mut event_bus = EventBus::new();
        
        // Add a battle
        resolver.initiate_combat(1, 2).unwrap();
        assert_eq!(resolver.get_active_battle_count(), 1);
        
        // Update should resolve all battles
        resolver.update(0.1, &mut event_bus).unwrap();
        assert_eq!(resolver.get_active_battle_count(), 0);
        
        // Should have emitted a combat resolved event
        assert!(!event_bus.queued_events.is_empty());
    }
}