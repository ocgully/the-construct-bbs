//! Static game data for Xodia - the Living MUD
//!
//! Contains predefined regions, items, NPCs, and game constants.

use serde::{Serialize, Deserialize};

// ============================================================================
// GAME CONSTANTS
// ============================================================================

/// Starting stats for new characters
pub const STARTING_STATS: CharacterStartStats = CharacterStartStats {
    strength: 10,
    dexterity: 10,
    constitution: 10,
    intelligence: 10,
    wisdom: 10,
    charisma: 10,
    gold: 50,
    level: 1,
};

#[derive(Debug, Clone)]
pub struct CharacterStartStats {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    pub gold: i64,
    pub level: u32,
}

/// Experience required for each level (index = level - 1)
pub const LEVEL_XP_REQUIREMENTS: [u64; 20] = [
    0,      // Level 1
    100,    // Level 2
    300,    // Level 3
    600,    // Level 4
    1000,   // Level 5
    1500,   // Level 6
    2100,   // Level 7
    2800,   // Level 8
    3600,   // Level 9
    4500,   // Level 10
    5500,   // Level 11
    6600,   // Level 12
    7800,   // Level 13
    9100,   // Level 14
    10500,  // Level 15
    12000,  // Level 16
    13600,  // Level 17
    15300,  // Level 18
    17100,  // Level 19
    19000,  // Level 20
];

// ============================================================================
// REGIONS - Core anchor locations in the world of Xodia
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub level_range: (u32, u32),
    pub atmosphere: &'static str,
}

pub static REGIONS: &[Region] = &[
    Region {
        key: "misthollow",
        name: "Misthollow Village",
        description: "A quiet village shrouded in perpetual mist. The last bastion of hope before the wilds.",
        level_range: (1, 3),
        atmosphere: "peaceful, mysterious, safe haven",
    },
    Region {
        key: "whispering_woods",
        name: "The Whispering Woods",
        description: "An ancient forest where the trees seem to speak. Fey creatures lurk in the shadows.",
        level_range: (2, 5),
        atmosphere: "dark, enchanted, mysterious",
    },
    Region {
        key: "saltmere",
        name: "Saltmere Port",
        description: "A bustling coastal trade hub where pirates and merchants mingle freely.",
        level_range: (3, 7),
        atmosphere: "lively, dangerous, commercial",
    },
    Region {
        key: "sunken_kingdom",
        name: "The Sunken Kingdom",
        description: "Underwater ruins of an ancient civilization, filled with powerful magic and forgotten treasures.",
        level_range: (6, 10),
        atmosphere: "mysterious, ancient, magical",
    },
    Region {
        key: "dragons_teeth",
        name: "Dragon's Teeth Mountains",
        description: "Towering peaks where dwarven holds lie carved into stone and dragons nest in high caves.",
        level_range: (8, 14),
        atmosphere: "harsh, dangerous, majestic",
    },
    Region {
        key: "obsidian_desert",
        name: "The Obsidian Desert",
        description: "A vast wasteland of black sand where nomads guard buried temples of forgotten gods.",
        level_range: (10, 16),
        atmosphere: "desolate, scorching, ancient",
    },
    Region {
        key: "spire_of_eternity",
        name: "The Spire of Eternity",
        description: "The endgame goal - a tower that touches the heavens, where the fate of the world will be decided.",
        level_range: (15, 20),
        atmosphere: "epic, otherworldly, climactic",
    },
];

pub fn get_region(key: &str) -> Option<&'static Region> {
    REGIONS.iter().find(|r| r.key == key)
}

// ============================================================================
// ITEM DEFINITIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Weapon,
    Armor,
    Shield,
    Potion,
    Scroll,
    Key,
    Quest,
    Misc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTemplate {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub item_type: ItemType,
    pub value: i64,
    pub weight: f32,
    /// For weapons: damage bonus. For armor: defense bonus.
    pub stat_bonus: i32,
}

pub static ITEMS: &[ItemTemplate] = &[
    // Starting weapons
    ItemTemplate {
        key: "rusty_dagger",
        name: "Rusty Dagger",
        description: "A worn dagger with a pitted blade. Better than nothing.",
        item_type: ItemType::Weapon,
        value: 5,
        weight: 0.5,
        stat_bonus: 1,
    },
    ItemTemplate {
        key: "wooden_staff",
        name: "Wooden Staff",
        description: "A simple quarterstaff of sturdy oak.",
        item_type: ItemType::Weapon,
        value: 8,
        weight: 2.0,
        stat_bonus: 2,
    },
    ItemTemplate {
        key: "short_sword",
        name: "Short Sword",
        description: "A reliable blade of modest length.",
        item_type: ItemType::Weapon,
        value: 25,
        weight: 1.5,
        stat_bonus: 3,
    },
    // Armor
    ItemTemplate {
        key: "leather_armor",
        name: "Leather Armor",
        description: "Basic protection made from tanned hides.",
        item_type: ItemType::Armor,
        value: 20,
        weight: 5.0,
        stat_bonus: 2,
    },
    ItemTemplate {
        key: "chain_mail",
        name: "Chain Mail",
        description: "Interlocking metal rings provide solid protection.",
        item_type: ItemType::Armor,
        value: 75,
        weight: 15.0,
        stat_bonus: 4,
    },
    // Potions
    ItemTemplate {
        key: "health_potion",
        name: "Health Potion",
        description: "A crimson liquid that restores vitality.",
        item_type: ItemType::Potion,
        value: 15,
        weight: 0.2,
        stat_bonus: 20, // Heals 20 HP
    },
    ItemTemplate {
        key: "mana_potion",
        name: "Mana Potion",
        description: "A blue elixir that restores magical energy.",
        item_type: ItemType::Potion,
        value: 20,
        weight: 0.2,
        stat_bonus: 15, // Restores 15 MP
    },
];

pub fn get_item_template(key: &str) -> Option<&'static ItemTemplate> {
    ITEMS.iter().find(|i| i.key == key)
}

// ============================================================================
// NPC TEMPLATES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NpcType {
    Friendly,
    Merchant,
    QuestGiver,
    Hostile,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcTemplate {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub npc_type: NpcType,
    pub level: u32,
    pub health: i32,
    pub damage: i32,
    pub dialogue_intro: &'static str,
}

pub static NPCS: &[NpcTemplate] = &[
    // Misthollow Village
    NpcTemplate {
        key: "elder_mira",
        name: "Elder Mira",
        description: "An ancient woman with knowing eyes who serves as the village keeper.",
        npc_type: NpcType::QuestGiver,
        level: 1,
        health: 20,
        damage: 0,
        dialogue_intro: "Welcome, Seeker. The dreams have shown me your coming.",
    },
    NpcTemplate {
        key: "blacksmith_torin",
        name: "Torin the Blacksmith",
        description: "A burly man with arms like oak branches and soot-stained hands.",
        npc_type: NpcType::Merchant,
        level: 1,
        health: 30,
        damage: 5,
        dialogue_intro: "Looking for steel? You've come to the right place.",
    },
    // Hostile creatures
    NpcTemplate {
        key: "forest_goblin",
        name: "Forest Goblin",
        description: "A small, green-skinned creature with sharp teeth and cunning eyes.",
        npc_type: NpcType::Hostile,
        level: 1,
        health: 15,
        damage: 3,
        dialogue_intro: "Hssss! Shiny things! Give!",
    },
    NpcTemplate {
        key: "dire_wolf",
        name: "Dire Wolf",
        description: "A massive wolf with silver-black fur and eyes that gleam with intelligence.",
        npc_type: NpcType::Hostile,
        level: 3,
        health: 35,
        damage: 8,
        dialogue_intro: "*growls menacingly*",
    },
    NpcTemplate {
        key: "shadow_wraith",
        name: "Shadow Wraith",
        description: "A dark, incorporeal being that drains the life from the living.",
        npc_type: NpcType::Hostile,
        level: 5,
        health: 45,
        damage: 12,
        dialogue_intro: "*an unearthly wail echoes through the air*",
    },
];

pub fn get_npc_template(key: &str) -> Option<&'static NpcTemplate> {
    NPCS.iter().find(|n| n.key == key)
}

// ============================================================================
// STARTING ROOMS
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct RoomTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub region: &'static str,
    pub exits: &'static [(&'static str, &'static str)], // (direction, room_id)
    pub npcs: &'static [&'static str],
    pub items: &'static [&'static str],
}

pub static STARTING_ROOMS: &[RoomTemplate] = &[
    RoomTemplate {
        id: "misthollow_square",
        name: "Misthollow Village Square",
        description: "The heart of Misthollow Village. A weathered stone fountain stands at the center, \
            its waters crystal clear despite the perpetual mist that clings to everything. \
            Cobblestone paths lead in all directions, and the soft glow of lanterns pierces the fog.",
        region: "misthollow",
        exits: &[
            ("north", "misthollow_elder"),
            ("east", "misthollow_smithy"),
            ("south", "misthollow_gate"),
            ("west", "misthollow_inn"),
        ],
        npcs: &[],
        items: &[],
    },
    RoomTemplate {
        id: "misthollow_elder",
        name: "Elder Mira's Cottage",
        description: "A cozy cottage filled with the scent of herbs and old parchment. \
            Crystals hang from the ceiling, catching what little light filters through the windows. \
            Elder Mira sits in a worn chair by the fire, watching you with ancient eyes.",
        region: "misthollow",
        exits: &[("south", "misthollow_square")],
        npcs: &["elder_mira"],
        items: &[],
    },
    RoomTemplate {
        id: "misthollow_smithy",
        name: "Torin's Smithy",
        description: "The clang of hammer on anvil fills this hot, smoky workshop. \
            Weapons and armor line the walls, each piece bearing Torin's distinctive mark. \
            The forge burns bright, casting dancing shadows across the stone floor.",
        region: "misthollow",
        exits: &[("west", "misthollow_square")],
        npcs: &["blacksmith_torin"],
        items: &["rusty_dagger", "short_sword", "leather_armor"],
    },
    RoomTemplate {
        id: "misthollow_gate",
        name: "Village Gate",
        description: "The southern gate of Misthollow stands open but guarded. \
            Beyond lies the path to the Whispering Woods, barely visible through the mist. \
            A weathered sign warns travelers of the dangers that await.",
        region: "misthollow",
        exits: &[
            ("north", "misthollow_square"),
            ("south", "forest_path_entrance"),
        ],
        npcs: &[],
        items: &[],
    },
    RoomTemplate {
        id: "misthollow_inn",
        name: "The Wanderer's Rest",
        description: "A warm tavern where travelers share tales and ale. \
            The innkeeper keeps the fire burning bright, and the smell of roasting meat \
            mingles with pipe smoke and old wood.",
        region: "misthollow",
        exits: &[("east", "misthollow_square")],
        npcs: &[],
        items: &["health_potion"],
    },
    // Whispering Woods entrance
    RoomTemplate {
        id: "forest_path_entrance",
        name: "Edge of the Whispering Woods",
        description: "The mist thins here, but shadows deepen. Ancient oaks tower overhead, \
            their branches intertwining to form a natural canopy. The path ahead splits, \
            and you hear faint whispers on the wind - or is that just your imagination?",
        region: "whispering_woods",
        exits: &[
            ("north", "misthollow_gate"),
            ("east", "forest_clearing"),
            ("south", "forest_deep_path"),
        ],
        npcs: &["forest_goblin"],
        items: &[],
    },
    RoomTemplate {
        id: "forest_clearing",
        name: "Moonlit Clearing",
        description: "A break in the canopy allows pale moonlight to filter down. \
            Strange mushrooms glow softly at the base of the trees, and you catch glimpses \
            of fireflies - or perhaps something else - dancing in the undergrowth.",
        region: "whispering_woods",
        exits: &[("west", "forest_path_entrance")],
        npcs: &[],
        items: &["wooden_staff"],
    },
    RoomTemplate {
        id: "forest_deep_path",
        name: "Deep Forest Path",
        description: "The woods grow darker here. The whispers are louder now, \
            and you could swear the trees are watching. Animal eyes gleam from the shadows, \
            and the path becomes increasingly difficult to follow.",
        region: "whispering_woods",
        exits: &[("north", "forest_path_entrance")],
        npcs: &["dire_wolf"],
        items: &[],
    },
];

pub fn get_room_template(id: &str) -> Option<&'static RoomTemplate> {
    STARTING_ROOMS.iter().find(|r| r.id == id)
}

// ============================================================================
// STORY CONSTANTS
// ============================================================================

/// The lore of Xodia for LLM context
pub const WORLD_LORE: &str = r#"
The World of Xodia

Long ago, the world was whole, united under the Light of the First Flame.
Then came the Sundering - the Flame was shattered into seven shards,
each falling to a corner of the world.

The Spire of Eternity holds the key to reuniting the shards.
But the path is treacherous, guarded by ancient evils awakened by the Sundering.

You are a Seeker, one called by dreams to find the Spire.
Your journey begins in Misthollow Village, where the last Keeper awaits.

The world is alive with side quests, factions, mysteries, and danger.
Every choice matters. Every action is remembered.

Key Locations:
- Misthollow Village: Starting safe zone, training area
- The Whispering Woods: Dark forest with fey creatures
- Saltmere Port: Coastal trade hub, pirates and merchants
- The Sunken Kingdom: Underwater ruins, ancient magic
- Dragon's Teeth Mountains: Dwarven holds and dragon lairs
- The Obsidian Desert: Nomads and buried temples
- The Spire of Eternity: The final destination

The tone should be fantasy adventure with moments of wonder, danger, and mystery.
Combat is tactical but narrated dramatically. NPCs have personalities and memories.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regions_exist() {
        assert!(REGIONS.len() >= 7);
        assert!(get_region("misthollow").is_some());
        assert!(get_region("spire_of_eternity").is_some());
    }

    #[test]
    fn test_items_exist() {
        assert!(get_item_template("rusty_dagger").is_some());
        assert!(get_item_template("health_potion").is_some());
    }

    #[test]
    fn test_npcs_exist() {
        assert!(get_npc_template("elder_mira").is_some());
        assert!(get_npc_template("forest_goblin").is_some());
    }

    #[test]
    fn test_starting_rooms_connected() {
        let square = get_room_template("misthollow_square").unwrap();
        assert_eq!(square.exits.len(), 4);

        // Verify all exits point to valid rooms
        for (_, room_id) in square.exits {
            assert!(get_room_template(room_id).is_some(), "Exit {} not found", room_id);
        }
    }

    #[test]
    fn test_level_xp_requirements() {
        assert_eq!(LEVEL_XP_REQUIREMENTS[0], 0); // Level 1 requires 0 XP
        assert!(LEVEL_XP_REQUIREMENTS[19] > LEVEL_XP_REQUIREMENTS[18]); // Higher levels need more
    }
}
