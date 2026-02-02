//! Static game data for Realm of Ralnar VGA
//!
//! Items, monsters, spells, and other game constants.

use serde::{Deserialize, Serialize};

/// Item types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Shield,
    Accessory,
    Consumable,
    KeyItem,
}

/// Item definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: &'static str,
    pub name: &'static str,
    pub item_type: ItemType,
    pub price: u32,
    pub stat_bonus: i16,
    pub effect: Option<&'static str>,
}

/// Monster definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub id: &'static str,
    pub name: &'static str,
    pub sprite: &'static str,
    pub hp: u16,
    pub attack: u8,
    pub defense: u8,
    pub exp: u32,
    pub gold: u32,
}

/// Spell definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub id: &'static str,
    pub name: &'static str,
    pub mp_cost: u8,
    pub power: u16,
    pub target: SpellTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellTarget {
    Self_,
    SingleEnemy,
    AllEnemies,
    SingleAlly,
    AllAllies,
}

// ============================================================================
// ITEMS DATABASE
// ============================================================================

pub const ITEMS: &[Item] = &[
    // Weapons
    Item {
        id: "dagger",
        name: "Dagger",
        item_type: ItemType::Weapon,
        price: 50,
        stat_bonus: 5,
        effect: None,
    },
    Item {
        id: "short_sword",
        name: "Short Sword",
        item_type: ItemType::Weapon,
        price: 150,
        stat_bonus: 10,
        effect: None,
    },
    Item {
        id: "long_sword",
        name: "Long Sword",
        item_type: ItemType::Weapon,
        price: 400,
        stat_bonus: 18,
        effect: None,
    },
    Item {
        id: "steel_sword",
        name: "Steel Sword",
        item_type: ItemType::Weapon,
        price: 1000,
        stat_bonus: 28,
        effect: None,
    },
    Item {
        id: "flame_sword",
        name: "Flame Sword",
        item_type: ItemType::Weapon,
        price: 5000,
        stat_bonus: 45,
        effect: Some("fire_damage"),
    },
    // Armor
    Item {
        id: "cloth_armor",
        name: "Cloth Armor",
        item_type: ItemType::Armor,
        price: 30,
        stat_bonus: 3,
        effect: None,
    },
    Item {
        id: "leather_armor",
        name: "Leather Armor",
        item_type: ItemType::Armor,
        price: 100,
        stat_bonus: 8,
        effect: None,
    },
    Item {
        id: "chain_mail",
        name: "Chain Mail",
        item_type: ItemType::Armor,
        price: 350,
        stat_bonus: 15,
        effect: None,
    },
    Item {
        id: "plate_armor",
        name: "Plate Armor",
        item_type: ItemType::Armor,
        price: 1200,
        stat_bonus: 25,
        effect: None,
    },
    // Shields
    Item {
        id: "wood_shield",
        name: "Wood Shield",
        item_type: ItemType::Shield,
        price: 40,
        stat_bonus: 2,
        effect: None,
    },
    Item {
        id: "iron_shield",
        name: "Iron Shield",
        item_type: ItemType::Shield,
        price: 200,
        stat_bonus: 6,
        effect: None,
    },
    Item {
        id: "steel_shield",
        name: "Steel Shield",
        item_type: ItemType::Shield,
        price: 800,
        stat_bonus: 12,
        effect: None,
    },
    // Consumables
    Item {
        id: "potion",
        name: "Potion",
        item_type: ItemType::Consumable,
        price: 30,
        stat_bonus: 0,
        effect: Some("heal_50"),
    },
    Item {
        id: "hi_potion",
        name: "Hi-Potion",
        item_type: ItemType::Consumable,
        price: 100,
        stat_bonus: 0,
        effect: Some("heal_150"),
    },
    Item {
        id: "ether",
        name: "Ether",
        item_type: ItemType::Consumable,
        price: 150,
        stat_bonus: 0,
        effect: Some("restore_mp_30"),
    },
    Item {
        id: "antidote",
        name: "Antidote",
        item_type: ItemType::Consumable,
        price: 50,
        stat_bonus: 0,
        effect: Some("cure_poison"),
    },
    Item {
        id: "tent",
        name: "Tent",
        item_type: ItemType::Consumable,
        price: 200,
        stat_bonus: 0,
        effect: Some("rest_field"),
    },
];

// ============================================================================
// MONSTERS DATABASE
// ============================================================================

pub const MONSTERS: &[Monster] = &[
    Monster {
        id: "slime",
        name: "Slime",
        sprite: "slime",
        hp: 15,
        attack: 4,
        defense: 2,
        exp: 5,
        gold: 3,
    },
    Monster {
        id: "spider",
        name: "Spider",
        sprite: "spider",
        hp: 20,
        attack: 6,
        defense: 3,
        exp: 8,
        gold: 5,
    },
    Monster {
        id: "bat",
        name: "Bat",
        sprite: "bbat",
        hp: 12,
        attack: 5,
        defense: 1,
        exp: 4,
        gold: 2,
    },
    Monster {
        id: "skeleton",
        name: "Skeleton",
        sprite: "skeleton",
        hp: 35,
        attack: 12,
        defense: 8,
        exp: 20,
        gold: 15,
    },
    Monster {
        id: "goblin",
        name: "Goblin",
        sprite: "goblin",
        hp: 25,
        attack: 8,
        defense: 4,
        exp: 12,
        gold: 10,
    },
    Monster {
        id: "knight",
        name: "Dark Knight",
        sprite: "knight",
        hp: 60,
        attack: 20,
        defense: 15,
        exp: 50,
        gold: 40,
    },
    Monster {
        id: "wizard",
        name: "Evil Wizard",
        sprite: "wizard",
        hp: 40,
        attack: 25,
        defense: 8,
        exp: 45,
        gold: 35,
    },
    Monster {
        id: "gspider",
        name: "Giant Spider",
        sprite: "gspider",
        hp: 80,
        attack: 22,
        defense: 12,
        exp: 60,
        gold: 50,
    },
    Monster {
        id: "fallen_armor",
        name: "Fallen Armor",
        sprite: "f_armor",
        hp: 100,
        attack: 30,
        defense: 25,
        exp: 80,
        gold: 60,
    },
    Monster {
        id: "fire_knight",
        name: "Fire Knight",
        sprite: "fknight",
        hp: 120,
        attack: 35,
        defense: 20,
        exp: 100,
        gold: 75,
    },
];

// ============================================================================
// SPELLS DATABASE
// ============================================================================

pub const SPELLS: &[Spell] = &[
    Spell {
        id: "heal",
        name: "Heal",
        mp_cost: 4,
        power: 30,
        target: SpellTarget::SingleAlly,
    },
    Spell {
        id: "healall",
        name: "Heal All",
        mp_cost: 12,
        power: 30,
        target: SpellTarget::AllAllies,
    },
    Spell {
        id: "fire",
        name: "Fire",
        mp_cost: 5,
        power: 25,
        target: SpellTarget::SingleEnemy,
    },
    Spell {
        id: "ice",
        name: "Ice",
        mp_cost: 5,
        power: 25,
        target: SpellTarget::SingleEnemy,
    },
    Spell {
        id: "bolt",
        name: "Bolt",
        mp_cost: 5,
        power: 25,
        target: SpellTarget::SingleEnemy,
    },
    Spell {
        id: "fire2",
        name: "Fire 2",
        mp_cost: 15,
        power: 60,
        target: SpellTarget::AllEnemies,
    },
    Spell {
        id: "cure",
        name: "Cure",
        mp_cost: 3,
        power: 0,
        target: SpellTarget::SingleAlly,
    },
    Spell {
        id: "protect",
        name: "Protect",
        mp_cost: 8,
        power: 20,
        target: SpellTarget::SingleAlly,
    },
];

// ============================================================================
// LOOKUP FUNCTIONS
// ============================================================================

pub fn get_item(id: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|i| i.id == id)
}

pub fn get_monster(id: &str) -> Option<&'static Monster> {
    MONSTERS.iter().find(|m| m.id == id)
}

pub fn get_spell(id: &str) -> Option<&'static Spell> {
    SPELLS.iter().find(|s| s.id == id)
}

/// Calculate experience needed for next level
pub fn exp_for_level(level: u8) -> u32 {
    // Classic exponential curve
    ((level as u32).pow(2) * 100) + ((level as u32) * 50)
}

/// Calculate damage
pub fn calc_damage(attack: u8, defense: u8, power: u16) -> u16 {
    let base = attack as u16 + power;
    let reduction = defense as u16 / 2;
    base.saturating_sub(reduction).max(1)
}
