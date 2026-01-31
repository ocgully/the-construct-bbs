//! Item system for Depths of Diablo
//!
//! Implements randomized loot with affixes and rarity tiers.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Item rarity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,     // White - no affixes
    Magic,      // Blue - 1 affix
    Rare,       // Yellow - 2-3 affixes
    Unique,     // Gold - special named items
}

impl ItemRarity {
    pub fn name(&self) -> &'static str {
        match self {
            ItemRarity::Common => "Common",
            ItemRarity::Magic => "Magic",
            ItemRarity::Rare => "Rare",
            ItemRarity::Unique => "Unique",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            ItemRarity::Common => "white",
            ItemRarity::Magic => "blue",
            ItemRarity::Rare => "yellow",
            ItemRarity::Unique => "gold",
        }
    }

    /// Roll rarity based on floor depth
    pub fn roll(floor: u32, luck_bonus: i32) -> ItemRarity {
        let mut rng = rand::thread_rng();
        let roll: i32 = rng.gen_range(0..100) + luck_bonus + (floor as i32 * 2);

        if roll >= 95 {
            ItemRarity::Unique
        } else if roll >= 75 {
            ItemRarity::Rare
        } else if roll >= 40 {
            ItemRarity::Magic
        } else {
            ItemRarity::Common
        }
    }
}

/// Item types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    // Weapons
    Sword,
    Axe,
    Mace,
    Bow,
    Staff,
    Dagger,
    // Armor
    Helm,
    Chest,
    Gloves,
    Boots,
    Shield,
    // Jewelry
    Ring,
    Amulet,
    // Consumables
    HealthPotion,
    ManaPotion,
    TownPortal,
}

impl ItemType {
    pub fn name(&self) -> &'static str {
        match self {
            ItemType::Sword => "Sword",
            ItemType::Axe => "Axe",
            ItemType::Mace => "Mace",
            ItemType::Bow => "Bow",
            ItemType::Staff => "Staff",
            ItemType::Dagger => "Dagger",
            ItemType::Helm => "Helm",
            ItemType::Chest => "Armor",
            ItemType::Gloves => "Gloves",
            ItemType::Boots => "Boots",
            ItemType::Shield => "Shield",
            ItemType::Ring => "Ring",
            ItemType::Amulet => "Amulet",
            ItemType::HealthPotion => "Health Potion",
            ItemType::ManaPotion => "Mana Potion",
            ItemType::TownPortal => "Town Portal",
        }
    }

    pub fn is_weapon(&self) -> bool {
        matches!(
            self,
            ItemType::Sword
                | ItemType::Axe
                | ItemType::Mace
                | ItemType::Bow
                | ItemType::Staff
                | ItemType::Dagger
        )
    }

    pub fn is_armor(&self) -> bool {
        matches!(
            self,
            ItemType::Helm
                | ItemType::Chest
                | ItemType::Gloves
                | ItemType::Boots
                | ItemType::Shield
        )
    }

    pub fn is_jewelry(&self) -> bool {
        matches!(self, ItemType::Ring | ItemType::Amulet)
    }

    pub fn is_consumable(&self) -> bool {
        matches!(
            self,
            ItemType::HealthPotion | ItemType::ManaPotion | ItemType::TownPortal
        )
    }

    pub fn slot(&self) -> Option<EquipSlot> {
        match self {
            ItemType::Sword | ItemType::Axe | ItemType::Mace | ItemType::Dagger => {
                Some(EquipSlot::MainHand)
            }
            ItemType::Bow | ItemType::Staff => Some(EquipSlot::TwoHand),
            ItemType::Shield => Some(EquipSlot::OffHand),
            ItemType::Helm => Some(EquipSlot::Head),
            ItemType::Chest => Some(EquipSlot::Body),
            ItemType::Gloves => Some(EquipSlot::Hands),
            ItemType::Boots => Some(EquipSlot::Feet),
            ItemType::Ring => Some(EquipSlot::Ring),
            ItemType::Amulet => Some(EquipSlot::Amulet),
            _ => None,
        }
    }

    pub fn base_damage(&self, floor: u32) -> i32 {
        let base = match self {
            ItemType::Sword => 10,
            ItemType::Axe => 12,
            ItemType::Mace => 11,
            ItemType::Bow => 8,
            ItemType::Staff => 6,
            ItemType::Dagger => 7,
            _ => 0,
        };
        base + (floor as i32 * 2)
    }

    pub fn base_armor(&self, floor: u32) -> i32 {
        let base = match self {
            ItemType::Helm => 5,
            ItemType::Chest => 15,
            ItemType::Gloves => 3,
            ItemType::Boots => 4,
            ItemType::Shield => 10,
            _ => 0,
        };
        base + (floor as i32)
    }
}

/// Equipment slots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipSlot {
    MainHand,
    OffHand,
    TwoHand,
    Head,
    Body,
    Hands,
    Feet,
    Ring,
    Amulet,
}

/// Item affix (prefix or suffix modifier)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affix {
    pub name: String,
    pub stat: AffixStat,
    pub value: i32,
    pub is_prefix: bool,
}

/// Stats that affixes can modify
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffixStat {
    Damage,
    Armor,
    Health,
    Mana,
    Strength,
    Dexterity,
    Intelligence,
    Vitality,
    CritChance,
    AttackSpeed,
    LifeSteal,
    ManaSteal,
    FireDamage,
    ColdDamage,
    LightningDamage,
    AllResist,
}

impl AffixStat {
    pub fn name(&self) -> &'static str {
        match self {
            AffixStat::Damage => "Damage",
            AffixStat::Armor => "Armor",
            AffixStat::Health => "Health",
            AffixStat::Mana => "Mana",
            AffixStat::Strength => "Strength",
            AffixStat::Dexterity => "Dexterity",
            AffixStat::Intelligence => "Intelligence",
            AffixStat::Vitality => "Vitality",
            AffixStat::CritChance => "Critical Chance",
            AffixStat::AttackSpeed => "Attack Speed",
            AffixStat::LifeSteal => "Life Steal",
            AffixStat::ManaSteal => "Mana Steal",
            AffixStat::FireDamage => "Fire Damage",
            AffixStat::ColdDamage => "Cold Damage",
            AffixStat::LightningDamage => "Lightning Damage",
            AffixStat::AllResist => "All Resist",
        }
    }
}

/// Prefix definitions
const PREFIXES: &[(&str, AffixStat, i32, i32)] = &[
    ("Sharp", AffixStat::Damage, 3, 10),
    ("Deadly", AffixStat::Damage, 8, 20),
    ("Vicious", AffixStat::Damage, 15, 35),
    ("Sturdy", AffixStat::Armor, 5, 15),
    ("Strong", AffixStat::Armor, 10, 25),
    ("Fortified", AffixStat::Armor, 20, 40),
    ("Healthy", AffixStat::Health, 10, 30),
    ("Vigorous", AffixStat::Health, 25, 60),
    ("Arcane", AffixStat::Mana, 10, 25),
    ("Mystic", AffixStat::Mana, 20, 50),
    ("Warrior's", AffixStat::Strength, 3, 8),
    ("Hunter's", AffixStat::Dexterity, 3, 8),
    ("Wizard's", AffixStat::Intelligence, 3, 8),
    ("Flaming", AffixStat::FireDamage, 5, 15),
    ("Frozen", AffixStat::ColdDamage, 5, 15),
    ("Shocking", AffixStat::LightningDamage, 5, 15),
];

/// Suffix definitions
const SUFFIXES: &[(&str, AffixStat, i32, i32)] = &[
    ("of Speed", AffixStat::AttackSpeed, 5, 15),
    ("of Swiftness", AffixStat::AttackSpeed, 10, 25),
    ("of the Leech", AffixStat::LifeSteal, 2, 5),
    ("of the Vampire", AffixStat::LifeSteal, 5, 10),
    ("of Precision", AffixStat::CritChance, 3, 8),
    ("of Slaughter", AffixStat::CritChance, 6, 15),
    ("of the Bear", AffixStat::Strength, 5, 12),
    ("of the Fox", AffixStat::Dexterity, 5, 12),
    ("of the Owl", AffixStat::Intelligence, 5, 12),
    ("of the Titan", AffixStat::Vitality, 5, 12),
    ("of Warding", AffixStat::AllResist, 5, 15),
    ("of Protection", AffixStat::AllResist, 10, 25),
];

/// Generate a random affix
fn generate_affix(floor: u32, is_prefix: bool) -> Affix {
    let mut rng = rand::thread_rng();
    let pool = if is_prefix { PREFIXES } else { SUFFIXES };
    let (name, stat, min_val, max_val) = pool[rng.gen_range(0..pool.len())];

    // Scale values with floor
    let floor_bonus = floor as i32 / 5;
    let value = rng.gen_range(min_val..=max_val) + floor_bonus;

    Affix {
        name: name.to_string(),
        stat,
        value,
        is_prefix,
    }
}

/// A game item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub item_type: ItemType,
    pub rarity: ItemRarity,
    pub name: String,
    pub base_damage: i32,
    pub base_armor: i32,
    pub affixes: Vec<Affix>,
    pub level_req: u32,
    pub floor_found: u32,
}

impl Item {
    /// Generate a random item for a given floor
    pub fn generate(id: u64, floor: u32, luck_bonus: i32) -> Self {
        let mut rng = rand::thread_rng();

        // Choose item type
        let item_types = [
            ItemType::Sword,
            ItemType::Axe,
            ItemType::Mace,
            ItemType::Bow,
            ItemType::Staff,
            ItemType::Dagger,
            ItemType::Helm,
            ItemType::Chest,
            ItemType::Gloves,
            ItemType::Boots,
            ItemType::Shield,
            ItemType::Ring,
            ItemType::Amulet,
        ];
        let item_type = item_types[rng.gen_range(0..item_types.len())];

        // Roll rarity
        let rarity = ItemRarity::roll(floor, luck_bonus);

        // Generate affixes based on rarity
        let affix_count = match rarity {
            ItemRarity::Common => 0,
            ItemRarity::Magic => 1,
            ItemRarity::Rare => rng.gen_range(2..=3),
            ItemRarity::Unique => rng.gen_range(3..=4),
        };

        let mut affixes = Vec::new();
        let mut has_prefix = false;
        let mut has_suffix = false;

        for _ in 0..affix_count {
            // Alternate between prefix and suffix
            let is_prefix = if !has_prefix {
                has_prefix = true;
                true
            } else if !has_suffix {
                has_suffix = true;
                false
            } else {
                rng.gen_bool(0.5)
            };
            affixes.push(generate_affix(floor, is_prefix));
        }

        // Generate name
        let name = Self::generate_name(item_type, &affixes, rarity);

        // Calculate base stats
        let base_damage = item_type.base_damage(floor);
        let base_armor = item_type.base_armor(floor);

        // Level requirement
        let level_req = (floor / 2).max(1);

        Item {
            id,
            item_type,
            rarity,
            name,
            base_damage,
            base_armor,
            affixes,
            level_req,
            floor_found: floor,
        }
    }

    /// Generate a potion
    pub fn generate_potion(id: u64, is_health: bool) -> Self {
        Item {
            id,
            item_type: if is_health {
                ItemType::HealthPotion
            } else {
                ItemType::ManaPotion
            },
            rarity: ItemRarity::Common,
            name: if is_health {
                "Health Potion".to_string()
            } else {
                "Mana Potion".to_string()
            },
            base_damage: 0,
            base_armor: 0,
            affixes: Vec::new(),
            level_req: 1,
            floor_found: 1,
        }
    }

    fn generate_name(item_type: ItemType, affixes: &[Affix], rarity: ItemRarity) -> String {
        let prefix = affixes
            .iter()
            .find(|a| a.is_prefix)
            .map(|a| a.name.as_str())
            .unwrap_or("");

        let suffix = affixes
            .iter()
            .find(|a| !a.is_prefix)
            .map(|a| a.name.as_str())
            .unwrap_or("");

        let base_name = match rarity {
            ItemRarity::Unique => format!("{} {}", "Legendary", item_type.name()),
            _ => item_type.name().to_string(),
        };

        let mut name = String::new();
        if !prefix.is_empty() {
            name.push_str(prefix);
            name.push(' ');
        }
        name.push_str(&base_name);
        if !suffix.is_empty() {
            name.push(' ');
            name.push_str(suffix);
        }

        name
    }

    /// Get total damage including affixes
    pub fn total_damage(&self) -> i32 {
        let affix_bonus: i32 = self
            .affixes
            .iter()
            .filter(|a| matches!(a.stat, AffixStat::Damage | AffixStat::FireDamage | AffixStat::ColdDamage | AffixStat::LightningDamage))
            .map(|a| a.value)
            .sum();
        self.base_damage + affix_bonus
    }

    /// Get total armor including affixes
    pub fn total_armor(&self) -> i32 {
        let affix_bonus: i32 = self
            .affixes
            .iter()
            .filter(|a| matches!(a.stat, AffixStat::Armor))
            .map(|a| a.value)
            .sum();
        self.base_armor + affix_bonus
    }

    /// Get stat bonus from affixes
    pub fn get_stat_bonus(&self, stat: AffixStat) -> i32 {
        self.affixes
            .iter()
            .filter(|a| a.stat == stat)
            .map(|a| a.value)
            .sum()
    }

    /// Check if item can be equipped
    pub fn can_equip(&self) -> bool {
        self.item_type.slot().is_some()
    }

    /// Get equipment slot
    pub fn slot(&self) -> Option<EquipSlot> {
        self.item_type.slot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarity_roll() {
        // With high floor and luck, should get better items
        let mut uniques = 0;
        for _ in 0..100 {
            let rarity = ItemRarity::roll(20, 50);
            if rarity == ItemRarity::Unique {
                uniques += 1;
            }
        }
        // Should get some uniques with high bonuses
        assert!(uniques > 0);
    }

    #[test]
    fn test_item_generation() {
        let item = Item::generate(1, 5, 0);
        assert!(item.level_req >= 1);
        assert!(item.floor_found == 5);
    }

    #[test]
    fn test_affix_count_by_rarity() {
        // Generate many items and check affix counts
        for _ in 0..50 {
            let item = Item::generate(1, 10, 100); // High luck for rares/uniques
            match item.rarity {
                ItemRarity::Common => assert_eq!(item.affixes.len(), 0),
                ItemRarity::Magic => assert_eq!(item.affixes.len(), 1),
                ItemRarity::Rare => assert!(item.affixes.len() >= 2 && item.affixes.len() <= 3),
                ItemRarity::Unique => assert!(item.affixes.len() >= 3),
            }
        }
    }

    #[test]
    fn test_item_slot() {
        assert_eq!(ItemType::Sword.slot(), Some(EquipSlot::MainHand));
        assert_eq!(ItemType::Bow.slot(), Some(EquipSlot::TwoHand));
        assert_eq!(ItemType::Helm.slot(), Some(EquipSlot::Head));
        assert_eq!(ItemType::HealthPotion.slot(), None);
    }

    #[test]
    fn test_potion_generation() {
        let hp = Item::generate_potion(1, true);
        assert_eq!(hp.item_type, ItemType::HealthPotion);

        let mp = Item::generate_potion(2, false);
        assert_eq!(mp.item_type, ItemType::ManaPotion);
    }
}
