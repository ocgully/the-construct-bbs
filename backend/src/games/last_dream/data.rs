//! Static game data for Last Dream
//! Classes, spells, enemies, items, equipment, and world locations

use rand::prelude::*;

// ============================================================================
// CHARACTER CLASSES
// ============================================================================

/// Character class definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassType {
    Warrior,
    Thief,
    Mage,
    Cleric,
    Monk,
    Knight,
}

impl ClassType {
    pub fn name(&self) -> &'static str {
        match self {
            ClassType::Warrior => "Warrior",
            ClassType::Thief => "Thief",
            ClassType::Mage => "Mage",
            ClassType::Cleric => "Cleric",
            ClassType::Monk => "Monk",
            ClassType::Knight => "Knight",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ClassType::Warrior => "High HP, STR. Heavy weapons/armor. No magic.",
            ClassType::Thief => "High Speed, luck. Can steal. Light weapons.",
            ClassType::Mage => "High INT, MP. Offensive spells. Fragile.",
            ClassType::Cleric => "Healing/support magic. Medium combat.",
            ClassType::Monk => "Unarmed specialist. High damage unequipped.",
            ClassType::Knight => "Balanced. Some white magic.",
        }
    }

    /// Base stats for this class: (HP, MP, STR, AGI, INT, VIT, LUCK)
    pub fn base_stats(&self) -> (u32, u32, u32, u32, u32, u32, u32) {
        match self {
            ClassType::Warrior => (40, 0, 12, 6, 3, 10, 5),
            ClassType::Thief => (25, 0, 6, 15, 5, 6, 12),
            ClassType::Mage => (20, 30, 3, 6, 15, 4, 6),
            ClassType::Cleric => (28, 25, 5, 5, 10, 7, 8),
            ClassType::Monk => (35, 5, 8, 10, 6, 8, 8),
            ClassType::Knight => (35, 10, 10, 7, 6, 9, 6),
        }
    }

    /// Stats gained per level: (HP, MP, STR, AGI, INT, VIT, LUCK)
    pub fn level_up_stats(&self) -> (u32, u32, u32, u32, u32, u32, u32) {
        match self {
            ClassType::Warrior => (12, 0, 4, 2, 1, 3, 1),
            ClassType::Thief => (8, 0, 2, 5, 1, 2, 4),
            ClassType::Mage => (5, 8, 1, 2, 5, 1, 2),
            ClassType::Cleric => (7, 6, 2, 2, 4, 2, 2),
            ClassType::Monk => (10, 2, 3, 3, 2, 3, 2),
            ClassType::Knight => (10, 3, 3, 2, 2, 3, 2),
        }
    }

    /// Can this class equip the given equipment type?
    pub fn can_equip(&self, equip_type: EquipmentType) -> bool {
        match (self, equip_type) {
            // Weapons
            (ClassType::Warrior, EquipmentType::Sword) => true,
            (ClassType::Warrior, EquipmentType::Axe) => true,
            (ClassType::Warrior, EquipmentType::HeavyArmor) => true,
            (ClassType::Warrior, EquipmentType::Shield) => true,
            (ClassType::Warrior, EquipmentType::Helmet) => true,

            (ClassType::Thief, EquipmentType::Dagger) => true,
            (ClassType::Thief, EquipmentType::Sword) => true,
            (ClassType::Thief, EquipmentType::LightArmor) => true,

            (ClassType::Mage, EquipmentType::Staff) => true,
            (ClassType::Mage, EquipmentType::Robe) => true,

            (ClassType::Cleric, EquipmentType::Staff) => true,
            (ClassType::Cleric, EquipmentType::Hammer) => true,
            (ClassType::Cleric, EquipmentType::Robe) => true,
            (ClassType::Cleric, EquipmentType::LightArmor) => true,
            (ClassType::Cleric, EquipmentType::Shield) => true,

            (ClassType::Monk, EquipmentType::Fist) => true,
            (ClassType::Monk, EquipmentType::Staff) => true,
            (ClassType::Monk, EquipmentType::Robe) => true,

            (ClassType::Knight, EquipmentType::Sword) => true,
            (ClassType::Knight, EquipmentType::HeavyArmor) => true,
            (ClassType::Knight, EquipmentType::LightArmor) => true,
            (ClassType::Knight, EquipmentType::Shield) => true,
            (ClassType::Knight, EquipmentType::Helmet) => true,

            // Accessories anyone can equip
            (_, EquipmentType::Accessory) => true,

            _ => false,
        }
    }

    /// Get learnable spells for this class at given level
    pub fn spells_at_level(&self, level: u8) -> Vec<&'static str> {
        let mut spells = Vec::new();

        match self {
            ClassType::Mage => {
                if level >= 1 { spells.push("fire"); }
                if level >= 3 { spells.push("thunder"); }
                if level >= 5 { spells.push("blizzard"); }
                if level >= 8 { spells.push("fira"); }
                if level >= 12 { spells.push("thundara"); }
                if level >= 15 { spells.push("blizzara"); }
                if level >= 20 { spells.push("firaga"); }
                if level >= 25 { spells.push("thundaga"); }
                if level >= 30 { spells.push("blizzaga"); }
                if level >= 35 { spells.push("flare"); }
            }
            ClassType::Cleric => {
                if level >= 1 { spells.push("cure"); }
                if level >= 3 { spells.push("protect"); }
                if level >= 5 { spells.push("antidote"); }
                if level >= 8 { spells.push("cura"); }
                if level >= 12 { spells.push("shell"); }
                if level >= 15 { spells.push("raise"); }
                if level >= 20 { spells.push("curaga"); }
                if level >= 25 { spells.push("holy"); }
                if level >= 30 { spells.push("arise"); }
                if level >= 35 { spells.push("full_cure"); }
            }
            ClassType::Knight => {
                if level >= 5 { spells.push("cure"); }
                if level >= 10 { spells.push("protect"); }
                if level >= 20 { spells.push("cura"); }
            }
            ClassType::Monk => {
                if level >= 10 { spells.push("focus"); }
                if level >= 20 { spells.push("chakra"); }
            }
            _ => {}
        }

        spells
    }
}

// ============================================================================
// SPELLS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Element {
    Fire,
    Ice,
    Lightning,
    Holy,
    Dark,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellTarget {
    SingleEnemy,
    AllEnemies,
    SingleAlly,
    AllAllies,
    Self_,
}

#[derive(Debug, Clone)]
pub struct SpellData {
    pub key: &'static str,
    pub name: &'static str,
    pub mp_cost: u32,
    pub power: u32,
    pub element: Element,
    pub target: SpellTarget,
    pub is_healing: bool,
    pub description: &'static str,
}

pub static SPELLS: &[SpellData] = &[
    // Black Magic
    SpellData { key: "fire", name: "Fire", mp_cost: 4, power: 20, element: Element::Fire, target: SpellTarget::SingleEnemy, is_healing: false, description: "Fire damage to one enemy." },
    SpellData { key: "fira", name: "Fira", mp_cost: 12, power: 50, element: Element::Fire, target: SpellTarget::SingleEnemy, is_healing: false, description: "Medium fire damage to one enemy." },
    SpellData { key: "firaga", name: "Firaga", mp_cost: 30, power: 120, element: Element::Fire, target: SpellTarget::AllEnemies, is_healing: false, description: "Heavy fire damage to all enemies." },
    SpellData { key: "thunder", name: "Thunder", mp_cost: 4, power: 22, element: Element::Lightning, target: SpellTarget::SingleEnemy, is_healing: false, description: "Lightning damage to one enemy." },
    SpellData { key: "thundara", name: "Thundara", mp_cost: 12, power: 55, element: Element::Lightning, target: SpellTarget::SingleEnemy, is_healing: false, description: "Medium lightning damage to one enemy." },
    SpellData { key: "thundaga", name: "Thundaga", mp_cost: 30, power: 130, element: Element::Lightning, target: SpellTarget::AllEnemies, is_healing: false, description: "Heavy lightning damage to all enemies." },
    SpellData { key: "blizzard", name: "Blizzard", mp_cost: 4, power: 18, element: Element::Ice, target: SpellTarget::SingleEnemy, is_healing: false, description: "Ice damage to one enemy." },
    SpellData { key: "blizzara", name: "Blizzara", mp_cost: 12, power: 48, element: Element::Ice, target: SpellTarget::SingleEnemy, is_healing: false, description: "Medium ice damage to one enemy." },
    SpellData { key: "blizzaga", name: "Blizzaga", mp_cost: 30, power: 110, element: Element::Ice, target: SpellTarget::AllEnemies, is_healing: false, description: "Heavy ice damage to all enemies." },
    SpellData { key: "flare", name: "Flare", mp_cost: 50, power: 200, element: Element::None, target: SpellTarget::SingleEnemy, is_healing: false, description: "Devastating non-elemental damage." },

    // White Magic
    SpellData { key: "cure", name: "Cure", mp_cost: 4, power: 30, element: Element::None, target: SpellTarget::SingleAlly, is_healing: true, description: "Restore HP to one ally." },
    SpellData { key: "cura", name: "Cura", mp_cost: 12, power: 80, element: Element::None, target: SpellTarget::SingleAlly, is_healing: true, description: "Restore more HP to one ally." },
    SpellData { key: "curaga", name: "Curaga", mp_cost: 30, power: 180, element: Element::None, target: SpellTarget::AllAllies, is_healing: true, description: "Restore HP to all allies." },
    SpellData { key: "full_cure", name: "Full Cure", mp_cost: 50, power: 999, element: Element::None, target: SpellTarget::SingleAlly, is_healing: true, description: "Fully restore one ally's HP." },
    SpellData { key: "raise", name: "Raise", mp_cost: 20, power: 50, element: Element::Holy, target: SpellTarget::SingleAlly, is_healing: true, description: "Revive fallen ally with some HP." },
    SpellData { key: "arise", name: "Arise", mp_cost: 50, power: 100, element: Element::Holy, target: SpellTarget::SingleAlly, is_healing: true, description: "Revive fallen ally with full HP." },
    SpellData { key: "protect", name: "Protect", mp_cost: 6, power: 0, element: Element::None, target: SpellTarget::SingleAlly, is_healing: false, description: "Increase ally's defense." },
    SpellData { key: "shell", name: "Shell", mp_cost: 6, power: 0, element: Element::None, target: SpellTarget::SingleAlly, is_healing: false, description: "Increase ally's magic defense." },
    SpellData { key: "antidote", name: "Antidote", mp_cost: 5, power: 0, element: Element::None, target: SpellTarget::SingleAlly, is_healing: true, description: "Cure poison status." },
    SpellData { key: "holy", name: "Holy", mp_cost: 45, power: 180, element: Element::Holy, target: SpellTarget::SingleEnemy, is_healing: false, description: "Holy damage to one enemy." },

    // Monk abilities
    SpellData { key: "focus", name: "Focus", mp_cost: 0, power: 0, element: Element::None, target: SpellTarget::Self_, is_healing: false, description: "Double next attack's damage." },
    SpellData { key: "chakra", name: "Chakra", mp_cost: 0, power: 50, element: Element::None, target: SpellTarget::Self_, is_healing: true, description: "Self-heal using inner energy." },
];

pub fn get_spell(key: &str) -> Option<&'static SpellData> {
    SPELLS.iter().find(|s| s.key == key)
}

// ============================================================================
// EQUIPMENT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EquipmentType {
    // Weapons
    Sword,
    Axe,
    Dagger,
    Staff,
    Hammer,
    Fist,
    // Armor
    HeavyArmor,
    LightArmor,
    Robe,
    Shield,
    Helmet,
    Accessory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Shield,
    Helmet,
    Accessory,
}

#[derive(Debug, Clone)]
pub struct EquipmentData {
    pub key: &'static str,
    pub name: &'static str,
    pub equip_type: EquipmentType,
    pub slot: EquipmentSlot,
    pub attack: u32,
    pub defense: u32,
    pub magic: i32,
    pub speed: i32,
    pub price: u32,
    pub description: &'static str,
}

pub static EQUIPMENT: &[EquipmentData] = &[
    // Swords
    EquipmentData { key: "wooden_sword", name: "Wooden Sword", equip_type: EquipmentType::Sword, slot: EquipmentSlot::Weapon, attack: 5, defense: 0, magic: 0, speed: 0, price: 50, description: "A practice sword." },
    EquipmentData { key: "short_sword", name: "Short Sword", equip_type: EquipmentType::Sword, slot: EquipmentSlot::Weapon, attack: 12, defense: 0, magic: 0, speed: 0, price: 200, description: "Standard blade." },
    EquipmentData { key: "long_sword", name: "Long Sword", equip_type: EquipmentType::Sword, slot: EquipmentSlot::Weapon, attack: 22, defense: 0, magic: 0, speed: 0, price: 600, description: "Longer reach." },
    EquipmentData { key: "silver_sword", name: "Silver Sword", equip_type: EquipmentType::Sword, slot: EquipmentSlot::Weapon, attack: 35, defense: 0, magic: 0, speed: 0, price: 1500, description: "Effective vs undead." },
    EquipmentData { key: "flame_sword", name: "Flame Sword", equip_type: EquipmentType::Sword, slot: EquipmentSlot::Weapon, attack: 50, defense: 0, magic: 5, speed: 0, price: 4000, description: "Burns with fire." },
    EquipmentData { key: "ice_brand", name: "Ice Brand", equip_type: EquipmentType::Sword, slot: EquipmentSlot::Weapon, attack: 52, defense: 0, magic: 5, speed: 0, price: 4500, description: "Cold as winter." },
    EquipmentData { key: "excalibur", name: "Excalibur", equip_type: EquipmentType::Sword, slot: EquipmentSlot::Weapon, attack: 85, defense: 5, magic: 10, speed: 5, price: 0, description: "Legendary holy sword." },

    // Daggers
    EquipmentData { key: "knife", name: "Knife", equip_type: EquipmentType::Dagger, slot: EquipmentSlot::Weapon, attack: 4, defense: 0, magic: 0, speed: 2, price: 30, description: "Small blade." },
    EquipmentData { key: "dagger", name: "Dagger", equip_type: EquipmentType::Dagger, slot: EquipmentSlot::Weapon, attack: 10, defense: 0, magic: 0, speed: 3, price: 150, description: "Quick strikes." },
    EquipmentData { key: "mythril_knife", name: "Mythril Knife", equip_type: EquipmentType::Dagger, slot: EquipmentSlot::Weapon, attack: 25, defense: 0, magic: 0, speed: 5, price: 1200, description: "Light and sharp." },
    EquipmentData { key: "assassin_dagger", name: "Assassin's Dagger", equip_type: EquipmentType::Dagger, slot: EquipmentSlot::Weapon, attack: 40, defense: 0, magic: 0, speed: 8, price: 5000, description: "Deadly precision." },

    // Staves
    EquipmentData { key: "wooden_staff", name: "Wooden Staff", equip_type: EquipmentType::Staff, slot: EquipmentSlot::Weapon, attack: 3, defense: 0, magic: 5, speed: 0, price: 40, description: "Focus for magic." },
    EquipmentData { key: "oak_staff", name: "Oak Staff", equip_type: EquipmentType::Staff, slot: EquipmentSlot::Weapon, attack: 6, defense: 0, magic: 12, speed: 0, price: 200, description: "Sturdy and wise." },
    EquipmentData { key: "wizard_staff", name: "Wizard Staff", equip_type: EquipmentType::Staff, slot: EquipmentSlot::Weapon, attack: 10, defense: 0, magic: 25, speed: 0, price: 2000, description: "Magical enhancement." },
    EquipmentData { key: "sage_staff", name: "Sage Staff", equip_type: EquipmentType::Staff, slot: EquipmentSlot::Weapon, attack: 15, defense: 0, magic: 40, speed: 0, price: 8000, description: "Wisdom incarnate." },

    // Axes
    EquipmentData { key: "hand_axe", name: "Hand Axe", equip_type: EquipmentType::Axe, slot: EquipmentSlot::Weapon, attack: 15, defense: 0, magic: 0, speed: -2, price: 300, description: "Heavy strikes." },
    EquipmentData { key: "battle_axe", name: "Battle Axe", equip_type: EquipmentType::Axe, slot: EquipmentSlot::Weapon, attack: 32, defense: 0, magic: 0, speed: -3, price: 1000, description: "Warrior's choice." },
    EquipmentData { key: "giant_axe", name: "Giant's Axe", equip_type: EquipmentType::Axe, slot: EquipmentSlot::Weapon, attack: 55, defense: 0, magic: 0, speed: -5, price: 5000, description: "Massive damage." },

    // Hammers
    EquipmentData { key: "mace", name: "Mace", equip_type: EquipmentType::Hammer, slot: EquipmentSlot::Weapon, attack: 10, defense: 0, magic: 0, speed: -1, price: 180, description: "Blunt force." },
    EquipmentData { key: "war_hammer", name: "War Hammer", equip_type: EquipmentType::Hammer, slot: EquipmentSlot::Weapon, attack: 28, defense: 0, magic: 3, speed: -2, price: 1500, description: "Holy symbol." },
    EquipmentData { key: "mjolnir", name: "Mjolnir", equip_type: EquipmentType::Hammer, slot: EquipmentSlot::Weapon, attack: 60, defense: 0, magic: 15, speed: 0, price: 0, description: "Thunder god's hammer." },

    // Fist weapons
    EquipmentData { key: "leather_gloves", name: "Leather Gloves", equip_type: EquipmentType::Fist, slot: EquipmentSlot::Weapon, attack: 8, defense: 1, magic: 0, speed: 2, price: 100, description: "Basic protection." },
    EquipmentData { key: "iron_knuckle", name: "Iron Knuckle", equip_type: EquipmentType::Fist, slot: EquipmentSlot::Weapon, attack: 20, defense: 2, magic: 0, speed: 2, price: 500, description: "Hard hits." },
    EquipmentData { key: "tiger_claws", name: "Tiger Claws", equip_type: EquipmentType::Fist, slot: EquipmentSlot::Weapon, attack: 45, defense: 3, magic: 0, speed: 5, price: 3500, description: "Swift and deadly." },

    // Heavy Armor
    EquipmentData { key: "leather_armor", name: "Leather Armor", equip_type: EquipmentType::HeavyArmor, slot: EquipmentSlot::Armor, attack: 0, defense: 5, magic: 0, speed: 0, price: 100, description: "Basic protection." },
    EquipmentData { key: "chain_mail", name: "Chain Mail", equip_type: EquipmentType::HeavyArmor, slot: EquipmentSlot::Armor, attack: 0, defense: 15, magic: 0, speed: -1, price: 500, description: "Linked rings." },
    EquipmentData { key: "plate_armor", name: "Plate Armor", equip_type: EquipmentType::HeavyArmor, slot: EquipmentSlot::Armor, attack: 0, defense: 28, magic: -5, speed: -2, price: 1500, description: "Heavy protection." },
    EquipmentData { key: "mythril_armor", name: "Mythril Armor", equip_type: EquipmentType::HeavyArmor, slot: EquipmentSlot::Armor, attack: 0, defense: 40, magic: 0, speed: 0, price: 5000, description: "Light yet strong." },
    EquipmentData { key: "dragon_armor", name: "Dragon Armor", equip_type: EquipmentType::HeavyArmor, slot: EquipmentSlot::Armor, attack: 0, defense: 55, magic: 10, speed: 0, price: 12000, description: "Dragon scales." },

    // Light Armor
    EquipmentData { key: "padded_vest", name: "Padded Vest", equip_type: EquipmentType::LightArmor, slot: EquipmentSlot::Armor, attack: 0, defense: 3, magic: 0, speed: 1, price: 60, description: "Minimal protection." },
    EquipmentData { key: "thief_outfit", name: "Thief's Outfit", equip_type: EquipmentType::LightArmor, slot: EquipmentSlot::Armor, attack: 0, defense: 10, magic: 0, speed: 5, price: 400, description: "Silent movement." },
    EquipmentData { key: "ninja_garb", name: "Ninja Garb", equip_type: EquipmentType::LightArmor, slot: EquipmentSlot::Armor, attack: 0, defense: 22, magic: 0, speed: 10, price: 3000, description: "Shadow wear." },

    // Robes
    EquipmentData { key: "cloth_robe", name: "Cloth Robe", equip_type: EquipmentType::Robe, slot: EquipmentSlot::Armor, attack: 0, defense: 2, magic: 5, speed: 0, price: 50, description: "Simple attire." },
    EquipmentData { key: "silk_robe", name: "Silk Robe", equip_type: EquipmentType::Robe, slot: EquipmentSlot::Armor, attack: 0, defense: 5, magic: 15, speed: 0, price: 500, description: "Magical enhancement." },
    EquipmentData { key: "wizard_robe", name: "Wizard Robe", equip_type: EquipmentType::Robe, slot: EquipmentSlot::Armor, attack: 0, defense: 10, magic: 30, speed: 0, price: 3000, description: "Power flows through." },
    EquipmentData { key: "white_robe", name: "White Robe", equip_type: EquipmentType::Robe, slot: EquipmentSlot::Armor, attack: 0, defense: 12, magic: 25, speed: 0, price: 4000, description: "Holy protection." },
    EquipmentData { key: "black_robe", name: "Black Robe", equip_type: EquipmentType::Robe, slot: EquipmentSlot::Armor, attack: 0, defense: 8, magic: 35, speed: 0, price: 4500, description: "Dark power." },

    // Shields
    EquipmentData { key: "buckler", name: "Buckler", equip_type: EquipmentType::Shield, slot: EquipmentSlot::Shield, attack: 0, defense: 4, magic: 0, speed: 0, price: 80, description: "Small shield." },
    EquipmentData { key: "iron_shield", name: "Iron Shield", equip_type: EquipmentType::Shield, slot: EquipmentSlot::Shield, attack: 0, defense: 10, magic: 0, speed: -1, price: 400, description: "Solid defense." },
    EquipmentData { key: "mythril_shield", name: "Mythril Shield", equip_type: EquipmentType::Shield, slot: EquipmentSlot::Shield, attack: 0, defense: 20, magic: 5, speed: 0, price: 2500, description: "Light metal." },
    EquipmentData { key: "crystal_shield", name: "Crystal Shield", equip_type: EquipmentType::Shield, slot: EquipmentSlot::Shield, attack: 0, defense: 30, magic: 10, speed: 0, price: 8000, description: "Reflects magic." },

    // Helmets
    EquipmentData { key: "leather_cap", name: "Leather Cap", equip_type: EquipmentType::Helmet, slot: EquipmentSlot::Helmet, attack: 0, defense: 2, magic: 0, speed: 0, price: 50, description: "Head protection." },
    EquipmentData { key: "iron_helm", name: "Iron Helm", equip_type: EquipmentType::Helmet, slot: EquipmentSlot::Helmet, attack: 0, defense: 6, magic: 0, speed: 0, price: 300, description: "Sturdy helmet." },
    EquipmentData { key: "great_helm", name: "Great Helm", equip_type: EquipmentType::Helmet, slot: EquipmentSlot::Helmet, attack: 0, defense: 12, magic: 0, speed: -1, price: 1200, description: "Full coverage." },
    EquipmentData { key: "crystal_helm", name: "Crystal Helm", equip_type: EquipmentType::Helmet, slot: EquipmentSlot::Helmet, attack: 0, defense: 18, magic: 8, speed: 0, price: 5000, description: "Magical defense." },

    // Accessories
    EquipmentData { key: "power_ring", name: "Power Ring", equip_type: EquipmentType::Accessory, slot: EquipmentSlot::Accessory, attack: 8, defense: 0, magic: 0, speed: 0, price: 1000, description: "Strength boost." },
    EquipmentData { key: "speed_ring", name: "Speed Ring", equip_type: EquipmentType::Accessory, slot: EquipmentSlot::Accessory, attack: 0, defense: 0, magic: 0, speed: 10, price: 1500, description: "Haste effect." },
    EquipmentData { key: "protect_ring", name: "Protect Ring", equip_type: EquipmentType::Accessory, slot: EquipmentSlot::Accessory, attack: 0, defense: 15, magic: 5, speed: 0, price: 2000, description: "Defensive aura." },
    EquipmentData { key: "crystal_orb", name: "Crystal Orb", equip_type: EquipmentType::Accessory, slot: EquipmentSlot::Accessory, attack: 0, defense: 5, magic: 20, speed: 0, price: 3500, description: "Magic amplifier." },
];

pub fn get_equipment(key: &str) -> Option<&'static EquipmentData> {
    EQUIPMENT.iter().find(|e| e.key == key)
}

// ============================================================================
// ITEMS
// ============================================================================

#[derive(Debug, Clone)]
pub struct ItemData {
    pub key: &'static str,
    pub name: &'static str,
    pub price: u32,
    pub heal_hp: u32,
    pub heal_mp: u32,
    pub revive: bool,
    pub cure_status: bool,
    pub description: &'static str,
}

pub static ITEMS: &[ItemData] = &[
    ItemData { key: "potion", name: "Potion", price: 50, heal_hp: 50, heal_mp: 0, revive: false, cure_status: false, description: "Restores 50 HP." },
    ItemData { key: "hi_potion", name: "Hi-Potion", price: 200, heal_hp: 150, heal_mp: 0, revive: false, cure_status: false, description: "Restores 150 HP." },
    ItemData { key: "x_potion", name: "X-Potion", price: 800, heal_hp: 500, heal_mp: 0, revive: false, cure_status: false, description: "Restores 500 HP." },
    ItemData { key: "elixir", name: "Elixir", price: 3000, heal_hp: 999, heal_mp: 999, revive: false, cure_status: false, description: "Full HP/MP restore." },
    ItemData { key: "ether", name: "Ether", price: 150, heal_hp: 0, heal_mp: 30, revive: false, cure_status: false, description: "Restores 30 MP." },
    ItemData { key: "hi_ether", name: "Hi-Ether", price: 500, heal_hp: 0, heal_mp: 100, revive: false, cure_status: false, description: "Restores 100 MP." },
    ItemData { key: "phoenix_down", name: "Phoenix Down", price: 500, heal_hp: 0, heal_mp: 0, revive: true, cure_status: false, description: "Revives fallen ally." },
    ItemData { key: "antidote_item", name: "Antidote", price: 30, heal_hp: 0, heal_mp: 0, revive: false, cure_status: true, description: "Cures poison." },
    ItemData { key: "tent", name: "Tent", price: 200, heal_hp: 100, heal_mp: 50, revive: false, cure_status: false, description: "Rest on world map." },
    ItemData { key: "cottage", name: "Cottage", price: 1000, heal_hp: 999, heal_mp: 999, revive: false, cure_status: false, description: "Full rest on world map." },
];

pub fn get_item(key: &str) -> Option<&'static ItemData> {
    ITEMS.iter().find(|i| i.key == key)
}

// ============================================================================
// ENEMIES
// ============================================================================

#[derive(Debug, Clone)]
pub struct EnemyData {
    pub key: &'static str,
    pub name: &'static str,
    pub hp: u32,
    pub mp: u32,
    pub attack: u32,
    pub defense: u32,
    pub magic: u32,
    pub speed: u32,
    pub exp: u32,
    pub gold: u32,
    pub weakness: Element,
    pub resistance: Element,
    pub is_boss: bool,
    pub description: &'static str,
}

pub static ENEMIES: &[EnemyData] = &[
    // Starting area enemies (level 1-5)
    EnemyData { key: "goblin", name: "Goblin", hp: 15, mp: 0, attack: 5, defense: 2, magic: 0, speed: 4, exp: 8, gold: 10, weakness: Element::None, resistance: Element::None, is_boss: false, description: "A small green pest." },
    EnemyData { key: "slime", name: "Slime", hp: 12, mp: 0, attack: 3, defense: 1, magic: 0, speed: 2, exp: 5, gold: 5, weakness: Element::Fire, resistance: Element::None, is_boss: false, description: "Gelatinous blob." },
    EnemyData { key: "wolf", name: "Wolf", hp: 20, mp: 0, attack: 8, defense: 3, magic: 0, speed: 8, exp: 12, gold: 12, weakness: Element::Fire, resistance: Element::None, is_boss: false, description: "Hungry predator." },
    EnemyData { key: "skeleton", name: "Skeleton", hp: 25, mp: 0, attack: 10, defense: 5, magic: 0, speed: 3, exp: 15, gold: 20, weakness: Element::Holy, resistance: Element::Dark, is_boss: false, description: "Animated bones." },
    EnemyData { key: "bat", name: "Vampire Bat", hp: 10, mp: 5, attack: 6, defense: 1, magic: 5, speed: 12, exp: 8, gold: 8, weakness: Element::Fire, resistance: Element::None, is_boss: false, description: "Bloodsucking wings." },

    // Mid-game enemies (level 6-15)
    EnemyData { key: "orc", name: "Orc", hp: 60, mp: 0, attack: 25, defense: 12, magic: 0, speed: 6, exp: 50, gold: 80, weakness: Element::None, resistance: Element::None, is_boss: false, description: "Brutal warrior." },
    EnemyData { key: "zombie", name: "Zombie", hp: 45, mp: 0, attack: 18, defense: 8, magic: 0, speed: 2, exp: 35, gold: 30, weakness: Element::Fire, resistance: Element::Dark, is_boss: false, description: "Walking dead." },
    EnemyData { key: "ghost", name: "Ghost", hp: 35, mp: 20, attack: 15, defense: 5, magic: 20, speed: 8, exp: 40, gold: 40, weakness: Element::Holy, resistance: Element::None, is_boss: false, description: "Ethereal specter." },
    EnemyData { key: "ogre", name: "Ogre", hp: 120, mp: 0, attack: 35, defense: 18, magic: 0, speed: 3, exp: 80, gold: 120, weakness: Element::None, resistance: Element::None, is_boss: false, description: "Massive brute." },
    EnemyData { key: "wyvern", name: "Wyvern", hp: 90, mp: 15, attack: 40, defense: 20, magic: 15, speed: 15, exp: 100, gold: 150, weakness: Element::Ice, resistance: Element::Fire, is_boss: false, description: "Flying serpent." },

    // Late-game enemies (level 16-30)
    EnemyData { key: "demon", name: "Demon", hp: 200, mp: 50, attack: 60, defense: 35, magic: 40, speed: 12, exp: 250, gold: 300, weakness: Element::Holy, resistance: Element::Dark, is_boss: false, description: "Hellspawn." },
    EnemyData { key: "dragon", name: "Dragon", hp: 350, mp: 80, attack: 80, defense: 50, magic: 60, speed: 10, exp: 500, gold: 600, weakness: Element::Ice, resistance: Element::Fire, is_boss: false, description: "Scaled terror." },
    EnemyData { key: "dark_knight", name: "Dark Knight", hp: 280, mp: 30, attack: 75, defense: 55, magic: 25, speed: 8, exp: 400, gold: 450, weakness: Element::Holy, resistance: Element::Dark, is_boss: false, description: "Corrupted warrior." },
    EnemyData { key: "lich", name: "Lich", hp: 220, mp: 150, attack: 40, defense: 30, magic: 80, speed: 6, exp: 450, gold: 500, weakness: Element::Holy, resistance: Element::Dark, is_boss: false, description: "Undead mage." },

    // Crystal Bosses
    EnemyData { key: "earth_fiend", name: "Lich - Earth Fiend", hp: 600, mp: 100, attack: 50, defense: 45, magic: 60, speed: 5, exp: 1500, gold: 2000, weakness: Element::Fire, resistance: Element::Dark, is_boss: true, description: "Guardian of Earth Crystal." },
    EnemyData { key: "fire_fiend", name: "Marilith - Fire Fiend", hp: 800, mp: 80, attack: 75, defense: 40, magic: 50, speed: 12, exp: 2500, gold: 3000, weakness: Element::Ice, resistance: Element::Fire, is_boss: true, description: "Guardian of Fire Crystal." },
    EnemyData { key: "water_fiend", name: "Kraken - Water Fiend", hp: 1000, mp: 120, attack: 65, defense: 50, magic: 70, speed: 8, exp: 3500, gold: 4000, weakness: Element::Lightning, resistance: Element::Ice, is_boss: true, description: "Guardian of Water Crystal." },
    EnemyData { key: "wind_fiend", name: "Tiamat - Wind Fiend", hp: 1200, mp: 150, attack: 85, defense: 55, magic: 80, speed: 15, exp: 5000, gold: 5000, weakness: Element::None, resistance: Element::None, is_boss: true, description: "Guardian of Wind Crystal." },

    // Final Boss
    EnemyData { key: "void_lord", name: "Void Lord", hp: 5000, mp: 500, attack: 120, defense: 80, magic: 100, speed: 20, exp: 0, gold: 0, weakness: Element::Holy, resistance: Element::Dark, is_boss: true, description: "The corrupted process." },
];

pub fn get_enemy(key: &str) -> Option<&'static EnemyData> {
    ENEMIES.iter().find(|e| e.key == key)
}

/// Get random enemies for an encounter based on area level
pub fn get_encounter_enemies(area_level: u8, count: usize) -> Vec<&'static EnemyData> {
    let mut rng = thread_rng();
    let eligible: Vec<_> = ENEMIES.iter()
        .filter(|e| !e.is_boss)
        .filter(|e| {
            let enemy_level = (e.exp / 10) as u8;
            enemy_level <= area_level + 3 && enemy_level >= area_level.saturating_sub(2)
        })
        .collect();

    if eligible.is_empty() {
        return vec![&ENEMIES[0]]; // Return goblin as fallback
    }

    (0..count.min(4))
        .map(|_| eligible[rng.gen_range(0..eligible.len())])
        .collect()
}

// ============================================================================
// SIMULATION BREADCRUMBS
// ============================================================================

/// Very rare hints about the simulation nature of the world
pub static SIMULATION_HINTS: &[&str] = &[
    // NPC glitches (very rare)
    "...where was I? Sometimes I feel like I've said this before.",
    "The sky... for a moment, I thought I saw a grid pattern.",
    "What's a 'computer'? I dreamed that word last night.",
    "The Architects who dreamed the world...",
    "ERROR: FILE NOT FOUND",

    // Anachronistic slips
    "I mean... the message stone. The... tel-e-graph? No, that's not right.",
    "We should reboot the-- I mean, restart the ritual.",
    "The process is corrupting the-- the Void! The Void spreads!",
];

/// Get a rare simulation hint (very low chance)
pub fn maybe_get_simulation_hint() -> Option<&'static str> {
    let mut rng = thread_rng();
    // 1 in 500 chance per check
    if rng.gen_range(0..500) == 0 {
        Some(SIMULATION_HINTS[rng.gen_range(0..SIMULATION_HINTS.len())])
    } else {
        None
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_base_stats() {
        let (hp, _, str, _, _, _, _) = ClassType::Warrior.base_stats();
        assert!(hp > 30);
        assert!(str > 10);
    }

    #[test]
    fn test_spell_lookup() {
        assert!(get_spell("fire").is_some());
        assert!(get_spell("cure").is_some());
        assert!(get_spell("nonexistent").is_none());
    }

    #[test]
    fn test_equipment_lookup() {
        assert!(get_equipment("wooden_sword").is_some());
        assert!(get_equipment("chain_mail").is_some());
    }

    #[test]
    fn test_item_lookup() {
        assert!(get_item("potion").is_some());
        assert!(get_item("elixir").is_some());
    }

    #[test]
    fn test_enemy_lookup() {
        assert!(get_enemy("goblin").is_some());
        assert!(get_enemy("void_lord").is_some());
    }

    #[test]
    fn test_class_equipment_restrictions() {
        assert!(ClassType::Warrior.can_equip(EquipmentType::Sword));
        assert!(!ClassType::Mage.can_equip(EquipmentType::HeavyArmor));
        assert!(ClassType::Mage.can_equip(EquipmentType::Staff));
    }

    #[test]
    fn test_class_spells() {
        let mage_spells = ClassType::Mage.spells_at_level(10);
        assert!(mage_spells.contains(&"fire"));
        assert!(mage_spells.contains(&"fira"));

        let warrior_spells = ClassType::Warrior.spells_at_level(10);
        assert!(warrior_spells.is_empty());
    }

    #[test]
    fn test_encounter_generation() {
        let enemies = get_encounter_enemies(5, 3);
        assert!(!enemies.is_empty());
        assert!(enemies.len() <= 4);
    }
}
