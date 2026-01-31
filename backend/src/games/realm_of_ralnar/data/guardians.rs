//! Guardian definitions for Realm of Ralnar
//!
//! Contains the five Guardian boss definitions - the tragic protectors
//! who the party unknowingly kills.

use super::items::Element;
use super::enemies::{EnemyStats, EnemyAttack, EnemyAIType, DamageType};

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Guardian phase based on HP percentage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardianPhase {
    /// 100-75% HP - Defensive only, trying to discourage
    Warning,
    /// 75-50% HP - Light retaliation, still mostly defensive
    Reluctant,
    /// 50-25% HP - Stronger defense + counter, survival mode
    Desperate,
    /// Below 25% HP - Full power, fighting for life
    LastStand,
}

impl GuardianPhase {
    /// Get the phase based on HP percentage
    pub fn from_hp_percent(hp_percent: f32) -> Self {
        if hp_percent > 0.75 {
            GuardianPhase::Warning
        } else if hp_percent > 0.50 {
            GuardianPhase::Reluctant
        } else if hp_percent > 0.25 {
            GuardianPhase::Desperate
        } else {
            GuardianPhase::LastStand
        }
    }
}

/// Full Guardian definition
#[derive(Debug, Clone)]
pub struct GuardianDef {
    pub id: &'static str,
    /// True name (revealed after death)
    pub name: &'static str,
    /// What the party sees under the illusion
    pub illusioned_name: &'static str,
    /// Element the Guardian represents
    pub element: Element,
    pub hp: i32,
    pub stats: EnemyStats,
    /// Guardian AI is always defensive
    pub ai_type: EnemyAIType,
    /// Which shrine (1-5)
    pub shrine: u8,
    /// Power granted to party after killing (the stolen divine essence)
    pub power_granted: &'static str,
    /// Last words before death (filtered by illusion during battle)
    pub death_message: &'static str,
    /// True last words (what they actually said)
    pub true_death_message: &'static str,
    /// Attacks available
    pub attacks: &'static [EnemyAttack],
    /// Defensive abilities
    pub defensive_abilities: &'static [&'static str],
    /// What the Guardian is trying to protect
    pub purpose: &'static str,
    /// Experience reward (meaningful for first 4, story-based for 5th)
    pub exp_reward: u32,
    /// Gold reward
    pub gold_min: u32,
    pub gold_max: u32,
}

// ============================================================================
// GUARDIAN DEFINITIONS
// ============================================================================

/// Spirata - Spirit Guardian (Shrine 1)
/// Element: Holy
/// First Guardian killed, grants Soul Sight
pub const SPIRATA: GuardianDef = GuardianDef {
    id: "spirata",
    name: "Spirit Guardian Spirata",
    illusioned_name: "Shade Demon",
    element: Element::Holy,
    hp: 500,
    stats: EnemyStats { attack: 30, defense: 20, magic: 40, magic_def: 30, agility: 15 },
    ai_type: EnemyAIType::Guardian,
    shrine: 1,
    power_granted: "soul_sight",
    death_message: "The light... fades...",
    true_death_message: "Why... why do you attack... the ones who protect you...",
    attacks: &[
        EnemyAttack {
            name: "Spirit Barrier",
            damage_type: DamageType::Magical,
            power: 0,
            accuracy: 1.0,
            element: Element::Holy,
            status_inflict: None,
            mp_cost: 10,
            weight: 100,
        },
        EnemyAttack {
            name: "Holy Light",
            damage_type: DamageType::Magical,
            power: 35,
            accuracy: 0.95,
            element: Element::Holy,
            status_inflict: None,
            mp_cost: 8,
            weight: 40,
        },
        EnemyAttack {
            name: "Purifying Wave",
            damage_type: DamageType::Magical,
            power: 50,
            accuracy: 0.90,
            element: Element::Holy,
            status_inflict: None,
            mp_cost: 15,
            weight: 20,
        },
    ],
    defensive_abilities: &["Spirit Shield", "Ethereal Form", "Light Barrier"],
    purpose: "Guardian of souls and the spirit realm. Spirata guides the departed and protects the living from malevolent spirits.",
    exp_reward: 500,
    gold_min: 200,
    gold_max: 400,
};

/// Terreth - Earth Guardian (Shrine 2)
/// Element: Earth
/// Second Guardian killed, grants Stone Skin
pub const TERRETH: GuardianDef = GuardianDef {
    id: "terreth",
    name: "Earth Guardian Terreth",
    illusioned_name: "Stone Colossus",
    element: Element::Earth,
    hp: 800,
    stats: EnemyStats { attack: 45, defense: 45, magic: 20, magic_def: 25, agility: 8 },
    ai_type: EnemyAIType::Guardian,
    shrine: 2,
    power_granted: "stone_skin",
    death_message: "The mountain... crumbles...",
    true_death_message: "I am... the last wall... between you and darkness...",
    attacks: &[
        EnemyAttack {
            name: "Stone Wall",
            damage_type: DamageType::Physical,
            power: 0,
            accuracy: 1.0,
            element: Element::Earth,
            status_inflict: None,
            mp_cost: 10,
            weight: 100,
        },
        EnemyAttack {
            name: "Boulder Strike",
            damage_type: DamageType::Physical,
            power: 50,
            accuracy: 0.85,
            element: Element::Earth,
            status_inflict: None,
            mp_cost: 5,
            weight: 40,
        },
        EnemyAttack {
            name: "Earthquake",
            damage_type: DamageType::Physical,
            power: 70,
            accuracy: 0.80,
            element: Element::Earth,
            status_inflict: None,
            mp_cost: 20,
            weight: 20,
        },
    ],
    defensive_abilities: &["Stone Armor", "Earth's Embrace", "Mountain's Resolve"],
    purpose: "Guardian of the land and its foundations. Terreth keeps the earth stable and prevents catastrophic shifts.",
    exp_reward: 800,
    gold_min: 300,
    gold_max: 600,
};

/// Aqualis - Water Guardian (Shrine 3)
/// Element: Water
/// Third Guardian killed, grants Healing Tide
pub const AQUALIS: GuardianDef = GuardianDef {
    id: "aqualis",
    name: "Water Guardian Aqualis",
    illusioned_name: "Sea Serpent",
    element: Element::Water,
    hp: 650,
    stats: EnemyStats { attack: 35, defense: 25, magic: 45, magic_def: 40, agility: 12 },
    ai_type: EnemyAIType::Guardian,
    shrine: 3,
    power_granted: "healing_tide",
    death_message: "The waters... still...",
    true_death_message: "I only wished... to heal... to nurture...",
    attacks: &[
        EnemyAttack {
            name: "Healing Waters",
            damage_type: DamageType::Magical,
            power: -50, // Negative = healing
            accuracy: 1.0,
            element: Element::Water,
            status_inflict: None,
            mp_cost: 15,
            weight: 100,
        },
        EnemyAttack {
            name: "Tidal Wave",
            damage_type: DamageType::Magical,
            power: 55,
            accuracy: 0.90,
            element: Element::Water,
            status_inflict: None,
            mp_cost: 12,
            weight: 30,
        },
        EnemyAttack {
            name: "Whirlpool",
            damage_type: DamageType::Magical,
            power: 40,
            accuracy: 0.95,
            element: Element::Water,
            status_inflict: None,
            mp_cost: 8,
            weight: 40,
        },
    ],
    defensive_abilities: &["Water Shield", "Cleansing Flow", "Ocean's Calm"],
    purpose: "Guardian of waters and healing. Aqualis purifies rivers, blesses rain, and mends wounds.",
    exp_reward: 700,
    gold_min: 250,
    gold_max: 500,
};

/// Ventus - Wind Guardian (Shrine 4)
/// Element: Wind
/// Fourth Guardian killed, grants Haste
pub const VENTUS: GuardianDef = GuardianDef {
    id: "ventus",
    name: "Wind Guardian Ventus",
    illusioned_name: "Storm Wraith",
    element: Element::Wind,
    hp: 550,
    stats: EnemyStats { attack: 40, defense: 18, magic: 38, magic_def: 35, agility: 25 },
    ai_type: EnemyAIType::Guardian,
    shrine: 4,
    power_granted: "haste",
    death_message: "The wind... stops...",
    true_death_message: "Freedom... I only wanted... to give you freedom...",
    attacks: &[
        EnemyAttack {
            name: "Gust Shield",
            damage_type: DamageType::Magical,
            power: 0,
            accuracy: 1.0,
            element: Element::Wind,
            status_inflict: None,
            mp_cost: 8,
            weight: 100,
        },
        EnemyAttack {
            name: "Wind Slash",
            damage_type: DamageType::Magical,
            power: 45,
            accuracy: 0.95,
            element: Element::Wind,
            status_inflict: None,
            mp_cost: 6,
            weight: 50,
        },
        EnemyAttack {
            name: "Tempest",
            damage_type: DamageType::Magical,
            power: 65,
            accuracy: 0.85,
            element: Element::Wind,
            status_inflict: None,
            mp_cost: 18,
            weight: 20,
        },
    ],
    defensive_abilities: &["Wind Barrier", "Evasive Currents", "Sky's Protection"],
    purpose: "Guardian of winds and freedom. Ventus carries messages, guides travelers, and ensures fair weather.",
    exp_reward: 750,
    gold_min: 280,
    gold_max: 550,
};

/// Pyreth - Fire Guardian (Shrine 5)
/// Element: Fire
/// Final Guardian killed, THE REVEAL
pub const PYRETH: GuardianDef = GuardianDef {
    id: "pyreth",
    name: "Fire Guardian Pyreth",
    illusioned_name: "Inferno Demon",
    element: Element::Fire,
    hp: 900,
    stats: EnemyStats { attack: 55, defense: 30, magic: 50, magic_def: 35, agility: 14 },
    ai_type: EnemyAIType::Guardian,
    shrine: 5,
    power_granted: "inferno",
    death_message: "The flame... extinguishes...",
    true_death_message: "Child of faith... you tried... I heard... your true voice...",
    attacks: &[
        EnemyAttack {
            name: "Blazing Shield",
            damage_type: DamageType::Magical,
            power: 0,
            accuracy: 1.0,
            element: Element::Fire,
            status_inflict: None,
            mp_cost: 10,
            weight: 100,
        },
        EnemyAttack {
            name: "Flame Burst",
            damage_type: DamageType::Magical,
            power: 55,
            accuracy: 0.90,
            element: Element::Fire,
            status_inflict: None,
            mp_cost: 10,
            weight: 40,
        },
        EnemyAttack {
            name: "Infernal Wave",
            damage_type: DamageType::Magical,
            power: 80,
            accuracy: 0.85,
            element: Element::Fire,
            status_inflict: None,
            mp_cost: 20,
            weight: 25,
        },
        EnemyAttack {
            name: "Purifying Flames",
            damage_type: DamageType::Magical,
            power: 100,
            accuracy: 0.80,
            element: Element::Fire,
            status_inflict: None,
            mp_cost: 30,
            weight: 10,
        },
    ],
    defensive_abilities: &["Fire Wall", "Phoenix Barrier", "Eternal Flame"],
    purpose: "Guardian of fire and passion. Pyreth lights the hearth, inspires courage, and burns away corruption.",
    exp_reward: 1000,
    gold_min: 350,
    gold_max: 700,
};

// ============================================================================
// GUARDIAN COLLECTION AND LOOKUP
// ============================================================================

/// All Guardians in order (by shrine number)
pub static ALL_GUARDIANS: &[&GuardianDef] = &[
    &SPIRATA,  // Shrine 1 - Spirit
    &TERRETH,  // Shrine 2 - Earth
    &AQUALIS,  // Shrine 3 - Water
    &VENTUS,   // Shrine 4 - Wind
    &PYRETH,   // Shrine 5 - Fire (THE REVEAL)
];

/// Look up a Guardian by ID
pub fn get_guardian(id: &str) -> Option<&'static GuardianDef> {
    ALL_GUARDIANS.iter().find(|g| g.id == id).copied()
}

/// Get a Guardian by shrine number (1-5)
pub fn get_guardian_by_shrine(shrine: u8) -> Option<&'static GuardianDef> {
    if shrine == 0 || shrine > 5 {
        return None;
    }
    Some(ALL_GUARDIANS[(shrine - 1) as usize])
}

/// Get a Guardian by element
pub fn get_guardian_by_element(element: Element) -> Option<&'static GuardianDef> {
    ALL_GUARDIANS.iter()
        .find(|g| g.element == element)
        .copied()
}

/// Get the power (spell ID) granted by killing a Guardian
pub fn get_guardian_power(shrine: u8) -> Option<&'static str> {
    get_guardian_by_shrine(shrine).map(|g| g.power_granted)
}

/// Get all Guardian powers in order
pub fn get_all_guardian_powers() -> Vec<&'static str> {
    ALL_GUARDIANS.iter().map(|g| g.power_granted).collect()
}

/// Get the illusioned (false) name for a Guardian
pub fn get_illusioned_name(shrine: u8) -> Option<&'static str> {
    get_guardian_by_shrine(shrine).map(|g| g.illusioned_name)
}

/// Get the true death message (what they actually said)
pub fn get_true_death_message(shrine: u8) -> Option<&'static str> {
    get_guardian_by_shrine(shrine).map(|g| g.true_death_message)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_guardians_have_unique_ids() {
        let mut ids: Vec<&str> = ALL_GUARDIANS.iter().map(|g| g.id).collect();
        ids.sort();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Duplicate Guardian IDs found");
    }

    #[test]
    fn test_exactly_five_guardians() {
        assert_eq!(ALL_GUARDIANS.len(), 5, "Should have exactly 5 Guardians");
    }

    #[test]
    fn test_guardian_shrine_numbers() {
        for (i, guardian) in ALL_GUARDIANS.iter().enumerate() {
            assert_eq!(guardian.shrine, (i + 1) as u8, "Guardian {} should be shrine {}", guardian.id, i + 1);
        }
    }

    #[test]
    fn test_get_guardian_by_shrine() {
        // Valid shrines
        for shrine in 1..=5 {
            let guardian = get_guardian_by_shrine(shrine);
            assert!(guardian.is_some(), "Should find Guardian for shrine {}", shrine);
            assert_eq!(guardian.unwrap().shrine, shrine);
        }

        // Invalid shrines
        assert!(get_guardian_by_shrine(0).is_none());
        assert!(get_guardian_by_shrine(6).is_none());
    }

    #[test]
    fn test_guardian_elements_unique() {
        let elements: Vec<Element> = ALL_GUARDIANS.iter().map(|g| g.element).collect();
        let expected = vec![Element::Holy, Element::Earth, Element::Water, Element::Wind, Element::Fire];

        for element in expected {
            assert!(elements.contains(&element), "Should have {:?} Guardian", element);
        }
    }

    #[test]
    fn test_guardian_powers_match_spells() {
        let powers = get_all_guardian_powers();
        assert_eq!(powers.len(), 5);
        assert!(powers.contains(&"soul_sight"));
        assert!(powers.contains(&"stone_skin"));
        assert!(powers.contains(&"healing_tide"));
        assert!(powers.contains(&"haste"));
        assert!(powers.contains(&"inferno"));
    }

    #[test]
    fn test_guardians_have_attacks() {
        for guardian in ALL_GUARDIANS.iter() {
            assert!(!guardian.attacks.is_empty(), "Guardian {} should have attacks", guardian.id);
        }
    }

    #[test]
    fn test_guardians_have_defensive_abilities() {
        for guardian in ALL_GUARDIANS.iter() {
            assert!(!guardian.defensive_abilities.is_empty(), "Guardian {} should have defensive abilities", guardian.id);
        }
    }

    #[test]
    fn test_guardian_ai_type() {
        for guardian in ALL_GUARDIANS.iter() {
            assert_eq!(guardian.ai_type, EnemyAIType::Guardian, "Guardian {} should have Guardian AI type", guardian.id);
        }
    }

    #[test]
    fn test_guardian_hp_positive() {
        for guardian in ALL_GUARDIANS.iter() {
            assert!(guardian.hp > 0, "Guardian {} should have positive HP", guardian.id);
        }
    }

    #[test]
    fn test_pyreth_is_reveal_guardian() {
        let pyreth = get_guardian_by_shrine(5);
        assert!(pyreth.is_some());
        let pyreth = pyreth.unwrap();
        assert_eq!(pyreth.id, "pyreth");
        assert_eq!(pyreth.element, Element::Fire);
        // Pyreth has the most HP and the most attacks (the climactic fight)
        assert!(pyreth.hp >= 900);
        assert!(pyreth.attacks.len() >= 4);
    }

    #[test]
    fn test_illusioned_names_different_from_true_names() {
        for guardian in ALL_GUARDIANS.iter() {
            assert_ne!(guardian.name, guardian.illusioned_name,
                "Guardian {} illusioned name should differ from true name", guardian.id);
        }
    }

    #[test]
    fn test_guardian_phase_calculation() {
        assert_eq!(GuardianPhase::from_hp_percent(1.0), GuardianPhase::Warning);
        assert_eq!(GuardianPhase::from_hp_percent(0.80), GuardianPhase::Warning);
        assert_eq!(GuardianPhase::from_hp_percent(0.75), GuardianPhase::Reluctant);
        assert_eq!(GuardianPhase::from_hp_percent(0.60), GuardianPhase::Reluctant);
        assert_eq!(GuardianPhase::from_hp_percent(0.50), GuardianPhase::Desperate);
        assert_eq!(GuardianPhase::from_hp_percent(0.30), GuardianPhase::Desperate);
        assert_eq!(GuardianPhase::from_hp_percent(0.25), GuardianPhase::LastStand);
        assert_eq!(GuardianPhase::from_hp_percent(0.10), GuardianPhase::LastStand);
    }
}
