//! Static game data for Ultimo
//!
//! Contains all the definitions for skills, items, monsters, NPCs, zones, etc.

use serde::{Deserialize, Serialize};

// ============================================================================
// SKILLS
// ============================================================================

/// Skill categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCategory {
    Combat,
    Magic,
    Crafting,
    Gathering,
    Miscellaneous,
}

/// Skill definition
#[derive(Debug, Clone)]
pub struct Skill {
    pub key: &'static str,
    pub name: &'static str,
    pub category: SkillCategory,
    pub description: &'static str,
    /// How fast this skill gains (1.0 = normal)
    pub gain_rate: f32,
}

/// All available skills in the game
pub static SKILLS: &[Skill] = &[
    // Combat skills
    Skill {
        key: "swordsmanship",
        name: "Swordsmanship",
        category: SkillCategory::Combat,
        description: "Proficiency with swords and bladed weapons",
        gain_rate: 1.0,
    },
    Skill {
        key: "mace_fighting",
        name: "Mace Fighting",
        category: SkillCategory::Combat,
        description: "Proficiency with maces and blunt weapons",
        gain_rate: 1.0,
    },
    Skill {
        key: "archery",
        name: "Archery",
        category: SkillCategory::Combat,
        description: "Proficiency with bows and crossbows",
        gain_rate: 1.0,
    },
    Skill {
        key: "tactics",
        name: "Tactics",
        category: SkillCategory::Combat,
        description: "Combat strategy and damage bonus",
        gain_rate: 0.9,
    },
    Skill {
        key: "parrying",
        name: "Parrying",
        category: SkillCategory::Combat,
        description: "Blocking attacks with shield or weapon",
        gain_rate: 1.0,
    },
    Skill {
        key: "wrestling",
        name: "Wrestling",
        category: SkillCategory::Combat,
        description: "Unarmed combat",
        gain_rate: 1.1,
    },
    // Magic skills
    Skill {
        key: "magery",
        name: "Magery",
        category: SkillCategory::Magic,
        description: "Casting arcane spells",
        gain_rate: 0.8,
    },
    Skill {
        key: "meditation",
        name: "Meditation",
        category: SkillCategory::Magic,
        description: "Recovering mana faster",
        gain_rate: 0.9,
    },
    Skill {
        key: "resist_spells",
        name: "Resisting Spells",
        category: SkillCategory::Magic,
        description: "Reducing magic damage taken",
        gain_rate: 1.0,
    },
    Skill {
        key: "eval_int",
        name: "Evaluating Intelligence",
        category: SkillCategory::Magic,
        description: "Assessing magical power",
        gain_rate: 1.0,
    },
    // Crafting skills
    Skill {
        key: "blacksmithing",
        name: "Blacksmithing",
        category: SkillCategory::Crafting,
        description: "Crafting metal weapons and armor",
        gain_rate: 0.8,
    },
    Skill {
        key: "tailoring",
        name: "Tailoring",
        category: SkillCategory::Crafting,
        description: "Crafting cloth and leather items",
        gain_rate: 0.9,
    },
    Skill {
        key: "carpentry",
        name: "Carpentry",
        category: SkillCategory::Crafting,
        description: "Crafting wooden items and furniture",
        gain_rate: 0.9,
    },
    Skill {
        key: "alchemy",
        name: "Alchemy",
        category: SkillCategory::Crafting,
        description: "Brewing potions and elixirs",
        gain_rate: 0.85,
    },
    Skill {
        key: "cooking",
        name: "Cooking",
        category: SkillCategory::Crafting,
        description: "Preparing food for sustenance",
        gain_rate: 1.2,
    },
    // Gathering skills
    Skill {
        key: "mining",
        name: "Mining",
        category: SkillCategory::Gathering,
        description: "Extracting ore from rocks",
        gain_rate: 0.9,
    },
    Skill {
        key: "lumberjacking",
        name: "Lumberjacking",
        category: SkillCategory::Gathering,
        description: "Chopping wood from trees",
        gain_rate: 1.0,
    },
    Skill {
        key: "fishing",
        name: "Fishing",
        category: SkillCategory::Gathering,
        description: "Catching fish from water",
        gain_rate: 1.1,
    },
    Skill {
        key: "herbalism",
        name: "Herbalism",
        category: SkillCategory::Gathering,
        description: "Gathering herbs and reagents",
        gain_rate: 1.0,
    },
    // Miscellaneous skills
    Skill {
        key: "healing",
        name: "Healing",
        category: SkillCategory::Miscellaneous,
        description: "Using bandages to heal wounds",
        gain_rate: 1.0,
    },
    Skill {
        key: "animal_taming",
        name: "Animal Taming",
        category: SkillCategory::Miscellaneous,
        description: "Taming wild creatures",
        gain_rate: 0.7,
    },
    Skill {
        key: "hiding",
        name: "Hiding",
        category: SkillCategory::Miscellaneous,
        description: "Concealing yourself from others",
        gain_rate: 1.0,
    },
    Skill {
        key: "stealth",
        name: "Stealth",
        category: SkillCategory::Miscellaneous,
        description: "Moving while hidden",
        gain_rate: 0.8,
    },
];

pub fn get_skill(key: &str) -> Option<&'static Skill> {
    SKILLS.iter().find(|s| s.key == key)
}

// ============================================================================
// ITEMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Shield,
    Consumable,
    Resource,
    Tool,
    Furniture,
    Container,
    Reagent,
    Misc,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub key: &'static str,
    pub name: &'static str,
    pub item_type: ItemType,
    pub description: &'static str,
    pub base_price: i64,
    pub weight: u32,
    /// For weapons: damage. For armor: defense. For consumables: potency.
    pub power: i32,
    /// Required skill to use effectively
    pub required_skill: Option<&'static str>,
    /// Minimum skill level to use
    pub required_skill_level: u32,
    /// Can be stacked
    pub stackable: bool,
}

pub static ITEMS: &[Item] = &[
    // Weapons - Swords
    Item {
        key: "dagger",
        name: "Dagger",
        item_type: ItemType::Weapon,
        description: "A small but deadly blade",
        base_price: 50,
        weight: 1,
        power: 5,
        required_skill: Some("swordsmanship"),
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "short_sword",
        name: "Short Sword",
        item_type: ItemType::Weapon,
        description: "A basic one-handed sword",
        base_price: 150,
        weight: 3,
        power: 10,
        required_skill: Some("swordsmanship"),
        required_skill_level: 10,
        stackable: false,
    },
    Item {
        key: "long_sword",
        name: "Long Sword",
        item_type: ItemType::Weapon,
        description: "A versatile sword for warriors",
        base_price: 350,
        weight: 5,
        power: 18,
        required_skill: Some("swordsmanship"),
        required_skill_level: 30,
        stackable: false,
    },
    Item {
        key: "broadsword",
        name: "Broadsword",
        item_type: ItemType::Weapon,
        description: "A heavy, powerful blade",
        base_price: 600,
        weight: 7,
        power: 25,
        required_skill: Some("swordsmanship"),
        required_skill_level: 50,
        stackable: false,
    },
    Item {
        key: "claymore",
        name: "Claymore",
        item_type: ItemType::Weapon,
        description: "A massive two-handed sword",
        base_price: 1200,
        weight: 12,
        power: 40,
        required_skill: Some("swordsmanship"),
        required_skill_level: 70,
        stackable: false,
    },
    // Weapons - Maces
    Item {
        key: "club",
        name: "Club",
        item_type: ItemType::Weapon,
        description: "A simple wooden club",
        base_price: 20,
        weight: 3,
        power: 6,
        required_skill: Some("mace_fighting"),
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "mace",
        name: "Mace",
        item_type: ItemType::Weapon,
        description: "A metal mace",
        base_price: 200,
        weight: 5,
        power: 14,
        required_skill: Some("mace_fighting"),
        required_skill_level: 20,
        stackable: false,
    },
    Item {
        key: "war_hammer",
        name: "War Hammer",
        item_type: ItemType::Weapon,
        description: "A devastating two-handed hammer",
        base_price: 800,
        weight: 10,
        power: 35,
        required_skill: Some("mace_fighting"),
        required_skill_level: 60,
        stackable: false,
    },
    // Weapons - Bows
    Item {
        key: "bow",
        name: "Bow",
        item_type: ItemType::Weapon,
        description: "A simple wooden bow",
        base_price: 100,
        weight: 3,
        power: 8,
        required_skill: Some("archery"),
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "crossbow",
        name: "Crossbow",
        item_type: ItemType::Weapon,
        description: "A mechanical crossbow",
        base_price: 400,
        weight: 6,
        power: 20,
        required_skill: Some("archery"),
        required_skill_level: 40,
        stackable: false,
    },
    Item {
        key: "heavy_crossbow",
        name: "Heavy Crossbow",
        item_type: ItemType::Weapon,
        description: "A powerful siege crossbow",
        base_price: 900,
        weight: 10,
        power: 32,
        required_skill: Some("archery"),
        required_skill_level: 70,
        stackable: false,
    },
    Item {
        key: "arrow",
        name: "Arrow",
        item_type: ItemType::Resource,
        description: "Ammunition for bows",
        base_price: 2,
        weight: 0,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "bolt",
        name: "Bolt",
        item_type: ItemType::Resource,
        description: "Ammunition for crossbows",
        base_price: 3,
        weight: 0,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    // Armor
    Item {
        key: "cloth_tunic",
        name: "Cloth Tunic",
        item_type: ItemType::Armor,
        description: "Basic cloth armor",
        base_price: 30,
        weight: 1,
        power: 2,
        required_skill: None,
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "leather_armor",
        name: "Leather Armor",
        item_type: ItemType::Armor,
        description: "Light leather protection",
        base_price: 200,
        weight: 5,
        power: 8,
        required_skill: None,
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "chain_mail",
        name: "Chain Mail",
        item_type: ItemType::Armor,
        description: "Interlocking metal rings",
        base_price: 600,
        weight: 15,
        power: 18,
        required_skill: None,
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "plate_armor",
        name: "Plate Armor",
        item_type: ItemType::Armor,
        description: "Heavy metal plate protection",
        base_price: 1500,
        weight: 30,
        power: 35,
        required_skill: None,
        required_skill_level: 0,
        stackable: false,
    },
    // Shields
    Item {
        key: "wooden_shield",
        name: "Wooden Shield",
        item_type: ItemType::Shield,
        description: "A basic wooden shield",
        base_price: 50,
        weight: 4,
        power: 5,
        required_skill: Some("parrying"),
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "metal_shield",
        name: "Metal Shield",
        item_type: ItemType::Shield,
        description: "A sturdy metal shield",
        base_price: 250,
        weight: 8,
        power: 12,
        required_skill: Some("parrying"),
        required_skill_level: 30,
        stackable: false,
    },
    Item {
        key: "tower_shield",
        name: "Tower Shield",
        item_type: ItemType::Shield,
        description: "A massive protective shield",
        base_price: 600,
        weight: 15,
        power: 22,
        required_skill: Some("parrying"),
        required_skill_level: 60,
        stackable: false,
    },
    // Consumables
    Item {
        key: "lesser_heal_potion",
        name: "Lesser Heal Potion",
        item_type: ItemType::Consumable,
        description: "Restores a small amount of health",
        base_price: 25,
        weight: 1,
        power: 20,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "heal_potion",
        name: "Heal Potion",
        item_type: ItemType::Consumable,
        description: "Restores health",
        base_price: 75,
        weight: 1,
        power: 50,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "greater_heal_potion",
        name: "Greater Heal Potion",
        item_type: ItemType::Consumable,
        description: "Restores a large amount of health",
        base_price: 200,
        weight: 1,
        power: 100,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "mana_potion",
        name: "Mana Potion",
        item_type: ItemType::Consumable,
        description: "Restores mana",
        base_price: 100,
        weight: 1,
        power: 30,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "bandage",
        name: "Bandage",
        item_type: ItemType::Consumable,
        description: "Used with Healing skill to heal wounds",
        base_price: 5,
        weight: 0,
        power: 0,
        required_skill: Some("healing"),
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "cooked_meat",
        name: "Cooked Meat",
        item_type: ItemType::Consumable,
        description: "A hearty meal",
        base_price: 10,
        weight: 1,
        power: 15,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "bread",
        name: "Bread",
        item_type: ItemType::Consumable,
        description: "Simple but filling",
        base_price: 5,
        weight: 1,
        power: 10,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    // Resources
    Item {
        key: "iron_ore",
        name: "Iron Ore",
        item_type: ItemType::Resource,
        description: "Raw iron ore for smelting",
        base_price: 10,
        weight: 2,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "iron_ingot",
        name: "Iron Ingot",
        item_type: ItemType::Resource,
        description: "Refined iron for crafting",
        base_price: 25,
        weight: 1,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "gold_ore",
        name: "Gold Ore",
        item_type: ItemType::Resource,
        description: "Precious gold ore",
        base_price: 50,
        weight: 2,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "gold_ingot",
        name: "Gold Ingot",
        item_type: ItemType::Resource,
        description: "Refined gold",
        base_price: 150,
        weight: 1,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "wood",
        name: "Wood",
        item_type: ItemType::Resource,
        description: "Logs for crafting",
        base_price: 5,
        weight: 2,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "board",
        name: "Board",
        item_type: ItemType::Resource,
        description: "Cut lumber for construction",
        base_price: 10,
        weight: 1,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "leather",
        name: "Leather",
        item_type: ItemType::Resource,
        description: "Tanned animal hide",
        base_price: 15,
        weight: 1,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "cloth",
        name: "Cloth",
        item_type: ItemType::Resource,
        description: "Woven fabric",
        base_price: 8,
        weight: 1,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "raw_fish",
        name: "Raw Fish",
        item_type: ItemType::Resource,
        description: "Fresh caught fish",
        base_price: 3,
        weight: 1,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "raw_meat",
        name: "Raw Meat",
        item_type: ItemType::Resource,
        description: "Uncooked meat",
        base_price: 5,
        weight: 1,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    // Reagents for alchemy/magic
    Item {
        key: "ginseng",
        name: "Ginseng",
        item_type: ItemType::Reagent,
        description: "A healing herb",
        base_price: 8,
        weight: 0,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "garlic",
        name: "Garlic",
        item_type: ItemType::Reagent,
        description: "Protective herb",
        base_price: 6,
        weight: 0,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "mandrake_root",
        name: "Mandrake Root",
        item_type: ItemType::Reagent,
        description: "Powerful magical reagent",
        base_price: 25,
        weight: 0,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "nightshade",
        name: "Nightshade",
        item_type: ItemType::Reagent,
        description: "Toxic but useful in potions",
        base_price: 15,
        weight: 0,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    Item {
        key: "spider_silk",
        name: "Spider Silk",
        item_type: ItemType::Reagent,
        description: "Used in binding spells",
        base_price: 12,
        weight: 0,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
    // Tools
    Item {
        key: "pickaxe",
        name: "Pickaxe",
        item_type: ItemType::Tool,
        description: "For mining ore",
        base_price: 50,
        weight: 5,
        power: 0,
        required_skill: Some("mining"),
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "hatchet",
        name: "Hatchet",
        item_type: ItemType::Tool,
        description: "For chopping wood",
        base_price: 40,
        weight: 4,
        power: 0,
        required_skill: Some("lumberjacking"),
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "fishing_pole",
        name: "Fishing Pole",
        item_type: ItemType::Tool,
        description: "For catching fish",
        base_price: 30,
        weight: 3,
        power: 0,
        required_skill: Some("fishing"),
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "smithy_hammer",
        name: "Smith's Hammer",
        item_type: ItemType::Tool,
        description: "For blacksmithing",
        base_price: 60,
        weight: 4,
        power: 0,
        required_skill: Some("blacksmithing"),
        required_skill_level: 0,
        stackable: false,
    },
    Item {
        key: "sewing_kit",
        name: "Sewing Kit",
        item_type: ItemType::Tool,
        description: "For tailoring",
        base_price: 35,
        weight: 1,
        power: 0,
        required_skill: Some("tailoring"),
        required_skill_level: 0,
        stackable: false,
    },
    // Gold (currency)
    Item {
        key: "gold_coin",
        name: "Gold Coin",
        item_type: ItemType::Misc,
        description: "Standard currency",
        base_price: 1,
        weight: 0,
        power: 0,
        required_skill: None,
        required_skill_level: 0,
        stackable: true,
    },
];

pub fn get_item(key: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|i| i.key == key)
}

// ============================================================================
// MONSTERS
// ============================================================================

#[derive(Debug, Clone)]
pub struct MonsterTemplate {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub ascii_art: &'static str,
    pub min_level: u32,
    pub max_level: u32,
    pub base_hp: i32,
    pub base_damage: i32,
    pub base_defense: i32,
    pub gold_drop_min: i64,
    pub gold_drop_max: i64,
    pub xp_reward: i64,
    /// Items that can drop (key, chance percentage)
    pub loot_table: &'static [(&'static str, u32)],
    /// Zones where this monster spawns
    pub spawn_zones: &'static [&'static str],
    /// Is this an aggressive monster?
    pub aggressive: bool,
}

/// Runtime monster instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub template_key: String,
    pub name: String,
    pub level: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub damage: i32,
    pub defense: i32,
    pub position: (i32, i32),
}

pub static MONSTERS: &[MonsterTemplate] = &[
    // Low level monsters (wilderness)
    MonsterTemplate {
        key: "rabbit",
        name: "Rabbit",
        description: "A harmless woodland creature",
        ascii_art: "(\\(\\",
        min_level: 1,
        max_level: 2,
        base_hp: 5,
        base_damage: 1,
        base_defense: 0,
        gold_drop_min: 0,
        gold_drop_max: 2,
        xp_reward: 5,
        loot_table: &[("raw_meat", 50)],
        spawn_zones: &["wilderness", "forest"],
        aggressive: false,
    },
    MonsterTemplate {
        key: "rat",
        name: "Giant Rat",
        description: "A large, diseased rodent",
        ascii_art: "~:3",
        min_level: 1,
        max_level: 3,
        base_hp: 10,
        base_damage: 3,
        base_defense: 1,
        gold_drop_min: 1,
        gold_drop_max: 5,
        xp_reward: 10,
        loot_table: &[("raw_meat", 30)],
        spawn_zones: &["wilderness", "dungeon"],
        aggressive: true,
    },
    MonsterTemplate {
        key: "mongbat",
        name: "Mongbat",
        description: "A winged nuisance",
        ascii_art: "/V\\",
        min_level: 2,
        max_level: 4,
        base_hp: 15,
        base_damage: 5,
        base_defense: 2,
        gold_drop_min: 3,
        gold_drop_max: 12,
        xp_reward: 20,
        loot_table: &[("leather", 20)],
        spawn_zones: &["wilderness", "dungeon"],
        aggressive: true,
    },
    MonsterTemplate {
        key: "skeleton",
        name: "Skeleton",
        description: "The animated bones of the dead",
        ascii_art: "_|_",
        min_level: 3,
        max_level: 6,
        base_hp: 25,
        base_damage: 8,
        base_defense: 5,
        gold_drop_min: 10,
        gold_drop_max: 30,
        xp_reward: 40,
        loot_table: &[("dagger", 10), ("gold_coin", 50)],
        spawn_zones: &["dungeon", "graveyard"],
        aggressive: true,
    },
    MonsterTemplate {
        key: "zombie",
        name: "Zombie",
        description: "A shambling corpse",
        ascii_art: "ZzZ",
        min_level: 4,
        max_level: 7,
        base_hp: 40,
        base_damage: 10,
        base_defense: 3,
        gold_drop_min: 5,
        gold_drop_max: 20,
        xp_reward: 50,
        loot_table: &[("bandage", 25), ("cloth", 15)],
        spawn_zones: &["dungeon", "graveyard"],
        aggressive: true,
    },
    MonsterTemplate {
        key: "orc",
        name: "Orc",
        description: "A savage humanoid warrior",
        ascii_art: "O>-",
        min_level: 5,
        max_level: 9,
        base_hp: 50,
        base_damage: 15,
        base_defense: 8,
        gold_drop_min: 20,
        gold_drop_max: 60,
        xp_reward: 80,
        loot_table: &[("short_sword", 10), ("leather_armor", 5), ("gold_coin", 60)],
        spawn_zones: &["wilderness", "orc_camp"],
        aggressive: true,
    },
    MonsterTemplate {
        key: "ettin",
        name: "Ettin",
        description: "A two-headed giant",
        ascii_art: "o'o",
        min_level: 8,
        max_level: 12,
        base_hp: 100,
        base_damage: 25,
        base_defense: 12,
        gold_drop_min: 50,
        gold_drop_max: 150,
        xp_reward: 150,
        loot_table: &[("club", 20), ("gold_coin", 80)],
        spawn_zones: &["wilderness", "mountains"],
        aggressive: true,
    },
    MonsterTemplate {
        key: "troll",
        name: "Troll",
        description: "A regenerating brute",
        ascii_art: "TRL",
        min_level: 10,
        max_level: 15,
        base_hp: 150,
        base_damage: 30,
        base_defense: 15,
        gold_drop_min: 80,
        gold_drop_max: 250,
        xp_reward: 250,
        loot_table: &[("mace", 10), ("gold_coin", 70)],
        spawn_zones: &["swamp", "dungeon"],
        aggressive: true,
    },
    MonsterTemplate {
        key: "lich",
        name: "Lich",
        description: "An undead sorcerer of great power",
        ascii_art: "~~@~~",
        min_level: 15,
        max_level: 20,
        base_hp: 200,
        base_damage: 40,
        base_defense: 20,
        gold_drop_min: 200,
        gold_drop_max: 500,
        xp_reward: 500,
        loot_table: &[("mandrake_root", 40), ("nightshade", 40), ("gold_coin", 90)],
        spawn_zones: &["dungeon_deep"],
        aggressive: true,
    },
    MonsterTemplate {
        key: "dragon",
        name: "Dragon",
        description: "The mightiest of all creatures",
        ascii_art: "<:=D~",
        min_level: 20,
        max_level: 30,
        base_hp: 500,
        base_damage: 80,
        base_defense: 40,
        gold_drop_min: 500,
        gold_drop_max: 2000,
        xp_reward: 2000,
        loot_table: &[("gold_ingot", 30), ("claymore", 5)],
        spawn_zones: &["dragon_lair"],
        aggressive: true,
    },
];

pub fn get_monster_template(key: &str) -> Option<&'static MonsterTemplate> {
    MONSTERS.iter().find(|m| m.key == key)
}

// ============================================================================
// ZONES (Maps)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    Town,       // Safe zone, no combat
    Wilderness, // Open world, PvE
    Dungeon,    // Dangerous PvE
    PvP,        // Player vs Player enabled
    Housing,    // Player housing area
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Grass,
    Water,
    Mountain,
    Forest,
    Sand,
    Stone,
    Road,
    Building,
    Wall,
    Door,
}

impl TerrainType {
    pub fn char(&self) -> char {
        match self {
            TerrainType::Grass => '.',
            TerrainType::Water => '~',
            TerrainType::Mountain => '^',
            TerrainType::Forest => 'T',
            TerrainType::Sand => ':',
            TerrainType::Stone => '#',
            TerrainType::Road => '=',
            TerrainType::Building => '+',
            TerrainType::Wall => '#',
            TerrainType::Door => 'D',
        }
    }

    pub fn passable(&self) -> bool {
        !matches!(self, TerrainType::Water | TerrainType::Mountain | TerrainType::Wall)
    }
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub zone_type: ZoneType,
    pub width: u32,
    pub height: u32,
    /// Minimum recommended level
    pub min_level: u32,
    /// Connected zones (key, x, y for entrance)
    pub exits: &'static [(&'static str, i32, i32)],
}

pub static ZONES: &[Zone] = &[
    Zone {
        key: "britain",
        name: "City of Britain",
        description: "The capital city of the realm, a bustling hub of trade and adventure",
        zone_type: ZoneType::Town,
        width: 60,
        height: 40,
        min_level: 1,
        exits: &[("britain_outskirts", 30, 39)],
    },
    Zone {
        key: "britain_outskirts",
        name: "Britain Outskirts",
        description: "The wilderness surrounding Britain, home to small creatures",
        zone_type: ZoneType::Wilderness,
        width: 80,
        height: 60,
        min_level: 1,
        exits: &[
            ("britain", 40, 0),
            ("britain_forest", 79, 30),
            ("britain_graveyard", 40, 59),
        ],
    },
    Zone {
        key: "britain_forest",
        name: "Britannian Forest",
        description: "A dense forest filled with wildlife and bandits",
        zone_type: ZoneType::Wilderness,
        width: 100,
        height: 80,
        min_level: 5,
        exits: &[("britain_outskirts", 0, 40), ("dungeon_despise", 80, 60)],
    },
    Zone {
        key: "britain_graveyard",
        name: "Britain Graveyard",
        description: "A haunted cemetery where the dead do not rest",
        zone_type: ZoneType::Wilderness,
        width: 40,
        height: 40,
        min_level: 3,
        exits: &[("britain_outskirts", 20, 0)],
    },
    Zone {
        key: "dungeon_despise",
        name: "Dungeon Despise",
        description: "A deep dungeon filled with orcs and trolls",
        zone_type: ZoneType::Dungeon,
        width: 60,
        height: 80,
        min_level: 8,
        exits: &[("britain_forest", 30, 0), ("dungeon_despise_deep", 30, 79)],
    },
    Zone {
        key: "dungeon_despise_deep",
        name: "Despise - Deep Level",
        description: "The darkest depths of Despise, home to powerful undead",
        zone_type: ZoneType::Dungeon,
        width: 50,
        height: 50,
        min_level: 15,
        exits: &[("dungeon_despise", 25, 0)],
    },
    Zone {
        key: "housing_district",
        name: "Housing District",
        description: "Land available for player housing",
        zone_type: ZoneType::Housing,
        width: 100,
        height: 100,
        min_level: 1,
        exits: &[("britain", 50, 0)],
    },
    Zone {
        key: "arena",
        name: "The Arena",
        description: "A gladiatorial arena where warriors test their skills against each other",
        zone_type: ZoneType::PvP,
        width: 30,
        height: 30,
        min_level: 10,
        exits: &[("britain", 15, 0)],
    },
];

pub fn get_zone(key: &str) -> Option<&'static Zone> {
    ZONES.iter().find(|z| z.key == key)
}

// ============================================================================
// NPCS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcType {
    Merchant,
    Trainer,
    QuestGiver,
    Banker,
    Innkeeper,
    Guard,
    Healer,
}

#[derive(Debug, Clone)]
pub struct Npc {
    pub key: &'static str,
    pub name: &'static str,
    pub npc_type: NpcType,
    pub zone: &'static str,
    pub position: (i32, i32),
    pub dialogue: &'static str,
    /// For merchants: items sold (key, price_multiplier)
    pub shop_inventory: &'static [(&'static str, f32)],
    /// For trainers: skills trained
    pub trains_skills: &'static [&'static str],
}

pub static NPCS: &[Npc] = &[
    // Britain NPCs
    Npc {
        key: "britain_weaponsmith",
        name: "Edgar the Weaponsmith",
        npc_type: NpcType::Merchant,
        zone: "britain",
        position: (15, 10),
        dialogue: "Welcome to my shop! I have the finest weapons in all of Britain.",
        shop_inventory: &[
            ("dagger", 1.0),
            ("short_sword", 1.0),
            ("long_sword", 1.0),
            ("broadsword", 1.0),
            ("claymore", 1.2),
            ("club", 1.0),
            ("mace", 1.0),
            ("war_hammer", 1.1),
            ("bow", 1.0),
            ("crossbow", 1.0),
            ("arrow", 1.0),
            ("bolt", 1.0),
        ],
        trains_skills: &[],
    },
    Npc {
        key: "britain_armorsmith",
        name: "Helena the Armorsmith",
        npc_type: NpcType::Merchant,
        zone: "britain",
        position: (20, 10),
        dialogue: "Protection for the adventurous soul. What do you need?",
        shop_inventory: &[
            ("cloth_tunic", 1.0),
            ("leather_armor", 1.0),
            ("chain_mail", 1.0),
            ("plate_armor", 1.2),
            ("wooden_shield", 1.0),
            ("metal_shield", 1.0),
            ("tower_shield", 1.1),
        ],
        trains_skills: &[],
    },
    Npc {
        key: "britain_healer",
        name: "Brother Marcus",
        npc_type: NpcType::Healer,
        zone: "britain",
        position: (30, 15),
        dialogue: "May the virtues guide you. I can heal your wounds.",
        shop_inventory: &[
            ("lesser_heal_potion", 1.0),
            ("heal_potion", 1.0),
            ("greater_heal_potion", 1.1),
            ("mana_potion", 1.0),
            ("bandage", 1.0),
        ],
        trains_skills: &[],
    },
    Npc {
        key: "britain_mage",
        name: "Mariah the Mage",
        npc_type: NpcType::Trainer,
        zone: "britain",
        position: (40, 20),
        dialogue:
            "The arcane arts are not for the faint of heart. Are you ready to learn?",
        shop_inventory: &[
            ("ginseng", 1.0),
            ("garlic", 1.0),
            ("mandrake_root", 1.0),
            ("nightshade", 1.0),
            ("spider_silk", 1.0),
        ],
        trains_skills: &["magery", "meditation", "eval_int"],
    },
    Npc {
        key: "britain_warrior_trainer",
        name: "Sir Geoffrey",
        npc_type: NpcType::Trainer,
        zone: "britain",
        position: (25, 25),
        dialogue: "A warrior must be strong in body and mind. Train with me!",
        shop_inventory: &[],
        trains_skills: &["swordsmanship", "mace_fighting", "tactics", "parrying"],
    },
    Npc {
        key: "britain_banker",
        name: "Lord British's Banker",
        npc_type: NpcType::Banker,
        zone: "britain",
        position: (35, 12),
        dialogue: "Your gold is safe with the Bank of Britain.",
        shop_inventory: &[],
        trains_skills: &[],
    },
    Npc {
        key: "britain_innkeeper",
        name: "Thomas the Innkeeper",
        npc_type: NpcType::Innkeeper,
        zone: "britain",
        position: (45, 30),
        dialogue: "Rest weary traveler. A room costs 50 gold per night.",
        shop_inventory: &[("bread", 1.0), ("cooked_meat", 1.0)],
        trains_skills: &[],
    },
    Npc {
        key: "britain_guard",
        name: "Guard",
        npc_type: NpcType::Guard,
        zone: "britain",
        position: (10, 5),
        dialogue: "Move along, citizen.",
        shop_inventory: &[],
        trains_skills: &[],
    },
    Npc {
        key: "britain_craftsman",
        name: "Robert the Craftsman",
        npc_type: NpcType::Trainer,
        zone: "britain",
        position: (50, 15),
        dialogue: "Crafting is an honorable profession. Let me teach you.",
        shop_inventory: &[
            ("pickaxe", 1.0),
            ("hatchet", 1.0),
            ("fishing_pole", 1.0),
            ("smithy_hammer", 1.0),
            ("sewing_kit", 1.0),
        ],
        trains_skills: &["blacksmithing", "tailoring", "carpentry", "mining", "lumberjacking"],
    },
    // Quest giver
    Npc {
        key: "britain_adventurer_guild",
        name: "Guildmaster Roland",
        npc_type: NpcType::QuestGiver,
        zone: "britain",
        position: (28, 8),
        dialogue: "Welcome, adventurer! The guild has tasks for brave souls.",
        shop_inventory: &[],
        trains_skills: &[],
    },
];

pub fn get_npc(key: &str) -> Option<&'static Npc> {
    NPCS.iter().find(|n| n.key == key)
}

pub fn get_npcs_in_zone(zone: &str) -> Vec<&'static Npc> {
    NPCS.iter().filter(|n| n.zone == zone).collect()
}

// ============================================================================
// QUESTS
// ============================================================================

#[derive(Debug, Clone)]
pub struct QuestRequirement {
    /// Kill X monsters of type
    pub kill_monsters: Option<(&'static str, u32)>,
    /// Collect X items
    pub collect_items: Option<(&'static str, u32)>,
    /// Visit a location
    pub visit_zone: Option<&'static str>,
    /// Reach skill level
    pub skill_level: Option<(&'static str, u32)>,
}

#[derive(Debug, Clone)]
pub struct QuestReward {
    pub gold: i64,
    pub xp: i64,
    pub items: &'static [(&'static str, u32)],
}

#[derive(Debug, Clone)]
pub struct Quest {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub giver_npc: &'static str,
    pub min_level: u32,
    pub requirements: QuestRequirement,
    pub reward: QuestReward,
    /// Quest that must be completed first
    pub prerequisite: Option<&'static str>,
}

pub static QUESTS: &[Quest] = &[
    Quest {
        key: "rats_in_cellar",
        name: "Rats in the Cellar",
        description: "Clear out the giant rats infesting the Britain cellars",
        giver_npc: "britain_adventurer_guild",
        min_level: 1,
        requirements: QuestRequirement {
            kill_monsters: Some(("rat", 5)),
            collect_items: None,
            visit_zone: None,
            skill_level: None,
        },
        reward: QuestReward {
            gold: 100,
            xp: 50,
            items: &[("lesser_heal_potion", 3)],
        },
        prerequisite: None,
    },
    Quest {
        key: "mongbat_menace",
        name: "The Mongbat Menace",
        description: "Mongbats are terrorizing travelers. Eliminate 10 of them.",
        giver_npc: "britain_adventurer_guild",
        min_level: 3,
        requirements: QuestRequirement {
            kill_monsters: Some(("mongbat", 10)),
            collect_items: None,
            visit_zone: None,
            skill_level: None,
        },
        reward: QuestReward {
            gold: 250,
            xp: 150,
            items: &[("leather_armor", 1)],
        },
        prerequisite: Some("rats_in_cellar"),
    },
    Quest {
        key: "orc_threat",
        name: "The Orc Threat",
        description: "Orcs have been raiding caravans. Defeat 15 orcs.",
        giver_npc: "britain_adventurer_guild",
        min_level: 6,
        requirements: QuestRequirement {
            kill_monsters: Some(("orc", 15)),
            collect_items: None,
            visit_zone: None,
            skill_level: None,
        },
        reward: QuestReward {
            gold: 500,
            xp: 400,
            items: &[("long_sword", 1)],
        },
        prerequisite: Some("mongbat_menace"),
    },
    Quest {
        key: "explore_despise",
        name: "Explore Dungeon Despise",
        description: "Scout the entrance to Dungeon Despise and report back.",
        giver_npc: "britain_adventurer_guild",
        min_level: 8,
        requirements: QuestRequirement {
            kill_monsters: None,
            collect_items: None,
            visit_zone: Some("dungeon_despise"),
            skill_level: None,
        },
        reward: QuestReward {
            gold: 300,
            xp: 300,
            items: &[("heal_potion", 5)],
        },
        prerequisite: Some("orc_threat"),
    },
    Quest {
        key: "troll_hunter",
        name: "Troll Hunter",
        description: "Trolls have become a serious threat. Kill 5 trolls.",
        giver_npc: "britain_adventurer_guild",
        min_level: 12,
        requirements: QuestRequirement {
            kill_monsters: Some(("troll", 5)),
            collect_items: None,
            visit_zone: None,
            skill_level: None,
        },
        reward: QuestReward {
            gold: 1000,
            xp: 800,
            items: &[("chain_mail", 1)],
        },
        prerequisite: Some("explore_despise"),
    },
    Quest {
        key: "lich_lord",
        name: "The Lich Lord",
        description: "A powerful lich threatens the realm. Destroy it!",
        giver_npc: "britain_adventurer_guild",
        min_level: 18,
        requirements: QuestRequirement {
            kill_monsters: Some(("lich", 1)),
            collect_items: None,
            visit_zone: None,
            skill_level: None,
        },
        reward: QuestReward {
            gold: 2000,
            xp: 1500,
            items: &[("greater_heal_potion", 10)],
        },
        prerequisite: Some("troll_hunter"),
    },
    // Gathering quest
    Quest {
        key: "iron_for_smith",
        name: "Iron for the Smith",
        description: "Edgar needs iron ore for his forge. Bring him 20 iron ore.",
        giver_npc: "britain_weaponsmith",
        min_level: 1,
        requirements: QuestRequirement {
            kill_monsters: None,
            collect_items: Some(("iron_ore", 20)),
            visit_zone: None,
            skill_level: None,
        },
        reward: QuestReward {
            gold: 200,
            xp: 100,
            items: &[("short_sword", 1)],
        },
        prerequisite: None,
    },
    // Skill quest
    Quest {
        key: "apprentice_mage",
        name: "Apprentice Mage",
        description: "Prove your magical aptitude by reaching 30 Magery skill.",
        giver_npc: "britain_mage",
        min_level: 1,
        requirements: QuestRequirement {
            kill_monsters: None,
            collect_items: None,
            visit_zone: None,
            skill_level: Some(("magery", 30)),
        },
        reward: QuestReward {
            gold: 300,
            xp: 200,
            items: &[("mana_potion", 5)],
        },
        prerequisite: None,
    },
];

pub fn get_quest(key: &str) -> Option<&'static Quest> {
    QUESTS.iter().find(|q| q.key == key)
}

// ============================================================================
// SPELLS
// ============================================================================

#[derive(Debug, Clone)]
pub struct Spell {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub mana_cost: i32,
    pub required_magery: u32,
    /// Damage/healing amount
    pub power: i32,
    /// Required reagents (key, amount)
    pub reagents: &'static [(&'static str, u32)],
}

pub static SPELLS: &[Spell] = &[
    Spell {
        key: "magic_arrow",
        name: "Magic Arrow",
        description: "A basic offensive spell",
        mana_cost: 4,
        required_magery: 0,
        power: 10,
        reagents: &[("spider_silk", 1)],
    },
    Spell {
        key: "heal",
        name: "Heal",
        description: "Heals minor wounds",
        mana_cost: 6,
        required_magery: 10,
        power: 20,
        reagents: &[("ginseng", 1), ("garlic", 1)],
    },
    Spell {
        key: "fireball",
        name: "Fireball",
        description: "A powerful ball of fire",
        mana_cost: 15,
        required_magery: 40,
        power: 40,
        reagents: &[("spider_silk", 1), ("nightshade", 1)],
    },
    Spell {
        key: "greater_heal",
        name: "Greater Heal",
        description: "Heals significant wounds",
        mana_cost: 20,
        required_magery: 50,
        power: 60,
        reagents: &[("ginseng", 2), ("garlic", 1), ("mandrake_root", 1)],
    },
    Spell {
        key: "lightning",
        name: "Lightning",
        description: "Strike with electrical energy",
        mana_cost: 25,
        required_magery: 60,
        power: 55,
        reagents: &[("mandrake_root", 1), ("spider_silk", 1)],
    },
    Spell {
        key: "energy_bolt",
        name: "Energy Bolt",
        description: "A devastating bolt of pure energy",
        mana_cost: 40,
        required_magery: 80,
        power: 80,
        reagents: &[("nightshade", 1), ("mandrake_root", 2)],
    },
];

pub fn get_spell(key: &str) -> Option<&'static Spell> {
    SPELLS.iter().find(|s| s.key == key)
}

// ============================================================================
// RESOURCE NODES
// ============================================================================

#[derive(Debug, Clone)]
pub struct ResourceNode {
    pub key: &'static str,
    pub name: &'static str,
    pub required_skill: &'static str,
    pub min_skill: u32,
    /// Items that can be gathered (key, chance, min, max)
    pub yields: &'static [(&'static str, u32, u32, u32)],
    pub zones: &'static [&'static str],
}

pub static RESOURCE_NODES: &[ResourceNode] = &[
    ResourceNode {
        key: "iron_vein",
        name: "Iron Vein",
        required_skill: "mining",
        min_skill: 0,
        yields: &[("iron_ore", 100, 1, 3)],
        zones: &["britain_outskirts", "dungeon_despise"],
    },
    ResourceNode {
        key: "gold_vein",
        name: "Gold Vein",
        required_skill: "mining",
        min_skill: 50,
        yields: &[("gold_ore", 100, 1, 2)],
        zones: &["dungeon_despise", "dungeon_despise_deep"],
    },
    ResourceNode {
        key: "tree",
        name: "Tree",
        required_skill: "lumberjacking",
        min_skill: 0,
        yields: &[("wood", 100, 2, 5)],
        zones: &["britain_outskirts", "britain_forest"],
    },
    ResourceNode {
        key: "fishing_spot",
        name: "Fishing Spot",
        required_skill: "fishing",
        min_skill: 0,
        yields: &[("raw_fish", 80, 1, 2)],
        zones: &["britain_outskirts"],
    },
    ResourceNode {
        key: "herb_patch",
        name: "Herb Patch",
        required_skill: "herbalism",
        min_skill: 0,
        yields: &[
            ("ginseng", 40, 1, 2),
            ("garlic", 40, 1, 2),
            ("nightshade", 20, 1, 1),
        ],
        zones: &["britain_outskirts", "britain_forest"],
    },
];

pub fn get_resource_node(key: &str) -> Option<&'static ResourceNode> {
    RESOURCE_NODES.iter().find(|r| r.key == key)
}
