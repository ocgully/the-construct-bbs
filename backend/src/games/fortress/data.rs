//! Static game data for Fortress
//!
//! Defines skills, workshops, room types, resources, and enemies.


/// Skill definition
#[derive(Debug, Clone)]
pub struct SkillDef {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

/// Workshop definition
#[derive(Debug, Clone)]
pub struct WorkshopDef {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub required_skill: &'static str,
    pub build_cost: ResourceCost,
    pub size: (u8, u8), // width, height
}

/// Room type definition
#[derive(Debug, Clone)]
pub struct RoomTypeDef {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub min_size: u32,
    pub required_furniture: &'static [&'static str],
    pub effect: &'static str,
}

/// Enemy type definition
#[derive(Debug, Clone)]
pub struct EnemyDef {
    pub key: &'static str,
    pub name: &'static str,
    pub health: u32,
    pub attack: u32,
    pub defense: u32,
    pub loot: &'static [(&'static str, u32)], // (resource, amount)
    pub threat_level: u8, // 1-5
}

/// Resource cost for crafting/building
#[derive(Debug, Clone, Default)]
pub struct ResourceCost {
    pub wood: u32,
    pub stone: u32,
    pub iron: u32,
    pub gold: u32,
    pub food: u32,
    pub drink: u32,
    pub cloth: u32,
    pub leather: u32,
}

impl ResourceCost {
    pub const fn new() -> Self {
        Self {
            wood: 0,
            stone: 0,
            iron: 0,
            gold: 0,
            food: 0,
            drink: 0,
            cloth: 0,
            leather: 0,
        }
    }

    pub const fn wood(mut self, amount: u32) -> Self {
        self.wood = amount;
        self
    }

    pub const fn stone(mut self, amount: u32) -> Self {
        self.stone = amount;
        self
    }

    pub const fn iron(mut self, amount: u32) -> Self {
        self.iron = amount;
        self
    }

    pub const fn gold(mut self, amount: u32) -> Self {
        self.gold = amount;
        self
    }

    pub const fn food(mut self, amount: u32) -> Self {
        self.food = amount;
        self
    }

    pub const fn drink(mut self, amount: u32) -> Self {
        self.drink = amount;
        self
    }

    pub const fn cloth(mut self, amount: u32) -> Self {
        self.cloth = amount;
        self
    }

    pub const fn leather(mut self, amount: u32) -> Self {
        self.leather = amount;
        self
    }
}

/// All available skills
pub static SKILLS: &[SkillDef] = &[
    SkillDef { key: "mining", name: "Mining", description: "Dig through rock and extract ore" },
    SkillDef { key: "woodcutting", name: "Woodcutting", description: "Fell trees for lumber" },
    SkillDef { key: "farming", name: "Farming", description: "Grow crops and tend fields" },
    SkillDef { key: "crafting", name: "Crafting", description: "Create items at workshops" },
    SkillDef { key: "cooking", name: "Cooking", description: "Prepare food and brew drinks" },
    SkillDef { key: "building", name: "Building", description: "Construct walls and furniture" },
    SkillDef { key: "combat", name: "Combat", description: "Fight enemies and defend the fortress" },
    SkillDef { key: "hauling", name: "Hauling", description: "Transport items between stockpiles" },
    SkillDef { key: "masonry", name: "Masonry", description: "Work with stone and gems" },
    SkillDef { key: "smithing", name: "Smithing", description: "Forge metal tools and weapons" },
    SkillDef { key: "brewing", name: "Brewing", description: "Create alcoholic beverages" },
    SkillDef { key: "healing", name: "Healing", description: "Treat injuries and illness" },
];

/// All available workshops
pub static WORKSHOPS: &[WorkshopDef] = &[
    WorkshopDef {
        key: "carpenter",
        name: "Carpenter's Workshop",
        description: "Craft wooden items and furniture",
        required_skill: "crafting",
        build_cost: ResourceCost::new().wood(30),
        size: (3, 3),
    },
    WorkshopDef {
        key: "mason",
        name: "Mason's Workshop",
        description: "Craft stone items and blocks",
        required_skill: "masonry",
        build_cost: ResourceCost::new().stone(30),
        size: (3, 3),
    },
    WorkshopDef {
        key: "smelter",
        name: "Smelter",
        description: "Smelt ore into metal bars",
        required_skill: "smithing",
        build_cost: ResourceCost::new().stone(20).wood(10),
        size: (3, 3),
    },
    WorkshopDef {
        key: "forge",
        name: "Metalsmith's Forge",
        description: "Craft metal items, weapons, and armor",
        required_skill: "smithing",
        build_cost: ResourceCost::new().stone(30).iron(10),
        size: (3, 3),
    },
    WorkshopDef {
        key: "kitchen",
        name: "Kitchen",
        description: "Prepare meals from raw ingredients",
        required_skill: "cooking",
        build_cost: ResourceCost::new().stone(20).wood(10),
        size: (3, 3),
    },
    WorkshopDef {
        key: "still",
        name: "Still",
        description: "Brew alcoholic beverages",
        required_skill: "brewing",
        build_cost: ResourceCost::new().wood(30).stone(10),
        size: (2, 2),
    },
    WorkshopDef {
        key: "craftsdwarf",
        name: "Craftsdwarf's Workshop",
        description: "Create crafts and trade goods",
        required_skill: "crafting",
        build_cost: ResourceCost::new().wood(20).stone(10),
        size: (3, 3),
    },
    WorkshopDef {
        key: "loom",
        name: "Loom",
        description: "Weave cloth from plant fibers",
        required_skill: "crafting",
        build_cost: ResourceCost::new().wood(30),
        size: (3, 3),
    },
    WorkshopDef {
        key: "tannery",
        name: "Tannery",
        description: "Process hides into leather",
        required_skill: "crafting",
        build_cost: ResourceCost::new().wood(20).stone(10),
        size: (3, 3),
    },
];

/// All room types
pub static ROOM_TYPES: &[RoomTypeDef] = &[
    RoomTypeDef {
        key: "bedroom",
        name: "Bedroom",
        description: "Private sleeping quarters for a dwarf",
        min_size: 4,
        required_furniture: &["bed"],
        effect: "rest_bonus",
    },
    RoomTypeDef {
        key: "dormitory",
        name: "Dormitory",
        description: "Shared sleeping quarters",
        min_size: 16,
        required_furniture: &["bed", "bed", "bed"],
        effect: "rest_basic",
    },
    RoomTypeDef {
        key: "dining",
        name: "Dining Hall",
        description: "Communal eating area",
        min_size: 25,
        required_furniture: &["table", "chair"],
        effect: "mood_bonus",
    },
    RoomTypeDef {
        key: "meeting",
        name: "Meeting Hall",
        description: "Social gathering space",
        min_size: 36,
        required_furniture: &["throne"],
        effect: "social_bonus",
    },
    RoomTypeDef {
        key: "hospital",
        name: "Hospital",
        description: "Medical treatment area",
        min_size: 16,
        required_furniture: &["bed", "table"],
        effect: "healing_bonus",
    },
    RoomTypeDef {
        key: "stockpile",
        name: "Stockpile",
        description: "Storage area for resources",
        min_size: 9,
        required_furniture: &[],
        effect: "storage",
    },
    RoomTypeDef {
        key: "barracks",
        name: "Barracks",
        description: "Military training area",
        min_size: 16,
        required_furniture: &["weapon_rack", "armor_stand"],
        effect: "combat_training",
    },
    RoomTypeDef {
        key: "throne_room",
        name: "Throne Room",
        description: "The fortress leader's audience chamber",
        min_size: 49,
        required_furniture: &["throne", "table"],
        effect: "leadership_bonus",
    },
];

/// Resource definitions
#[derive(Debug, Clone)]
pub struct ResourceDef {
    pub key: &'static str,
    pub name: &'static str,
    pub category: ResourceCategory,
    pub base_value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceCategory {
    Raw,
    Processed,
    Food,
    Drink,
    Crafted,
}

pub static RESOURCES: &[ResourceDef] = &[
    // Raw materials
    ResourceDef { key: "wood", name: "Wood", category: ResourceCategory::Raw, base_value: 5 },
    ResourceDef { key: "stone", name: "Stone", category: ResourceCategory::Raw, base_value: 3 },
    ResourceDef { key: "iron_ore", name: "Iron Ore", category: ResourceCategory::Raw, base_value: 10 },
    ResourceDef { key: "copper_ore", name: "Copper Ore", category: ResourceCategory::Raw, base_value: 8 },
    ResourceDef { key: "gold_ore", name: "Gold Ore", category: ResourceCategory::Raw, base_value: 50 },
    ResourceDef { key: "gem", name: "Rough Gem", category: ResourceCategory::Raw, base_value: 100 },
    ResourceDef { key: "plant_fiber", name: "Plant Fiber", category: ResourceCategory::Raw, base_value: 2 },
    ResourceDef { key: "hide", name: "Hide", category: ResourceCategory::Raw, base_value: 15 },

    // Processed materials
    ResourceDef { key: "iron", name: "Iron Bar", category: ResourceCategory::Processed, base_value: 25 },
    ResourceDef { key: "copper", name: "Copper Bar", category: ResourceCategory::Processed, base_value: 20 },
    ResourceDef { key: "gold", name: "Gold Bar", category: ResourceCategory::Processed, base_value: 150 },
    ResourceDef { key: "cut_gem", name: "Cut Gem", category: ResourceCategory::Processed, base_value: 200 },
    ResourceDef { key: "cloth", name: "Cloth", category: ResourceCategory::Processed, base_value: 10 },
    ResourceDef { key: "leather", name: "Leather", category: ResourceCategory::Processed, base_value: 30 },
    ResourceDef { key: "plank", name: "Wooden Plank", category: ResourceCategory::Processed, base_value: 10 },
    ResourceDef { key: "block", name: "Stone Block", category: ResourceCategory::Processed, base_value: 8 },

    // Food
    ResourceDef { key: "meat", name: "Meat", category: ResourceCategory::Food, base_value: 10 },
    ResourceDef { key: "fish", name: "Fish", category: ResourceCategory::Food, base_value: 8 },
    ResourceDef { key: "vegetable", name: "Vegetables", category: ResourceCategory::Food, base_value: 5 },
    ResourceDef { key: "grain", name: "Grain", category: ResourceCategory::Food, base_value: 3 },
    ResourceDef { key: "meal", name: "Prepared Meal", category: ResourceCategory::Food, base_value: 20 },
    ResourceDef { key: "plump_helmet", name: "Plump Helmet", category: ResourceCategory::Food, base_value: 5 },

    // Drinks
    ResourceDef { key: "water", name: "Water", category: ResourceCategory::Drink, base_value: 1 },
    ResourceDef { key: "ale", name: "Dwarven Ale", category: ResourceCategory::Drink, base_value: 15 },
    ResourceDef { key: "wine", name: "Wine", category: ResourceCategory::Drink, base_value: 20 },
    ResourceDef { key: "mead", name: "Mead", category: ResourceCategory::Drink, base_value: 25 },

    // Crafted goods
    ResourceDef { key: "furniture", name: "Furniture", category: ResourceCategory::Crafted, base_value: 30 },
    ResourceDef { key: "tool", name: "Tool", category: ResourceCategory::Crafted, base_value: 25 },
    ResourceDef { key: "weapon", name: "Weapon", category: ResourceCategory::Crafted, base_value: 50 },
    ResourceDef { key: "armor", name: "Armor", category: ResourceCategory::Crafted, base_value: 75 },
    ResourceDef { key: "craft", name: "Craft", category: ResourceCategory::Crafted, base_value: 40 },
];

/// Enemy types for invasions
pub static ENEMIES: &[EnemyDef] = &[
    EnemyDef {
        key: "goblin",
        name: "Goblin",
        health: 20,
        attack: 5,
        defense: 2,
        loot: &[("gold_ore", 5)],
        threat_level: 1,
    },
    EnemyDef {
        key: "goblin_warrior",
        name: "Goblin Warrior",
        health: 35,
        attack: 10,
        defense: 5,
        loot: &[("iron", 2), ("weapon", 1)],
        threat_level: 2,
    },
    EnemyDef {
        key: "troll",
        name: "Cave Troll",
        health: 80,
        attack: 20,
        defense: 10,
        loot: &[("hide", 5), ("gem", 1)],
        threat_level: 3,
    },
    EnemyDef {
        key: "forgotten_beast",
        name: "Forgotten Beast",
        health: 150,
        attack: 35,
        defense: 20,
        loot: &[("gem", 3), ("gold", 5)],
        threat_level: 4,
    },
    EnemyDef {
        key: "dragon",
        name: "Dragon",
        health: 300,
        attack: 50,
        defense: 30,
        loot: &[("gold", 20), ("gem", 10), ("cut_gem", 5)],
        threat_level: 5,
    },
    EnemyDef {
        key: "undead_dwarf",
        name: "Undead Dwarf",
        health: 25,
        attack: 8,
        defense: 3,
        loot: &[("armor", 1)],
        threat_level: 2,
    },
    EnemyDef {
        key: "giant_spider",
        name: "Giant Spider",
        health: 40,
        attack: 15,
        defense: 5,
        loot: &[("plant_fiber", 10)],
        threat_level: 2,
    },
    EnemyDef {
        key: "siege_leader",
        name: "Goblin Siege Leader",
        health: 60,
        attack: 25,
        defense: 15,
        loot: &[("gold", 10), ("weapon", 2), ("armor", 1)],
        threat_level: 3,
    },
];

/// Furniture definitions for rooms
#[derive(Debug, Clone)]
pub struct FurnitureDef {
    pub key: &'static str,
    pub name: &'static str,
    pub build_cost: ResourceCost,
    pub comfort_bonus: i32,
}

pub static FURNITURE: &[FurnitureDef] = &[
    FurnitureDef { key: "bed", name: "Bed", build_cost: ResourceCost::new().wood(10), comfort_bonus: 5 },
    FurnitureDef { key: "table", name: "Table", build_cost: ResourceCost::new().wood(5), comfort_bonus: 2 },
    FurnitureDef { key: "chair", name: "Chair", build_cost: ResourceCost::new().wood(3), comfort_bonus: 2 },
    FurnitureDef { key: "throne", name: "Throne", build_cost: ResourceCost::new().stone(20).gold(5), comfort_bonus: 10 },
    FurnitureDef { key: "cabinet", name: "Cabinet", build_cost: ResourceCost::new().wood(8), comfort_bonus: 1 },
    FurnitureDef { key: "door", name: "Door", build_cost: ResourceCost::new().wood(5), comfort_bonus: 0 },
    FurnitureDef { key: "weapon_rack", name: "Weapon Rack", build_cost: ResourceCost::new().wood(10).iron(5), comfort_bonus: 0 },
    FurnitureDef { key: "armor_stand", name: "Armor Stand", build_cost: ResourceCost::new().wood(8).iron(3), comfort_bonus: 0 },
    FurnitureDef { key: "statue", name: "Statue", build_cost: ResourceCost::new().stone(30), comfort_bonus: 5 },
    FurnitureDef { key: "coffin", name: "Coffin", build_cost: ResourceCost::new().stone(15), comfort_bonus: 0 },
];

/// Get a skill by key
pub fn get_skill(key: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.key == key)
}

/// Get a workshop by key
pub fn get_workshop(key: &str) -> Option<&'static WorkshopDef> {
    WORKSHOPS.iter().find(|w| w.key == key)
}

/// Get a room type by key
pub fn get_room_type(key: &str) -> Option<&'static RoomTypeDef> {
    ROOM_TYPES.iter().find(|r| r.key == key)
}

/// Get an enemy by key
pub fn get_enemy(key: &str) -> Option<&'static EnemyDef> {
    ENEMIES.iter().find(|e| e.key == key)
}

/// Get a resource by key
pub fn get_resource(key: &str) -> Option<&'static ResourceDef> {
    RESOURCES.iter().find(|r| r.key == key)
}

/// Get a furniture by key
pub fn get_furniture(key: &str) -> Option<&'static FurnitureDef> {
    FURNITURE.iter().find(|f| f.key == key)
}

/// Crafting recipe
#[derive(Debug, Clone)]
pub struct Recipe {
    pub key: &'static str,
    pub name: &'static str,
    pub workshop: &'static str,
    pub skill: &'static str,
    pub skill_level: u8,
    pub inputs: &'static [(&'static str, u32)],
    pub outputs: &'static [(&'static str, u32)],
    pub work_time: u32, // in ticks
}

pub static RECIPES: &[Recipe] = &[
    // Smelting
    Recipe {
        key: "smelt_iron",
        name: "Smelt Iron",
        workshop: "smelter",
        skill: "smithing",
        skill_level: 1,
        inputs: &[("iron_ore", 2), ("wood", 1)],
        outputs: &[("iron", 1)],
        work_time: 5,
    },
    Recipe {
        key: "smelt_copper",
        name: "Smelt Copper",
        workshop: "smelter",
        skill: "smithing",
        skill_level: 1,
        inputs: &[("copper_ore", 2), ("wood", 1)],
        outputs: &[("copper", 1)],
        work_time: 4,
    },
    Recipe {
        key: "smelt_gold",
        name: "Smelt Gold",
        workshop: "smelter",
        skill: "smithing",
        skill_level: 3,
        inputs: &[("gold_ore", 2), ("wood", 2)],
        outputs: &[("gold", 1)],
        work_time: 8,
    },

    // Carpentry
    Recipe {
        key: "make_plank",
        name: "Make Planks",
        workshop: "carpenter",
        skill: "crafting",
        skill_level: 1,
        inputs: &[("wood", 1)],
        outputs: &[("plank", 4)],
        work_time: 2,
    },
    Recipe {
        key: "make_bed",
        name: "Make Bed",
        workshop: "carpenter",
        skill: "crafting",
        skill_level: 2,
        inputs: &[("plank", 3), ("cloth", 1)],
        outputs: &[("furniture", 1)],
        work_time: 5,
    },
    Recipe {
        key: "make_barrel",
        name: "Make Barrel",
        workshop: "carpenter",
        skill: "crafting",
        skill_level: 1,
        inputs: &[("plank", 2)],
        outputs: &[("furniture", 1)],
        work_time: 3,
    },

    // Masonry
    Recipe {
        key: "cut_blocks",
        name: "Cut Stone Blocks",
        workshop: "mason",
        skill: "masonry",
        skill_level: 1,
        inputs: &[("stone", 2)],
        outputs: &[("block", 4)],
        work_time: 3,
    },
    Recipe {
        key: "cut_gem",
        name: "Cut Gem",
        workshop: "mason",
        skill: "masonry",
        skill_level: 4,
        inputs: &[("gem", 1)],
        outputs: &[("cut_gem", 1)],
        work_time: 10,
    },

    // Forge
    Recipe {
        key: "forge_weapon",
        name: "Forge Weapon",
        workshop: "forge",
        skill: "smithing",
        skill_level: 2,
        inputs: &[("iron", 2), ("wood", 1)],
        outputs: &[("weapon", 1)],
        work_time: 8,
    },
    Recipe {
        key: "forge_armor",
        name: "Forge Armor",
        workshop: "forge",
        skill: "smithing",
        skill_level: 3,
        inputs: &[("iron", 4), ("leather", 1)],
        outputs: &[("armor", 1)],
        work_time: 12,
    },
    Recipe {
        key: "forge_tool",
        name: "Forge Tool",
        workshop: "forge",
        skill: "smithing",
        skill_level: 1,
        inputs: &[("iron", 1), ("wood", 1)],
        outputs: &[("tool", 1)],
        work_time: 5,
    },

    // Kitchen
    Recipe {
        key: "prepare_meal",
        name: "Prepare Meal",
        workshop: "kitchen",
        skill: "cooking",
        skill_level: 1,
        inputs: &[("meat", 1), ("vegetable", 1)],
        outputs: &[("meal", 2)],
        work_time: 3,
    },
    Recipe {
        key: "cook_roast",
        name: "Cook Roast",
        workshop: "kitchen",
        skill: "cooking",
        skill_level: 2,
        inputs: &[("meat", 2)],
        outputs: &[("meal", 3)],
        work_time: 5,
    },

    // Still
    Recipe {
        key: "brew_ale",
        name: "Brew Ale",
        workshop: "still",
        skill: "brewing",
        skill_level: 1,
        inputs: &[("grain", 3), ("water", 1)],
        outputs: &[("ale", 5)],
        work_time: 6,
    },
    Recipe {
        key: "brew_wine",
        name: "Brew Wine",
        workshop: "still",
        skill: "brewing",
        skill_level: 2,
        inputs: &[("plump_helmet", 5), ("water", 1)],
        outputs: &[("wine", 5)],
        work_time: 8,
    },

    // Loom
    Recipe {
        key: "weave_cloth",
        name: "Weave Cloth",
        workshop: "loom",
        skill: "crafting",
        skill_level: 1,
        inputs: &[("plant_fiber", 3)],
        outputs: &[("cloth", 1)],
        work_time: 4,
    },

    // Tannery
    Recipe {
        key: "tan_hide",
        name: "Tan Hide",
        workshop: "tannery",
        skill: "crafting",
        skill_level: 1,
        inputs: &[("hide", 1)],
        outputs: &[("leather", 1)],
        work_time: 5,
    },

    // Craftsdwarf
    Recipe {
        key: "make_craft",
        name: "Make Craft",
        workshop: "craftsdwarf",
        skill: "crafting",
        skill_level: 1,
        inputs: &[("stone", 1)],
        outputs: &[("craft", 1)],
        work_time: 3,
    },
    Recipe {
        key: "make_gem_craft",
        name: "Make Gem Craft",
        workshop: "craftsdwarf",
        skill: "crafting",
        skill_level: 3,
        inputs: &[("cut_gem", 1), ("gold", 1)],
        outputs: &[("craft", 3)],
        work_time: 8,
    },
];

/// Get a recipe by key
pub fn get_recipe(key: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.key == key)
}

/// Get all recipes for a workshop
pub fn get_workshop_recipes(workshop: &str) -> Vec<&'static Recipe> {
    RECIPES.iter().filter(|r| r.workshop == workshop).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_lookup() {
        assert!(get_skill("mining").is_some());
        assert!(get_skill("invalid").is_none());
    }

    #[test]
    fn test_workshop_lookup() {
        assert!(get_workshop("forge").is_some());
        assert!(get_workshop("invalid").is_none());
    }

    #[test]
    fn test_enemy_threat_levels() {
        for enemy in ENEMIES {
            assert!(enemy.threat_level >= 1 && enemy.threat_level <= 5);
        }
    }

    #[test]
    fn test_all_recipes_have_valid_workshops() {
        for recipe in RECIPES {
            assert!(get_workshop(recipe.workshop).is_some(),
                "Recipe {} references invalid workshop {}", recipe.key, recipe.workshop);
        }
    }

    #[test]
    fn test_workshop_recipes() {
        let forge_recipes = get_workshop_recipes("forge");
        assert!(!forge_recipes.is_empty());
        assert!(forge_recipes.iter().any(|r| r.key == "forge_weapon"));
    }
}
