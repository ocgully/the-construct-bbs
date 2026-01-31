//! Static game data for Mineteria
//!
//! Defines blocks, items, tools, recipes, biomes, and monsters.

use serde::{Deserialize, Serialize};

// ============================================================================
// BLOCK TYPES
// ============================================================================

/// Block types that can exist in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockType {
    // Natural blocks
    Air,
    Dirt,
    Grass,
    Stone,
    Sand,
    Gravel,
    Clay,
    Snow,
    Ice,

    // Ores
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    CopperOre,

    // Organic
    Wood,
    Leaves,
    Cactus,

    // Crafted blocks
    Planks,
    CobbleStone,
    StoneBrick,
    Torch,
    Workbench,
    Furnace,
    Chest,
    Door,
    Ladder,

    // Special
    Bedrock,
    Water,
    Lava,
}

/// Block definition with properties
#[derive(Debug, Clone)]
pub struct Block {
    pub block_type: BlockType,
    pub name: &'static str,
    pub hardness: u8,           // 0 = instant break, 255 = unbreakable
    pub tool_required: Option<ToolType>,
    pub min_tool_tier: u8,      // 0 = any, 1 = stone, 2 = iron, etc.
    pub drops: Option<BlockType>, // What it drops when mined (None = itself)
    pub light_level: u8,        // 0-15, 0 = no light
    pub solid: bool,
    pub char_display: char,
}

impl BlockType {
    pub fn get_block(&self) -> Block {
        match self {
            BlockType::Air => Block {
                block_type: *self,
                name: "Air",
                hardness: 0,
                tool_required: None,
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: false,
                char_display: ' ',
            },
            BlockType::Dirt => Block {
                block_type: *self,
                name: "Dirt",
                hardness: 2,
                tool_required: Some(ToolType::Shovel),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: '.',
            },
            BlockType::Grass => Block {
                block_type: *self,
                name: "Grass",
                hardness: 2,
                tool_required: Some(ToolType::Shovel),
                min_tool_tier: 0,
                drops: Some(BlockType::Dirt),
                light_level: 0,
                solid: true,
                char_display: '"',
            },
            BlockType::Stone => Block {
                block_type: *self,
                name: "Stone",
                hardness: 6,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 0,
                drops: Some(BlockType::CobbleStone),
                light_level: 0,
                solid: true,
                char_display: '#',
            },
            BlockType::Sand => Block {
                block_type: *self,
                name: "Sand",
                hardness: 2,
                tool_required: Some(ToolType::Shovel),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: ':',
            },
            BlockType::Gravel => Block {
                block_type: *self,
                name: "Gravel",
                hardness: 3,
                tool_required: Some(ToolType::Shovel),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: ';',
            },
            BlockType::Clay => Block {
                block_type: *self,
                name: "Clay",
                hardness: 3,
                tool_required: Some(ToolType::Shovel),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'c',
            },
            BlockType::Snow => Block {
                block_type: *self,
                name: "Snow",
                hardness: 1,
                tool_required: Some(ToolType::Shovel),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: '*',
            },
            BlockType::Ice => Block {
                block_type: *self,
                name: "Ice",
                hardness: 4,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'I',
            },
            BlockType::CoalOre => Block {
                block_type: *self,
                name: "Coal Ore",
                hardness: 6,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'C',
            },
            BlockType::IronOre => Block {
                block_type: *self,
                name: "Iron Ore",
                hardness: 8,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 1,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'F',
            },
            BlockType::GoldOre => Block {
                block_type: *self,
                name: "Gold Ore",
                hardness: 8,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 2,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'G',
            },
            BlockType::DiamondOre => Block {
                block_type: *self,
                name: "Diamond Ore",
                hardness: 10,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 2,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'D',
            },
            BlockType::CopperOre => Block {
                block_type: *self,
                name: "Copper Ore",
                hardness: 6,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'o',
            },
            BlockType::Wood => Block {
                block_type: *self,
                name: "Wood",
                hardness: 4,
                tool_required: Some(ToolType::Axe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: '|',
            },
            BlockType::Leaves => Block {
                block_type: *self,
                name: "Leaves",
                hardness: 1,
                tool_required: None,
                min_tool_tier: 0,
                drops: None, // Sometimes drops saplings
                light_level: 0,
                solid: false,
                char_display: '&',
            },
            BlockType::Cactus => Block {
                block_type: *self,
                name: "Cactus",
                hardness: 2,
                tool_required: None,
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'Y',
            },
            BlockType::Planks => Block {
                block_type: *self,
                name: "Planks",
                hardness: 4,
                tool_required: Some(ToolType::Axe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: '=',
            },
            BlockType::CobbleStone => Block {
                block_type: *self,
                name: "Cobblestone",
                hardness: 6,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: '%',
            },
            BlockType::StoneBrick => Block {
                block_type: *self,
                name: "Stone Brick",
                hardness: 7,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'B',
            },
            BlockType::Torch => Block {
                block_type: *self,
                name: "Torch",
                hardness: 0,
                tool_required: None,
                min_tool_tier: 0,
                drops: None,
                light_level: 14,
                solid: false,
                char_display: 'i',
            },
            BlockType::Workbench => Block {
                block_type: *self,
                name: "Workbench",
                hardness: 4,
                tool_required: Some(ToolType::Axe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: 'W',
            },
            BlockType::Furnace => Block {
                block_type: *self,
                name: "Furnace",
                hardness: 6,
                tool_required: Some(ToolType::Pickaxe),
                min_tool_tier: 0,
                drops: None,
                light_level: 13,
                solid: true,
                char_display: 'M',
            },
            BlockType::Chest => Block {
                block_type: *self,
                name: "Chest",
                hardness: 4,
                tool_required: Some(ToolType::Axe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: '[',
            },
            BlockType::Door => Block {
                block_type: *self,
                name: "Door",
                hardness: 3,
                tool_required: Some(ToolType::Axe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: false, // Can walk through when open
                char_display: 'O',
            },
            BlockType::Ladder => Block {
                block_type: *self,
                name: "Ladder",
                hardness: 2,
                tool_required: Some(ToolType::Axe),
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: false,
                char_display: 'H',
            },
            BlockType::Bedrock => Block {
                block_type: *self,
                name: "Bedrock",
                hardness: 255,
                tool_required: None,
                min_tool_tier: 255,
                drops: None,
                light_level: 0,
                solid: true,
                char_display: '@',
            },
            BlockType::Water => Block {
                block_type: *self,
                name: "Water",
                hardness: 0,
                tool_required: None,
                min_tool_tier: 0,
                drops: None,
                light_level: 0,
                solid: false,
                char_display: '~',
            },
            BlockType::Lava => Block {
                block_type: *self,
                name: "Lava",
                hardness: 0,
                tool_required: None,
                min_tool_tier: 0,
                drops: None,
                light_level: 15,
                solid: false,
                char_display: '^',
            },
        }
    }
}

// ============================================================================
// ITEM TYPES
// ============================================================================

/// Type of tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
}

/// Material tier for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ToolMaterial {
    Wood = 0,
    Stone = 1,
    Copper = 2,
    Iron = 3,
    Gold = 4,      // Fast but low durability
    Diamond = 5,
}

impl ToolMaterial {
    pub fn tier(&self) -> u8 {
        match self {
            ToolMaterial::Wood => 0,
            ToolMaterial::Stone => 1,
            ToolMaterial::Copper => 1,
            ToolMaterial::Iron => 2,
            ToolMaterial::Gold => 2,
            ToolMaterial::Diamond => 3,
        }
    }

    pub fn durability(&self) -> u16 {
        match self {
            ToolMaterial::Wood => 59,
            ToolMaterial::Stone => 131,
            ToolMaterial::Copper => 160,
            ToolMaterial::Iron => 250,
            ToolMaterial::Gold => 32,
            ToolMaterial::Diamond => 1561,
        }
    }

    pub fn speed(&self) -> f32 {
        match self {
            ToolMaterial::Wood => 2.0,
            ToolMaterial::Stone => 4.0,
            ToolMaterial::Copper => 5.0,
            ToolMaterial::Iron => 6.0,
            ToolMaterial::Gold => 12.0,
            ToolMaterial::Diamond => 8.0,
        }
    }

    pub fn damage_bonus(&self) -> i32 {
        match self {
            ToolMaterial::Wood => 0,
            ToolMaterial::Stone => 1,
            ToolMaterial::Copper => 2,
            ToolMaterial::Iron => 3,
            ToolMaterial::Gold => 0,
            ToolMaterial::Diamond => 4,
        }
    }
}

/// Item types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    // Blocks (placeable)
    Block(BlockType),

    // Tools
    Tool(ToolType, ToolMaterial),

    // Raw materials
    Coal,
    IronIngot,
    GoldIngot,
    CopperIngot,
    Diamond,
    Stick,

    // Consumables
    Apple,
    Bread,
    CookedMeat,
    RawMeat,

    // Combat
    Arrow,
    Bow,
}

/// Full item definition
#[derive(Debug, Clone)]
pub struct Item {
    pub item_type: ItemType,
    pub name: &'static str,
    pub max_stack: u8,
    pub char_display: char,
}

impl ItemType {
    pub fn get_item(&self) -> Item {
        match self {
            ItemType::Block(block) => {
                let b = block.get_block();
                Item {
                    item_type: *self,
                    name: b.name,
                    max_stack: 64,
                    char_display: b.char_display,
                }
            },
            ItemType::Tool(tool_type, material) => {
                let name = match (tool_type, material) {
                    (ToolType::Pickaxe, ToolMaterial::Wood) => "Wooden Pickaxe",
                    (ToolType::Pickaxe, ToolMaterial::Stone) => "Stone Pickaxe",
                    (ToolType::Pickaxe, ToolMaterial::Copper) => "Copper Pickaxe",
                    (ToolType::Pickaxe, ToolMaterial::Iron) => "Iron Pickaxe",
                    (ToolType::Pickaxe, ToolMaterial::Gold) => "Golden Pickaxe",
                    (ToolType::Pickaxe, ToolMaterial::Diamond) => "Diamond Pickaxe",
                    (ToolType::Axe, ToolMaterial::Wood) => "Wooden Axe",
                    (ToolType::Axe, ToolMaterial::Stone) => "Stone Axe",
                    (ToolType::Axe, ToolMaterial::Copper) => "Copper Axe",
                    (ToolType::Axe, ToolMaterial::Iron) => "Iron Axe",
                    (ToolType::Axe, ToolMaterial::Gold) => "Golden Axe",
                    (ToolType::Axe, ToolMaterial::Diamond) => "Diamond Axe",
                    (ToolType::Shovel, ToolMaterial::Wood) => "Wooden Shovel",
                    (ToolType::Shovel, ToolMaterial::Stone) => "Stone Shovel",
                    (ToolType::Shovel, ToolMaterial::Copper) => "Copper Shovel",
                    (ToolType::Shovel, ToolMaterial::Iron) => "Iron Shovel",
                    (ToolType::Shovel, ToolMaterial::Gold) => "Golden Shovel",
                    (ToolType::Shovel, ToolMaterial::Diamond) => "Diamond Shovel",
                    (ToolType::Sword, ToolMaterial::Wood) => "Wooden Sword",
                    (ToolType::Sword, ToolMaterial::Stone) => "Stone Sword",
                    (ToolType::Sword, ToolMaterial::Copper) => "Copper Sword",
                    (ToolType::Sword, ToolMaterial::Iron) => "Iron Sword",
                    (ToolType::Sword, ToolMaterial::Gold) => "Golden Sword",
                    (ToolType::Sword, ToolMaterial::Diamond) => "Diamond Sword",
                    (ToolType::Hoe, ToolMaterial::Wood) => "Wooden Hoe",
                    (ToolType::Hoe, ToolMaterial::Stone) => "Stone Hoe",
                    (ToolType::Hoe, ToolMaterial::Copper) => "Copper Hoe",
                    (ToolType::Hoe, ToolMaterial::Iron) => "Iron Hoe",
                    (ToolType::Hoe, ToolMaterial::Gold) => "Golden Hoe",
                    (ToolType::Hoe, ToolMaterial::Diamond) => "Diamond Hoe",
                };
                Item {
                    item_type: *self,
                    name,
                    max_stack: 1,
                    char_display: match tool_type {
                        ToolType::Pickaxe => 'P',
                        ToolType::Axe => 'A',
                        ToolType::Shovel => 'S',
                        ToolType::Sword => '/',
                        ToolType::Hoe => 'h',
                    },
                }
            },
            ItemType::Coal => Item {
                item_type: *self,
                name: "Coal",
                max_stack: 64,
                char_display: 'c',
            },
            ItemType::IronIngot => Item {
                item_type: *self,
                name: "Iron Ingot",
                max_stack: 64,
                char_display: 'f',
            },
            ItemType::GoldIngot => Item {
                item_type: *self,
                name: "Gold Ingot",
                max_stack: 64,
                char_display: 'g',
            },
            ItemType::CopperIngot => Item {
                item_type: *self,
                name: "Copper Ingot",
                max_stack: 64,
                char_display: 'o',
            },
            ItemType::Diamond => Item {
                item_type: *self,
                name: "Diamond",
                max_stack: 64,
                char_display: 'd',
            },
            ItemType::Stick => Item {
                item_type: *self,
                name: "Stick",
                max_stack: 64,
                char_display: '!',
            },
            ItemType::Apple => Item {
                item_type: *self,
                name: "Apple",
                max_stack: 64,
                char_display: 'a',
            },
            ItemType::Bread => Item {
                item_type: *self,
                name: "Bread",
                max_stack: 64,
                char_display: 'b',
            },
            ItemType::CookedMeat => Item {
                item_type: *self,
                name: "Cooked Meat",
                max_stack: 64,
                char_display: 'm',
            },
            ItemType::RawMeat => Item {
                item_type: *self,
                name: "Raw Meat",
                max_stack: 64,
                char_display: 'r',
            },
            ItemType::Arrow => Item {
                item_type: *self,
                name: "Arrow",
                max_stack: 64,
                char_display: '>',
            },
            ItemType::Bow => Item {
                item_type: *self,
                name: "Bow",
                max_stack: 1,
                char_display: ')',
            },
        }
    }
}

/// Tool with durability tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub tool_type: ToolType,
    pub material: ToolMaterial,
    pub durability: u16,
    pub max_durability: u16,
}

impl Tool {
    pub fn new(tool_type: ToolType, material: ToolMaterial) -> Self {
        let max = material.durability();
        Self {
            tool_type,
            material,
            durability: max,
            max_durability: max,
        }
    }

    pub fn use_tool(&mut self) -> bool {
        if self.durability > 0 {
            self.durability -= 1;
            true
        } else {
            false // Tool is broken
        }
    }

    pub fn is_broken(&self) -> bool {
        self.durability == 0
    }
}

// ============================================================================
// CRAFTING RECIPES
// ============================================================================

/// Crafting station requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftingStation {
    Hand,       // Can craft anywhere
    Workbench,  // Needs workbench nearby
    Furnace,    // Needs furnace (smelting)
    Anvil,      // Needs anvil (advanced crafting)
}

/// A crafting recipe
#[derive(Debug, Clone)]
pub struct Recipe {
    pub output: ItemType,
    pub output_count: u8,
    pub ingredients: &'static [(ItemType, u8)],
    pub station: CraftingStation,
}

/// All crafting recipes
pub static RECIPES: &[Recipe] = &[
    // Hand crafting
    Recipe {
        output: ItemType::Block(BlockType::Planks),
        output_count: 4,
        ingredients: &[(ItemType::Block(BlockType::Wood), 1)],
        station: CraftingStation::Hand,
    },
    Recipe {
        output: ItemType::Stick,
        output_count: 4,
        ingredients: &[(ItemType::Block(BlockType::Planks), 2)],
        station: CraftingStation::Hand,
    },
    Recipe {
        output: ItemType::Block(BlockType::Workbench),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::Planks), 4)],
        station: CraftingStation::Hand,
    },
    Recipe {
        output: ItemType::Block(BlockType::Torch),
        output_count: 4,
        ingredients: &[(ItemType::Coal, 1), (ItemType::Stick, 1)],
        station: CraftingStation::Hand,
    },

    // Workbench crafting - Wooden tools
    Recipe {
        output: ItemType::Tool(ToolType::Pickaxe, ToolMaterial::Wood),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::Planks), 3), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Axe, ToolMaterial::Wood),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::Planks), 3), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Shovel, ToolMaterial::Wood),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::Planks), 1), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Sword, ToolMaterial::Wood),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::Planks), 2), (ItemType::Stick, 1)],
        station: CraftingStation::Workbench,
    },

    // Stone tools
    Recipe {
        output: ItemType::Tool(ToolType::Pickaxe, ToolMaterial::Stone),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::CobbleStone), 3), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Axe, ToolMaterial::Stone),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::CobbleStone), 3), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Shovel, ToolMaterial::Stone),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::CobbleStone), 1), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Sword, ToolMaterial::Stone),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::CobbleStone), 2), (ItemType::Stick, 1)],
        station: CraftingStation::Workbench,
    },

    // Iron tools
    Recipe {
        output: ItemType::Tool(ToolType::Pickaxe, ToolMaterial::Iron),
        output_count: 1,
        ingredients: &[(ItemType::IronIngot, 3), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Axe, ToolMaterial::Iron),
        output_count: 1,
        ingredients: &[(ItemType::IronIngot, 3), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Shovel, ToolMaterial::Iron),
        output_count: 1,
        ingredients: &[(ItemType::IronIngot, 1), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Sword, ToolMaterial::Iron),
        output_count: 1,
        ingredients: &[(ItemType::IronIngot, 2), (ItemType::Stick, 1)],
        station: CraftingStation::Workbench,
    },

    // Diamond tools
    Recipe {
        output: ItemType::Tool(ToolType::Pickaxe, ToolMaterial::Diamond),
        output_count: 1,
        ingredients: &[(ItemType::Diamond, 3), (ItemType::Stick, 2)],
        station: CraftingStation::Workbench,
    },
    Recipe {
        output: ItemType::Tool(ToolType::Sword, ToolMaterial::Diamond),
        output_count: 1,
        ingredients: &[(ItemType::Diamond, 2), (ItemType::Stick, 1)],
        station: CraftingStation::Workbench,
    },

    // Furnace
    Recipe {
        output: ItemType::Block(BlockType::Furnace),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::CobbleStone), 8)],
        station: CraftingStation::Workbench,
    },

    // Chest
    Recipe {
        output: ItemType::Block(BlockType::Chest),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::Planks), 8)],
        station: CraftingStation::Workbench,
    },

    // Ladder
    Recipe {
        output: ItemType::Block(BlockType::Ladder),
        output_count: 3,
        ingredients: &[(ItemType::Stick, 7)],
        station: CraftingStation::Workbench,
    },

    // Door
    Recipe {
        output: ItemType::Block(BlockType::Door),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::Planks), 6)],
        station: CraftingStation::Workbench,
    },

    // Smelting recipes
    Recipe {
        output: ItemType::IronIngot,
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::IronOre), 1)],
        station: CraftingStation::Furnace,
    },
    Recipe {
        output: ItemType::GoldIngot,
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::GoldOre), 1)],
        station: CraftingStation::Furnace,
    },
    Recipe {
        output: ItemType::CopperIngot,
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::CopperOre), 1)],
        station: CraftingStation::Furnace,
    },
    Recipe {
        output: ItemType::Block(BlockType::StoneBrick),
        output_count: 1,
        ingredients: &[(ItemType::Block(BlockType::Stone), 1)],
        station: CraftingStation::Furnace,
    },
    Recipe {
        output: ItemType::CookedMeat,
        output_count: 1,
        ingredients: &[(ItemType::RawMeat, 1)],
        station: CraftingStation::Furnace,
    },
];

// ============================================================================
// BIOMES
// ============================================================================

/// Biome types for world generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Biome {
    Forest,
    Desert,
    Tundra,
    Swamp,
    Mountains,
    Plains,
    Ocean,
    Cave,
}

impl Biome {
    pub fn surface_block(&self) -> BlockType {
        match self {
            Biome::Forest => BlockType::Grass,
            Biome::Desert => BlockType::Sand,
            Biome::Tundra => BlockType::Snow,
            Biome::Swamp => BlockType::Grass,
            Biome::Mountains => BlockType::Stone,
            Biome::Plains => BlockType::Grass,
            Biome::Ocean => BlockType::Water,
            Biome::Cave => BlockType::Stone,
        }
    }

    pub fn subsurface_block(&self) -> BlockType {
        match self {
            Biome::Forest => BlockType::Dirt,
            Biome::Desert => BlockType::Sand,
            Biome::Tundra => BlockType::Dirt,
            Biome::Swamp => BlockType::Clay,
            Biome::Mountains => BlockType::Stone,
            Biome::Plains => BlockType::Dirt,
            Biome::Ocean => BlockType::Sand,
            Biome::Cave => BlockType::Stone,
        }
    }

    pub fn has_trees(&self) -> bool {
        matches!(self, Biome::Forest | Biome::Swamp | Biome::Plains)
    }

    pub fn temperature(&self) -> i8 {
        match self {
            Biome::Forest => 15,
            Biome::Desert => 35,
            Biome::Tundra => -10,
            Biome::Swamp => 20,
            Biome::Mountains => 5,
            Biome::Plains => 18,
            Biome::Ocean => 12,
            Biome::Cave => 10,
        }
    }
}

// ============================================================================
// MONSTERS
// ============================================================================

/// Monster types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterType {
    Zombie,
    Skeleton,
    Spider,
    Slime,
    Creeper,
    Bat,
    // Boss monsters
    GiantSpider,
    UndeadKing,
}

/// Monster definition
#[derive(Debug, Clone)]
pub struct Monster {
    pub monster_type: MonsterType,
    pub name: &'static str,
    pub max_health: i32,
    pub damage: i32,
    pub defense: i32,
    pub xp_reward: i32,
    pub spawns_at_night: bool,
    pub spawns_underground: bool,
    pub char_display: char,
}

impl MonsterType {
    pub fn get_monster(&self) -> Monster {
        match self {
            MonsterType::Zombie => Monster {
                monster_type: *self,
                name: "Zombie",
                max_health: 20,
                damage: 3,
                defense: 2,
                xp_reward: 5,
                spawns_at_night: true,
                spawns_underground: true,
                char_display: 'Z',
            },
            MonsterType::Skeleton => Monster {
                monster_type: *self,
                name: "Skeleton",
                max_health: 15,
                damage: 4,
                defense: 1,
                xp_reward: 5,
                spawns_at_night: true,
                spawns_underground: true,
                char_display: 'K',
            },
            MonsterType::Spider => Monster {
                monster_type: *self,
                name: "Spider",
                max_health: 16,
                damage: 2,
                defense: 0,
                xp_reward: 5,
                spawns_at_night: true,
                spawns_underground: true,
                char_display: 'X',
            },
            MonsterType::Slime => Monster {
                monster_type: *self,
                name: "Slime",
                max_health: 8,
                damage: 1,
                defense: 0,
                xp_reward: 2,
                spawns_at_night: false,
                spawns_underground: true,
                char_display: 'O',
            },
            MonsterType::Creeper => Monster {
                monster_type: *self,
                name: "Creeper",
                max_health: 20,
                damage: 10, // Explosion damage
                defense: 0,
                xp_reward: 8,
                spawns_at_night: true,
                spawns_underground: false,
                char_display: 'C',
            },
            MonsterType::Bat => Monster {
                monster_type: *self,
                name: "Bat",
                max_health: 6,
                damage: 1,
                defense: 0,
                xp_reward: 1,
                spawns_at_night: false,
                spawns_underground: true,
                char_display: 'v',
            },
            MonsterType::GiantSpider => Monster {
                monster_type: *self,
                name: "Giant Spider",
                max_health: 100,
                damage: 8,
                defense: 5,
                xp_reward: 50,
                spawns_at_night: false,
                spawns_underground: true,
                char_display: 'Q',
            },
            MonsterType::UndeadKing => Monster {
                monster_type: *self,
                name: "Undead King",
                max_health: 200,
                damage: 15,
                defense: 10,
                xp_reward: 100,
                spawns_at_night: false,
                spawns_underground: true,
                char_display: 'U',
            },
        }
    }
}

// ============================================================================
// TIME
// ============================================================================

/// Day/night cycle constants
pub const DAY_LENGTH_TICKS: u32 = 24000;      // Full day in ticks
pub const DAY_START_TICK: u32 = 0;            // Dawn
pub const NOON_TICK: u32 = 6000;              // Midday
pub const DUSK_TICK: u32 = 12000;             // Evening
pub const NIGHT_START_TICK: u32 = 13000;      // Night begins
pub const MIDNIGHT_TICK: u32 = 18000;         // Midnight
pub const DAWN_TICK: u32 = 23000;             // Dawn approaching

pub fn is_daytime(tick: u32) -> bool {
    let time = tick % DAY_LENGTH_TICKS;
    time < NIGHT_START_TICK || time >= DAWN_TICK
}

pub fn time_of_day_name(tick: u32) -> &'static str {
    let time = tick % DAY_LENGTH_TICKS;
    if time < 3000 {
        "Dawn"
    } else if time < NOON_TICK {
        "Morning"
    } else if time < DUSK_TICK {
        "Afternoon"
    } else if time < NIGHT_START_TICK {
        "Dusk"
    } else if time < MIDNIGHT_TICK {
        "Night"
    } else if time < DAWN_TICK {
        "Late Night"
    } else {
        "Dawn"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_properties() {
        let stone = BlockType::Stone.get_block();
        assert_eq!(stone.name, "Stone");
        assert!(stone.solid);
        assert_eq!(stone.tool_required, Some(ToolType::Pickaxe));
    }

    #[test]
    fn test_tool_material_progression() {
        assert!(ToolMaterial::Stone > ToolMaterial::Wood);
        assert!(ToolMaterial::Iron > ToolMaterial::Stone);
        assert!(ToolMaterial::Diamond > ToolMaterial::Iron);
    }

    #[test]
    fn test_tool_durability() {
        let mut tool = Tool::new(ToolType::Pickaxe, ToolMaterial::Wood);
        assert_eq!(tool.durability, 59);
        assert!(!tool.is_broken());

        for _ in 0..59 {
            assert!(tool.use_tool());
        }
        assert!(tool.is_broken());
        assert!(!tool.use_tool());
    }

    #[test]
    fn test_time_of_day() {
        assert!(is_daytime(0));       // Dawn
        assert!(is_daytime(6000));    // Noon
        assert!(!is_daytime(15000));  // Night
        assert!(is_daytime(23500));   // Dawn approaching
    }

    #[test]
    fn test_biome_blocks() {
        assert_eq!(Biome::Forest.surface_block(), BlockType::Grass);
        assert_eq!(Biome::Desert.surface_block(), BlockType::Sand);
        assert_eq!(Biome::Tundra.surface_block(), BlockType::Snow);
    }

    #[test]
    fn test_monster_stats() {
        let zombie = MonsterType::Zombie.get_monster();
        assert_eq!(zombie.name, "Zombie");
        assert!(zombie.spawns_at_night);

        let boss = MonsterType::UndeadKing.get_monster();
        assert_eq!(boss.max_health, 200);
        assert_eq!(boss.xp_reward, 100);
    }
}
