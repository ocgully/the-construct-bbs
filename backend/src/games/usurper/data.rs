//! Static game data for Usurper
//!
//! Defines dungeons, monsters, equipment, substances, and other game constants.

/// Character class definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    Warrior,    // High STR, VIT - melee focus
    Rogue,      // High AGI, CHA - stealth and critical hits
    Mage,       // High INT - magic damage and utility
    Cleric,     // Balanced - healing and support
    Berserker,  // Very high STR, low mental stability
}

impl CharacterClass {
    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Rogue => "Rogue",
            CharacterClass::Mage => "Mage",
            CharacterClass::Cleric => "Cleric",
            CharacterClass::Berserker => "Berserker",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Masters of combat, high health and damage",
            CharacterClass::Rogue => "Swift and deadly, critical hit specialists",
            CharacterClass::Mage => "Wield arcane power, devastating spells",
            CharacterClass::Cleric => "Divine warriors, healing and protection",
            CharacterClass::Berserker => "Rage-fueled fighters, unstable but powerful",
        }
    }

    pub fn base_stats(&self) -> (u32, u32, u32, u32, u32, u32) {
        // (STR, AGI, VIT, INT, CHA, mental_stability)
        match self {
            CharacterClass::Warrior => (15, 10, 14, 8, 10, 100),
            CharacterClass::Rogue => (10, 15, 10, 10, 14, 100),
            CharacterClass::Mage => (8, 10, 8, 16, 12, 100),
            CharacterClass::Cleric => (12, 10, 12, 12, 14, 100),
            CharacterClass::Berserker => (18, 12, 15, 6, 8, 70),
        }
    }
}

/// Dungeon level definition
#[derive(Debug, Clone)]
pub struct Dungeon {
    pub level: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub tier: DungeonTier,
    pub monster_types: &'static [&'static str],
    pub loot_quality: u32,      // 1-10 multiplier for loot drops
    pub danger_rating: u32,     // Affects encounter frequency
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonTier {
    Surface,      // Level 1-10
    Upper,        // Level 11-25
    Deep,         // Level 26-50
    Abyss,        // Level 51-75
    Depths,       // Level 76-100
    Bottom,       // Level 101+ (The Supreme Being)
}

impl DungeonTier {
    pub fn name(&self) -> &'static str {
        match self {
            DungeonTier::Surface => "Surface Caves",
            DungeonTier::Upper => "Upper Dungeons",
            DungeonTier::Deep => "Deep Caves",
            DungeonTier::Abyss => "The Abyss",
            DungeonTier::Depths => "The Depths",
            DungeonTier::Bottom => "The Bottom",
        }
    }

    pub fn color_hint(&self) -> &'static str {
        match self {
            DungeonTier::Surface => "green",
            DungeonTier::Upper => "yellow",
            DungeonTier::Deep => "brown",
            DungeonTier::Abyss => "magenta",
            DungeonTier::Depths => "red",
            DungeonTier::Bottom => "white",
        }
    }
}

/// Monster definition
#[derive(Debug, Clone)]
pub struct Monster {
    pub key: &'static str,
    pub name: &'static str,
    pub min_level: u32,
    pub max_level: u32,
    pub base_hp: u32,
    pub base_damage: u32,
    pub base_defense: u32,
    pub xp_reward: u32,
    pub gold_drop: (u32, u32),  // min, max
    pub is_boss: bool,
    pub description: &'static str,
}

/// Equipment item definition
#[derive(Debug, Clone)]
pub struct EquipmentItem {
    pub key: &'static str,
    pub name: &'static str,
    pub slot: EquipmentSlot,
    pub min_level: u32,
    pub quality: ItemQuality,
    pub stat_bonuses: StatBonuses,
    pub price: u32,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    Weapon,
    Shield,
    Helmet,
    Armor,
    Gloves,
    Boots,
    RingLeft,
    RingRight,
    Amulet,
    Cloak,
}

impl EquipmentSlot {
    pub fn name(&self) -> &'static str {
        match self {
            EquipmentSlot::Weapon => "Weapon",
            EquipmentSlot::Shield => "Shield",
            EquipmentSlot::Helmet => "Helmet",
            EquipmentSlot::Armor => "Armor",
            EquipmentSlot::Gloves => "Gloves",
            EquipmentSlot::Boots => "Boots",
            EquipmentSlot::RingLeft => "Ring (L)",
            EquipmentSlot::RingRight => "Ring (R)",
            EquipmentSlot::Amulet => "Amulet",
            EquipmentSlot::Cloak => "Cloak",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemQuality {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemQuality {
    pub fn name(&self) -> &'static str {
        match self {
            ItemQuality::Common => "Common",
            ItemQuality::Uncommon => "Uncommon",
            ItemQuality::Rare => "Rare",
            ItemQuality::Epic => "Epic",
            ItemQuality::Legendary => "Legendary",
        }
    }

    pub fn multiplier(&self) -> f32 {
        match self {
            ItemQuality::Common => 1.0,
            ItemQuality::Uncommon => 1.25,
            ItemQuality::Rare => 1.5,
            ItemQuality::Epic => 2.0,
            ItemQuality::Legendary => 3.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StatBonuses {
    pub strength: i32,
    pub agility: i32,
    pub vitality: i32,
    pub intelligence: i32,
    pub charisma: i32,
    pub damage: i32,
    pub defense: i32,
    pub hp_bonus: i32,
    pub mental_stability: i32,
}

impl StatBonuses {
    /// Const default for use in static arrays
    pub const fn zero() -> Self {
        StatBonuses {
            strength: 0,
            agility: 0,
            vitality: 0,
            intelligence: 0,
            charisma: 0,
            damage: 0,
            defense: 0,
            hp_bonus: 0,
            mental_stability: 0,
        }
    }
}

/// Substance (drug/steroid) definition
#[derive(Debug, Clone)]
pub struct Substance {
    pub key: &'static str,
    pub name: &'static str,
    pub category: SubstanceCategory,
    pub effects: SubstanceEffect,
    pub duration_turns: u32,
    pub mental_cost: i32,       // Negative = costs mental stability
    pub addiction_chance: u32,  // Percentage 0-100
    pub price: u32,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstanceCategory {
    Steroid,    // Strength/physical boosts
    Stimulant,  // Speed/action boosts
    Sedative,   // Defense/mental recovery
    Psychedelic,// Magic/perception boosts, high mental cost
    Alchemical, // Special effects
}

impl SubstanceCategory {
    pub fn name(&self) -> &'static str {
        match self {
            SubstanceCategory::Steroid => "Steroid",
            SubstanceCategory::Stimulant => "Stimulant",
            SubstanceCategory::Sedative => "Sedative",
            SubstanceCategory::Psychedelic => "Psychedelic",
            SubstanceCategory::Alchemical => "Alchemical",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SubstanceEffect {
    pub strength_mod: i32,
    pub agility_mod: i32,
    pub vitality_mod: i32,
    pub intelligence_mod: i32,
    pub damage_mod: i32,
    pub defense_mod: i32,
    pub action_bonus: u32,
    pub healing: i32,
    pub invincible_turns: u32,
}

impl SubstanceEffect {
    /// Const default for use in static arrays
    pub const fn zero() -> Self {
        SubstanceEffect {
            strength_mod: 0,
            agility_mod: 0,
            vitality_mod: 0,
            intelligence_mod: 0,
            damage_mod: 0,
            defense_mod: 0,
            action_bonus: 0,
            healing: 0,
            invincible_turns: 0,
        }
    }
}

// ============================================================================
// STATIC DATA
// ============================================================================

pub static DUNGEONS: &[Dungeon] = &[
    // SURFACE CAVES (Level 1-10) - Training grounds
    Dungeon {
        level: 1, name: "Mountain Entrance",
        description: "Daylight still reaches here. Cobwebs and rat droppings litter the floor.",
        tier: DungeonTier::Surface,
        monster_types: &["rat", "bat", "spider"],
        loot_quality: 1, danger_rating: 1,
    },
    Dungeon {
        level: 2, name: "Moss Caverns",
        description: "Bioluminescent moss provides an eerie green glow.",
        tier: DungeonTier::Surface,
        monster_types: &["rat", "goblin", "spider"],
        loot_quality: 1, danger_rating: 2,
    },
    Dungeon {
        level: 5, name: "Goblin Warrens",
        description: "The stench of goblin filth burns your nostrils.",
        tier: DungeonTier::Surface,
        monster_types: &["goblin", "goblin_shaman", "wolf"],
        loot_quality: 2, danger_rating: 3,
    },
    Dungeon {
        level: 10, name: "Surface Gate",
        description: "Ancient dwarven architecture marks the boundary to the depths below.",
        tier: DungeonTier::Surface,
        monster_types: &["skeleton", "zombie", "goblin_chief"],
        loot_quality: 3, danger_rating: 4,
    },

    // UPPER DUNGEONS (Level 11-25)
    Dungeon {
        level: 15, name: "Abandoned Mines",
        description: "Rusted minecarts and collapsed tunnels. Something lurks in the darkness.",
        tier: DungeonTier::Upper,
        monster_types: &["skeleton", "ghost", "dark_elf"],
        loot_quality: 3, danger_rating: 5,
    },
    Dungeon {
        level: 20, name: "Crypts of the Forgotten",
        description: "Nameless kings rest here, their treasures cursed.",
        tier: DungeonTier::Upper,
        monster_types: &["wraith", "vampire_spawn", "mummy"],
        loot_quality: 4, danger_rating: 6,
    },
    Dungeon {
        level: 25, name: "Temple of Shadows",
        description: "An unholy cathedral to dark gods. Blood stains the altar.",
        tier: DungeonTier::Upper,
        monster_types: &["cultist", "demon_imp", "shadow_priest"],
        loot_quality: 5, danger_rating: 7,
    },

    // DEEP CAVES (Level 26-50)
    Dungeon {
        level: 30, name: "Fungal Forests",
        description: "Giant mushrooms tower overhead. Spores cloud the air.",
        tier: DungeonTier::Deep,
        monster_types: &["myconid", "sporeling", "fungal_beast"],
        loot_quality: 5, danger_rating: 7,
    },
    Dungeon {
        level: 40, name: "Drow Outpost",
        description: "Dark elves watch from the shadows. Their poison is legendary.",
        tier: DungeonTier::Deep,
        monster_types: &["drow_warrior", "drow_mage", "drider"],
        loot_quality: 6, danger_rating: 8,
    },
    Dungeon {
        level: 50, name: "Gates of the Abyss",
        description: "Reality warps here. Madness whispers at the edge of perception.",
        tier: DungeonTier::Deep,
        monster_types: &["demon", "chaos_spawn", "abyss_watcher"],
        loot_quality: 7, danger_rating: 9,
    },

    // THE ABYSS (Level 51-75)
    Dungeon {
        level: 60, name: "Nightmare Realm",
        description: "Your worst fears manifest. Sanity slips away.",
        tier: DungeonTier::Abyss,
        monster_types: &["nightmare", "fear_demon", "mind_flayer"],
        loot_quality: 7, danger_rating: 9,
    },
    Dungeon {
        level: 70, name: "Halls of Torment",
        description: "Eternal screams echo. The damned writhe in chains.",
        tier: DungeonTier::Abyss,
        monster_types: &["torturer", "pain_demon", "soul_reaper"],
        loot_quality: 8, danger_rating: 10,
    },
    Dungeon {
        level: 75, name: "Threshold of Oblivion",
        description: "Beyond lies only darkness. Few return from here.",
        tier: DungeonTier::Abyss,
        monster_types: &["void_horror", "oblivion_knight", "abyssal_lord"],
        loot_quality: 8, danger_rating: 10,
    },

    // THE DEPTHS (Level 76-100)
    Dungeon {
        level: 80, name: "Bone Gardens",
        description: "Mountains of bones from countless fallen adventurers.",
        tier: DungeonTier::Depths,
        monster_types: &["bone_colossus", "death_knight", "lich"],
        loot_quality: 9, danger_rating: 10,
    },
    Dungeon {
        level: 90, name: "Dragon's Graveyard",
        description: "Ancient dragon skeletons line the walls. Some still stir.",
        tier: DungeonTier::Depths,
        monster_types: &["dracolich", "elder_wyrm", "dragon_zombie"],
        loot_quality: 9, danger_rating: 10,
    },
    Dungeon {
        level: 100, name: "Antechamber of the Supreme",
        description: "The final barrier. Beyond lies destiny itself.",
        tier: DungeonTier::Depths,
        monster_types: &["supreme_guardian", "eternal_watcher"],
        loot_quality: 10, danger_rating: 10,
    },

    // THE BOTTOM (Level 101+)
    Dungeon {
        level: 101, name: "Throne of the Supreme Being",
        description: "You stand before the most powerful entity in existence.",
        tier: DungeonTier::Bottom,
        monster_types: &["supreme_being"],
        loot_quality: 10, danger_rating: 10,
    },
];

pub static MONSTERS: &[Monster] = &[
    // Surface tier
    Monster { key: "rat", name: "Giant Rat", min_level: 1, max_level: 3,
        base_hp: 10, base_damage: 3, base_defense: 1, xp_reward: 5, gold_drop: (1, 10),
        is_boss: false, description: "Mangy and disease-ridden" },
    Monster { key: "bat", name: "Cave Bat", min_level: 1, max_level: 4,
        base_hp: 8, base_damage: 4, base_defense: 2, xp_reward: 6, gold_drop: (2, 12),
        is_boss: false, description: "Swoops from the darkness" },
    Monster { key: "spider", name: "Giant Spider", min_level: 2, max_level: 6,
        base_hp: 15, base_damage: 5, base_defense: 2, xp_reward: 10, gold_drop: (5, 20),
        is_boss: false, description: "Venomous and quick" },
    Monster { key: "goblin", name: "Goblin", min_level: 3, max_level: 8,
        base_hp: 20, base_damage: 6, base_defense: 3, xp_reward: 15, gold_drop: (10, 30),
        is_boss: false, description: "Cunning and cowardly" },
    Monster { key: "goblin_shaman", name: "Goblin Shaman", min_level: 5, max_level: 10,
        base_hp: 25, base_damage: 10, base_defense: 2, xp_reward: 25, gold_drop: (20, 50),
        is_boss: false, description: "Wields dark magics" },
    Monster { key: "wolf", name: "Dire Wolf", min_level: 4, max_level: 9,
        base_hp: 30, base_damage: 8, base_defense: 4, xp_reward: 20, gold_drop: (5, 25),
        is_boss: false, description: "Fangs drip with hunger" },
    Monster { key: "skeleton", name: "Skeleton Warrior", min_level: 6, max_level: 12,
        base_hp: 35, base_damage: 9, base_defense: 5, xp_reward: 30, gold_drop: (15, 40),
        is_boss: false, description: "Bones rattle with malice" },
    Monster { key: "zombie", name: "Rotting Zombie", min_level: 6, max_level: 11,
        base_hp: 50, base_damage: 7, base_defense: 3, xp_reward: 28, gold_drop: (10, 35),
        is_boss: false, description: "Shambles toward fresh meat" },
    Monster { key: "goblin_chief", name: "Goblin Chieftain", min_level: 8, max_level: 12,
        base_hp: 80, base_damage: 15, base_defense: 8, xp_reward: 100, gold_drop: (50, 150),
        is_boss: true, description: "Scarred leader of the horde" },

    // Upper dungeons
    Monster { key: "ghost", name: "Restless Spirit", min_level: 12, max_level: 18,
        base_hp: 40, base_damage: 12, base_defense: 6, xp_reward: 50, gold_drop: (30, 80),
        is_boss: false, description: "Wails echo through stone" },
    Monster { key: "dark_elf", name: "Dark Elf Scout", min_level: 14, max_level: 22,
        base_hp: 55, base_damage: 15, base_defense: 8, xp_reward: 70, gold_drop: (40, 100),
        is_boss: false, description: "Silent and deadly" },
    Monster { key: "wraith", name: "Wraith", min_level: 18, max_level: 25,
        base_hp: 60, base_damage: 18, base_defense: 10, xp_reward: 90, gold_drop: (60, 150),
        is_boss: false, description: "Drains life essence" },
    Monster { key: "vampire_spawn", name: "Vampire Spawn", min_level: 18, max_level: 26,
        base_hp: 75, base_damage: 20, base_defense: 12, xp_reward: 120, gold_drop: (80, 200),
        is_boss: false, description: "Hungers for blood" },
    Monster { key: "mummy", name: "Ancient Mummy", min_level: 20, max_level: 28,
        base_hp: 100, base_damage: 16, base_defense: 15, xp_reward: 130, gold_drop: (100, 250),
        is_boss: false, description: "Wrapped in cursed bandages" },
    Monster { key: "cultist", name: "Dark Cultist", min_level: 22, max_level: 28,
        base_hp: 65, base_damage: 22, base_defense: 8, xp_reward: 100, gold_drop: (70, 180),
        is_boss: false, description: "Chants forbidden words" },
    Monster { key: "demon_imp", name: "Demon Imp", min_level: 22, max_level: 30,
        base_hp: 50, base_damage: 25, base_defense: 5, xp_reward: 110, gold_drop: (60, 160),
        is_boss: false, description: "Cackles with malicious glee" },
    Monster { key: "shadow_priest", name: "Shadow Priest", min_level: 24, max_level: 30,
        base_hp: 120, base_damage: 30, base_defense: 15, xp_reward: 250, gold_drop: (150, 400),
        is_boss: true, description: "High servant of darkness" },

    // Deep caves
    Monster { key: "myconid", name: "Myconid Sovereign", min_level: 28, max_level: 35,
        base_hp: 90, base_damage: 20, base_defense: 18, xp_reward: 150, gold_drop: (100, 250),
        is_boss: false, description: "Telepathic fungus entity" },
    Monster { key: "sporeling", name: "Toxic Sporeling", min_level: 26, max_level: 34,
        base_hp: 60, base_damage: 28, base_defense: 10, xp_reward: 140, gold_drop: (80, 200),
        is_boss: false, description: "Explodes into poison" },
    Monster { key: "drow_warrior", name: "Drow Warrior", min_level: 35, max_level: 45,
        base_hp: 120, base_damage: 35, base_defense: 22, xp_reward: 200, gold_drop: (150, 350),
        is_boss: false, description: "Elite dark elf soldier" },
    Monster { key: "drow_mage", name: "Drow Archmage", min_level: 38, max_level: 48,
        base_hp: 90, base_damage: 50, base_defense: 15, xp_reward: 280, gold_drop: (200, 500),
        is_boss: false, description: "Master of shadow magic" },
    Monster { key: "drider", name: "Drider", min_level: 40, max_level: 50,
        base_hp: 180, base_damage: 45, base_defense: 25, xp_reward: 400, gold_drop: (250, 600),
        is_boss: true, description: "Cursed spider-elf abomination" },
    Monster { key: "demon", name: "Pit Demon", min_level: 45, max_level: 55,
        base_hp: 200, base_damage: 55, base_defense: 30, xp_reward: 500, gold_drop: (300, 700),
        is_boss: false, description: "Flames wreath its form" },
    Monster { key: "abyss_watcher", name: "Abyss Watcher", min_level: 48, max_level: 55,
        base_hp: 300, base_damage: 60, base_defense: 35, xp_reward: 800, gold_drop: (500, 1000),
        is_boss: true, description: "Guardian of the abyss gates" },

    // Abyss tier
    Monster { key: "nightmare", name: "Living Nightmare", min_level: 55, max_level: 65,
        base_hp: 250, base_damage: 65, base_defense: 35, xp_reward: 600, gold_drop: (400, 900),
        is_boss: false, description: "Feeds on fear itself" },
    Monster { key: "mind_flayer", name: "Mind Flayer", min_level: 58, max_level: 68,
        base_hp: 220, base_damage: 80, base_defense: 30, xp_reward: 750, gold_drop: (500, 1100),
        is_boss: false, description: "Devours thoughts" },
    Monster { key: "void_horror", name: "Void Horror", min_level: 70, max_level: 80,
        base_hp: 400, base_damage: 90, base_defense: 45, xp_reward: 1200, gold_drop: (800, 1800),
        is_boss: true, description: "Tears at reality itself" },

    // Depths tier
    Monster { key: "death_knight", name: "Death Knight", min_level: 75, max_level: 85,
        base_hp: 500, base_damage: 100, base_defense: 55, xp_reward: 1500, gold_drop: (1000, 2500),
        is_boss: false, description: "Fallen paladin of darkness" },
    Monster { key: "lich", name: "Lich King", min_level: 80, max_level: 90,
        base_hp: 450, base_damage: 130, base_defense: 50, xp_reward: 2000, gold_drop: (1500, 3500),
        is_boss: true, description: "Ancient undead sorcerer" },
    Monster { key: "dracolich", name: "Dracolich", min_level: 85, max_level: 95,
        base_hp: 800, base_damage: 150, base_defense: 70, xp_reward: 3000, gold_drop: (2000, 5000),
        is_boss: true, description: "Undead dragon terror" },
    Monster { key: "supreme_guardian", name: "Supreme Guardian", min_level: 95, max_level: 100,
        base_hp: 1500, base_damage: 200, base_defense: 100, xp_reward: 5000, gold_drop: (5000, 10000),
        is_boss: true, description: "Final defender before the throne" },

    // THE SUPREME BEING
    Monster { key: "supreme_being", name: "The Supreme Being", min_level: 100, max_level: 999,
        base_hp: 10000, base_damage: 500, base_defense: 200, xp_reward: 100000, gold_drop: (50000, 100000),
        is_boss: true, description: "The ultimate power. The destiny of worlds." },
];

pub static EQUIPMENT_ITEMS: &[EquipmentItem] = &[
    // WEAPONS
    EquipmentItem {
        key: "rusty_sword", name: "Rusty Sword", slot: EquipmentSlot::Weapon,
        min_level: 1, quality: ItemQuality::Common,
        stat_bonuses: StatBonuses { damage: 5, ..StatBonuses::zero() },
        price: 50, description: "Better than nothing",
    },
    EquipmentItem {
        key: "iron_sword", name: "Iron Sword", slot: EquipmentSlot::Weapon,
        min_level: 5, quality: ItemQuality::Common,
        stat_bonuses: StatBonuses { damage: 12, strength: 1, ..StatBonuses::zero() },
        price: 200, description: "Reliable blade",
    },
    EquipmentItem {
        key: "steel_sword", name: "Steel Sword", slot: EquipmentSlot::Weapon,
        min_level: 15, quality: ItemQuality::Uncommon,
        stat_bonuses: StatBonuses { damage: 25, strength: 3, ..StatBonuses::zero() },
        price: 800, description: "Well-forged steel",
    },
    EquipmentItem {
        key: "shadow_blade", name: "Shadow Blade", slot: EquipmentSlot::Weapon,
        min_level: 30, quality: ItemQuality::Rare,
        stat_bonuses: StatBonuses { damage: 45, agility: 5, ..StatBonuses::zero() },
        price: 3000, description: "Wreathed in darkness",
    },
    EquipmentItem {
        key: "demon_slayer", name: "Demonslayer", slot: EquipmentSlot::Weapon,
        min_level: 50, quality: ItemQuality::Epic,
        stat_bonuses: StatBonuses { damage: 80, strength: 10, vitality: 5, ..StatBonuses::zero() },
        price: 15000, description: "Forged to destroy evil",
    },
    EquipmentItem {
        key: "godsbane", name: "Godsbane", slot: EquipmentSlot::Weapon,
        min_level: 80, quality: ItemQuality::Legendary,
        stat_bonuses: StatBonuses { damage: 150, strength: 20, agility: 10, vitality: 10, ..StatBonuses::zero() },
        price: 100000, description: "The blade that slew a god",
    },

    // ARMOR
    EquipmentItem {
        key: "leather_armor", name: "Leather Armor", slot: EquipmentSlot::Armor,
        min_level: 1, quality: ItemQuality::Common,
        stat_bonuses: StatBonuses { defense: 5, ..StatBonuses::zero() },
        price: 100, description: "Basic protection",
    },
    EquipmentItem {
        key: "chainmail", name: "Chainmail", slot: EquipmentSlot::Armor,
        min_level: 10, quality: ItemQuality::Uncommon,
        stat_bonuses: StatBonuses { defense: 15, vitality: 2, ..StatBonuses::zero() },
        price: 500, description: "Interlocking rings of steel",
    },
    EquipmentItem {
        key: "plate_armor", name: "Plate Armor", slot: EquipmentSlot::Armor,
        min_level: 25, quality: ItemQuality::Rare,
        stat_bonuses: StatBonuses { defense: 35, vitality: 5, hp_bonus: 20, ..StatBonuses::zero() },
        price: 2500, description: "Heavy but protective",
    },
    EquipmentItem {
        key: "dragon_scale", name: "Dragon Scale Armor", slot: EquipmentSlot::Armor,
        min_level: 60, quality: ItemQuality::Epic,
        stat_bonuses: StatBonuses { defense: 70, vitality: 15, hp_bonus: 50, ..StatBonuses::zero() },
        price: 25000, description: "Scales of an ancient dragon",
    },

    // SHIELDS
    EquipmentItem {
        key: "wooden_shield", name: "Wooden Shield", slot: EquipmentSlot::Shield,
        min_level: 1, quality: ItemQuality::Common,
        stat_bonuses: StatBonuses { defense: 3, ..StatBonuses::zero() },
        price: 30, description: "Splintery but functional",
    },
    EquipmentItem {
        key: "iron_shield", name: "Iron Shield", slot: EquipmentSlot::Shield,
        min_level: 12, quality: ItemQuality::Uncommon,
        stat_bonuses: StatBonuses { defense: 12, vitality: 1, ..StatBonuses::zero() },
        price: 400, description: "Heavy iron protection",
    },

    // HELMETS
    EquipmentItem {
        key: "leather_cap", name: "Leather Cap", slot: EquipmentSlot::Helmet,
        min_level: 1, quality: ItemQuality::Common,
        stat_bonuses: StatBonuses { defense: 2, ..StatBonuses::zero() },
        price: 25, description: "Basic head protection",
    },
    EquipmentItem {
        key: "steel_helm", name: "Steel Helm", slot: EquipmentSlot::Helmet,
        min_level: 20, quality: ItemQuality::Rare,
        stat_bonuses: StatBonuses { defense: 15, vitality: 3, ..StatBonuses::zero() },
        price: 1200, description: "Full head protection",
    },

    // RINGS
    EquipmentItem {
        key: "ring_strength", name: "Ring of Might", slot: EquipmentSlot::RingLeft,
        min_level: 10, quality: ItemQuality::Uncommon,
        stat_bonuses: StatBonuses { strength: 5, ..StatBonuses::zero() },
        price: 600, description: "Pulses with power",
    },
    EquipmentItem {
        key: "ring_agility", name: "Ring of Shadows", slot: EquipmentSlot::RingRight,
        min_level: 15, quality: ItemQuality::Rare,
        stat_bonuses: StatBonuses { agility: 8, ..StatBonuses::zero() },
        price: 1500, description: "Swift as darkness",
    },
    EquipmentItem {
        key: "ring_madness", name: "Ring of Madness", slot: EquipmentSlot::RingLeft,
        min_level: 40, quality: ItemQuality::Epic,
        stat_bonuses: StatBonuses { strength: 15, damage: 25, mental_stability: -20, ..StatBonuses::zero() },
        price: 8000, description: "Power at a terrible cost",
    },

    // AMULETS
    EquipmentItem {
        key: "amulet_protection", name: "Amulet of Protection", slot: EquipmentSlot::Amulet,
        min_level: 8, quality: ItemQuality::Uncommon,
        stat_bonuses: StatBonuses { defense: 8, hp_bonus: 15, ..StatBonuses::zero() },
        price: 700, description: "Wards against harm",
    },
    EquipmentItem {
        key: "amulet_clarity", name: "Amulet of Clarity", slot: EquipmentSlot::Amulet,
        min_level: 25, quality: ItemQuality::Rare,
        stat_bonuses: StatBonuses { intelligence: 10, mental_stability: 15, ..StatBonuses::zero() },
        price: 3500, description: "Maintains sanity in darkness",
    },

    // BOOTS
    EquipmentItem {
        key: "leather_boots", name: "Leather Boots", slot: EquipmentSlot::Boots,
        min_level: 1, quality: ItemQuality::Common,
        stat_bonuses: StatBonuses { agility: 1, ..StatBonuses::zero() },
        price: 40, description: "Comfortable footwear",
    },
    EquipmentItem {
        key: "boots_speed", name: "Boots of Speed", slot: EquipmentSlot::Boots,
        min_level: 20, quality: ItemQuality::Rare,
        stat_bonuses: StatBonuses { agility: 8, ..StatBonuses::zero() },
        price: 2000, description: "Swift as the wind",
    },

    // GLOVES
    EquipmentItem {
        key: "leather_gloves", name: "Leather Gloves", slot: EquipmentSlot::Gloves,
        min_level: 1, quality: ItemQuality::Common,
        stat_bonuses: StatBonuses { damage: 2, ..StatBonuses::zero() },
        price: 35, description: "Grip enhancement",
    },
    EquipmentItem {
        key: "gauntlets_power", name: "Gauntlets of Power", slot: EquipmentSlot::Gloves,
        min_level: 35, quality: ItemQuality::Epic,
        stat_bonuses: StatBonuses { strength: 10, damage: 15, ..StatBonuses::zero() },
        price: 5000, description: "Crushing force",
    },

    // CLOAKS
    EquipmentItem {
        key: "travelers_cloak", name: "Traveler's Cloak", slot: EquipmentSlot::Cloak,
        min_level: 1, quality: ItemQuality::Common,
        stat_bonuses: StatBonuses { defense: 1, ..StatBonuses::zero() },
        price: 20, description: "Keeps the rain off",
    },
    EquipmentItem {
        key: "shadow_cloak", name: "Shadow Cloak", slot: EquipmentSlot::Cloak,
        min_level: 30, quality: ItemQuality::Rare,
        stat_bonuses: StatBonuses { agility: 6, defense: 10, ..StatBonuses::zero() },
        price: 2800, description: "Blend with darkness",
    },
];

pub static SUBSTANCES: &[Substance] = &[
    // STEROIDS - Physical enhancement
    Substance {
        key: "basic_steroid", name: "Basic Steroids",
        category: SubstanceCategory::Steroid,
        effects: SubstanceEffect { strength_mod: 10, vitality_mod: 5, ..SubstanceEffect::zero() },
        duration_turns: 10, mental_cost: -5, addiction_chance: 10, price: 100,
        description: "Common muscle enhancer. Modest gains, modest risks.",
    },
    Substance {
        key: "power_enhancer", name: "Power Enhancer",
        category: SubstanceCategory::Steroid,
        effects: SubstanceEffect { strength_mod: 20, damage_mod: 15, ..SubstanceEffect::zero() },
        duration_turns: 8, mental_cost: -10, addiction_chance: 20, price: 350,
        description: "Military-grade compound. Serious strength boost.",
    },
    Substance {
        key: "rage_inducer", name: "Rage Inducer",
        category: SubstanceCategory::Steroid,
        effects: SubstanceEffect { strength_mod: 30, damage_mod: 25, defense_mod: -10, ..SubstanceEffect::zero() },
        duration_turns: 5, mental_cost: -15, addiction_chance: 35, price: 600,
        description: "Unleashes primal fury. Defense suffers as rage consumes.",
    },
    Substance {
        key: "titan_serum", name: "Titan Serum",
        category: SubstanceCategory::Steroid,
        effects: SubstanceEffect { strength_mod: 50, vitality_mod: 30, damage_mod: 40, ..SubstanceEffect::zero() },
        duration_turns: 6, mental_cost: -25, addiction_chance: 50, price: 2000,
        description: "Legendary compound. Transform into a monster temporarily.",
    },

    // STIMULANTS - Speed and action boosts
    Substance {
        key: "speed_powder", name: "Speed Powder",
        category: SubstanceCategory::Stimulant,
        effects: SubstanceEffect { agility_mod: 15, action_bonus: 1, ..SubstanceEffect::zero() },
        duration_turns: 8, mental_cost: -5, addiction_chance: 15, price: 150,
        description: "Heightens reflexes. Grants additional actions.",
    },
    Substance {
        key: "battle_stim", name: "Battle Stim",
        category: SubstanceCategory::Stimulant,
        effects: SubstanceEffect { agility_mod: 25, damage_mod: 10, action_bonus: 2, ..SubstanceEffect::zero() },
        duration_turns: 5, mental_cost: -12, addiction_chance: 30, price: 500,
        description: "Combat enhancement drug. Time seems to slow.",
    },
    Substance {
        key: "temporal_rush", name: "Temporal Rush",
        category: SubstanceCategory::Stimulant,
        effects: SubstanceEffect { agility_mod: 40, action_bonus: 3, ..SubstanceEffect::zero() },
        duration_turns: 3, mental_cost: -20, addiction_chance: 45, price: 1500,
        description: "Bends perception of time. Extremely taxing.",
    },

    // SEDATIVES - Recovery and defense
    Substance {
        key: "calming_draught", name: "Calming Draught",
        category: SubstanceCategory::Sedative,
        effects: SubstanceEffect { defense_mod: 10, healing: 20, ..SubstanceEffect::zero() },
        duration_turns: 10, mental_cost: 5, addiction_chance: 5, price: 80,
        description: "Soothes mind and body. Minor healing.",
    },
    Substance {
        key: "fortifying_elixir", name: "Fortifying Elixir",
        category: SubstanceCategory::Sedative,
        effects: SubstanceEffect { defense_mod: 25, vitality_mod: 15, healing: 40, ..SubstanceEffect::zero() },
        duration_turns: 8, mental_cost: 10, addiction_chance: 10, price: 300,
        description: "Strengthens constitution. Restores mental stability.",
    },
    Substance {
        key: "iron_will", name: "Iron Will Tonic",
        category: SubstanceCategory::Sedative,
        effects: SubstanceEffect { defense_mod: 40, vitality_mod: 25, ..SubstanceEffect::zero() },
        duration_turns: 6, mental_cost: 20, addiction_chance: 5, price: 800,
        description: "Impenetrable mental fortress. Excellent for recovery.",
    },

    // PSYCHEDELICS - Magic and perception
    Substance {
        key: "vision_moss", name: "Vision Moss",
        category: SubstanceCategory::Psychedelic,
        effects: SubstanceEffect { intelligence_mod: 15, ..SubstanceEffect::zero() },
        duration_turns: 12, mental_cost: -8, addiction_chance: 15, price: 200,
        description: "Opens the third eye. Common in mage circles.",
    },
    Substance {
        key: "soul_shroom", name: "Soul Shroom",
        category: SubstanceCategory::Psychedelic,
        effects: SubstanceEffect { intelligence_mod: 30, damage_mod: 20, ..SubstanceEffect::zero() },
        duration_turns: 8, mental_cost: -15, addiction_chance: 25, price: 600,
        description: "Glimpse the spirit realm. Enhances magical damage.",
    },
    Substance {
        key: "void_essence", name: "Void Essence",
        category: SubstanceCategory::Psychedelic,
        effects: SubstanceEffect { intelligence_mod: 50, damage_mod: 40, defense_mod: -20, ..SubstanceEffect::zero() },
        duration_turns: 5, mental_cost: -30, addiction_chance: 60, price: 2500,
        description: "Touch the void itself. Devastating power, devastating cost.",
    },

    // ALCHEMICAL - Special effects
    Substance {
        key: "healing_potion", name: "Healing Potion",
        category: SubstanceCategory::Alchemical,
        effects: SubstanceEffect { healing: 50, ..SubstanceEffect::zero() },
        duration_turns: 1, mental_cost: 0, addiction_chance: 0, price: 100,
        description: "Instant health restoration.",
    },
    Substance {
        key: "greater_healing", name: "Greater Healing Potion",
        category: SubstanceCategory::Alchemical,
        effects: SubstanceEffect { healing: 150, ..SubstanceEffect::zero() },
        duration_turns: 1, mental_cost: 0, addiction_chance: 0, price: 400,
        description: "Significant health restoration.",
    },
    Substance {
        key: "invincibility_elixir", name: "Invincibility Elixir",
        category: SubstanceCategory::Alchemical,
        effects: SubstanceEffect { invincible_turns: 3, ..SubstanceEffect::zero() },
        duration_turns: 3, mental_cost: -5, addiction_chance: 0, price: 5000,
        description: "Temporary invulnerability. Extremely rare.",
    },
    Substance {
        key: "berserker_blood", name: "Berserker Blood",
        category: SubstanceCategory::Alchemical,
        effects: SubstanceEffect { strength_mod: 40, damage_mod: 50, defense_mod: -30, ..SubstanceEffect::zero() },
        duration_turns: 4, mental_cost: -35, addiction_chance: 40, price: 3000,
        description: "The blood of a mad berserker. Ultimate power, ultimate risk.",
    },
];

// ============================================================================
// LOOKUP FUNCTIONS
// ============================================================================

pub fn get_dungeon(level: u32) -> Option<&'static Dungeon> {
    // Find the dungeon at or below the requested level
    DUNGEONS.iter()
        .filter(|d| d.level <= level)
        .max_by_key(|d| d.level)
}

pub fn get_dungeon_by_level(level: u32) -> &'static Dungeon {
    get_dungeon(level).unwrap_or(&DUNGEONS[0])
}

pub fn get_monster(key: &str) -> Option<&'static Monster> {
    MONSTERS.iter().find(|m| m.key == key)
}

pub fn get_equipment(key: &str) -> Option<&'static EquipmentItem> {
    EQUIPMENT_ITEMS.iter().find(|e| e.key == key)
}

pub fn get_substance(key: &str) -> Option<&'static Substance> {
    SUBSTANCES.iter().find(|s| s.key == key)
}

pub fn get_monsters_for_dungeon(dungeon: &Dungeon) -> Vec<&'static Monster> {
    dungeon.monster_types.iter()
        .filter_map(|key| get_monster(key))
        .collect()
}

pub fn get_equipment_for_level(level: u32) -> Vec<&'static EquipmentItem> {
    EQUIPMENT_ITEMS.iter()
        .filter(|e| e.min_level <= level)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_lookup() {
        let d = get_dungeon(5).unwrap();
        assert_eq!(d.level, 5);
        assert_eq!(d.name, "Goblin Warrens");
    }

    #[test]
    fn test_dungeon_fallback() {
        let d = get_dungeon(7).unwrap();
        assert_eq!(d.level, 5); // Falls back to level 5
    }

    #[test]
    fn test_monster_lookup() {
        let m = get_monster("goblin").unwrap();
        assert_eq!(m.name, "Goblin");
    }

    #[test]
    fn test_substance_categories() {
        let steroid = get_substance("basic_steroid").unwrap();
        assert_eq!(steroid.category, SubstanceCategory::Steroid);
        assert!(steroid.effects.strength_mod > 0);
        assert!(steroid.mental_cost < 0);
    }

    #[test]
    fn test_equipment_slots() {
        let sword = get_equipment("rusty_sword").unwrap();
        assert_eq!(sword.slot, EquipmentSlot::Weapon);
    }

    #[test]
    fn test_class_stats() {
        let (str, _agi, _vit, _int, _cha, mental) = CharacterClass::Warrior.base_stats();
        assert_eq!(str, 15);
        assert_eq!(mental, 100);

        let (str, _, _, _, _, mental) = CharacterClass::Berserker.base_stats();
        assert_eq!(str, 18);
        assert_eq!(mental, 70); // Berserkers start with lower mental stability
    }
}
