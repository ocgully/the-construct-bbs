//! Static game data - races, personalities, buildings, units, spells

use serde::{Serialize, Deserialize};

/// Races available to players, each with different bonuses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Race {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// Population growth bonus (100 = normal, 110 = +10%)
    pub pop_growth: u32,
    /// Gold income bonus (100 = normal)
    pub gold_bonus: u32,
    /// Food production bonus (100 = normal)
    pub food_bonus: u32,
    /// Attack strength bonus (100 = normal)
    pub attack_bonus: u32,
    /// Defense strength bonus (100 = normal)
    pub defense_bonus: u32,
    /// Magic power bonus (100 = normal)
    pub magic_bonus: u32,
    /// Thief effectiveness bonus (100 = normal)
    pub thief_bonus: u32,
    /// Special ability description
    pub special: &'static str,
}

pub static RACES: &[Race] = &[
    Race {
        key: "human",
        name: "Human",
        description: "Balanced and adaptable.",
        pop_growth: 105,
        gold_bonus: 105,
        food_bonus: 100,
        attack_bonus: 100,
        defense_bonus: 100,
        magic_bonus: 100,
        thief_bonus: 100,
        special: "Bonus gold and population growth",
    },
    Race {
        key: "elf",
        name: "Elf",
        description: "Masters of magic and nature.",
        pop_growth: 90,
        gold_bonus: 95,
        food_bonus: 110,
        attack_bonus: 95,
        defense_bonus: 100,
        magic_bonus: 125,
        thief_bonus: 105,
        special: "Superior magic power, better farming",
    },
    Race {
        key: "dwarf",
        name: "Dwarf",
        description: "Hardy miners and craftsmen.",
        pop_growth: 95,
        gold_bonus: 120,
        food_bonus: 90,
        attack_bonus: 105,
        defense_bonus: 115,
        magic_bonus: 80,
        thief_bonus: 90,
        special: "Excellent defense and gold income",
    },
    Race {
        key: "orc",
        name: "Orc",
        description: "Brutal warriors who live for battle.",
        pop_growth: 110,
        gold_bonus: 85,
        food_bonus: 95,
        attack_bonus: 125,
        defense_bonus: 95,
        magic_bonus: 75,
        thief_bonus: 80,
        special: "Devastating attacks, fast breeding",
    },
    Race {
        key: "undead",
        name: "Undead",
        description: "The restless dead who need no food.",
        pop_growth: 85,
        gold_bonus: 90,
        food_bonus: 0, // Don't need food
        attack_bonus: 105,
        defense_bonus: 110,
        magic_bonus: 115,
        thief_bonus: 95,
        special: "No food required, immune to plague",
    },
    Race {
        key: "faery",
        name: "Faery",
        description: "Magical tricksters from the twilight realm.",
        pop_growth: 100,
        gold_bonus: 90,
        food_bonus: 100,
        attack_bonus: 80,
        defense_bonus: 90,
        magic_bonus: 130,
        thief_bonus: 120,
        special: "Best thieves and spellcasters",
    },
    Race {
        key: "halfling",
        name: "Halfling",
        description: "Peaceful folk with surprising resilience.",
        pop_growth: 115,
        gold_bonus: 110,
        food_bonus: 120,
        attack_bonus: 85,
        defense_bonus: 105,
        magic_bonus: 95,
        thief_bonus: 110,
        special: "Best farmers, excellent population growth",
    },
    Race {
        key: "dark_elf",
        name: "Dark Elf",
        description: "Cruel masters of shadow and steel.",
        pop_growth: 95,
        gold_bonus: 100,
        food_bonus: 95,
        attack_bonus: 115,
        defense_bonus: 100,
        magic_bonus: 110,
        thief_bonus: 115,
        special: "Balanced offense with strong magic/thief",
    },
];

/// Personalities determine playstyle bonuses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Personality {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// Building cost reduction (100 = normal, 90 = -10% cost)
    pub build_cost: u32,
    /// Military training speed (100 = normal, 110 = +10% faster)
    pub train_speed: u32,
    /// Science research speed (100 = normal)
    pub science_speed: u32,
    /// Exploration bonus for land gains (100 = normal)
    pub explore_bonus: u32,
}

pub static PERSONALITIES: &[Personality] = &[
    Personality {
        key: "merchant",
        name: "The Merchant",
        description: "Focus on trade and economy.",
        build_cost: 90,
        train_speed: 100,
        science_speed: 100,
        explore_bonus: 100,
    },
    Personality {
        key: "warrior",
        name: "The Warrior",
        description: "Born for battle.",
        build_cost: 100,
        train_speed: 115,
        science_speed: 95,
        explore_bonus: 105,
    },
    Personality {
        key: "sage",
        name: "The Sage",
        description: "Seeker of knowledge.",
        build_cost: 100,
        train_speed: 95,
        science_speed: 120,
        explore_bonus: 100,
    },
    Personality {
        key: "rogue",
        name: "The Rogue",
        description: "Master of covert operations.",
        build_cost: 100,
        train_speed: 100,
        science_speed: 100,
        explore_bonus: 115,
    },
    Personality {
        key: "mystic",
        name: "The Mystic",
        description: "Wielder of arcane power.",
        build_cost: 105,
        train_speed: 95,
        science_speed: 105,
        explore_bonus: 100,
    },
    Personality {
        key: "tactician",
        name: "The Tactician",
        description: "Master of strategy.",
        build_cost: 95,
        train_speed: 105,
        science_speed: 105,
        explore_bonus: 100,
    },
];

/// Building types and their effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    Home,           // Increases max population
    Farm,           // Produces food
    Bank,           // Increases gold income
    Barracks,       // Reduces military upkeep
    TrainingGround, // Trains troops faster
    Fort,           // Defense bonus
    Tower,          // Magic power
    ThievesDen,     // Thief operations
    WatchTower,     // Protection from thieves
    Stable,         // Cavalry bonus
    University,     // Science speed
    Hospital,       // Reduces casualties
    Armoury,        // Attack bonus
    Dungeon,        // Prisoner capacity
    Guildhall,      // Specialist training
}

impl BuildingType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Farm => "Farm",
            Self::Bank => "Bank",
            Self::Barracks => "Barracks",
            Self::TrainingGround => "Training Ground",
            Self::Fort => "Fort",
            Self::Tower => "Tower",
            Self::ThievesDen => "Thieves' Den",
            Self::WatchTower => "Watch Tower",
            Self::Stable => "Stable",
            Self::University => "University",
            Self::Hospital => "Hospital",
            Self::Armoury => "Armoury",
            Self::Dungeon => "Dungeon",
            Self::Guildhall => "Guildhall",
        }
    }

    #[allow(dead_code)] // Used in tests and for UI tooltips
    pub fn description(&self) -> &'static str {
        match self {
            Self::Home => "Houses 25 peasants",
            Self::Farm => "Produces 60 bushels of food",
            Self::Bank => "+2% gold income per bank",
            Self::Barracks => "-1.5% military upkeep",
            Self::TrainingGround => "+2% training speed",
            Self::Fort => "+50 defense per fort",
            Self::Tower => "+2% magic power",
            Self::ThievesDen => "Trains 2 thieves per tick",
            Self::WatchTower => "+5% thief protection",
            Self::Stable => "+2% cavalry effectiveness",
            Self::University => "+2% science speed",
            Self::Hospital => "-2% military casualties",
            Self::Armoury => "+1% attack strength",
            Self::Dungeon => "Holds 25 prisoners",
            Self::Guildhall => "Trains 1 specialist per tick",
        }
    }

    pub fn base_cost(&self) -> u32 {
        match self {
            Self::Home => 200,
            Self::Farm => 150,
            Self::Bank => 500,
            Self::Barracks => 300,
            Self::TrainingGround => 400,
            Self::Fort => 600,
            Self::Tower => 700,
            Self::ThievesDen => 450,
            Self::WatchTower => 350,
            Self::Stable => 500,
            Self::University => 800,
            Self::Hospital => 550,
            Self::Armoury => 650,
            Self::Dungeon => 400,
            Self::Guildhall => 750,
        }
    }

    /// All building types for iteration
    pub fn all() -> &'static [BuildingType] {
        &[
            Self::Home,
            Self::Farm,
            Self::Bank,
            Self::Barracks,
            Self::TrainingGround,
            Self::Fort,
            Self::Tower,
            Self::ThievesDen,
            Self::WatchTower,
            Self::Stable,
            Self::University,
            Self::Hospital,
            Self::Armoury,
            Self::Dungeon,
            Self::Guildhall,
        ]
    }
}

/// Military unit types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    Soldier,    // Basic infantry
    Archer,     // Ranged unit
    Knight,     // Heavy cavalry
    Thief,      // Covert operations
    Wizard,     // Magic user
    Elite,      // Race-specific elite unit
}

impl UnitType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Soldier => "Soldier",
            Self::Archer => "Archer",
            Self::Knight => "Knight",
            Self::Thief => "Thief",
            Self::Wizard => "Wizard",
            Self::Elite => "Elite",
        }
    }

    pub fn attack(&self) -> u32 {
        match self {
            Self::Soldier => 3,
            Self::Archer => 4,
            Self::Knight => 7,
            Self::Thief => 1,
            Self::Wizard => 4,
            Self::Elite => 10,
        }
    }

    pub fn defense(&self) -> u32 {
        match self {
            Self::Soldier => 3,
            Self::Archer => 2,
            Self::Knight => 6,
            Self::Thief => 1,
            Self::Wizard => 3,
            Self::Elite => 8,
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            Self::Soldier => 250,
            Self::Archer => 300,
            Self::Knight => 600,
            Self::Thief => 400,
            Self::Wizard => 750,
            Self::Elite => 1000,
        }
    }

    pub fn upkeep(&self) -> u32 {
        match self {
            Self::Soldier => 5,
            Self::Archer => 6,
            Self::Knight => 12,
            Self::Thief => 8,
            Self::Wizard => 15,
            Self::Elite => 20,
        }
    }
}

/// Attack types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackType {
    TraditionalMarch, // Capture land
    Raid,            // Steal resources
    Plunder,         // Destroy buildings
    Massacre,        // Kill peasants
    Learn,           // Steal science
}

impl AttackType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TraditionalMarch => "Traditional March",
            Self::Raid => "Raid",
            Self::Plunder => "Plunder",
            Self::Massacre => "Massacre",
            Self::Learn => "Learn",
        }
    }

    #[allow(dead_code)] // Used for UI tooltips
    pub fn description(&self) -> &'static str {
        match self {
            Self::TraditionalMarch => "Capture enemy land and buildings",
            Self::Raid => "Steal gold and resources",
            Self::Plunder => "Destroy enemy buildings",
            Self::Massacre => "Kill enemy peasants",
            Self::Learn => "Steal enemy sciences",
        }
    }
}

/// Thief operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThiefOp {
    IntelGather,     // Gather intel on target
    Sabotage,        // Destroy buildings
    Assassinate,     // Kill specialists
    PropagandaWar,   // Reduce morale
    StealGold,       // Steal money
    Kidnap,          // Capture peasants
}

impl ThiefOp {
    pub fn name(&self) -> &'static str {
        match self {
            Self::IntelGather => "Intel Gathering",
            Self::Sabotage => "Sabotage",
            Self::Assassinate => "Assassination",
            Self::PropagandaWar => "Propaganda",
            Self::StealGold => "Steal Gold",
            Self::Kidnap => "Kidnap",
        }
    }
}

/// Spell types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellType {
    // Offensive
    Fireball,        // Damage troops
    Lightning,       // Damage buildings
    Plague,          // Kill peasants
    Drought,         // Reduce food
    // Defensive
    Shield,          // Increase defense
    Barrier,         // Protect from spells
    Heal,            // Restore troops
    // Utility
    Clairvoyance,    // View province info
    Haste,           // Speed up training
    Prosperity,      // Increase income
}

impl SpellType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Fireball => "Fireball",
            Self::Lightning => "Lightning",
            Self::Plague => "Plague",
            Self::Drought => "Drought",
            Self::Shield => "Shield",
            Self::Barrier => "Barrier",
            Self::Heal => "Heal",
            Self::Clairvoyance => "Clairvoyance",
            Self::Haste => "Haste",
            Self::Prosperity => "Prosperity",
        }
    }

    pub fn rune_cost(&self) -> u32 {
        match self {
            Self::Fireball => 200,
            Self::Lightning => 350,
            Self::Plague => 500,
            Self::Drought => 400,
            Self::Shield => 150,
            Self::Barrier => 300,
            Self::Heal => 250,
            Self::Clairvoyance => 100,
            Self::Haste => 200,
            Self::Prosperity => 300,
        }
    }
}

/// Helper functions
pub fn get_race(key: &str) -> Option<&'static Race> {
    RACES.iter().find(|r| r.key == key)
}

pub fn get_personality(key: &str) -> Option<&'static Personality> {
    PERSONALITIES.iter().find(|p| p.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_race_lookup() {
        assert!(get_race("human").is_some());
        assert!(get_race("elf").is_some());
        assert!(get_race("nonexistent").is_none());
    }

    #[test]
    fn test_personality_lookup() {
        assert!(get_personality("merchant").is_some());
        assert!(get_personality("warrior").is_some());
        assert!(get_personality("nonexistent").is_none());
    }

    #[test]
    fn test_building_costs() {
        for building in BuildingType::all() {
            assert!(building.base_cost() > 0);
            assert!(!building.name().is_empty());
            assert!(!building.description().is_empty());
        }
    }

    #[test]
    fn test_unit_stats() {
        // Knights should be stronger than soldiers
        assert!(UnitType::Knight.attack() > UnitType::Soldier.attack());
        assert!(UnitType::Knight.cost() > UnitType::Soldier.cost());
    }
}
