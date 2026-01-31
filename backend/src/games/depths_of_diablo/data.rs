//! Static game data for Depths of Diablo
//!
//! Contains character classes, monsters, skills, and item definitions.

use serde::{Deserialize, Serialize};

/// Character class definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    Warrior,
    Rogue,
    Sorcerer,
}

impl CharacterClass {
    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Rogue => "Rogue",
            CharacterClass::Sorcerer => "Sorcerer",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "A mighty fighter with high HP and armor. Excels in melee combat.",
            CharacterClass::Rogue => "A swift striker with high speed and critical hits. Hybrid ranged/melee.",
            CharacterClass::Sorcerer => "A master of arcane arts. High mana and spell damage, but fragile.",
        }
    }

    pub fn base_stats(&self) -> ClassStats {
        match self {
            CharacterClass::Warrior => ClassStats {
                health: 150,
                mana: 30,
                strength: 25,
                dexterity: 15,
                intelligence: 10,
                vitality: 25,
                armor: 20,
                attack_speed: 100,
            },
            CharacterClass::Rogue => ClassStats {
                health: 100,
                mana: 50,
                strength: 15,
                dexterity: 25,
                intelligence: 15,
                vitality: 15,
                armor: 10,
                attack_speed: 130,
            },
            CharacterClass::Sorcerer => ClassStats {
                health: 80,
                mana: 100,
                strength: 10,
                dexterity: 15,
                intelligence: 30,
                vitality: 10,
                armor: 5,
                attack_speed: 90,
            },
        }
    }

    pub fn starting_skills(&self) -> Vec<&'static str> {
        match self {
            CharacterClass::Warrior => vec!["bash"],
            CharacterClass::Rogue => vec!["multishot"],
            CharacterClass::Sorcerer => vec!["fireball"],
        }
    }

    pub fn ascii_art(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => r#"
    /\
   /  \
  |    |
  | [] |
 /|    |\
/ |====| \
  |    |
  |    |
 /|    |\
/ |    | \
"#,
            CharacterClass::Rogue => r#"
    o
   /|\
  / | \
    |
   /|
  / |
 /  |
    |\
    | \
   /   \
"#,
            CharacterClass::Sorcerer => r#"
   /\
  /  \
 | () |
  \  /
   ||
  /||\
 / || \
   ||
   ||
  /  \
"#,
        }
    }
}

impl std::fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Base stats for a character class
#[derive(Debug, Clone, Copy)]
pub struct ClassStats {
    pub health: i32,
    pub mana: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub vitality: i32,
    pub armor: i32,
    pub attack_speed: i32, // percentage, 100 = normal
}

/// All available classes
pub const CLASSES: &[CharacterClass] = &[
    CharacterClass::Warrior,
    CharacterClass::Rogue,
    CharacterClass::Sorcerer,
];

/// Monster type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonsterType {
    // Cathedral (floors 1-5)
    Zombie,
    Skeleton,
    FallenOne,
    SkeletonArcher,
    // Catacombs (floors 6-10)
    Goatman,
    HiddenOne,
    Gargoyle,
    Overlord,
    // Caves (floors 11-15)
    SandRaider,
    CaveBat,
    Viper,
    Balrog,
    // Hell (floors 16-20)
    SuccubusLesser,
    HellSpawn,
    BloodKnight,
    Advocate,
    // Bosses
    ButcherBoss,
    SkeletonKingBoss,
    LazarusBoss,
    DiabloBoss,
}

impl MonsterType {
    pub fn name(&self) -> &'static str {
        match self {
            MonsterType::Zombie => "Zombie",
            MonsterType::Skeleton => "Skeleton",
            MonsterType::FallenOne => "Fallen One",
            MonsterType::SkeletonArcher => "Skeleton Archer",
            MonsterType::Goatman => "Goatman",
            MonsterType::HiddenOne => "Hidden One",
            MonsterType::Gargoyle => "Gargoyle",
            MonsterType::Overlord => "Overlord",
            MonsterType::SandRaider => "Sand Raider",
            MonsterType::CaveBat => "Cave Bat",
            MonsterType::Viper => "Viper",
            MonsterType::Balrog => "Balrog",
            MonsterType::SuccubusLesser => "Succubus",
            MonsterType::HellSpawn => "Hell Spawn",
            MonsterType::BloodKnight => "Blood Knight",
            MonsterType::Advocate => "Advocate",
            MonsterType::ButcherBoss => "The Butcher",
            MonsterType::SkeletonKingBoss => "Skeleton King",
            MonsterType::LazarusBoss => "Lazarus",
            MonsterType::DiabloBoss => "Diablo",
        }
    }

    pub fn stats(&self) -> MonsterStats {
        match self {
            // Cathedral (floors 1-5)
            MonsterType::Zombie => MonsterStats {
                health: 30,
                damage: 5,
                armor: 2,
                speed: 50,
                xp: 10,
            },
            MonsterType::Skeleton => MonsterStats {
                health: 25,
                damage: 8,
                armor: 5,
                speed: 80,
                xp: 15,
            },
            MonsterType::FallenOne => MonsterStats {
                health: 20,
                damage: 6,
                armor: 0,
                speed: 100,
                xp: 12,
            },
            MonsterType::SkeletonArcher => MonsterStats {
                health: 20,
                damage: 10,
                armor: 3,
                speed: 70,
                xp: 18,
            },
            // Catacombs (floors 6-10)
            MonsterType::Goatman => MonsterStats {
                health: 50,
                damage: 15,
                armor: 8,
                speed: 90,
                xp: 25,
            },
            MonsterType::HiddenOne => MonsterStats {
                health: 35,
                damage: 20,
                armor: 5,
                speed: 110,
                xp: 30,
            },
            MonsterType::Gargoyle => MonsterStats {
                health: 60,
                damage: 18,
                armor: 15,
                speed: 85,
                xp: 35,
            },
            MonsterType::Overlord => MonsterStats {
                health: 80,
                damage: 22,
                armor: 12,
                speed: 75,
                xp: 45,
            },
            // Caves (floors 11-15)
            MonsterType::SandRaider => MonsterStats {
                health: 90,
                damage: 25,
                armor: 10,
                speed: 100,
                xp: 50,
            },
            MonsterType::CaveBat => MonsterStats {
                health: 40,
                damage: 15,
                armor: 0,
                speed: 150,
                xp: 35,
            },
            MonsterType::Viper => MonsterStats {
                health: 70,
                damage: 30,
                armor: 8,
                speed: 95,
                xp: 55,
            },
            MonsterType::Balrog => MonsterStats {
                health: 150,
                damage: 40,
                armor: 20,
                speed: 70,
                xp: 80,
            },
            // Hell (floors 16-20)
            MonsterType::SuccubusLesser => MonsterStats {
                health: 100,
                damage: 35,
                armor: 5,
                speed: 110,
                xp: 70,
            },
            MonsterType::HellSpawn => MonsterStats {
                health: 180,
                damage: 45,
                armor: 25,
                speed: 80,
                xp: 100,
            },
            MonsterType::BloodKnight => MonsterStats {
                health: 250,
                damage: 55,
                armor: 35,
                speed: 90,
                xp: 120,
            },
            MonsterType::Advocate => MonsterStats {
                health: 120,
                damage: 60,
                armor: 15,
                speed: 85,
                xp: 110,
            },
            // Bosses
            MonsterType::ButcherBoss => MonsterStats {
                health: 400,
                damage: 50,
                armor: 30,
                speed: 80,
                xp: 500,
            },
            MonsterType::SkeletonKingBoss => MonsterStats {
                health: 600,
                damage: 65,
                armor: 40,
                speed: 70,
                xp: 800,
            },
            MonsterType::LazarusBoss => MonsterStats {
                health: 800,
                damage: 80,
                armor: 35,
                speed: 90,
                xp: 1200,
            },
            MonsterType::DiabloBoss => MonsterStats {
                health: 1500,
                damage: 120,
                armor: 50,
                speed: 100,
                xp: 3000,
            },
        }
    }

    pub fn ascii_char(&self) -> char {
        match self {
            MonsterType::Zombie => 'z',
            MonsterType::Skeleton => 's',
            MonsterType::FallenOne => 'f',
            MonsterType::SkeletonArcher => 'a',
            MonsterType::Goatman => 'g',
            MonsterType::HiddenOne => 'h',
            MonsterType::Gargoyle => 'G',
            MonsterType::Overlord => 'O',
            MonsterType::SandRaider => 'r',
            MonsterType::CaveBat => 'b',
            MonsterType::Viper => 'v',
            MonsterType::Balrog => 'B',
            MonsterType::SuccubusLesser => 'S',
            MonsterType::HellSpawn => 'H',
            MonsterType::BloodKnight => 'K',
            MonsterType::Advocate => 'A',
            MonsterType::ButcherBoss => 'X',
            MonsterType::SkeletonKingBoss => 'X',
            MonsterType::LazarusBoss => 'X',
            MonsterType::DiabloBoss => 'D',
        }
    }

    pub fn is_boss(&self) -> bool {
        matches!(
            self,
            MonsterType::ButcherBoss
                | MonsterType::SkeletonKingBoss
                | MonsterType::LazarusBoss
                | MonsterType::DiabloBoss
        )
    }

    /// Get monsters for a given floor
    pub fn for_floor(floor: u32) -> Vec<MonsterType> {
        match floor {
            1..=5 => vec![
                MonsterType::Zombie,
                MonsterType::Skeleton,
                MonsterType::FallenOne,
                MonsterType::SkeletonArcher,
            ],
            6..=10 => vec![
                MonsterType::Goatman,
                MonsterType::HiddenOne,
                MonsterType::Gargoyle,
                MonsterType::Overlord,
            ],
            11..=15 => vec![
                MonsterType::SandRaider,
                MonsterType::CaveBat,
                MonsterType::Viper,
                MonsterType::Balrog,
            ],
            16..=20 => vec![
                MonsterType::SuccubusLesser,
                MonsterType::HellSpawn,
                MonsterType::BloodKnight,
                MonsterType::Advocate,
            ],
            _ => vec![MonsterType::Zombie],
        }
    }

    /// Get boss for a given floor (every 5 floors)
    pub fn boss_for_floor(floor: u32) -> Option<MonsterType> {
        match floor {
            5 => Some(MonsterType::ButcherBoss),
            10 => Some(MonsterType::SkeletonKingBoss),
            15 => Some(MonsterType::LazarusBoss),
            20 => Some(MonsterType::DiabloBoss),
            _ => None,
        }
    }
}

/// Monster stats
#[derive(Debug, Clone, Copy)]
pub struct MonsterStats {
    pub health: i32,
    pub damage: i32,
    pub armor: i32,
    pub speed: i32,  // movement/attack speed percentage
    pub xp: i32,     // experience awarded on kill
}

/// List of all monsters
pub const MONSTERS: &[MonsterType] = &[
    MonsterType::Zombie,
    MonsterType::Skeleton,
    MonsterType::FallenOne,
    MonsterType::SkeletonArcher,
    MonsterType::Goatman,
    MonsterType::HiddenOne,
    MonsterType::Gargoyle,
    MonsterType::Overlord,
    MonsterType::SandRaider,
    MonsterType::CaveBat,
    MonsterType::Viper,
    MonsterType::Balrog,
    MonsterType::SuccubusLesser,
    MonsterType::HellSpawn,
    MonsterType::BloodKnight,
    MonsterType::Advocate,
];

/// Skill definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillDef {
    pub key: &'static str,
    pub name: &'static str,
    pub class: CharacterClass,
    pub mana_cost: i32,
    pub cooldown_ms: u64,
    pub damage_multiplier: i32, // percentage, 100 = normal
    pub description: &'static str,
}

/// All available skills
pub const SKILLS: &[SkillDef] = &[
    // Warrior skills
    SkillDef {
        key: "bash",
        name: "Bash",
        class: CharacterClass::Warrior,
        mana_cost: 5,
        cooldown_ms: 2000,
        damage_multiplier: 150,
        description: "A powerful strike that stuns briefly",
    },
    SkillDef {
        key: "whirlwind",
        name: "Whirlwind",
        class: CharacterClass::Warrior,
        mana_cost: 20,
        cooldown_ms: 5000,
        damage_multiplier: 80,
        description: "Spin attack hitting all nearby enemies",
    },
    SkillDef {
        key: "battle_cry",
        name: "Battle Cry",
        class: CharacterClass::Warrior,
        mana_cost: 15,
        cooldown_ms: 10000,
        damage_multiplier: 0,
        description: "Increases damage for 10 seconds",
    },
    SkillDef {
        key: "iron_skin",
        name: "Iron Skin",
        class: CharacterClass::Warrior,
        mana_cost: 10,
        cooldown_ms: 15000,
        damage_multiplier: 0,
        description: "Reduces damage taken for 8 seconds",
    },
    // Rogue skills
    SkillDef {
        key: "multishot",
        name: "Multishot",
        class: CharacterClass::Rogue,
        mana_cost: 10,
        cooldown_ms: 2500,
        damage_multiplier: 70,
        description: "Fire multiple arrows at once",
    },
    SkillDef {
        key: "trap",
        name: "Trap",
        class: CharacterClass::Rogue,
        mana_cost: 15,
        cooldown_ms: 8000,
        damage_multiplier: 200,
        description: "Place a trap that explodes when triggered",
    },
    SkillDef {
        key: "shadow_step",
        name: "Shadow Step",
        class: CharacterClass::Rogue,
        mana_cost: 25,
        cooldown_ms: 6000,
        damage_multiplier: 0,
        description: "Teleport behind an enemy",
    },
    SkillDef {
        key: "poison_strike",
        name: "Poison Strike",
        class: CharacterClass::Rogue,
        mana_cost: 12,
        cooldown_ms: 4000,
        damage_multiplier: 80,
        description: "Attack that poisons the target",
    },
    // Sorcerer skills
    SkillDef {
        key: "fireball",
        name: "Fireball",
        class: CharacterClass::Sorcerer,
        mana_cost: 15,
        cooldown_ms: 2000,
        damage_multiplier: 180,
        description: "Hurl a ball of fire at enemies",
    },
    SkillDef {
        key: "frost_nova",
        name: "Frost Nova",
        class: CharacterClass::Sorcerer,
        mana_cost: 25,
        cooldown_ms: 6000,
        damage_multiplier: 100,
        description: "Freeze all nearby enemies",
    },
    SkillDef {
        key: "teleport",
        name: "Teleport",
        class: CharacterClass::Sorcerer,
        mana_cost: 30,
        cooldown_ms: 4000,
        damage_multiplier: 0,
        description: "Instantly move to a new location",
    },
    SkillDef {
        key: "chain_lightning",
        name: "Chain Lightning",
        class: CharacterClass::Sorcerer,
        mana_cost: 35,
        cooldown_ms: 3000,
        damage_multiplier: 120,
        description: "Lightning that jumps between enemies",
    },
];

pub fn get_skill(key: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.key == key)
}

pub fn get_class_skills(class: CharacterClass) -> Vec<&'static SkillDef> {
    SKILLS.iter().filter(|s| s.class == class).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_base_stats() {
        let warrior = CharacterClass::Warrior.base_stats();
        assert_eq!(warrior.health, 150);
        assert!(warrior.strength > warrior.intelligence);

        let sorc = CharacterClass::Sorcerer.base_stats();
        assert!(sorc.intelligence > sorc.strength);
        assert!(sorc.mana > warrior.mana);
    }

    #[test]
    fn test_monster_for_floor() {
        let floor1 = MonsterType::for_floor(1);
        assert!(floor1.contains(&MonsterType::Zombie));

        let floor10 = MonsterType::for_floor(10);
        assert!(floor10.contains(&MonsterType::Goatman));
    }

    #[test]
    fn test_boss_for_floor() {
        assert_eq!(MonsterType::boss_for_floor(5), Some(MonsterType::ButcherBoss));
        assert_eq!(MonsterType::boss_for_floor(20), Some(MonsterType::DiabloBoss));
        assert_eq!(MonsterType::boss_for_floor(3), None);
    }

    #[test]
    fn test_get_skill() {
        let bash = get_skill("bash").unwrap();
        assert_eq!(bash.class, CharacterClass::Warrior);

        let fireball = get_skill("fireball").unwrap();
        assert_eq!(fireball.class, CharacterClass::Sorcerer);
    }

    #[test]
    fn test_get_class_skills() {
        let warrior_skills = get_class_skills(CharacterClass::Warrior);
        assert_eq!(warrior_skills.len(), 4);
        assert!(warrior_skills.iter().all(|s| s.class == CharacterClass::Warrior));
    }
}
