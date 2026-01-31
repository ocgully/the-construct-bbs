//! Item definitions for Realm of Ralnar
//!
//! Contains all consumables, weapons, armor, accessories, and key items.

use super::config::CharacterClass;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Types of items in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Consumable,
    Weapon,
    Armor,
    Accessory,
    KeyItem,
}

/// Elemental types for weapons and spells
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Status effects that can be inflicted or cured
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffect {
    Poison,
    Stone,
    Dead,
    Unconscious,
    Confused,
    Asleep,
    Blind,
    Silence,
    Slow,
    Haste,
    Regen,
    Protect,
    Shell,
}

/// Effect an item has when used or equipped
#[derive(Debug, Clone)]
pub enum ItemEffect {
    /// Restores HP
    HealHP(i32),
    /// Restores MP
    HealMP(i32),
    /// Restores both HP and MP
    HealBoth { hp: i32, mp: i32 },
    /// Fully restores HP and MP
    FullRestore,
    /// Cures a status effect
    CureStatus(StatusEffect),
    /// Cures all negative status effects
    CureAll,
    /// Revives from death with HP percentage
    Revive(f32),
    /// Weapon stats (attack power, optional element)
    WeaponStats { attack: i32, element: Element },
    /// Armor stats (defense, magic defense)
    ArmorStats { defense: i32, magic_def: i32 },
    /// Accessory stats (various bonuses)
    AccessoryStats {
        hp_bonus: i32,
        mp_bonus: i32,
        stat_bonus: i32,
        element_resist: Option<Element>,
    },
    /// Temporary stat boost in battle
    StatBoost { stat: &'static str, amount: i32, duration: u8 },
    /// Deals damage to enemies
    Damage { power: i32, element: Element },
    /// No direct effect (key items, story items)
    None,
}

/// Full item definition
#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub item_type: ItemType,
    pub cost: u32,
    pub effect: ItemEffect,
    /// Which classes can equip (empty = all can use/equip)
    pub usable_by: &'static [CharacterClass],
}

// ============================================================================
// CONSUMABLE ITEMS
// ============================================================================

pub const POTION: ItemDef = ItemDef {
    id: "potion",
    name: "Potion",
    description: "Restores 100 HP",
    item_type: ItemType::Consumable,
    cost: 50,
    effect: ItemEffect::HealHP(100),
    usable_by: &[],
};

pub const HI_POTION: ItemDef = ItemDef {
    id: "hi_potion",
    name: "Hi-Potion",
    description: "Restores 300 HP",
    item_type: ItemType::Consumable,
    cost: 200,
    effect: ItemEffect::HealHP(300),
    usable_by: &[],
};

pub const MEGA_POTION: ItemDef = ItemDef {
    id: "mega_potion",
    name: "Mega Potion",
    description: "Restores 700 HP",
    item_type: ItemType::Consumable,
    cost: 500,
    effect: ItemEffect::HealHP(700),
    usable_by: &[],
};

pub const ELIXIR: ItemDef = ItemDef {
    id: "elixir",
    name: "Elixir",
    description: "Fully restores HP and MP",
    item_type: ItemType::Consumable,
    cost: 5000,
    effect: ItemEffect::FullRestore,
    usable_by: &[],
};

pub const ETHER: ItemDef = ItemDef {
    id: "ether",
    name: "Ether",
    description: "Restores 30 MP",
    item_type: ItemType::Consumable,
    cost: 150,
    effect: ItemEffect::HealMP(30),
    usable_by: &[],
};

pub const HI_ETHER: ItemDef = ItemDef {
    id: "hi_ether",
    name: "Hi-Ether",
    description: "Restores 80 MP",
    item_type: ItemType::Consumable,
    cost: 400,
    effect: ItemEffect::HealMP(80),
    usable_by: &[],
};

pub const ANTIDOTE: ItemDef = ItemDef {
    id: "antidote",
    name: "Antidote",
    description: "Cures poison",
    item_type: ItemType::Consumable,
    cost: 30,
    effect: ItemEffect::CureStatus(StatusEffect::Poison),
    usable_by: &[],
};

pub const SOFT: ItemDef = ItemDef {
    id: "soft",
    name: "Soft",
    description: "Cures petrification",
    item_type: ItemType::Consumable,
    cost: 200,
    effect: ItemEffect::CureStatus(StatusEffect::Stone),
    usable_by: &[],
};

pub const ECHO_HERBS: ItemDef = ItemDef {
    id: "echo_herbs",
    name: "Echo Herbs",
    description: "Cures silence",
    item_type: ItemType::Consumable,
    cost: 50,
    effect: ItemEffect::CureStatus(StatusEffect::Silence),
    usable_by: &[],
};

pub const EYE_DROPS: ItemDef = ItemDef {
    id: "eye_drops",
    name: "Eye Drops",
    description: "Cures blindness",
    item_type: ItemType::Consumable,
    cost: 40,
    effect: ItemEffect::CureStatus(StatusEffect::Blind),
    usable_by: &[],
};

pub const SMELLING_SALTS: ItemDef = ItemDef {
    id: "smelling_salts",
    name: "Smelling Salts",
    description: "Cures sleep and confusion",
    item_type: ItemType::Consumable,
    cost: 60,
    effect: ItemEffect::CureStatus(StatusEffect::Asleep),
    usable_by: &[],
};

pub const REMEDY: ItemDef = ItemDef {
    id: "remedy",
    name: "Remedy",
    description: "Cures all status ailments",
    item_type: ItemType::Consumable,
    cost: 500,
    effect: ItemEffect::CureAll,
    usable_by: &[],
};

pub const PHOENIX_DOWN: ItemDef = ItemDef {
    id: "phoenix_down",
    name: "Phoenix Down",
    description: "Revives fallen ally with 25% HP",
    item_type: ItemType::Consumable,
    cost: 300,
    effect: ItemEffect::Revive(0.25),
    usable_by: &[],
};

pub const PHOENIX_PLUME: ItemDef = ItemDef {
    id: "phoenix_plume",
    name: "Phoenix Plume",
    description: "Revives fallen ally with full HP",
    item_type: ItemType::Consumable,
    cost: 2000,
    effect: ItemEffect::Revive(1.0),
    usable_by: &[],
};

pub const BOMB_FRAGMENT: ItemDef = ItemDef {
    id: "bomb_fragment",
    name: "Bomb Fragment",
    description: "Deals fire damage to one enemy",
    item_type: ItemType::Consumable,
    cost: 100,
    effect: ItemEffect::Damage { power: 50, element: Element::Fire },
    usable_by: &[],
};

pub const ARCTIC_WIND: ItemDef = ItemDef {
    id: "arctic_wind",
    name: "Arctic Wind",
    description: "Deals ice damage to one enemy",
    item_type: ItemType::Consumable,
    cost: 100,
    effect: ItemEffect::Damage { power: 50, element: Element::Ice },
    usable_by: &[],
};

pub const ZEUS_RAGE: ItemDef = ItemDef {
    id: "zeus_rage",
    name: "Zeus's Rage",
    description: "Deals lightning damage to all enemies",
    item_type: ItemType::Consumable,
    cost: 250,
    effect: ItemEffect::Damage { power: 80, element: Element::Lightning },
    usable_by: &[],
};

// ============================================================================
// WEAPONS
// ============================================================================

pub const IRON_SWORD: ItemDef = ItemDef {
    id: "iron_sword",
    name: "Iron Sword",
    description: "A basic iron blade",
    item_type: ItemType::Weapon,
    cost: 100,
    effect: ItemEffect::WeaponStats { attack: 10, element: Element::None },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const STEEL_SWORD: ItemDef = ItemDef {
    id: "steel_sword",
    name: "Steel Sword",
    description: "A well-crafted steel sword",
    item_type: ItemType::Weapon,
    cost: 350,
    effect: ItemEffect::WeaponStats { attack: 18, element: Element::None },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const SILVER_SWORD: ItemDef = ItemDef {
    id: "silver_sword",
    name: "Silver Sword",
    description: "A blessed silver blade",
    item_type: ItemType::Weapon,
    cost: 800,
    effect: ItemEffect::WeaponStats { attack: 28, element: Element::Holy },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const FLAME_SWORD: ItemDef = ItemDef {
    id: "flame_sword",
    name: "Flame Sword",
    description: "A blade wreathed in eternal fire",
    item_type: ItemType::Weapon,
    cost: 1500,
    effect: ItemEffect::WeaponStats { attack: 35, element: Element::Fire },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const ICE_BRAND: ItemDef = ItemDef {
    id: "ice_brand",
    name: "Ice Brand",
    description: "A blade of frozen steel",
    item_type: ItemType::Weapon,
    cost: 1500,
    effect: ItemEffect::WeaponStats { attack: 35, element: Element::Ice },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const THUNDER_BLADE: ItemDef = ItemDef {
    id: "thunder_blade",
    name: "Thunder Blade",
    description: "Crackles with lightning",
    item_type: ItemType::Weapon,
    cost: 1500,
    effect: ItemEffect::WeaponStats { attack: 35, element: Element::Lightning },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const GUARDIAN_BLADE: ItemDef = ItemDef {
    id: "guardian_blade",
    name: "Guardian Blade",
    description: "Forged by the ancients",
    item_type: ItemType::Weapon,
    cost: 5000,
    effect: ItemEffect::WeaponStats { attack: 55, element: Element::Holy },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const CUTLASS: ItemDef = ItemDef {
    id: "cutlass",
    name: "Cutlass",
    description: "A swift sailor's blade",
    item_type: ItemType::Weapon,
    cost: 200,
    effect: ItemEffect::WeaponStats { attack: 12, element: Element::None },
    usable_by: &[CharacterClass::Swashbuckler, CharacterClass::Thief],
};

pub const RAPIER: ItemDef = ItemDef {
    id: "rapier",
    name: "Rapier",
    description: "A precise dueling weapon",
    item_type: ItemType::Weapon,
    cost: 500,
    effect: ItemEffect::WeaponStats { attack: 22, element: Element::None },
    usable_by: &[CharacterClass::Swashbuckler, CharacterClass::Thief],
};

pub const MYTHRIL_RAPIER: ItemDef = ItemDef {
    id: "mythril_rapier",
    name: "Mythril Rapier",
    description: "Light as air, sharp as fate",
    item_type: ItemType::Weapon,
    cost: 2000,
    effect: ItemEffect::WeaponStats { attack: 40, element: Element::None },
    usable_by: &[CharacterClass::Swashbuckler, CharacterClass::Thief],
};

pub const DAGGER: ItemDef = ItemDef {
    id: "dagger",
    name: "Dagger",
    description: "A simple throwing knife",
    item_type: ItemType::Weapon,
    cost: 50,
    effect: ItemEffect::WeaponStats { attack: 6, element: Element::None },
    usable_by: &[CharacterClass::Thief, CharacterClass::Wizard, CharacterClass::Sage],
};

pub const ASSASSIN_DAGGER: ItemDef = ItemDef {
    id: "assassin_dagger",
    name: "Assassin's Dagger",
    description: "Coated with deadly poison",
    item_type: ItemType::Weapon,
    cost: 1200,
    effect: ItemEffect::WeaponStats { attack: 25, element: Element::Dark },
    usable_by: &[CharacterClass::Thief],
};

pub const OAK_STAFF: ItemDef = ItemDef {
    id: "oak_staff",
    name: "Oak Staff",
    description: "A simple wooden staff",
    item_type: ItemType::Weapon,
    cost: 80,
    effect: ItemEffect::WeaponStats { attack: 5, element: Element::None },
    usable_by: &[CharacterClass::Wizard, CharacterClass::Sage, CharacterClass::Cleric],
};

pub const MAGE_STAFF: ItemDef = ItemDef {
    id: "mage_staff",
    name: "Mage Staff",
    description: "Enhances magical power",
    item_type: ItemType::Weapon,
    cost: 400,
    effect: ItemEffect::WeaponStats { attack: 8, element: Element::None },
    usable_by: &[CharacterClass::Wizard, CharacterClass::Sage, CharacterClass::Cleric],
};

pub const WIZARD_STAFF: ItemDef = ItemDef {
    id: "wizard_staff",
    name: "Wizard Staff",
    description: "Channels arcane energy",
    item_type: ItemType::Weapon,
    cost: 1000,
    effect: ItemEffect::WeaponStats { attack: 12, element: Element::None },
    usable_by: &[CharacterClass::Wizard, CharacterClass::Sage],
};

pub const ARCANE_ROD: ItemDef = ItemDef {
    id: "arcane_rod",
    name: "Arcane Rod",
    description: "Pulses with raw magic",
    item_type: ItemType::Weapon,
    cost: 3000,
    effect: ItemEffect::WeaponStats { attack: 20, element: Element::None },
    usable_by: &[CharacterClass::Wizard, CharacterClass::Sage],
};

pub const HOLY_MACE: ItemDef = ItemDef {
    id: "holy_mace",
    name: "Holy Mace",
    description: "Blessed by the Guardians",
    item_type: ItemType::Weapon,
    cost: 600,
    effect: ItemEffect::WeaponStats { attack: 15, element: Element::Holy },
    usable_by: &[CharacterClass::Cleric, CharacterClass::Paladin],
};

pub const JUDGMENT_HAMMER: ItemDef = ItemDef {
    id: "judgment_hammer",
    name: "Judgment Hammer",
    description: "Smites the wicked",
    item_type: ItemType::Weapon,
    cost: 2500,
    effect: ItemEffect::WeaponStats { attack: 38, element: Element::Holy },
    usable_by: &[CharacterClass::Cleric, CharacterClass::Paladin],
};

pub const SHORT_BOW: ItemDef = ItemDef {
    id: "short_bow",
    name: "Short Bow",
    description: "A simple hunting bow",
    item_type: ItemType::Weapon,
    cost: 120,
    effect: ItemEffect::WeaponStats { attack: 9, element: Element::None },
    usable_by: &[CharacterClass::Archer],
};

pub const LONG_BOW: ItemDef = ItemDef {
    id: "long_bow",
    name: "Long Bow",
    description: "Greater range and power",
    item_type: ItemType::Weapon,
    cost: 450,
    effect: ItemEffect::WeaponStats { attack: 20, element: Element::None },
    usable_by: &[CharacterClass::Archer],
};

pub const WIND_BOW: ItemDef = ItemDef {
    id: "wind_bow",
    name: "Wind Bow",
    description: "Arrows fly with the wind",
    item_type: ItemType::Weapon,
    cost: 1800,
    effect: ItemEffect::WeaponStats { attack: 36, element: Element::Wind },
    usable_by: &[CharacterClass::Archer],
};

pub const ARTEMIS_BOW: ItemDef = ItemDef {
    id: "artemis_bow",
    name: "Artemis Bow",
    description: "Never misses its mark",
    item_type: ItemType::Weapon,
    cost: 4000,
    effect: ItemEffect::WeaponStats { attack: 50, element: Element::None },
    usable_by: &[CharacterClass::Archer],
};

// ============================================================================
// ARMOR
// ============================================================================

pub const LEATHER_ARMOR: ItemDef = ItemDef {
    id: "leather_armor",
    name: "Leather Armor",
    description: "Basic protection",
    item_type: ItemType::Armor,
    cost: 80,
    effect: ItemEffect::ArmorStats { defense: 5, magic_def: 2 },
    usable_by: &[],
};

pub const CHAIN_MAIL: ItemDef = ItemDef {
    id: "chain_mail",
    name: "Chain Mail",
    description: "Linked metal rings",
    item_type: ItemType::Armor,
    cost: 300,
    effect: ItemEffect::ArmorStats { defense: 12, magic_def: 4 },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const PLATE_ARMOR: ItemDef = ItemDef {
    id: "plate_armor",
    name: "Plate Armor",
    description: "Heavy steel plates",
    item_type: ItemType::Armor,
    cost: 800,
    effect: ItemEffect::ArmorStats { defense: 22, magic_def: 6 },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const MYTHRIL_ARMOR: ItemDef = ItemDef {
    id: "mythril_armor",
    name: "Mythril Armor",
    description: "Light yet strong",
    item_type: ItemType::Armor,
    cost: 2500,
    effect: ItemEffect::ArmorStats { defense: 35, magic_def: 15 },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const GUARDIAN_ARMOR: ItemDef = ItemDef {
    id: "guardian_armor",
    name: "Guardian Armor",
    description: "Worn by ancient protectors",
    item_type: ItemType::Armor,
    cost: 6000,
    effect: ItemEffect::ArmorStats { defense: 50, magic_def: 25 },
    usable_by: &[CharacterClass::Warrior, CharacterClass::Paladin, CharacterClass::Knight],
};

pub const ROBES: ItemDef = ItemDef {
    id: "robes",
    name: "Robes",
    description: "Simple cloth robes",
    item_type: ItemType::Armor,
    cost: 50,
    effect: ItemEffect::ArmorStats { defense: 2, magic_def: 5 },
    usable_by: &[CharacterClass::Wizard, CharacterClass::Sage, CharacterClass::Cleric],
};

pub const MAGE_ROBES: ItemDef = ItemDef {
    id: "mage_robes",
    name: "Mage Robes",
    description: "Enchanted for protection",
    item_type: ItemType::Armor,
    cost: 400,
    effect: ItemEffect::ArmorStats { defense: 6, magic_def: 15 },
    usable_by: &[CharacterClass::Wizard, CharacterClass::Sage, CharacterClass::Cleric],
};

pub const ARCHMAGE_ROBES: ItemDef = ItemDef {
    id: "archmage_robes",
    name: "Archmage Robes",
    description: "Woven with magical threads",
    item_type: ItemType::Armor,
    cost: 2000,
    effect: ItemEffect::ArmorStats { defense: 12, magic_def: 35 },
    usable_by: &[CharacterClass::Wizard, CharacterClass::Sage, CharacterClass::Cleric],
};

pub const BRIGANDINE: ItemDef = ItemDef {
    id: "brigandine",
    name: "Brigandine",
    description: "Flexible studded armor",
    item_type: ItemType::Armor,
    cost: 200,
    effect: ItemEffect::ArmorStats { defense: 8, magic_def: 4 },
    usable_by: &[CharacterClass::Swashbuckler, CharacterClass::Thief, CharacterClass::Archer],
};

pub const SHADOW_VEST: ItemDef = ItemDef {
    id: "shadow_vest",
    name: "Shadow Vest",
    description: "Dark and light as shadow",
    item_type: ItemType::Armor,
    cost: 1500,
    effect: ItemEffect::ArmorStats { defense: 18, magic_def: 12 },
    usable_by: &[CharacterClass::Swashbuckler, CharacterClass::Thief],
};

pub const WIND_CLOAK: ItemDef = ItemDef {
    id: "wind_cloak",
    name: "Wind Cloak",
    description: "Dances with the breeze",
    item_type: ItemType::Armor,
    cost: 1800,
    effect: ItemEffect::ArmorStats { defense: 15, magic_def: 18 },
    usable_by: &[CharacterClass::Archer],
};

// ============================================================================
// ACCESSORIES
// ============================================================================

pub const POWER_RING: ItemDef = ItemDef {
    id: "power_ring",
    name: "Power Ring",
    description: "Increases strength",
    item_type: ItemType::Accessory,
    cost: 500,
    effect: ItemEffect::AccessoryStats { hp_bonus: 0, mp_bonus: 0, stat_bonus: 5, element_resist: None },
    usable_by: &[],
};

pub const MAGIC_RING: ItemDef = ItemDef {
    id: "magic_ring",
    name: "Magic Ring",
    description: "Increases MP",
    item_type: ItemType::Accessory,
    cost: 500,
    effect: ItemEffect::AccessoryStats { hp_bonus: 0, mp_bonus: 20, stat_bonus: 0, element_resist: None },
    usable_by: &[],
};

pub const LIFE_RING: ItemDef = ItemDef {
    id: "life_ring",
    name: "Life Ring",
    description: "Increases max HP",
    item_type: ItemType::Accessory,
    cost: 500,
    effect: ItemEffect::AccessoryStats { hp_bonus: 50, mp_bonus: 0, stat_bonus: 0, element_resist: None },
    usable_by: &[],
};

pub const FIRE_RING: ItemDef = ItemDef {
    id: "fire_ring",
    name: "Fire Ring",
    description: "Resists fire damage",
    item_type: ItemType::Accessory,
    cost: 800,
    effect: ItemEffect::AccessoryStats { hp_bonus: 0, mp_bonus: 0, stat_bonus: 0, element_resist: Some(Element::Fire) },
    usable_by: &[],
};

pub const ICE_RING: ItemDef = ItemDef {
    id: "ice_ring",
    name: "Ice Ring",
    description: "Resists ice damage",
    item_type: ItemType::Accessory,
    cost: 800,
    effect: ItemEffect::AccessoryStats { hp_bonus: 0, mp_bonus: 0, stat_bonus: 0, element_resist: Some(Element::Ice) },
    usable_by: &[],
};

pub const LIGHTNING_RING: ItemDef = ItemDef {
    id: "lightning_ring",
    name: "Lightning Ring",
    description: "Resists lightning damage",
    item_type: ItemType::Accessory,
    cost: 800,
    effect: ItemEffect::AccessoryStats { hp_bonus: 0, mp_bonus: 0, stat_bonus: 0, element_resist: Some(Element::Lightning) },
    usable_by: &[],
};

pub const GUARDIAN_AMULET: ItemDef = ItemDef {
    id: "guardian_amulet",
    name: "Guardian Amulet",
    description: "Ancient protective charm",
    item_type: ItemType::Accessory,
    cost: 3000,
    effect: ItemEffect::AccessoryStats { hp_bonus: 30, mp_bonus: 15, stat_bonus: 3, element_resist: Some(Element::Holy) },
    usable_by: &[],
};

pub const THIEF_GLOVES: ItemDef = ItemDef {
    id: "thief_gloves",
    name: "Thief Gloves",
    description: "Increases luck and agility",
    item_type: ItemType::Accessory,
    cost: 1200,
    effect: ItemEffect::AccessoryStats { hp_bonus: 0, mp_bonus: 0, stat_bonus: 7, element_resist: None },
    usable_by: &[CharacterClass::Thief],
};

// ============================================================================
// KEY ITEMS
// ============================================================================

pub const SPIRIT_CHALICE: ItemDef = ItemDef {
    id: "spirit_chalice",
    name: "Spirit Chalice",
    description: "Vessel for Spirit Guardian's echo",
    item_type: ItemType::KeyItem,
    cost: 0,
    effect: ItemEffect::None,
    usable_by: &[],
};

pub const EARTH_CROWN: ItemDef = ItemDef {
    id: "earth_crown",
    name: "Earth Crown",
    description: "Vessel for Earth Guardian's echo",
    item_type: ItemType::KeyItem,
    cost: 0,
    effect: ItemEffect::None,
    usable_by: &[],
};

pub const WATER_PEARL: ItemDef = ItemDef {
    id: "water_pearl",
    name: "Water Pearl",
    description: "Vessel for Water Guardian's echo",
    item_type: ItemType::KeyItem,
    cost: 0,
    effect: ItemEffect::None,
    usable_by: &[],
};

pub const WIND_FEATHER: ItemDef = ItemDef {
    id: "wind_feather",
    name: "Wind Feather",
    description: "Vessel for Wind Guardian's echo",
    item_type: ItemType::KeyItem,
    cost: 0,
    effect: ItemEffect::None,
    usable_by: &[],
};

pub const FIRE_HEART: ItemDef = ItemDef {
    id: "fire_heart",
    name: "Fire Heart",
    description: "Vessel for Fire Guardian's echo",
    item_type: ItemType::KeyItem,
    cost: 0,
    effect: ItemEffect::None,
    usable_by: &[],
};

pub const SHIP_KEY: ItemDef = ItemDef {
    id: "ship_key",
    name: "Ship Key",
    description: "Captain John's vessel key",
    item_type: ItemType::KeyItem,
    cost: 0,
    effect: ItemEffect::None,
    usable_by: &[],
};

pub const AIRSHIP_CRYSTAL: ItemDef = ItemDef {
    id: "airship_crystal",
    name: "Airship Crystal",
    description: "Powers the Sky Nomad airship",
    item_type: ItemType::KeyItem,
    cost: 0,
    effect: ItemEffect::None,
    usable_by: &[],
};

pub const DORL_LETTER: ItemDef = ItemDef {
    id: "dorl_letter",
    name: "Dorl's Letter",
    description: "A letter of introduction from Dorl",
    item_type: ItemType::KeyItem,
    cost: 0,
    effect: ItemEffect::None,
    usable_by: &[],
};

// ============================================================================
// ITEM COLLECTION AND LOOKUP
// ============================================================================

/// All items in the game
pub static ALL_ITEMS: &[&ItemDef] = &[
    // Consumables
    &POTION, &HI_POTION, &MEGA_POTION, &ELIXIR,
    &ETHER, &HI_ETHER,
    &ANTIDOTE, &SOFT, &ECHO_HERBS, &EYE_DROPS, &SMELLING_SALTS, &REMEDY,
    &PHOENIX_DOWN, &PHOENIX_PLUME,
    &BOMB_FRAGMENT, &ARCTIC_WIND, &ZEUS_RAGE,
    // Weapons
    &IRON_SWORD, &STEEL_SWORD, &SILVER_SWORD, &FLAME_SWORD, &ICE_BRAND, &THUNDER_BLADE, &GUARDIAN_BLADE,
    &CUTLASS, &RAPIER, &MYTHRIL_RAPIER,
    &DAGGER, &ASSASSIN_DAGGER,
    &OAK_STAFF, &MAGE_STAFF, &WIZARD_STAFF, &ARCANE_ROD,
    &HOLY_MACE, &JUDGMENT_HAMMER,
    &SHORT_BOW, &LONG_BOW, &WIND_BOW, &ARTEMIS_BOW,
    // Armor
    &LEATHER_ARMOR, &CHAIN_MAIL, &PLATE_ARMOR, &MYTHRIL_ARMOR, &GUARDIAN_ARMOR,
    &ROBES, &MAGE_ROBES, &ARCHMAGE_ROBES,
    &BRIGANDINE, &SHADOW_VEST, &WIND_CLOAK,
    // Accessories
    &POWER_RING, &MAGIC_RING, &LIFE_RING,
    &FIRE_RING, &ICE_RING, &LIGHTNING_RING,
    &GUARDIAN_AMULET, &THIEF_GLOVES,
    // Key Items
    &SPIRIT_CHALICE, &EARTH_CROWN, &WATER_PEARL, &WIND_FEATHER, &FIRE_HEART,
    &SHIP_KEY, &AIRSHIP_CRYSTAL, &DORL_LETTER,
];

/// Look up an item by its ID
pub fn get_item(id: &str) -> Option<&'static ItemDef> {
    ALL_ITEMS.iter().find(|item| item.id == id).copied()
}

/// Get all items of a specific type
pub fn get_items_by_type(item_type: ItemType) -> Vec<&'static ItemDef> {
    ALL_ITEMS.iter()
        .filter(|item| item.item_type == item_type)
        .copied()
        .collect()
}

/// Get all items usable by a specific class
pub fn get_items_for_class(class: CharacterClass) -> Vec<&'static ItemDef> {
    ALL_ITEMS.iter()
        .filter(|item| item.usable_by.is_empty() || item.usable_by.contains(&class))
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
    fn test_all_items_have_unique_ids() {
        let mut ids: Vec<&str> = ALL_ITEMS.iter().map(|item| item.id).collect();
        ids.sort();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Duplicate item IDs found");
    }

    #[test]
    fn test_all_items_lookup() {
        for item in ALL_ITEMS.iter() {
            let found = get_item(item.id);
            assert!(found.is_some(), "Failed to find item: {}", item.id);
            assert_eq!(found.unwrap().id, item.id);
        }
    }

    #[test]
    fn test_get_item_not_found() {
        assert!(get_item("nonexistent_item").is_none());
    }

    #[test]
    fn test_consumables_have_positive_cost() {
        for item in get_items_by_type(ItemType::Consumable) {
            assert!(item.cost > 0, "Consumable {} should have positive cost", item.id);
        }
    }

    #[test]
    fn test_key_items_are_free() {
        for item in get_items_by_type(ItemType::KeyItem) {
            assert_eq!(item.cost, 0, "Key item {} should be free", item.id);
        }
    }

    #[test]
    fn test_vessel_key_items_exist() {
        // Verify all 5 Guardian vessels exist
        assert!(get_item("spirit_chalice").is_some());
        assert!(get_item("earth_crown").is_some());
        assert!(get_item("water_pearl").is_some());
        assert!(get_item("wind_feather").is_some());
        assert!(get_item("fire_heart").is_some());
    }
}
