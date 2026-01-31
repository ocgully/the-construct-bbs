//! Static game data for Dragon Slayer
//! Monsters, masters, weapons, armor, locations, skills

use rand::prelude::*;

/// Monster definition
#[derive(Debug, Clone)]
pub struct Monster {
    pub key: &'static str,
    pub name: &'static str,
    pub level_min: u8,
    pub level_max: u8,
    pub hp_base: u32,
    pub hp_per_level: u32,
    pub strength: u32,
    pub defense: u32,
    pub gold_min: i64,
    pub gold_max: i64,
    pub xp_base: i64,
    #[allow(dead_code)]
    pub description: &'static str,
}

/// Master definition - must defeat to level up
#[derive(Debug, Clone)]
pub struct Master {
    pub level: u8,
    pub name: &'static str,
    pub hp: u32,
    pub strength: u32,
    pub defense: u32,
    pub xp_required: i64,
    #[allow(dead_code)]
    pub quote_on_defeat: &'static str,
    #[allow(dead_code)]
    pub quote_on_victory: &'static str,
}

/// Weapon definition
#[derive(Debug, Clone)]
pub struct Weapon {
    pub key: &'static str,
    pub name: &'static str,
    pub damage: u32,
    pub price: i64,
    pub level_required: u8,
}

/// Armor definition
#[derive(Debug, Clone)]
pub struct Armor {
    pub key: &'static str,
    pub name: &'static str,
    pub defense: u32,
    pub price: i64,
    pub level_required: u8,
}

/// Location in the town
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Location {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub hotkey: char,
}

/// Skill definition
#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub key: &'static str,
    #[allow(dead_code)]
    pub name: &'static str,
    pub path: SkillPath,
    pub level_required: u8,
    pub uses_per_day: u8,
    #[allow(dead_code)]
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillPath {
    DeathKnight,
    Mystic,
    Thief,
}

// ============================================================================
// STATIC DATA - MONSTERS
// ============================================================================

pub static MONSTERS: &[Monster] = &[
    // Level 1-2 monsters
    Monster {
        key: "slime",
        name: "Small Slime",
        level_min: 1, level_max: 2,
        hp_base: 5, hp_per_level: 2,
        strength: 3, defense: 1,
        gold_min: 1, gold_max: 5,
        xp_base: 5,
        description: "A quivering blob of green goo.",
    },
    Monster {
        key: "rat",
        name: "Giant Rat",
        level_min: 1, level_max: 2,
        hp_base: 8, hp_per_level: 3,
        strength: 5, defense: 2,
        gold_min: 2, gold_max: 8,
        xp_base: 8,
        description: "Red eyes gleam in the darkness.",
    },
    Monster {
        key: "bat",
        name: "Vampire Bat",
        level_min: 1, level_max: 3,
        hp_base: 6, hp_per_level: 2,
        strength: 4, defense: 1,
        gold_min: 1, gold_max: 6,
        xp_base: 6,
        description: "It screeches and dives at you!",
    },

    // Level 2-4 monsters
    Monster {
        key: "goblin",
        name: "Goblin",
        level_min: 2, level_max: 4,
        hp_base: 12, hp_per_level: 4,
        strength: 8, defense: 3,
        gold_min: 5, gold_max: 20,
        xp_base: 15,
        description: "A snickering green creature with a rusty dagger.",
    },
    Monster {
        key: "wolf",
        name: "Dire Wolf",
        level_min: 2, level_max: 4,
        hp_base: 15, hp_per_level: 5,
        strength: 10, defense: 4,
        gold_min: 8, gold_max: 25,
        xp_base: 20,
        description: "Yellow eyes and dripping fangs.",
    },
    Monster {
        key: "skeleton",
        name: "Skeleton Warrior",
        level_min: 3, level_max: 5,
        hp_base: 18, hp_per_level: 5,
        strength: 12, defense: 5,
        gold_min: 10, gold_max: 35,
        xp_base: 30,
        description: "Bones rattle as it raises its sword.",
    },

    // Level 4-6 monsters
    Monster {
        key: "orc",
        name: "Orc Berserker",
        level_min: 4, level_max: 6,
        hp_base: 30, hp_per_level: 8,
        strength: 18, defense: 8,
        gold_min: 20, gold_max: 60,
        xp_base: 50,
        description: "Muscles ripple beneath green skin.",
    },
    Monster {
        key: "dark_elf",
        name: "Dark Elf Assassin",
        level_min: 4, level_max: 6,
        hp_base: 25, hp_per_level: 6,
        strength: 22, defense: 6,
        gold_min: 30, gold_max: 80,
        xp_base: 60,
        description: "Silent and deadly, daggers gleaming.",
    },
    Monster {
        key: "ogre",
        name: "Swamp Ogre",
        level_min: 5, level_max: 7,
        hp_base: 50, hp_per_level: 12,
        strength: 25, defense: 10,
        gold_min: 40, gold_max: 100,
        xp_base: 80,
        description: "The stench is overwhelming.",
    },

    // Level 6-8 monsters
    Monster {
        key: "troll",
        name: "Mountain Troll",
        level_min: 6, level_max: 8,
        hp_base: 80, hp_per_level: 15,
        strength: 35, defense: 15,
        gold_min: 60, gold_max: 150,
        xp_base: 120,
        description: "Regenerating flesh makes it hard to kill.",
    },
    Monster {
        key: "necromancer",
        name: "Necromancer",
        level_min: 6, level_max: 8,
        hp_base: 45, hp_per_level: 10,
        strength: 40, defense: 12,
        gold_min: 80, gold_max: 200,
        xp_base: 150,
        description: "Dark energy crackles around skeletal hands.",
    },
    Monster {
        key: "wraith",
        name: "Vengeful Wraith",
        level_min: 7, level_max: 9,
        hp_base: 60, hp_per_level: 12,
        strength: 45, defense: 20,
        gold_min: 100, gold_max: 250,
        xp_base: 180,
        description: "A tortured spirit seeking revenge.",
    },

    // Level 8-10 monsters
    Monster {
        key: "wyvern",
        name: "Young Wyvern",
        level_min: 8, level_max: 10,
        hp_base: 120, hp_per_level: 20,
        strength: 55, defense: 25,
        gold_min: 150, gold_max: 350,
        xp_base: 250,
        description: "Smaller cousin of the dragon, equally vicious.",
    },
    Monster {
        key: "demon",
        name: "Lesser Demon",
        level_min: 8, level_max: 10,
        hp_base: 100, hp_per_level: 18,
        strength: 60, defense: 22,
        gold_min: 200, gold_max: 400,
        xp_base: 300,
        description: "Brimstone and hellfire incarnate.",
    },
    Monster {
        key: "vampire",
        name: "Ancient Vampire",
        level_min: 9, level_max: 11,
        hp_base: 90, hp_per_level: 15,
        strength: 65, defense: 28,
        gold_min: 250, gold_max: 500,
        xp_base: 400,
        description: "Centuries of evil in a single form.",
    },

    // Level 10-12 monsters
    Monster {
        key: "elder_dragon",
        name: "Elder Drake",
        level_min: 10, level_max: 12,
        hp_base: 200, hp_per_level: 30,
        strength: 80, defense: 35,
        gold_min: 400, gold_max: 800,
        xp_base: 600,
        description: "Not the Red Dragon, but fearsome nonetheless.",
    },
    Monster {
        key: "lich",
        name: "Lich Lord",
        level_min: 10, level_max: 12,
        hp_base: 150, hp_per_level: 25,
        strength: 90, defense: 30,
        gold_min: 500, gold_max: 1000,
        xp_base: 700,
        description: "Undead sorcerer of terrible power.",
    },
    Monster {
        key: "chaos_knight",
        name: "Chaos Knight",
        level_min: 11, level_max: 12,
        hp_base: 180, hp_per_level: 28,
        strength: 100, defense: 40,
        gold_min: 600, gold_max: 1200,
        xp_base: 850,
        description: "Armor of darkness, sword of doom.",
    },
];

// ============================================================================
// STATIC DATA - MASTERS
// ============================================================================

pub static MASTERS: &[Master] = &[
    Master {
        level: 1,
        name: "Halder the Trainer",
        hp: 15,
        strength: 10,
        defense: 5,
        xp_required: 100,
        quote_on_defeat: "Not bad, youngling! You have potential.",
        quote_on_victory: "Come back when you've learned something!",
    },
    Master {
        level: 2,
        name: "Buga the Fighter",
        hp: 30,
        strength: 15,
        defense: 8,
        xp_required: 400,
        quote_on_defeat: "Ugh! You hit harder than you look!",
        quote_on_victory: "Pathetic! Train more in the forest!",
    },
    Master {
        level: 3,
        name: "Atsuko Sensei",
        hp: 50,
        strength: 20,
        defense: 12,
        xp_required: 1_000,
        quote_on_defeat: "Your spirit is strong. Honor to you.",
        quote_on_victory: "Discipline. You lack discipline.",
    },
    Master {
        level: 4,
        name: "Sandtiger",
        hp: 80,
        strength: 30,
        defense: 18,
        xp_required: 4_000,
        quote_on_defeat: "The desert breeds warriors. So does this town!",
        quote_on_victory: "The sands will claim your bones.",
    },
    Master {
        level: 5,
        name: "Sparhawk the Knight",
        hp: 150,
        strength: 50,
        defense: 25,
        xp_required: 10_000,
        quote_on_defeat: "A worthy opponent! The kingdom needs you.",
        quote_on_victory: "Return when you're ready for true combat.",
    },
    Master {
        level: 6,
        name: "Aladdin",
        hp: 250,
        strength: 75,
        defense: 35,
        xp_required: 40_000,
        quote_on_defeat: "By the lamp! You have grown powerful!",
        quote_on_victory: "Even genies couldn't help you beat me!",
    },
    Master {
        level: 7,
        name: "Prince Caspian",
        hp: 400,
        strength: 100,
        defense: 50,
        xp_required: 100_000,
        quote_on_defeat: "For Narnia... and for you. Well fought!",
        quote_on_victory: "The Deep Magic protects me still.",
    },
    Master {
        level: 8,
        name: "Gandalf the Grey",
        hp: 600,
        strength: 150,
        defense: 70,
        xp_required: 400_000,
        quote_on_defeat: "Fly, you fool... to greater heights!",
        quote_on_victory: "YOU SHALL NOT PASS to the next level!",
    },
    Master {
        level: 9,
        name: "Turgon the Master",
        hp: 1000,
        strength: 200,
        defense: 90,
        xp_required: 1_000_000,
        quote_on_defeat: "I have trained many, but none like you.",
        quote_on_victory: "I am still the Master here.",
    },
    Master {
        level: 10,
        name: "Merlin the Enchanter",
        hp: 2000,
        strength: 350,
        defense: 120,
        xp_required: 4_000_000,
        quote_on_defeat: "Time itself bends to your will!",
        quote_on_victory: "My magic is too strong for you.",
    },
    Master {
        level: 11,
        name: "Pellinore the Eternal",
        hp: 4000,
        strength: 500,
        defense: 160,
        xp_required: 10_000_000,
        quote_on_defeat: "The Questing Beast... it is yours to hunt.",
        quote_on_victory: "None shall surpass Pellinore!",
    },
    // Level 12 has no master - it's the dragon slaying level
];

// ============================================================================
// STATIC DATA - WEAPONS
// ============================================================================

pub static WEAPONS: &[Weapon] = &[
    Weapon { key: "stick", name: "Stick", damage: 1, price: 0, level_required: 1 },
    Weapon { key: "dagger", name: "Dagger", damage: 3, price: 100, level_required: 1 },
    Weapon { key: "short_sword", name: "Short Sword", damage: 5, price: 500, level_required: 1 },
    Weapon { key: "long_sword", name: "Long Sword", damage: 10, price: 1_500, level_required: 2 },
    Weapon { key: "battle_axe", name: "Battle Axe", damage: 15, price: 3_000, level_required: 3 },
    Weapon { key: "morning_star", name: "Morning Star", damage: 20, price: 6_000, level_required: 4 },
    Weapon { key: "broadsword", name: "Broadsword", damage: 30, price: 12_000, level_required: 5 },
    Weapon { key: "crystal_blade", name: "Crystal Blade", damage: 45, price: 25_000, level_required: 6 },
    Weapon { key: "flaming_sword", name: "Flaming Sword", damage: 60, price: 50_000, level_required: 7 },
    Weapon { key: "shadow_blade", name: "Shadow Blade", damage: 80, price: 100_000, level_required: 8 },
    Weapon { key: "dragon_slayer", name: "Dragon Slayer", damage: 100, price: 200_000, level_required: 9 },
    Weapon { key: "excalibur", name: "Excalibur", damage: 150, price: 500_000, level_required: 10 },
    Weapon { key: "soul_reaver", name: "Soul Reaver", damage: 200, price: 1_000_000, level_required: 11 },
];

// ============================================================================
// STATIC DATA - ARMOR
// ============================================================================

pub static ARMOR: &[Armor] = &[
    Armor { key: "rags", name: "Tattered Rags", defense: 0, price: 0, level_required: 1 },
    Armor { key: "leather", name: "Leather Armor", defense: 3, price: 200, level_required: 1 },
    Armor { key: "studded", name: "Studded Leather", defense: 5, price: 800, level_required: 2 },
    Armor { key: "chain", name: "Chain Mail", defense: 10, price: 2_000, level_required: 3 },
    Armor { key: "scale", name: "Scale Mail", defense: 15, price: 5_000, level_required: 4 },
    Armor { key: "plate", name: "Plate Armor", defense: 25, price: 12_000, level_required: 5 },
    Armor { key: "mithril", name: "Mithril Armor", defense: 40, price: 30_000, level_required: 6 },
    Armor { key: "dragon_scale", name: "Dragon Scale", defense: 55, price: 75_000, level_required: 7 },
    Armor { key: "shadow", name: "Shadow Armor", defense: 70, price: 150_000, level_required: 8 },
    Armor { key: "enchanted", name: "Enchanted Plate", defense: 90, price: 300_000, level_required: 9 },
    Armor { key: "celestial", name: "Celestial Armor", defense: 120, price: 600_000, level_required: 10 },
    Armor { key: "godplate", name: "Armor of the Gods", defense: 160, price: 1_200_000, level_required: 11 },
];

// ============================================================================
// STATIC DATA - LOCATIONS
// ============================================================================

#[allow(dead_code)]
pub static LOCATIONS: &[Location] = &[
    Location {
        key: "inn",
        name: "The Red Dragon Inn",
        description: "Rest, recover, and hear the latest gossip.",
        hotkey: 'I',
    },
    Location {
        key: "forest",
        name: "The Dark Forest",
        description: "Hunt monsters for gold and experience.",
        hotkey: 'F',
    },
    Location {
        key: "training",
        name: "Turgon's Training Grounds",
        description: "Challenge masters to advance your level.",
        hotkey: 'T',
    },
    Location {
        key: "weapons",
        name: "The Weapons Shop",
        description: "Arm yourself for battle.",
        hotkey: 'W',
    },
    Location {
        key: "armor",
        name: "The Armor Shop",
        description: "Protect yourself from harm.",
        hotkey: 'A',
    },
    Location {
        key: "healer",
        name: "The Healer's Hut",
        description: "Restore your health for a price.",
        hotkey: 'H',
    },
    Location {
        key: "bank",
        name: "The Bank",
        description: "Store your gold safely.",
        hotkey: 'B',
    },
    Location {
        key: "court",
        name: "King's Court",
        description: "Daily news, player rankings, and quests.",
        hotkey: 'K',
    },
    Location {
        key: "violet",
        name: "Violet's House",
        description: "Visit the charming barmaid.",
        hotkey: 'V',
    },
    Location {
        key: "seth",
        name: "Seth's Tavern",
        description: "The handsome bard performs nightly.",
        hotkey: 'S',
    },
    Location {
        key: "arena",
        name: "The Slaughter Arena",
        description: "Challenge other warriors to combat.",
        hotkey: 'P',
    },
    Location {
        key: "other",
        name: "Other Places",
        description: "IGM module locations.",
        hotkey: 'O',
    },
];

// ============================================================================
// STATIC DATA - SKILLS
// ============================================================================

pub static SKILL_DATA: &[SkillInfo] = &[
    // Death Knight skills
    SkillInfo {
        key: "power_strike",
        name: "Power Strike",
        path: SkillPath::DeathKnight,
        level_required: 2,
        uses_per_day: 5,
        description: "Deal double damage on your next attack.",
    },
    SkillInfo {
        key: "death_wish",
        name: "Death Wish",
        path: SkillPath::DeathKnight,
        level_required: 4,
        uses_per_day: 3,
        description: "Sacrifice 20% HP for 3x damage.",
    },
    SkillInfo {
        key: "assault",
        name: "Assault",
        path: SkillPath::DeathKnight,
        level_required: 7,
        uses_per_day: 1,
        description: "Devastating attack dealing 5x damage.",
    },

    // Mystic skills
    SkillInfo {
        key: "fireball",
        name: "Fireball",
        path: SkillPath::Mystic,
        level_required: 2,
        uses_per_day: 8,
        description: "Hurl a ball of fire at your enemy.",
    },
    SkillInfo {
        key: "heal",
        name: "Heal",
        path: SkillPath::Mystic,
        level_required: 3,
        uses_per_day: 5,
        description: "Restore 30% of your maximum HP.",
    },
    SkillInfo {
        key: "lightning",
        name: "Lightning Bolt",
        path: SkillPath::Mystic,
        level_required: 5,
        uses_per_day: 4,
        description: "Strike with the fury of the storm.",
    },
    SkillInfo {
        key: "transport",
        name: "Mystical Transport",
        path: SkillPath::Mystic,
        level_required: 8,
        uses_per_day: 2,
        description: "Instantly return to town.",
    },

    // Thief skills
    SkillInfo {
        key: "pick_pocket",
        name: "Pick Pocket",
        path: SkillPath::Thief,
        level_required: 2,
        uses_per_day: 8,
        description: "Steal gold from your enemy.",
    },
    SkillInfo {
        key: "sneak_attack",
        name: "Sneak Attack",
        path: SkillPath::Thief,
        level_required: 4,
        uses_per_day: 4,
        description: "Attack from the shadows for 2.5x damage.",
    },
    SkillInfo {
        key: "dodge",
        name: "Dodge",
        path: SkillPath::Thief,
        level_required: 5,
        uses_per_day: 3,
        description: "Automatically avoid the next attack.",
    },
    SkillInfo {
        key: "fairy_catch",
        name: "Catch Fairy",
        path: SkillPath::Thief,
        level_required: 7,
        uses_per_day: 1,
        description: "Attempt to catch a fairy for protection.",
    },
];

// ============================================================================
// LOOKUP FUNCTIONS
// ============================================================================

#[allow(dead_code)]
pub fn get_monster(key: &str) -> Option<&'static Monster> {
    MONSTERS.iter().find(|m| m.key == key)
}

pub fn get_master(level: u8) -> Option<&'static Master> {
    MASTERS.iter().find(|m| m.level == level)
}

pub fn get_weapon(key: &str) -> Option<&'static Weapon> {
    WEAPONS.iter().find(|w| w.key == key)
}

pub fn get_armor(key: &str) -> Option<&'static Armor> {
    ARMOR.iter().find(|a| a.key == key)
}

#[allow(dead_code)]
pub fn get_location(key: &str) -> Option<&'static Location> {
    LOCATIONS.iter().find(|l| l.key == key)
}

pub fn get_skill(key: &str) -> Option<&'static SkillInfo> {
    SKILL_DATA.iter().find(|s| s.key == key)
}

/// Get all monsters that can appear at a given player level
pub fn get_monsters_for_level(level: u8) -> Vec<&'static Monster> {
    MONSTERS.iter()
        .filter(|m| level >= m.level_min && level <= m.level_max + 2)
        .collect()
}

/// Get a random monster appropriate for the player's level
pub fn get_random_monster(level: u8) -> Option<&'static Monster> {
    let candidates = get_monsters_for_level(level);
    if candidates.is_empty() {
        return None;
    }
    let mut rng = thread_rng();
    Some(candidates[rng.gen_range(0..candidates.len())])
}

/// Get weapons available for purchase at a given level
#[allow(dead_code)]
pub fn get_weapons_for_level(level: u8) -> Vec<&'static Weapon> {
    WEAPONS.iter()
        .filter(|w| w.level_required <= level)
        .collect()
}

/// Get armor available for purchase at a given level
#[allow(dead_code)]
pub fn get_armor_for_level(level: u8) -> Vec<&'static Armor> {
    ARMOR.iter()
        .filter(|a| a.level_required <= level)
        .collect()
}

/// Calculate the Red Dragon's stats based on player level
pub fn get_red_dragon_stats() -> (u32, u32, u32) {
    // HP, Strength, Defense
    (10_000, 800, 200)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_lookup() {
        assert!(get_monster("slime").is_some());
        assert!(get_monster("nonexistent").is_none());
    }

    #[test]
    fn test_master_lookup() {
        let master = get_master(1).unwrap();
        assert_eq!(master.name, "Halder the Trainer");
        assert!(get_master(12).is_none()); // No master at level 12
    }

    #[test]
    fn test_monsters_for_level() {
        let level_1_monsters = get_monsters_for_level(1);
        assert!(!level_1_monsters.is_empty());
        // All returned monsters should be valid for level 1
        for m in level_1_monsters {
            assert!(1 >= m.level_min);
        }
    }

    #[test]
    fn test_weapon_progression() {
        let weapons = get_weapons_for_level(5);
        assert!(weapons.len() > 1);
        // All weapons should be purchasable at level 5
        for w in weapons {
            assert!(w.level_required <= 5);
        }
    }

    #[test]
    fn test_armor_progression() {
        let armor = get_armor_for_level(5);
        assert!(armor.len() > 1);
    }

    #[test]
    fn test_red_dragon_stats() {
        let (hp, str, def) = get_red_dragon_stats();
        assert!(hp > 5000);
        assert!(str > 500);
        assert!(def > 100);
    }
}
