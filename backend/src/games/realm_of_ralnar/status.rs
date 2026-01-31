//! Status Effects for Realm of Ralnar combat
//!
//! Handles various status effects that can affect combatants during battle.

use serde::{Deserialize, Serialize};

/// Status effects that can be applied to combatants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffect {
    /// Damage over time - 10% max HP per turn
    Poison,
    /// Can't act, immune to physical damage
    Stone,
    /// Can't act, 0 HP
    Dead,
    /// Can't act, wakes on damage
    Sleep,
    /// Random target selection
    Confused,
    /// Extra action (acts twice)
    Haste,
    /// Defense boost (+50%)
    Protect,
    /// Magic defense boost (+50%)
    Shell,
    /// Silence - can't use magic
    Silence,
    /// Blind - reduced accuracy
    Blind,
    /// Slow - reduced speed
    Slow,
    /// Regen - heal 5% max HP per turn
    Regen,
    /// Berserk - attack only, but +50% damage
    Berserk,
}

impl StatusEffect {
    /// Get the display name for this status effect
    pub fn name(&self) -> &'static str {
        match self {
            StatusEffect::Poison => "Poison",
            StatusEffect::Stone => "Stone",
            StatusEffect::Dead => "Dead",
            StatusEffect::Sleep => "Sleep",
            StatusEffect::Confused => "Confused",
            StatusEffect::Haste => "Haste",
            StatusEffect::Protect => "Protect",
            StatusEffect::Shell => "Shell",
            StatusEffect::Silence => "Silence",
            StatusEffect::Blind => "Blind",
            StatusEffect::Slow => "Slow",
            StatusEffect::Regen => "Regen",
            StatusEffect::Berserk => "Berserk",
        }
    }

    /// Check if this status prevents taking actions
    pub fn prevents_action(&self) -> bool {
        matches!(
            self,
            StatusEffect::Stone | StatusEffect::Dead | StatusEffect::Sleep
        )
    }

    /// Check if this is a negative (debuff) effect
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            StatusEffect::Poison
                | StatusEffect::Stone
                | StatusEffect::Dead
                | StatusEffect::Sleep
                | StatusEffect::Confused
                | StatusEffect::Silence
                | StatusEffect::Blind
                | StatusEffect::Slow
                | StatusEffect::Berserk
        )
    }

    /// Check if this is a positive (buff) effect
    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            StatusEffect::Haste
                | StatusEffect::Protect
                | StatusEffect::Shell
                | StatusEffect::Regen
        )
    }

    /// Get the base duration in turns (0 means permanent until cured/death)
    pub fn base_duration(&self) -> u8 {
        match self {
            StatusEffect::Poison => 0,    // Until cured or battle ends
            StatusEffect::Stone => 0,     // Until cured
            StatusEffect::Dead => 0,      // Until revived
            StatusEffect::Sleep => 0,     // Until damaged or cured
            StatusEffect::Confused => 4,  // 4 turns
            StatusEffect::Haste => 5,     // 5 turns
            StatusEffect::Protect => 5,   // 5 turns
            StatusEffect::Shell => 5,     // 5 turns
            StatusEffect::Silence => 4,   // 4 turns
            StatusEffect::Blind => 4,     // 4 turns
            StatusEffect::Slow => 4,      // 4 turns
            StatusEffect::Regen => 5,     // 5 turns
            StatusEffect::Berserk => 4,   // 4 turns
        }
    }

    /// Check if this status is removed on damage
    pub fn removed_on_damage(&self) -> bool {
        matches!(self, StatusEffect::Sleep)
    }

    /// Get an icon character for display
    pub fn icon(&self) -> char {
        match self {
            StatusEffect::Poison => 'P',
            StatusEffect::Stone => 'X',
            StatusEffect::Dead => 'D',
            StatusEffect::Sleep => 'Z',
            StatusEffect::Confused => '?',
            StatusEffect::Haste => 'H',
            StatusEffect::Protect => 'P',
            StatusEffect::Shell => 'S',
            StatusEffect::Silence => 'M',
            StatusEffect::Blind => 'B',
            StatusEffect::Slow => '-',
            StatusEffect::Regen => 'R',
            StatusEffect::Berserk => '!',
        }
    }
}

/// A status effect with its remaining duration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStatus {
    pub effect: StatusEffect,
    /// Remaining turns (0 = permanent until cured)
    pub duration: u8,
}

impl ActiveStatus {
    pub fn new(effect: StatusEffect) -> Self {
        Self {
            duration: effect.base_duration(),
            effect,
        }
    }

    pub fn with_duration(effect: StatusEffect, duration: u8) -> Self {
        Self { effect, duration }
    }

    /// Tick down duration. Returns true if effect expired.
    pub fn tick(&mut self) -> bool {
        if self.duration > 0 {
            self.duration -= 1;
            self.duration == 0
        } else {
            false // Permanent effects don't expire from ticking
        }
    }
}

/// Collection of active status effects on a combatant
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusCollection {
    pub effects: Vec<ActiveStatus>,
}

impl StatusCollection {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Add a status effect (replaces if already present)
    pub fn add(&mut self, effect: StatusEffect) {
        // Remove existing effect of same type
        self.effects.retain(|s| s.effect != effect);
        self.effects.push(ActiveStatus::new(effect));
    }

    /// Add with specific duration
    pub fn add_with_duration(&mut self, effect: StatusEffect, duration: u8) {
        self.effects.retain(|s| s.effect != effect);
        self.effects.push(ActiveStatus::with_duration(effect, duration));
    }

    /// Remove a specific status effect
    pub fn remove(&mut self, effect: StatusEffect) {
        self.effects.retain(|s| s.effect != effect);
    }

    /// Check if a status effect is active
    pub fn has(&self, effect: StatusEffect) -> bool {
        self.effects.iter().any(|s| s.effect == effect)
    }

    /// Check if any action-preventing status is active
    pub fn can_act(&self) -> bool {
        !self.effects.iter().any(|s| s.effect.prevents_action())
    }

    /// Tick all effects and remove expired ones
    pub fn tick_all(&mut self) -> Vec<StatusEffect> {
        let mut expired = Vec::new();
        self.effects.retain_mut(|s| {
            if s.tick() {
                expired.push(s.effect);
                false
            } else {
                true
            }
        });
        expired
    }

    /// Remove statuses that are removed on damage
    pub fn on_damage(&mut self) -> Vec<StatusEffect> {
        let mut removed = Vec::new();
        self.effects.retain(|s| {
            if s.effect.removed_on_damage() {
                removed.push(s.effect);
                false
            } else {
                true
            }
        });
        removed
    }

    /// Clear all status effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    /// Clear all negative effects
    pub fn clear_negative(&mut self) {
        self.effects.retain(|s| !s.effect.is_negative());
    }

    /// Clear all positive effects (dispel)
    pub fn clear_positive(&mut self) {
        self.effects.retain(|s| !s.effect.is_positive());
    }

    /// Get all active status effects
    pub fn active_effects(&self) -> Vec<StatusEffect> {
        self.effects.iter().map(|s| s.effect).collect()
    }

    /// Check if is dead
    pub fn is_dead(&self) -> bool {
        self.has(StatusEffect::Dead)
    }

    /// Check if is petrified
    pub fn is_stone(&self) -> bool {
        self.has(StatusEffect::Stone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_effect_names() {
        assert_eq!(StatusEffect::Poison.name(), "Poison");
        assert_eq!(StatusEffect::Dead.name(), "Dead");
        assert_eq!(StatusEffect::Haste.name(), "Haste");
    }

    #[test]
    fn test_status_prevents_action() {
        assert!(StatusEffect::Dead.prevents_action());
        assert!(StatusEffect::Stone.prevents_action());
        assert!(StatusEffect::Sleep.prevents_action());
        assert!(!StatusEffect::Poison.prevents_action());
        assert!(!StatusEffect::Haste.prevents_action());
    }

    #[test]
    fn test_status_is_negative() {
        assert!(StatusEffect::Poison.is_negative());
        assert!(StatusEffect::Blind.is_negative());
        assert!(!StatusEffect::Haste.is_negative());
        assert!(!StatusEffect::Protect.is_negative());
    }

    #[test]
    fn test_status_is_positive() {
        assert!(StatusEffect::Haste.is_positive());
        assert!(StatusEffect::Protect.is_positive());
        assert!(StatusEffect::Regen.is_positive());
        assert!(!StatusEffect::Poison.is_positive());
    }

    #[test]
    fn test_status_collection_add_remove() {
        let mut collection = StatusCollection::new();

        collection.add(StatusEffect::Poison);
        assert!(collection.has(StatusEffect::Poison));

        collection.remove(StatusEffect::Poison);
        assert!(!collection.has(StatusEffect::Poison));
    }

    #[test]
    fn test_status_collection_can_act() {
        let mut collection = StatusCollection::new();
        assert!(collection.can_act());

        collection.add(StatusEffect::Poison);
        assert!(collection.can_act());

        collection.add(StatusEffect::Sleep);
        assert!(!collection.can_act());

        collection.remove(StatusEffect::Sleep);
        assert!(collection.can_act());
    }

    #[test]
    fn test_status_collection_tick() {
        let mut collection = StatusCollection::new();
        collection.add_with_duration(StatusEffect::Haste, 2);

        let expired = collection.tick_all();
        assert!(expired.is_empty());
        assert!(collection.has(StatusEffect::Haste));

        let expired = collection.tick_all();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0], StatusEffect::Haste);
        assert!(!collection.has(StatusEffect::Haste));
    }

    #[test]
    fn test_status_collection_on_damage() {
        let mut collection = StatusCollection::new();
        collection.add(StatusEffect::Sleep);
        collection.add(StatusEffect::Poison);

        let removed = collection.on_damage();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], StatusEffect::Sleep);
        assert!(!collection.has(StatusEffect::Sleep));
        assert!(collection.has(StatusEffect::Poison));
    }

    #[test]
    fn test_status_collection_clear_negative() {
        let mut collection = StatusCollection::new();
        collection.add(StatusEffect::Poison);
        collection.add(StatusEffect::Haste);
        collection.add(StatusEffect::Blind);

        collection.clear_negative();
        assert!(!collection.has(StatusEffect::Poison));
        assert!(!collection.has(StatusEffect::Blind));
        assert!(collection.has(StatusEffect::Haste));
    }

    #[test]
    fn test_status_replaces_existing() {
        let mut collection = StatusCollection::new();
        collection.add_with_duration(StatusEffect::Haste, 2);
        collection.add_with_duration(StatusEffect::Haste, 5);

        // Should only have one Haste effect
        let count = collection
            .effects
            .iter()
            .filter(|s| s.effect == StatusEffect::Haste)
            .count();
        assert_eq!(count, 1);

        // And it should have the new duration
        let haste = collection
            .effects
            .iter()
            .find(|s| s.effect == StatusEffect::Haste)
            .unwrap();
        assert_eq!(haste.duration, 5);
    }

    #[test]
    fn test_permanent_status_doesnt_expire() {
        let mut collection = StatusCollection::new();
        collection.add(StatusEffect::Poison); // Permanent until cured

        // Tick many times
        for _ in 0..10 {
            collection.tick_all();
        }

        // Should still be poisoned
        assert!(collection.has(StatusEffect::Poison));
    }
}
