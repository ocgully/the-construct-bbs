//! Magic System for Realm of Ralnar
//!
//! Defines spells, elements, and targeting for the combat system.

use serde::{Deserialize, Serialize};

/// Elemental types for spells and resistances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    Fire,
    Ice,
    Lightning,
    Earth,
    Water,
    Wind,
    Holy,
    Dark,
    None,
}

impl Element {
    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Element::Fire => "Fire",
            Element::Ice => "Ice",
            Element::Lightning => "Lightning",
            Element::Earth => "Earth",
            Element::Water => "Water",
            Element::Wind => "Wind",
            Element::Holy => "Holy",
            Element::Dark => "Dark",
            Element::None => "Non-elemental",
        }
    }

    /// Get the element this is weak to
    pub fn weakness(&self) -> Option<Element> {
        match self {
            Element::Fire => Some(Element::Water),
            Element::Ice => Some(Element::Fire),
            Element::Lightning => Some(Element::Earth),
            Element::Earth => Some(Element::Wind),
            Element::Water => Some(Element::Lightning),
            Element::Wind => Some(Element::Ice),
            Element::Holy => Some(Element::Dark),
            Element::Dark => Some(Element::Holy),
            Element::None => None,
        }
    }

    /// Get the element this resists
    pub fn resistance(&self) -> Option<Element> {
        // Resist the element we're strong against
        self.weakness().and_then(|w| w.weakness())
    }
}

/// What a spell can target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType {
    /// One enemy
    SingleEnemy,
    /// All enemies
    AllEnemies,
    /// One ally
    SingleAlly,
    /// All allies
    AllAllies,
    /// The caster only
    Self_,
}

impl TargetType {
    /// Check if this targets enemies
    pub fn targets_enemies(&self) -> bool {
        matches!(self, TargetType::SingleEnemy | TargetType::AllEnemies)
    }

    /// Check if this targets allies
    pub fn targets_allies(&self) -> bool {
        matches!(
            self,
            TargetType::SingleAlly | TargetType::AllAllies | TargetType::Self_
        )
    }

    /// Check if this targets multiple
    pub fn is_multi_target(&self) -> bool {
        matches!(self, TargetType::AllEnemies | TargetType::AllAllies)
    }
}

/// Type of spell effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellType {
    /// Deals damage
    Damage,
    /// Restores HP
    Heal,
    /// Applies a buff
    Buff,
    /// Applies a debuff
    Debuff,
    /// Inflicts a status ailment
    StatusInflict,
    /// Cures a status ailment
    StatusCure,
    /// Revives from KO
    Revive,
}

/// Definition of a spell
#[derive(Debug, Clone)]
pub struct Spell {
    pub id: &'static str,
    pub name: &'static str,
    pub mp_cost: i32,
    pub spell_type: SpellType,
    pub element: Element,
    pub power: i32,
    pub target_type: TargetType,
    pub level_required: u8,
    pub description: &'static str,
}

/// All available spells
pub static SPELLS: &[Spell] = &[
    // ========== BLACK MAGIC (Damage) ==========
    Spell {
        id: "fire",
        name: "Fire",
        mp_cost: 4,
        spell_type: SpellType::Damage,
        element: Element::Fire,
        power: 20,
        target_type: TargetType::SingleEnemy,
        level_required: 1,
        description: "Fire damage to one enemy.",
    },
    Spell {
        id: "fira",
        name: "Fira",
        mp_cost: 12,
        spell_type: SpellType::Damage,
        element: Element::Fire,
        power: 50,
        target_type: TargetType::SingleEnemy,
        level_required: 8,
        description: "Medium fire damage to one enemy.",
    },
    Spell {
        id: "firaga",
        name: "Firaga",
        mp_cost: 30,
        spell_type: SpellType::Damage,
        element: Element::Fire,
        power: 120,
        target_type: TargetType::AllEnemies,
        level_required: 20,
        description: "Heavy fire damage to all enemies.",
    },
    Spell {
        id: "blizzard",
        name: "Blizzard",
        mp_cost: 4,
        spell_type: SpellType::Damage,
        element: Element::Ice,
        power: 18,
        target_type: TargetType::SingleEnemy,
        level_required: 1,
        description: "Ice damage to one enemy.",
    },
    Spell {
        id: "blizzara",
        name: "Blizzara",
        mp_cost: 12,
        spell_type: SpellType::Damage,
        element: Element::Ice,
        power: 48,
        target_type: TargetType::SingleEnemy,
        level_required: 8,
        description: "Medium ice damage to one enemy.",
    },
    Spell {
        id: "blizzaga",
        name: "Blizzaga",
        mp_cost: 30,
        spell_type: SpellType::Damage,
        element: Element::Ice,
        power: 110,
        target_type: TargetType::AllEnemies,
        level_required: 20,
        description: "Heavy ice damage to all enemies.",
    },
    Spell {
        id: "thunder",
        name: "Thunder",
        mp_cost: 4,
        spell_type: SpellType::Damage,
        element: Element::Lightning,
        power: 22,
        target_type: TargetType::SingleEnemy,
        level_required: 3,
        description: "Lightning damage to one enemy.",
    },
    Spell {
        id: "thundara",
        name: "Thundara",
        mp_cost: 12,
        spell_type: SpellType::Damage,
        element: Element::Lightning,
        power: 55,
        target_type: TargetType::SingleEnemy,
        level_required: 10,
        description: "Medium lightning damage to one enemy.",
    },
    Spell {
        id: "thundaga",
        name: "Thundaga",
        mp_cost: 30,
        spell_type: SpellType::Damage,
        element: Element::Lightning,
        power: 130,
        target_type: TargetType::AllEnemies,
        level_required: 22,
        description: "Heavy lightning damage to all enemies.",
    },
    Spell {
        id: "quake",
        name: "Quake",
        mp_cost: 25,
        spell_type: SpellType::Damage,
        element: Element::Earth,
        power: 100,
        target_type: TargetType::AllEnemies,
        level_required: 18,
        description: "Earth damage to all enemies.",
    },
    Spell {
        id: "aero",
        name: "Aero",
        mp_cost: 5,
        spell_type: SpellType::Damage,
        element: Element::Wind,
        power: 25,
        target_type: TargetType::SingleEnemy,
        level_required: 5,
        description: "Wind damage to one enemy.",
    },
    Spell {
        id: "water",
        name: "Water",
        mp_cost: 5,
        spell_type: SpellType::Damage,
        element: Element::Water,
        power: 25,
        target_type: TargetType::SingleEnemy,
        level_required: 5,
        description: "Water damage to one enemy.",
    },
    Spell {
        id: "flare",
        name: "Flare",
        mp_cost: 50,
        spell_type: SpellType::Damage,
        element: Element::None,
        power: 200,
        target_type: TargetType::SingleEnemy,
        level_required: 35,
        description: "Devastating non-elemental damage.",
    },
    Spell {
        id: "holy",
        name: "Holy",
        mp_cost: 45,
        spell_type: SpellType::Damage,
        element: Element::Holy,
        power: 180,
        target_type: TargetType::SingleEnemy,
        level_required: 30,
        description: "Holy damage to one enemy. Strong vs undead.",
    },
    Spell {
        id: "dark",
        name: "Dark",
        mp_cost: 8,
        spell_type: SpellType::Damage,
        element: Element::Dark,
        power: 35,
        target_type: TargetType::SingleEnemy,
        level_required: 7,
        description: "Dark damage to one enemy.",
    },
    // ========== WHITE MAGIC (Healing/Support) ==========
    Spell {
        id: "cure",
        name: "Cure",
        mp_cost: 4,
        spell_type: SpellType::Heal,
        element: Element::None,
        power: 30,
        target_type: TargetType::SingleAlly,
        level_required: 1,
        description: "Restore HP to one ally.",
    },
    Spell {
        id: "cura",
        name: "Cura",
        mp_cost: 12,
        spell_type: SpellType::Heal,
        element: Element::None,
        power: 80,
        target_type: TargetType::SingleAlly,
        level_required: 8,
        description: "Restore more HP to one ally.",
    },
    Spell {
        id: "curaga",
        name: "Curaga",
        mp_cost: 30,
        spell_type: SpellType::Heal,
        element: Element::None,
        power: 180,
        target_type: TargetType::AllAllies,
        level_required: 20,
        description: "Restore HP to all allies.",
    },
    Spell {
        id: "full_cure",
        name: "Full Cure",
        mp_cost: 50,
        spell_type: SpellType::Heal,
        element: Element::None,
        power: 999,
        target_type: TargetType::SingleAlly,
        level_required: 35,
        description: "Fully restore one ally's HP.",
    },
    Spell {
        id: "regen",
        name: "Regen",
        mp_cost: 10,
        spell_type: SpellType::Buff,
        element: Element::None,
        power: 0,
        target_type: TargetType::SingleAlly,
        level_required: 12,
        description: "Grant regeneration to one ally.",
    },
    Spell {
        id: "raise",
        name: "Raise",
        mp_cost: 20,
        spell_type: SpellType::Revive,
        element: Element::Holy,
        power: 25, // 25% HP on revive
        target_type: TargetType::SingleAlly,
        level_required: 15,
        description: "Revive fallen ally with some HP.",
    },
    Spell {
        id: "arise",
        name: "Arise",
        mp_cost: 50,
        spell_type: SpellType::Revive,
        element: Element::Holy,
        power: 100, // Full HP on revive
        target_type: TargetType::SingleAlly,
        level_required: 30,
        description: "Revive fallen ally with full HP.",
    },
    // ========== BUFFS ==========
    Spell {
        id: "protect",
        name: "Protect",
        mp_cost: 6,
        spell_type: SpellType::Buff,
        element: Element::None,
        power: 0,
        target_type: TargetType::SingleAlly,
        level_required: 3,
        description: "Increase ally's defense.",
    },
    Spell {
        id: "shell",
        name: "Shell",
        mp_cost: 6,
        spell_type: SpellType::Buff,
        element: Element::None,
        power: 0,
        target_type: TargetType::SingleAlly,
        level_required: 6,
        description: "Increase ally's magic defense.",
    },
    Spell {
        id: "haste",
        name: "Haste",
        mp_cost: 15,
        spell_type: SpellType::Buff,
        element: Element::None,
        power: 0,
        target_type: TargetType::SingleAlly,
        level_required: 12,
        description: "Speed up ally's actions.",
    },
    Spell {
        id: "protectga",
        name: "Protectga",
        mp_cost: 20,
        spell_type: SpellType::Buff,
        element: Element::None,
        power: 0,
        target_type: TargetType::AllAllies,
        level_required: 18,
        description: "Increase all allies' defense.",
    },
    Spell {
        id: "shellga",
        name: "Shellga",
        mp_cost: 20,
        spell_type: SpellType::Buff,
        element: Element::None,
        power: 0,
        target_type: TargetType::AllAllies,
        level_required: 18,
        description: "Increase all allies' magic defense.",
    },
    // ========== DEBUFFS / STATUS ==========
    Spell {
        id: "poison",
        name: "Poison",
        mp_cost: 5,
        spell_type: SpellType::StatusInflict,
        element: Element::None,
        power: 50, // 50% chance
        target_type: TargetType::SingleEnemy,
        level_required: 4,
        description: "Inflict poison on one enemy.",
    },
    Spell {
        id: "sleep",
        name: "Sleep",
        mp_cost: 8,
        spell_type: SpellType::StatusInflict,
        element: Element::None,
        power: 40, // 40% chance
        target_type: TargetType::SingleEnemy,
        level_required: 6,
        description: "Put one enemy to sleep.",
    },
    Spell {
        id: "silence",
        name: "Silence",
        mp_cost: 6,
        spell_type: SpellType::StatusInflict,
        element: Element::None,
        power: 45, // 45% chance
        target_type: TargetType::SingleEnemy,
        level_required: 5,
        description: "Prevent enemy from casting spells.",
    },
    Spell {
        id: "blind",
        name: "Blind",
        mp_cost: 5,
        spell_type: SpellType::StatusInflict,
        element: Element::None,
        power: 50, // 50% chance
        target_type: TargetType::SingleEnemy,
        level_required: 4,
        description: "Reduce enemy's accuracy.",
    },
    Spell {
        id: "slow",
        name: "Slow",
        mp_cost: 10,
        spell_type: SpellType::StatusInflict,
        element: Element::None,
        power: 40, // 40% chance
        target_type: TargetType::SingleEnemy,
        level_required: 8,
        description: "Slow down enemy's actions.",
    },
    Spell {
        id: "break",
        name: "Break",
        mp_cost: 25,
        spell_type: SpellType::StatusInflict,
        element: Element::Earth,
        power: 20, // 20% chance
        target_type: TargetType::SingleEnemy,
        level_required: 15,
        description: "Attempt to petrify one enemy.",
    },
    Spell {
        id: "death",
        name: "Death",
        mp_cost: 35,
        spell_type: SpellType::StatusInflict,
        element: Element::Dark,
        power: 10, // 10% chance
        target_type: TargetType::SingleEnemy,
        level_required: 25,
        description: "Attempt to instantly kill one enemy.",
    },
    // ========== STATUS CURE ==========
    Spell {
        id: "esuna",
        name: "Esuna",
        mp_cost: 8,
        spell_type: SpellType::StatusCure,
        element: Element::None,
        power: 0,
        target_type: TargetType::SingleAlly,
        level_required: 10,
        description: "Cure all negative status from one ally.",
    },
    Spell {
        id: "stona",
        name: "Stona",
        mp_cost: 12,
        spell_type: SpellType::StatusCure,
        element: Element::None,
        power: 0,
        target_type: TargetType::SingleAlly,
        level_required: 12,
        description: "Cure petrification from one ally.",
    },
];

/// Look up a spell by ID
pub fn get_spell(id: &str) -> Option<&'static Spell> {
    SPELLS.iter().find(|s| s.id == id)
}

/// Get all spells learnable at or below a level
pub fn get_spells_for_level(level: u8) -> Vec<&'static Spell> {
    SPELLS
        .iter()
        .filter(|s| s.level_required <= level)
        .collect()
}

/// Get all spells of a specific type
pub fn get_spells_by_type(spell_type: SpellType) -> Vec<&'static Spell> {
    SPELLS
        .iter()
        .filter(|s| s.spell_type == spell_type)
        .collect()
}

/// Get all damage spells of a specific element
pub fn get_damage_spells_by_element(element: Element) -> Vec<&'static Spell> {
    SPELLS
        .iter()
        .filter(|s| s.spell_type == SpellType::Damage && s.element == element)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_names() {
        assert_eq!(Element::Fire.name(), "Fire");
        assert_eq!(Element::Holy.name(), "Holy");
        assert_eq!(Element::None.name(), "Non-elemental");
    }

    #[test]
    fn test_element_weakness() {
        assert_eq!(Element::Fire.weakness(), Some(Element::Water));
        assert_eq!(Element::Ice.weakness(), Some(Element::Fire));
        assert_eq!(Element::Holy.weakness(), Some(Element::Dark));
        assert_eq!(Element::None.weakness(), None);
    }

    #[test]
    fn test_target_type_enemies() {
        assert!(TargetType::SingleEnemy.targets_enemies());
        assert!(TargetType::AllEnemies.targets_enemies());
        assert!(!TargetType::SingleAlly.targets_enemies());
    }

    #[test]
    fn test_target_type_allies() {
        assert!(TargetType::SingleAlly.targets_allies());
        assert!(TargetType::AllAllies.targets_allies());
        assert!(TargetType::Self_.targets_allies());
        assert!(!TargetType::SingleEnemy.targets_allies());
    }

    #[test]
    fn test_target_type_multi() {
        assert!(TargetType::AllEnemies.is_multi_target());
        assert!(TargetType::AllAllies.is_multi_target());
        assert!(!TargetType::SingleEnemy.is_multi_target());
    }

    #[test]
    fn test_get_spell() {
        let fire = get_spell("fire").unwrap();
        assert_eq!(fire.name, "Fire");
        assert_eq!(fire.element, Element::Fire);
        assert_eq!(fire.spell_type, SpellType::Damage);

        let cure = get_spell("cure").unwrap();
        assert_eq!(cure.name, "Cure");
        assert_eq!(cure.spell_type, SpellType::Heal);
    }

    #[test]
    fn test_get_spell_not_found() {
        assert!(get_spell("nonexistent").is_none());
    }

    #[test]
    fn test_get_spells_for_level() {
        let level_1_spells = get_spells_for_level(1);
        assert!(level_1_spells.iter().any(|s| s.id == "fire"));
        assert!(level_1_spells.iter().any(|s| s.id == "cure"));
        assert!(!level_1_spells.iter().any(|s| s.id == "flare")); // Level 35

        let level_35_spells = get_spells_for_level(35);
        assert!(level_35_spells.iter().any(|s| s.id == "flare"));
    }

    #[test]
    fn test_get_spells_by_type() {
        let damage_spells = get_spells_by_type(SpellType::Damage);
        assert!(damage_spells.iter().all(|s| s.spell_type == SpellType::Damage));
        assert!(damage_spells.iter().any(|s| s.id == "fire"));

        let heal_spells = get_spells_by_type(SpellType::Heal);
        assert!(heal_spells.iter().all(|s| s.spell_type == SpellType::Heal));
        assert!(heal_spells.iter().any(|s| s.id == "cure"));
    }

    #[test]
    fn test_get_damage_spells_by_element() {
        let fire_spells = get_damage_spells_by_element(Element::Fire);
        assert!(fire_spells.iter().all(|s| s.element == Element::Fire));
        assert!(fire_spells.iter().any(|s| s.id == "fire"));
        assert!(fire_spells.iter().any(|s| s.id == "fira"));
        assert!(fire_spells.iter().any(|s| s.id == "firaga"));
    }

    #[test]
    fn test_spell_costs_and_power_scaling() {
        let fire = get_spell("fire").unwrap();
        let fira = get_spell("fira").unwrap();
        let firaga = get_spell("firaga").unwrap();

        // Higher tier spells cost more MP
        assert!(fira.mp_cost > fire.mp_cost);
        assert!(firaga.mp_cost > fira.mp_cost);

        // Higher tier spells are more powerful
        assert!(fira.power > fire.power);
        assert!(firaga.power > fira.power);

        // Higher tier spells require higher level
        assert!(fira.level_required > fire.level_required);
        assert!(firaga.level_required > fira.level_required);
    }

    #[test]
    fn test_all_spells_have_valid_data() {
        for spell in SPELLS {
            assert!(!spell.id.is_empty());
            assert!(!spell.name.is_empty());
            assert!(spell.mp_cost >= 0);
            assert!(!spell.description.is_empty());
        }
    }
}
