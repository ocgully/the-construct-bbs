//! Enemy definitions for Realm of Ralnar
//!
//! Contains all regular enemies and non-guardian bosses.

use super::items::Element;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Enemy AI behavior patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EnemyAIType {
    /// Normal balanced behavior
    Normal,
    /// Aggressive, focuses on damage
    Aggressive,
    /// Defensive, buff-focused
    Defensive,
    /// Magic-focused attacks
    Magical,
    /// Mage enemy type (prefers spells)
    Mage,
    /// Uses status effects
    Debuffer,
    /// Healer enemy type (supports allies)
    Healer,
    /// Summons other enemies
    Summoner,
    /// Aggressive, high damage, berserker style
    Berserker,
    /// Boss-level AI (smarter, more varied)
    Boss,
    /// Guardian-style defensive (for the tragic Guardian fights)
    Guardian,
}

impl EnemyAIType {
    /// Get the display name for this AI type
    pub fn name(&self) -> &'static str {
        match self {
            EnemyAIType::Normal => "Normal",
            EnemyAIType::Aggressive => "Aggressive",
            EnemyAIType::Defensive => "Defensive",
            EnemyAIType::Magical => "Magical",
            EnemyAIType::Mage => "Mage",
            EnemyAIType::Debuffer => "Debuffer",
            EnemyAIType::Healer => "Healer",
            EnemyAIType::Summoner => "Summoner",
            EnemyAIType::Berserker => "Berserker",
            EnemyAIType::Boss => "Boss",
            EnemyAIType::Guardian => "Guardian",
        }
    }
}

/// Enemy attack definition
#[derive(Debug, Clone)]
pub struct EnemyAttack {
    pub name: &'static str,
    pub damage_type: DamageType,
    pub power: i32,
    pub accuracy: f32,
    pub element: Element,
    pub status_inflict: Option<(StatusInflict, f32)>,
    pub mp_cost: i32,
    pub weight: u32,
}

impl EnemyAttack {
    /// Create a basic physical attack
    pub const fn basic(name: &'static str, power: i32) -> Self {
        Self {
            name,
            damage_type: DamageType::Physical,
            power,
            accuracy: 0.90,
            element: Element::None,
            status_inflict: None,
            mp_cost: 0,
            weight: 100,
        }
    }

    /// Create a magical attack
    pub const fn magical(name: &'static str, power: i32, element: Element) -> Self {
        Self {
            name,
            damage_type: DamageType::Magical,
            power,
            accuracy: 0.95,
            element,
            status_inflict: None,
            mp_cost: 5,
            weight: 50,
        }
    }
}

/// Damage type for attacks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Magical,
}

/// Status effects enemies can inflict
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusInflict {
    Poison,
    Stone,
    Confusion,
    Sleep,
    Blind,
    Silence,
    Slow,
}

/// Enemy stats
#[derive(Debug, Clone, Copy)]
pub struct EnemyStats {
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub magic_def: i32,
    pub agility: i32,
}

/// Full enemy definition
#[derive(Debug, Clone)]
pub struct EnemyDef {
    pub id: &'static str,
    pub name: &'static str,
    pub hp: i32,
    pub stats: EnemyStats,
    pub attacks: &'static [EnemyAttack],
    pub weaknesses: &'static [Element],
    pub resistances: &'static [Element],
    pub exp_reward: u32,
    pub gold_min: u32,
    pub gold_max: u32,
    pub drops: &'static [(&'static str, f32)],
    pub ai_type: EnemyAIType,
    pub is_boss: bool,
}

// ============================================================================
// EARLY GAME ENEMIES (Regions 1-2)
// ============================================================================

pub const SLIME: EnemyDef = EnemyDef {
    id: "slime",
    name: "Slime",
    hp: 15,
    stats: EnemyStats { attack: 5, defense: 2, magic: 0, magic_def: 5, agility: 3 },
    attacks: &[EnemyAttack::basic("Tackle", 5)],
    weaknesses: &[Element::Fire],
    resistances: &[],
    exp_reward: 10,
    gold_min: 5,
    gold_max: 15,
    drops: &[("potion", 0.10)],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

pub const WOLF: EnemyDef = EnemyDef {
    id: "wolf",
    name: "Wolf",
    hp: 25,
    stats: EnemyStats { attack: 8, defense: 3, magic: 0, magic_def: 3, agility: 8 },
    attacks: &[
        EnemyAttack::basic("Bite", 8),
        EnemyAttack::basic("Claw", 6),
    ],
    weaknesses: &[Element::Fire],
    resistances: &[],
    exp_reward: 18,
    gold_min: 8,
    gold_max: 20,
    drops: &[("antidote", 0.15)],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

pub const GOBLIN: EnemyDef = EnemyDef {
    id: "goblin",
    name: "Goblin",
    hp: 30,
    stats: EnemyStats { attack: 10, defense: 5, magic: 2, magic_def: 4, agility: 6 },
    attacks: &[
        EnemyAttack::basic("Club Swing", 10),
        EnemyAttack::basic("Thrown Rock", 7),
    ],
    weaknesses: &[Element::Holy],
    resistances: &[],
    exp_reward: 22,
    gold_min: 15,
    gold_max: 35,
    drops: &[("potion", 0.15), ("dagger", 0.05)],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

pub const GIANT_SPIDER: EnemyDef = EnemyDef {
    id: "giant_spider",
    name: "Giant Spider",
    hp: 45,
    stats: EnemyStats { attack: 12, defense: 6, magic: 0, magic_def: 5, agility: 10 },
    attacks: &[
        EnemyAttack::basic("Fang Strike", 12),
        EnemyAttack {
            name: "Venom Bite",
            damage_type: DamageType::Physical,
            power: 10,
            accuracy: 0.85,
            element: Element::None,
            status_inflict: Some((StatusInflict::Poison, 0.40)),
            mp_cost: 0,
            weight: 50,
        },
    ],
    weaknesses: &[Element::Fire],
    resistances: &[],
    exp_reward: 35,
    gold_min: 20,
    gold_max: 45,
    drops: &[("antidote", 0.25), ("potion", 0.10)],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

pub const BAT: EnemyDef = EnemyDef {
    id: "bat",
    name: "Bat",
    hp: 18,
    stats: EnemyStats { attack: 6, defense: 2, magic: 0, magic_def: 4, agility: 12 },
    attacks: &[EnemyAttack::basic("Wing Attack", 6)],
    weaknesses: &[Element::Wind],
    resistances: &[],
    exp_reward: 12,
    gold_min: 5,
    gold_max: 12,
    drops: &[],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

pub const SKELETON: EnemyDef = EnemyDef {
    id: "skeleton",
    name: "Skeleton",
    hp: 40,
    stats: EnemyStats { attack: 12, defense: 8, magic: 0, magic_def: 3, agility: 5 },
    attacks: &[
        EnemyAttack::basic("Bone Club", 12),
        EnemyAttack::basic("Skull Throw", 10),
    ],
    weaknesses: &[Element::Holy, Element::Fire],
    resistances: &[Element::Dark],
    exp_reward: 30,
    gold_min: 25,
    gold_max: 50,
    drops: &[("potion", 0.10), ("iron_sword", 0.03)],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

// ============================================================================
// MID GAME ENEMIES (Regions 2-3)
// ============================================================================

pub const ORC: EnemyDef = EnemyDef {
    id: "orc",
    name: "Orc",
    hp: 80,
    stats: EnemyStats { attack: 18, defense: 12, magic: 2, magic_def: 6, agility: 6 },
    attacks: &[
        EnemyAttack::basic("Axe Swing", 18),
        EnemyAttack::basic("Shield Bash", 12),
    ],
    weaknesses: &[Element::Holy],
    resistances: &[],
    exp_reward: 55,
    gold_min: 35,
    gold_max: 70,
    drops: &[("hi_potion", 0.10), ("chain_mail", 0.03)],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

pub const HARPY: EnemyDef = EnemyDef {
    id: "harpy",
    name: "Harpy",
    hp: 55,
    stats: EnemyStats { attack: 14, defense: 6, magic: 12, magic_def: 10, agility: 14 },
    attacks: &[
        EnemyAttack::basic("Talon Rake", 14),
        EnemyAttack::magical("Screech", 15, Element::Wind),
    ],
    weaknesses: &[Element::Lightning],
    resistances: &[Element::Wind],
    exp_reward: 48,
    gold_min: 30,
    gold_max: 60,
    drops: &[("echo_herbs", 0.20)],
    ai_type: EnemyAIType::Magical,
    is_boss: false,
};

pub const DARK_MAGE: EnemyDef = EnemyDef {
    id: "dark_mage",
    name: "Dark Mage",
    hp: 50,
    stats: EnemyStats { attack: 8, defense: 5, magic: 22, magic_def: 18, agility: 8 },
    attacks: &[
        EnemyAttack::magical("Shadow Bolt", 25, Element::Dark),
        EnemyAttack::magical("Dark Flame", 20, Element::Fire),
        EnemyAttack::basic("Staff Strike", 8),
    ],
    weaknesses: &[Element::Holy],
    resistances: &[Element::Dark],
    exp_reward: 65,
    gold_min: 40,
    gold_max: 80,
    drops: &[("ether", 0.15), ("mage_staff", 0.05)],
    ai_type: EnemyAIType::Magical,
    is_boss: false,
};

pub const GOLEM: EnemyDef = EnemyDef {
    id: "golem",
    name: "Golem",
    hp: 120,
    stats: EnemyStats { attack: 25, defense: 25, magic: 0, magic_def: 15, agility: 2 },
    attacks: &[
        EnemyAttack::basic("Stone Fist", 25),
        EnemyAttack::basic("Ground Pound", 30),
    ],
    weaknesses: &[Element::Water],
    resistances: &[Element::Fire, Element::Lightning, Element::Earth],
    exp_reward: 85,
    gold_min: 50,
    gold_max: 100,
    drops: &[("soft", 0.20)],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

pub const WRAITH: EnemyDef = EnemyDef {
    id: "wraith",
    name: "Wraith",
    hp: 65,
    stats: EnemyStats { attack: 15, defense: 5, magic: 18, magic_def: 20, agility: 10 },
    attacks: &[
        EnemyAttack::magical("Life Drain", 18, Element::Dark),
        EnemyAttack {
            name: "Chilling Touch",
            damage_type: DamageType::Magical,
            power: 15,
            accuracy: 0.90,
            element: Element::Ice,
            status_inflict: Some((StatusInflict::Slow, 0.30)),
            mp_cost: 5,
            weight: 50,
        },
    ],
    weaknesses: &[Element::Holy, Element::Fire],
    resistances: &[Element::Dark, Element::Ice],
    exp_reward: 70,
    gold_min: 45,
    gold_max: 90,
    drops: &[("hi_ether", 0.10)],
    ai_type: EnemyAIType::Magical,
    is_boss: false,
};

pub const MEDUSA: EnemyDef = EnemyDef {
    id: "medusa",
    name: "Medusa",
    hp: 75,
    stats: EnemyStats { attack: 12, defense: 8, magic: 20, magic_def: 16, agility: 9 },
    attacks: &[
        EnemyAttack::basic("Snake Bite", 12),
        EnemyAttack {
            name: "Petrifying Gaze",
            damage_type: DamageType::Magical,
            power: 0,
            accuracy: 0.35,
            element: Element::Earth,
            status_inflict: Some((StatusInflict::Stone, 1.0)),
            mp_cost: 10,
            weight: 30,
        },
        EnemyAttack::magical("Venom Spray", 16, Element::None),
    ],
    weaknesses: &[Element::Holy],
    resistances: &[Element::Earth],
    exp_reward: 80,
    gold_min: 50,
    gold_max: 95,
    drops: &[("soft", 0.30), ("antidote", 0.20)],
    ai_type: EnemyAIType::Debuffer,
    is_boss: false,
};

// ============================================================================
// LATE GAME ENEMIES (Regions 4-5)
// ============================================================================

pub const DARK_KNIGHT: EnemyDef = EnemyDef {
    id: "dark_knight",
    name: "Dark Knight",
    hp: 150,
    stats: EnemyStats { attack: 35, defense: 30, magic: 10, magic_def: 20, agility: 10 },
    attacks: &[
        EnemyAttack::basic("Dark Blade", 35),
        EnemyAttack::basic("Shield Slam", 25),
        EnemyAttack::magical("Shadow Strike", 30, Element::Dark),
    ],
    weaknesses: &[Element::Holy],
    resistances: &[Element::Dark],
    exp_reward: 150,
    gold_min: 80,
    gold_max: 150,
    drops: &[("hi_potion", 0.20), ("steel_sword", 0.05)],
    ai_type: EnemyAIType::Normal,
    is_boss: false,
};

pub const FIRE_ELEMENTAL: EnemyDef = EnemyDef {
    id: "fire_elemental",
    name: "Fire Elemental",
    hp: 100,
    stats: EnemyStats { attack: 15, defense: 10, magic: 35, magic_def: 25, agility: 12 },
    attacks: &[
        EnemyAttack::magical("Fireball", 40, Element::Fire),
        EnemyAttack::magical("Flame Burst", 30, Element::Fire),
    ],
    weaknesses: &[Element::Water, Element::Ice],
    resistances: &[Element::Fire],
    exp_reward: 120,
    gold_min: 60,
    gold_max: 120,
    drops: &[("bomb_fragment", 0.25), ("fire_ring", 0.03)],
    ai_type: EnemyAIType::Magical,
    is_boss: false,
};

pub const ICE_ELEMENTAL: EnemyDef = EnemyDef {
    id: "ice_elemental",
    name: "Ice Elemental",
    hp: 100,
    stats: EnemyStats { attack: 15, defense: 12, magic: 35, magic_def: 25, agility: 10 },
    attacks: &[
        EnemyAttack::magical("Blizzard", 40, Element::Ice),
        EnemyAttack::magical("Frost Breath", 30, Element::Ice),
    ],
    weaknesses: &[Element::Fire],
    resistances: &[Element::Ice, Element::Water],
    exp_reward: 120,
    gold_min: 60,
    gold_max: 120,
    drops: &[("arctic_wind", 0.25), ("ice_ring", 0.03)],
    ai_type: EnemyAIType::Magical,
    is_boss: false,
};

pub const DEMON: EnemyDef = EnemyDef {
    id: "demon",
    name: "Demon",
    hp: 180,
    stats: EnemyStats { attack: 40, defense: 25, magic: 30, magic_def: 28, agility: 14 },
    attacks: &[
        EnemyAttack::basic("Demon Claw", 40),
        EnemyAttack::magical("Hellfire", 45, Element::Fire),
        EnemyAttack::magical("Dark Blast", 35, Element::Dark),
    ],
    weaknesses: &[Element::Holy],
    resistances: &[Element::Fire, Element::Dark],
    exp_reward: 200,
    gold_min: 100,
    gold_max: 200,
    drops: &[("hi_potion", 0.15), ("phoenix_down", 0.08)],
    ai_type: EnemyAIType::Berserker,
    is_boss: false,
};

pub const LICH: EnemyDef = EnemyDef {
    id: "lich",
    name: "Lich",
    hp: 140,
    stats: EnemyStats { attack: 20, defense: 15, magic: 45, magic_def: 35, agility: 8 },
    attacks: &[
        EnemyAttack::magical("Death Touch", 50, Element::Dark),
        EnemyAttack::magical("Soul Drain", 35, Element::Dark),
        EnemyAttack {
            name: "Curse",
            damage_type: DamageType::Magical,
            power: 20,
            accuracy: 0.80,
            element: Element::Dark,
            status_inflict: Some((StatusInflict::Poison, 0.50)),
            mp_cost: 8,
            weight: 40,
        },
    ],
    weaknesses: &[Element::Holy, Element::Fire],
    resistances: &[Element::Dark, Element::Ice],
    exp_reward: 180,
    gold_min: 90,
    gold_max: 180,
    drops: &[("hi_ether", 0.20), ("archmage_robes", 0.02)],
    ai_type: EnemyAIType::Magical,
    is_boss: false,
};

pub const DRAGON: EnemyDef = EnemyDef {
    id: "dragon",
    name: "Dragon",
    hp: 250,
    stats: EnemyStats { attack: 50, defense: 40, magic: 35, magic_def: 35, agility: 8 },
    attacks: &[
        EnemyAttack::basic("Claw Swipe", 50),
        EnemyAttack::basic("Tail Whip", 40),
        EnemyAttack::magical("Dragon Breath", 60, Element::Fire),
    ],
    weaknesses: &[Element::Ice],
    resistances: &[Element::Fire],
    exp_reward: 350,
    gold_min: 150,
    gold_max: 300,
    drops: &[("mega_potion", 0.15), ("flame_sword", 0.03)],
    ai_type: EnemyAIType::Berserker,
    is_boss: false,
};

// ============================================================================
// NON-GUARDIAN BOSSES
// ============================================================================

pub const THORNWICK_SPIDER: EnemyDef = EnemyDef {
    id: "thornwick_spider",
    name: "Thornwick Spider Queen",
    hp: 200,
    stats: EnemyStats { attack: 18, defense: 10, magic: 8, magic_def: 8, agility: 12 },
    attacks: &[
        EnemyAttack::basic("Massive Fang", 18),
        EnemyAttack {
            name: "Web Spray",
            damage_type: DamageType::Physical,
            power: 12,
            accuracy: 0.95,
            element: Element::None,
            status_inflict: Some((StatusInflict::Slow, 0.60)),
            mp_cost: 0,
            weight: 40,
        },
        EnemyAttack {
            name: "Venom Spray",
            damage_type: DamageType::Physical,
            power: 15,
            accuracy: 0.90,
            element: Element::None,
            status_inflict: Some((StatusInflict::Poison, 0.50)),
            mp_cost: 0,
            weight: 40,
        },
    ],
    weaknesses: &[Element::Fire],
    resistances: &[],
    exp_reward: 150,
    gold_min: 100,
    gold_max: 200,
    drops: &[("antidote", 0.50), ("hi_potion", 0.30)],
    ai_type: EnemyAIType::Debuffer,
    is_boss: true,
};

pub const BANDIT_KING: EnemyDef = EnemyDef {
    id: "bandit_king",
    name: "Bandit King",
    hp: 350,
    stats: EnemyStats { attack: 28, defense: 18, magic: 5, magic_def: 12, agility: 14 },
    attacks: &[
        EnemyAttack::basic("Cutthroat", 28),
        EnemyAttack::basic("Dual Strike", 35),
        EnemyAttack::basic("Throw Dagger", 20),
    ],
    weaknesses: &[],
    resistances: &[],
    exp_reward: 300,
    gold_min: 500,
    gold_max: 1000,
    drops: &[("rapier", 0.30), ("thief_gloves", 0.10)],
    ai_type: EnemyAIType::Berserker,
    is_boss: true,
};

pub const CORRUPTED_KNIGHT: EnemyDef = EnemyDef {
    id: "corrupted_knight",
    name: "Corrupted Knight",
    hp: 500,
    stats: EnemyStats { attack: 40, defense: 35, magic: 15, magic_def: 25, agility: 8 },
    attacks: &[
        EnemyAttack::basic("Dark Cleave", 45),
        EnemyAttack::magical("Shadow Wave", 35, Element::Dark),
        EnemyAttack::basic("Shield Wall", 0), // Defensive
    ],
    weaknesses: &[Element::Holy],
    resistances: &[Element::Dark],
    exp_reward: 500,
    gold_min: 300,
    gold_max: 500,
    drops: &[("plate_armor", 0.20), ("silver_sword", 0.10)],
    ai_type: EnemyAIType::Defensive,
    is_boss: true,
};

pub const STORM_SERPENT: EnemyDef = EnemyDef {
    id: "storm_serpent",
    name: "Storm Serpent",
    hp: 600,
    stats: EnemyStats { attack: 35, defense: 20, magic: 40, magic_def: 30, agility: 16 },
    attacks: &[
        EnemyAttack::basic("Constrict", 35),
        EnemyAttack::magical("Lightning Bolt", 45, Element::Lightning),
        EnemyAttack::magical("Tempest", 55, Element::Wind),
    ],
    weaknesses: &[Element::Earth],
    resistances: &[Element::Lightning, Element::Wind],
    exp_reward: 600,
    gold_min: 400,
    gold_max: 600,
    drops: &[("zeus_rage", 0.30), ("lightning_ring", 0.10)],
    ai_type: EnemyAIType::Magical,
    is_boss: true,
};

// Morveth - Final Boss (fought after all echoes gathered)
pub const MORVETH: EnemyDef = EnemyDef {
    id: "morveth",
    name: "Morveth, The Deceiver",
    hp: 3000,
    stats: EnemyStats { attack: 60, defense: 40, magic: 70, magic_def: 50, agility: 15 },
    attacks: &[
        EnemyAttack::basic("Deceiver's Strike", 60),
        EnemyAttack::magical("Void Blast", 80, Element::Dark),
        EnemyAttack::magical("Reality Tear", 70, Element::None),
        EnemyAttack {
            name: "Illusion",
            damage_type: DamageType::Magical,
            power: 40,
            accuracy: 1.0,
            element: Element::Dark,
            status_inflict: Some((StatusInflict::Confusion, 0.40)),
            mp_cost: 15,
            weight: 30,
        },
        EnemyAttack::magical("Consume Hope", 90, Element::Dark),
    ],
    weaknesses: &[Element::Holy],
    resistances: &[Element::Dark, Element::Fire, Element::Ice],
    exp_reward: 0, // Final boss
    gold_min: 0,
    gold_max: 0,
    drops: &[],
    ai_type: EnemyAIType::Magical,
    is_boss: true,
};

// ============================================================================
// ENEMY COLLECTION AND LOOKUP
// ============================================================================

/// All enemies in the game
pub static ALL_ENEMIES: &[&EnemyDef] = &[
    // Early game
    &SLIME, &WOLF, &GOBLIN, &GIANT_SPIDER, &BAT, &SKELETON,
    // Mid game
    &ORC, &HARPY, &DARK_MAGE, &GOLEM, &WRAITH, &MEDUSA,
    // Late game
    &DARK_KNIGHT, &FIRE_ELEMENTAL, &ICE_ELEMENTAL, &DEMON, &LICH, &DRAGON,
    // Bosses
    &THORNWICK_SPIDER, &BANDIT_KING, &CORRUPTED_KNIGHT, &STORM_SERPENT, &MORVETH,
];

/// Look up an enemy by its ID
pub fn get_enemy(id: &str) -> Option<&'static EnemyDef> {
    ALL_ENEMIES.iter().find(|enemy| enemy.id == id).copied()
}

/// Get all regular (non-boss) enemies
pub fn get_regular_enemies() -> Vec<&'static EnemyDef> {
    ALL_ENEMIES.iter()
        .filter(|enemy| !enemy.is_boss)
        .copied()
        .collect()
}

/// Get all boss enemies
pub fn get_bosses() -> Vec<&'static EnemyDef> {
    ALL_ENEMIES.iter()
        .filter(|enemy| enemy.is_boss)
        .copied()
        .collect()
}

/// Get enemies by AI type
pub fn get_enemies_by_ai(ai_type: EnemyAIType) -> Vec<&'static EnemyDef> {
    ALL_ENEMIES.iter()
        .filter(|enemy| enemy.ai_type == ai_type)
        .copied()
        .collect()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_enemies_have_unique_ids() {
        let mut ids: Vec<&str> = ALL_ENEMIES.iter().map(|e| e.id).collect();
        ids.sort();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Duplicate enemy IDs found");
    }

    #[test]
    fn test_all_enemies_lookup() {
        for enemy in ALL_ENEMIES.iter() {
            let found = get_enemy(enemy.id);
            assert!(found.is_some(), "Failed to find enemy: {}", enemy.id);
            assert_eq!(found.unwrap().id, enemy.id);
        }
    }

    #[test]
    fn test_get_enemy_not_found() {
        assert!(get_enemy("nonexistent_enemy").is_none());
    }

    #[test]
    fn test_enemies_have_positive_hp() {
        for enemy in ALL_ENEMIES.iter() {
            assert!(enemy.hp > 0, "Enemy {} should have positive HP", enemy.id);
        }
    }

    #[test]
    fn test_enemies_have_attacks() {
        for enemy in ALL_ENEMIES.iter() {
            assert!(!enemy.attacks.is_empty(), "Enemy {} should have at least one attack", enemy.id);
        }
    }

    #[test]
    fn test_regular_enemies_give_exp() {
        for enemy in get_regular_enemies() {
            assert!(enemy.exp_reward > 0, "Regular enemy {} should give EXP", enemy.id);
        }
    }

    #[test]
    fn test_morveth_is_boss() {
        let morveth = get_enemy("morveth");
        assert!(morveth.is_some());
        assert!(morveth.unwrap().is_boss);
    }

    #[test]
    fn test_boss_count() {
        let bosses = get_bosses();
        assert!(bosses.len() >= 5, "Should have at least 5 bosses");
    }
}
