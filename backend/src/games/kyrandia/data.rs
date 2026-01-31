//! Static game data for Kyrandia
//! Rooms, items, spells, NPCs, and world structure

#![allow(dead_code)]

// ============================================================================
// REGIONS & ROOMS
// ============================================================================

/// Region of the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    Village,
    DarkForest,
    GoldenForest,
    DragonCastle,
}

impl Region {
    pub fn name(&self) -> &'static str {
        match self {
            Region::Village => "The Village",
            Region::DarkForest => "The Dark Forest",
            Region::GoldenForest => "The Golden Forest",
            Region::DragonCastle => "Dragon Castle",
        }
    }

    pub fn required_level(&self) -> u8 {
        match self {
            Region::Village => 1,
            Region::DarkForest => 2,
            Region::GoldenForest => 4,
            Region::DragonCastle => 6,
        }
    }
}

/// Room definition
#[derive(Debug, Clone)]
pub struct Room {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub region: Region,
    pub exits: &'static [(&'static str, &'static str)],  // (direction, room_key)
    pub items: &'static [&'static str],  // Default items in room
    pub npcs: &'static [&'static str],   // NPCs present
    pub special: Option<RoomSpecial>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomSpecial {
    Inn,
    Shop,
    Library,
    Fountain,
    Training,
    Altar,
    ThroneRoom,
    DragonLair,
}

pub static ROOMS: &[Room] = &[
    // === VILLAGE ===
    Room {
        key: "village_square",
        name: "Village Square",
        description: "The heart of the humble village. A weathered stone fountain sits in the center, surrounded by thatched-roof cottages. Villagers go about their daily routines.",
        region: Region::Village,
        exits: &[("north", "village_inn"), ("east", "village_shop"), ("south", "village_gate"), ("west", "village_library")],
        items: &[],
        npcs: &["elder_quinn"],
        special: None,
    },
    Room {
        key: "village_inn",
        name: "The Rusty Cauldron Inn",
        description: "A cozy inn with a roaring fireplace. The smell of stew fills the air. A few travelers sit at worn wooden tables.",
        region: Region::Village,
        exits: &[("south", "village_square"), ("up", "inn_rooms")],
        items: &[],
        npcs: &["innkeeper_mira"],
        special: Some(RoomSpecial::Inn),
    },
    Room {
        key: "inn_rooms",
        name: "Inn Upper Floor",
        description: "A narrow hallway with several doors leading to modest guest rooms. The floorboards creak underfoot.",
        region: Region::Village,
        exits: &[("down", "village_inn")],
        items: &["pine_cone"],
        npcs: &[],
        special: None,
    },
    Room {
        key: "village_shop",
        name: "Mystic Supplies",
        description: "Shelves lined with curious artifacts, spell components, and adventuring gear. Crystals hang from the ceiling, casting rainbow light.",
        region: Region::Village,
        exits: &[("west", "village_square")],
        items: &[],
        npcs: &["merchant_felix"],
        special: Some(RoomSpecial::Shop),
    },
    Room {
        key: "village_library",
        name: "The Dusty Archive",
        description: "Towering bookshelves filled with ancient tomes. Dust motes dance in shafts of light from high windows. Knowledge of ages past awaits those who seek it.",
        region: Region::Village,
        exits: &[("east", "village_square")],
        items: &["scroll_light"],
        npcs: &["sage_orion"],
        special: Some(RoomSpecial::Library),
    },
    Room {
        key: "village_gate",
        name: "Village Gate",
        description: "The old wooden gate marks the boundary between the safe village and the untamed wilderness. A guard watches the road warily.",
        region: Region::Village,
        exits: &[("north", "village_square"), ("south", "forest_entrance")],
        items: &[],
        npcs: &["guard_bran"],
        special: None,
    },
    Room {
        key: "village_training",
        name: "Training Grounds",
        description: "A packed-dirt yard behind the barracks. Training dummies and weapon racks line the perimeter.",
        region: Region::Village,
        exits: &[("east", "village_square")],
        items: &[],
        npcs: &["trainer_grok"],
        special: Some(RoomSpecial::Training),
    },

    // === DARK FOREST ===
    Room {
        key: "forest_entrance",
        name: "Forest Entrance",
        description: "The path narrows as ancient trees loom overhead. Shadows seem to move between the gnarled trunks. The village gate is visible to the north.",
        region: Region::DarkForest,
        exits: &[("north", "village_gate"), ("south", "dark_path"), ("east", "pine_grove")],
        items: &[],
        npcs: &[],
        special: None,
    },
    Room {
        key: "pine_grove",
        name: "Pine Grove",
        description: "A cluster of tall pine trees. Pine cones litter the ground, some still fresh and green. The air smells of resin.",
        region: Region::DarkForest,
        exits: &[("west", "forest_entrance"), ("south", "mushroom_hollow")],
        items: &["pine_cone", "pine_cone", "pine_cone"],
        npcs: &[],
        special: None,
    },
    Room {
        key: "dark_path",
        name: "The Dark Path",
        description: "The canopy is so thick here that barely any light reaches the ground. Strange sounds echo from unseen sources.",
        region: Region::DarkForest,
        exits: &[("north", "forest_entrance"), ("south", "crossroads"), ("west", "haunted_hollow")],
        items: &[],
        npcs: &[],
        special: None,
    },
    Room {
        key: "haunted_hollow",
        name: "Haunted Hollow",
        description: "A cold mist hangs over this depression in the forest floor. Faint whispers seem to come from nowhere and everywhere.",
        region: Region::DarkForest,
        exits: &[("east", "dark_path")],
        items: &["ghost_orchid"],
        npcs: &["spirit_elara"],
        special: None,
    },
    Room {
        key: "mushroom_hollow",
        name: "Mushroom Hollow",
        description: "Giant mushrooms of every color grow in this damp hollow. Some glow with an inner light. The largest are big enough to shelter under.",
        region: Region::DarkForest,
        exits: &[("north", "pine_grove"), ("east", "crossroads")],
        items: &["glowing_mushroom"],
        npcs: &[],
        special: None,
    },
    Room {
        key: "crossroads",
        name: "Forest Crossroads",
        description: "Four paths meet at a stone marker worn smooth by time. Cryptic runes are barely visible on its surface.",
        region: Region::DarkForest,
        exits: &[("north", "dark_path"), ("east", "golden_gate"), ("south", "deep_forest"), ("west", "mushroom_hollow")],
        items: &[],
        npcs: &["wanderer_zeph"],
        special: None,
    },
    Room {
        key: "deep_forest",
        name: "Deep Forest",
        description: "Ancient trees with trunks wider than houses surround you. Creatures move in the shadows, watching.",
        region: Region::DarkForest,
        exits: &[("north", "crossroads")],
        items: &["pine_cone"],
        npcs: &[],
        special: None,
    },

    // === GOLDEN FOREST ===
    Room {
        key: "golden_gate",
        name: "Golden Gate",
        description: "An archway of intertwined golden branches marks the entrance to the Golden Forest. Warm light spills through, unlike the darkness behind you.",
        region: Region::GoldenForest,
        exits: &[("west", "crossroads"), ("east", "sunlit_path")],
        items: &[],
        npcs: &["gatekeeper_lumina"],
        special: None,
    },
    Room {
        key: "sunlit_path",
        name: "Sunlit Path",
        description: "Trees with golden leaves line a path of white stones. Butterflies with glowing wings flit between flowers.",
        region: Region::GoldenForest,
        exits: &[("west", "golden_gate"), ("north", "fountain_clearing"), ("east", "crystal_cave_entrance"), ("south", "ancient_grove")],
        items: &[],
        npcs: &[],
        special: None,
    },
    Room {
        key: "fountain_clearing",
        name: "The Fountain of Scrolls",
        description: "A magnificent crystal fountain rises from a pool of starlit water. Magic radiates from its depths. Pine cones thrown in are said to transform into spell scrolls.",
        region: Region::GoldenForest,
        exits: &[("south", "sunlit_path")],
        items: &[],
        npcs: &["fountain_sprite"],
        special: Some(RoomSpecial::Fountain),
    },
    Room {
        key: "ancient_grove",
        name: "Ancient Grove",
        description: "The oldest trees in the forest grow here, their roots forming natural seats. This is a place of deep magic and older secrets.",
        region: Region::GoldenForest,
        exits: &[("north", "sunlit_path"), ("south", "altar_of_tashanna")],
        items: &["golden_leaf"],
        npcs: &["treant_elder"],
        special: None,
    },
    Room {
        key: "altar_of_tashanna",
        name: "Altar of Tashanna",
        description: "A marble altar stands in a perfect circle of standing stones. The statue of Tashanna, Lady of Legends, gazes down with serene wisdom.",
        region: Region::GoldenForest,
        exits: &[("north", "ancient_grove")],
        items: &[],
        npcs: &[],
        special: Some(RoomSpecial::Altar),
    },
    Room {
        key: "crystal_cave_entrance",
        name: "Crystal Cave Entrance",
        description: "A cave mouth glitters with embedded crystals. A cool breeze carries hints of ancient power from within.",
        region: Region::GoldenForest,
        exits: &[("west", "sunlit_path"), ("in", "crystal_cave")],
        items: &[],
        npcs: &[],
        special: None,
    },
    Room {
        key: "crystal_cave",
        name: "Crystal Cave",
        description: "Crystals of every color form the walls, floor, and ceiling. They hum with magical energy. Deep within, you sense a passage upward.",
        region: Region::GoldenForest,
        exits: &[("out", "crystal_cave_entrance"), ("up", "castle_secret_entrance")],
        items: &["magic_crystal"],
        npcs: &[],
        special: None,
    },

    // === DRAGON CASTLE ===
    Room {
        key: "castle_gate",
        name: "Castle Gate",
        description: "Massive iron gates, twisted and melted by dragon fire. The castle looms above, dark clouds perpetually swirling around its highest tower.",
        region: Region::DragonCastle,
        exits: &[("in", "castle_courtyard")],
        items: &[],
        npcs: &[],
        special: None,
    },
    Room {
        key: "castle_secret_entrance",
        name: "Secret Passage",
        description: "A hidden passage emerges from the crystal cave into the castle. Cobwebs suggest it hasn't been used in ages.",
        region: Region::DragonCastle,
        exits: &[("down", "crystal_cave"), ("north", "castle_corridor")],
        items: &[],
        npcs: &[],
        special: None,
    },
    Room {
        key: "castle_courtyard",
        name: "Castle Courtyard",
        description: "Once beautiful gardens now lie scorched and dead. Statues of past arch-mages stand in a circle, their stone faces worn by time.",
        region: Region::DragonCastle,
        exits: &[("out", "castle_gate"), ("north", "great_hall"), ("east", "armory"), ("west", "castle_library")],
        items: &["charred_rose"],
        npcs: &[],
        special: None,
    },
    Room {
        key: "great_hall",
        name: "Great Hall",
        description: "Enormous columns support a vaulted ceiling. Faded tapestries depict the history of Kyrandia's mages. A grand staircase leads up.",
        region: Region::DragonCastle,
        exits: &[("south", "castle_courtyard"), ("up", "upper_hall")],
        items: &[],
        npcs: &["ghost_archmage"],
        special: None,
    },
    Room {
        key: "armory",
        name: "Castle Armory",
        description: "Racks of ancient weapons line the walls. Most are rusted beyond use, but some still gleam with enchantment.",
        region: Region::DragonCastle,
        exits: &[("west", "castle_courtyard")],
        items: &["enchanted_staff"],
        npcs: &[],
        special: None,
    },
    Room {
        key: "castle_library",
        name: "Castle Library",
        description: "The greatest magical library in the realm. Many books are destroyed, but some survive, containing powerful secrets.",
        region: Region::DragonCastle,
        exits: &[("east", "castle_courtyard")],
        items: &["tome_arcane"],
        npcs: &[],
        special: Some(RoomSpecial::Library),
    },
    Room {
        key: "castle_corridor",
        name: "Castle Corridor",
        description: "A long hallway with portraits of past arch-mages. Their painted eyes seem to follow you.",
        region: Region::DragonCastle,
        exits: &[("south", "castle_secret_entrance"), ("north", "upper_hall")],
        items: &[],
        npcs: &[],
        special: None,
    },
    Room {
        key: "upper_hall",
        name: "Upper Hall",
        description: "The upper level of the castle. Grand windows overlook the scorched lands below. The throne room beckons to the north.",
        region: Region::DragonCastle,
        exits: &[("down", "great_hall"), ("south", "castle_corridor"), ("north", "throne_room")],
        items: &[],
        npcs: &[],
        special: None,
    },
    Room {
        key: "throne_room",
        name: "Throne Room",
        description: "The seat of power for the Arch-Mage of Legends. An empty throne waits for one worthy. Beyond it, a passage leads to the dragon's lair.",
        region: Region::DragonCastle,
        exits: &[("south", "upper_hall"), ("north", "dragon_lair")],
        items: &[],
        npcs: &[],
        special: Some(RoomSpecial::ThroneRoom),
    },
    Room {
        key: "dragon_lair",
        name: "Dragon's Lair",
        description: "A vast cavern filled with treasure and bones. Heat radiates from the center where the vicious guardian dragon sleeps.",
        region: Region::DragonCastle,
        exits: &[("south", "throne_room")],
        items: &["dragon_egg"],
        npcs: &["dragon_pyraxis"],
        special: Some(RoomSpecial::DragonLair),
    },
];

pub fn get_room(key: &str) -> Option<&'static Room> {
    ROOMS.iter().find(|r| r.key == key)
}

// ============================================================================
// ITEMS
// ============================================================================

#[derive(Debug, Clone)]
pub struct Item {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub weight: u8,
    pub item_type: ItemType,
    pub value: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemType {
    Consumable,
    SpellComponent,
    Scroll,
    Weapon,
    Armor,
    Quest,
    Key,
    Treasure,
}

pub static ITEMS: &[Item] = &[
    Item {
        key: "pine_cone",
        name: "Pine Cone",
        description: "A fresh green pine cone. These are used at the Fountain of Scrolls to create spell scrolls.",
        weight: 1,
        item_type: ItemType::SpellComponent,
        value: 5,
    },
    Item {
        key: "scroll_light",
        name: "Scroll of Light",
        description: "A magical scroll containing the Light spell. Reading it will teach you the spell permanently.",
        weight: 1,
        item_type: ItemType::Scroll,
        value: 50,
    },
    Item {
        key: "scroll_fireball",
        name: "Scroll of Fireball",
        description: "A dangerous scroll crackling with barely contained flame. Teaches the Fireball spell.",
        weight: 1,
        item_type: ItemType::Scroll,
        value: 200,
    },
    Item {
        key: "scroll_heal",
        name: "Scroll of Healing",
        description: "A soothing scroll that glows with gentle light. Teaches the Heal spell.",
        weight: 1,
        item_type: ItemType::Scroll,
        value: 150,
    },
    Item {
        key: "scroll_shield",
        name: "Scroll of Shield",
        description: "A scroll with defensive runes. Teaches the Shield spell.",
        weight: 1,
        item_type: ItemType::Scroll,
        value: 100,
    },
    Item {
        key: "scroll_teleport",
        name: "Scroll of Teleport",
        description: "A scroll that shimmers with spatial distortion. Teaches the Teleport spell.",
        weight: 1,
        item_type: ItemType::Scroll,
        value: 300,
    },
    Item {
        key: "health_potion",
        name: "Health Potion",
        description: "A red potion that restores health when consumed.",
        weight: 1,
        item_type: ItemType::Consumable,
        value: 25,
    },
    Item {
        key: "mana_potion",
        name: "Mana Potion",
        description: "A blue potion that restores mana when consumed.",
        weight: 1,
        item_type: ItemType::Consumable,
        value: 30,
    },
    Item {
        key: "glowing_mushroom",
        name: "Glowing Mushroom",
        description: "A bioluminescent fungus from the Mushroom Hollow. Has minor healing properties.",
        weight: 1,
        item_type: ItemType::Consumable,
        value: 15,
    },
    Item {
        key: "ghost_orchid",
        name: "Ghost Orchid",
        description: "A pale flower that only grows in haunted places. Useful in spirit-related magic.",
        weight: 1,
        item_type: ItemType::SpellComponent,
        value: 75,
    },
    Item {
        key: "golden_leaf",
        name: "Golden Leaf",
        description: "A leaf from the trees of the Golden Forest. It never wilts and shimmers with magic.",
        weight: 1,
        item_type: ItemType::SpellComponent,
        value: 100,
    },
    Item {
        key: "magic_crystal",
        name: "Magic Crystal",
        description: "A crystal pulsing with raw magical energy. Can be used to recharge mana or enhance spells.",
        weight: 2,
        item_type: ItemType::SpellComponent,
        value: 150,
    },
    Item {
        key: "charred_rose",
        name: "Charred Rose",
        description: "A rose burnt by dragon fire but somehow still alive. A symbol of resilience.",
        weight: 1,
        item_type: ItemType::Quest,
        value: 50,
    },
    Item {
        key: "enchanted_staff",
        name: "Enchanted Staff",
        description: "A staff that enhances magical power. Increases spell damage by 25%.",
        weight: 3,
        item_type: ItemType::Weapon,
        value: 500,
    },
    Item {
        key: "tome_arcane",
        name: "Tome of Arcane Secrets",
        description: "An ancient book containing forgotten magical knowledge. Required for the final ritual.",
        weight: 3,
        item_type: ItemType::Quest,
        value: 1000,
    },
    Item {
        key: "dragon_egg",
        name: "Dragon Egg",
        description: "A scaled egg radiating warmth. Worth a fortune to the right buyer.",
        weight: 5,
        item_type: ItemType::Treasure,
        value: 5000,
    },
    Item {
        key: "apprentice_robe",
        name: "Apprentice Robe",
        description: "A simple robe worn by magic students. Provides minimal protection.",
        weight: 2,
        item_type: ItemType::Armor,
        value: 20,
    },
    Item {
        key: "mage_robe",
        name: "Mage Robe",
        description: "A fine robe woven with protective enchantments.",
        weight: 2,
        item_type: ItemType::Armor,
        value: 200,
    },
    Item {
        key: "archmage_robe",
        name: "Arch-Mage's Robe",
        description: "The legendary robe worn by the Arch-Mage of Legends. Grants great magical protection.",
        weight: 2,
        item_type: ItemType::Armor,
        value: 2000,
    },
    Item {
        key: "wooden_staff",
        name: "Wooden Staff",
        description: "A simple staff carved from oak. Better than nothing.",
        weight: 2,
        item_type: ItemType::Weapon,
        value: 15,
    },
    Item {
        key: "crystal_wand",
        name: "Crystal Wand",
        description: "A wand tipped with a focusing crystal. Increases spell accuracy.",
        weight: 1,
        item_type: ItemType::Weapon,
        value: 100,
    },
    Item {
        key: "tashanna_amulet",
        name: "Amulet of Tashanna",
        description: "A holy amulet blessed by the goddess. Required for the final ritual.",
        weight: 1,
        item_type: ItemType::Quest,
        value: 1500,
    },
    Item {
        key: "golden_key",
        name: "Golden Key",
        description: "A key that glows with golden light. Opens the way to the Golden Forest.",
        weight: 1,
        item_type: ItemType::Key,
        value: 250,
    },
    Item {
        key: "dragon_key",
        name: "Dragon Key",
        description: "A key forged in dragon fire. Opens the gate to Dragon Castle.",
        weight: 1,
        item_type: ItemType::Key,
        value: 500,
    },
];

pub fn get_item(key: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|i| i.key == key)
}

// ============================================================================
// SPELLS
// ============================================================================

#[derive(Debug, Clone)]
pub struct Spell {
    pub key: &'static str,
    pub name: &'static str,
    pub incantation: &'static str,  // The phrase players must type to cast
    pub description: &'static str,
    pub mana_cost: u32,
    pub spell_type: SpellType,
    pub power: u32,
    pub required_level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpellType {
    Combat,
    Healing,
    Utility,
    Defense,
}

pub static SPELLS: &[Spell] = &[
    Spell {
        key: "light",
        name: "Light",
        incantation: "luminos",
        description: "Creates a magical light to illuminate dark areas.",
        mana_cost: 5,
        spell_type: SpellType::Utility,
        power: 0,
        required_level: 1,
    },
    Spell {
        key: "heal",
        name: "Heal",
        incantation: "vitae restauro",
        description: "Restores health to the caster.",
        mana_cost: 15,
        spell_type: SpellType::Healing,
        power: 25,
        required_level: 1,
    },
    Spell {
        key: "shield",
        name: "Shield",
        incantation: "protego magnus",
        description: "Creates a magical barrier that reduces incoming damage.",
        mana_cost: 10,
        spell_type: SpellType::Defense,
        power: 20,
        required_level: 2,
    },
    Spell {
        key: "fireball",
        name: "Fireball",
        incantation: "ignis sphaera",
        description: "Hurls a ball of fire at the enemy.",
        mana_cost: 20,
        spell_type: SpellType::Combat,
        power: 30,
        required_level: 2,
    },
    Spell {
        key: "lightning",
        name: "Lightning Bolt",
        incantation: "fulgur caeli",
        description: "Calls down a bolt of lightning to strike the enemy.",
        mana_cost: 25,
        spell_type: SpellType::Combat,
        power: 40,
        required_level: 3,
    },
    Spell {
        key: "ice_shard",
        name: "Ice Shard",
        incantation: "glacies acuta",
        description: "Launches a shard of ice that damages and slows the enemy.",
        mana_cost: 18,
        spell_type: SpellType::Combat,
        power: 25,
        required_level: 2,
    },
    Spell {
        key: "teleport",
        name: "Teleport",
        incantation: "transitus locus",
        description: "Teleports the caster to a known location.",
        mana_cost: 30,
        spell_type: SpellType::Utility,
        power: 0,
        required_level: 4,
    },
    Spell {
        key: "detect_magic",
        name: "Detect Magic",
        incantation: "revelo arcanum",
        description: "Reveals hidden magical items and passages.",
        mana_cost: 10,
        spell_type: SpellType::Utility,
        power: 0,
        required_level: 2,
    },
    Spell {
        key: "ward",
        name: "Ward",
        incantation: "custos liminis",
        description: "Creates a protective ward that prevents entry.",
        mana_cost: 20,
        spell_type: SpellType::Defense,
        power: 30,
        required_level: 3,
    },
    Spell {
        key: "greater_heal",
        name: "Greater Heal",
        incantation: "vitae magnus restauro",
        description: "Powerfully restores health to the caster.",
        mana_cost: 35,
        spell_type: SpellType::Healing,
        power: 60,
        required_level: 4,
    },
    Spell {
        key: "inferno",
        name: "Inferno",
        incantation: "inferno totalis",
        description: "Creates a devastating ring of fire around the caster.",
        mana_cost: 50,
        spell_type: SpellType::Combat,
        power: 75,
        required_level: 5,
    },
    Spell {
        key: "arcane_blast",
        name: "Arcane Blast",
        incantation: "vis arcana explodo",
        description: "Releases pure magical energy in a devastating blast.",
        mana_cost: 60,
        spell_type: SpellType::Combat,
        power: 100,
        required_level: 6,
    },
    Spell {
        key: "resurrection",
        name: "Resurrection",
        incantation: "anima redux vitae",
        description: "Returns a fallen ally to life (can only be used at altars).",
        mana_cost: 100,
        spell_type: SpellType::Healing,
        power: 0,
        required_level: 6,
    },
    Spell {
        key: "glory_tashanna",
        name: "Glory to Tashanna",
        incantation: "glory be to tashanna",
        description: "A sacred incantation that reveals hidden truths.",
        mana_cost: 0,
        spell_type: SpellType::Utility,
        power: 0,
        required_level: 3,
    },
];

pub fn get_spell(key: &str) -> Option<&'static Spell> {
    SPELLS.iter().find(|s| s.key == key)
}

pub fn get_spell_by_incantation(incantation: &str) -> Option<&'static Spell> {
    let lower = incantation.to_lowercase();
    SPELLS.iter().find(|s| s.incantation == lower)
}

// ============================================================================
// NPCS
// ============================================================================

#[derive(Debug, Clone)]
pub struct Npc {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub npc_type: NpcType,
    pub dialogue: &'static [&'static str],
    pub shop_items: &'static [&'static str],
    pub is_romanceable: bool,
    pub gender: Gender,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NpcType {
    Friendly,
    Merchant,
    Trainer,
    Quest,
    Enemy,
    Boss,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gender {
    Male,
    Female,
    Other,
}

pub static NPCS: &[Npc] = &[
    Npc {
        key: "elder_quinn",
        name: "Elder Quinn",
        description: "A wise old man with a long white beard and kind eyes. He has guided many apprentices on their journey.",
        npc_type: NpcType::Quest,
        dialogue: &[
            "Welcome, young apprentice. Kyrandia has long awaited one with your potential.",
            "The path to becoming Arch-Mage is fraught with danger. But great rewards await the worthy.",
            "Seek the Fountain of Scrolls in the Golden Forest. Pine cones thrown within become spell scrolls.",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Male,
    },
    Npc {
        key: "innkeeper_mira",
        name: "Innkeeper Mira",
        description: "A cheerful woman with rosy cheeks and a warm smile. She runs the Rusty Cauldron Inn.",
        npc_type: NpcType::Friendly,
        dialogue: &[
            "Rest well, traveler! A good night's sleep works wonders.",
            "I've heard strange sounds from the forest lately. Be careful out there.",
            "My stew is famous across the land. Would you like a bowl?",
        ],
        shop_items: &[],
        is_romanceable: true,
        gender: Gender::Female,
    },
    Npc {
        key: "merchant_felix",
        name: "Merchant Felix",
        description: "A shrewd-looking man surrounded by magical wares. His eyes glitter with the promise of deals.",
        npc_type: NpcType::Merchant,
        dialogue: &[
            "Looking to buy or sell? I deal in only the finest magical goods.",
            "Everything has a price, my friend. Everything.",
        ],
        shop_items: &["health_potion", "mana_potion", "wooden_staff", "apprentice_robe", "scroll_light"],
        is_romanceable: false,
        gender: Gender::Male,
    },
    Npc {
        key: "sage_orion",
        name: "Sage Orion",
        description: "An elderly scholar surrounded by towering stacks of books. His spectacles are always perched on his nose.",
        npc_type: NpcType::Quest,
        dialogue: &[
            "Knowledge is the greatest magic of all, young one.",
            "The library holds many secrets. Study well.",
            "I've been researching the ancient ritual. It requires three items of power...",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Male,
    },
    Npc {
        key: "guard_bran",
        name: "Guard Bran",
        description: "A stern-faced guard in worn armor. He takes his duty seriously.",
        npc_type: NpcType::Friendly,
        dialogue: &[
            "The forest is dangerous. Don't go unprepared.",
            "I've seen many young mages head out. Not all return.",
        ],
        shop_items: &[],
        is_romanceable: true,
        gender: Gender::Male,
    },
    Npc {
        key: "trainer_grok",
        name: "Trainer Grok",
        description: "A muscular half-orc with surprising patience. He trains apprentices in combat magic.",
        npc_type: NpcType::Trainer,
        dialogue: &[
            "Magic is nothing without focus! Again!",
            "You have potential. Let's see if you can survive my training.",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Male,
    },
    Npc {
        key: "spirit_elara",
        name: "Spirit Elara",
        description: "A translucent female spirit hovering sadly in the hollow. She seems trapped between worlds.",
        npc_type: NpcType::Quest,
        dialogue: &[
            "I was once a mage like you... before the darkness claimed me...",
            "Free me from this curse and I will reward you with ancient knowledge.",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Female,
    },
    Npc {
        key: "wanderer_zeph",
        name: "Wanderer Zeph",
        description: "A mysterious traveler wrapped in a tattered cloak. They seem to know more than they let on.",
        npc_type: NpcType::Quest,
        dialogue: &[
            "The crossroads... where paths meet and fates intertwine.",
            "Beware the mages who came before. Not all sought glory for noble ends.",
        ],
        shop_items: &[],
        is_romanceable: true,
        gender: Gender::Other,
    },
    Npc {
        key: "gatekeeper_lumina",
        name: "Gatekeeper Lumina",
        description: "A luminous being of pure light guards the entrance to the Golden Forest.",
        npc_type: NpcType::Quest,
        dialogue: &[
            "Only those with the Golden Key may pass into the sacred forest.",
            "Prove your worth, apprentice, and the path will open.",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Female,
    },
    Npc {
        key: "fountain_sprite",
        name: "Fountain Sprite",
        description: "A playful water spirit dancing on the fountain's surface.",
        npc_type: NpcType::Friendly,
        dialogue: &[
            "Throw pine cones into my waters, three at a time!",
            "The magic will transform them into a scroll of power!",
            "The spell you receive depends on many things... luck, skill, destiny!",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Female,
    },
    Npc {
        key: "treant_elder",
        name: "Elder Treant",
        description: "An ancient tree creature whose bark-covered face shows wisdom beyond measure.",
        npc_type: NpcType::Quest,
        dialogue: &[
            "I have stood in this grove for a thousand years...",
            "The young mages come and go. So few reach the castle now.",
            "To defeat the dragon, you must understand its fire. It is lonely...",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Other,
    },
    Npc {
        key: "ghost_archmage",
        name: "Ghost of the Last Arch-Mage",
        description: "A spectral figure in tattered robes. He was the last to hold the title before the dragon came.",
        npc_type: NpcType::Quest,
        dialogue: &[
            "I failed... the dragon was too powerful...",
            "You must not repeat my mistakes. Gather the three artifacts!",
            "The Tome, the Amulet, and the Charred Rose. Only with all three...",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Male,
    },
    Npc {
        key: "dragon_pyraxis",
        name: "Pyraxis, the Flame Guardian",
        description: "An enormous red dragon with scales like burning embers. Its golden eyes hold ancient intelligence.",
        npc_type: NpcType::Boss,
        dialogue: &[
            "ANOTHER MORTAL DARES ENTER MY LAIR?",
            "I HAVE GUARDED THIS CASTLE FOR CENTURIES. NONE SHALL CLAIM THE THRONE!",
            "VERY WELL... IF YOU SEEK GLORY, FACE MY FLAMES!",
        ],
        shop_items: &[],
        is_romanceable: false,
        gender: Gender::Male,
    },
];

pub fn get_npc(key: &str) -> Option<&'static Npc> {
    NPCS.iter().find(|n| n.key == key)
}

// ============================================================================
// MONSTERS
// ============================================================================

#[derive(Debug, Clone)]
pub struct Monster {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub region: Region,
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub xp_reward: u64,
    pub gold_reward: i64,
    pub drops: &'static [(&'static str, u8)],  // (item_key, chance_percent)
}

pub static MONSTERS: &[Monster] = &[
    // Village area (level 1)
    Monster {
        key: "rat",
        name: "Giant Rat",
        description: "An oversized rodent with sharp teeth.",
        region: Region::Village,
        hp: 10,
        attack: 5,
        defense: 2,
        xp_reward: 5,
        gold_reward: 3,
        drops: &[],
    },
    Monster {
        key: "goblin_scout",
        name: "Goblin Scout",
        description: "A small green creature armed with a crude dagger.",
        region: Region::Village,
        hp: 15,
        attack: 8,
        defense: 3,
        xp_reward: 10,
        gold_reward: 8,
        drops: &[("health_potion", 10)],
    },

    // Dark Forest (level 2-3)
    Monster {
        key: "shadow_wolf",
        name: "Shadow Wolf",
        description: "A wolf-like creature made of living shadow.",
        region: Region::DarkForest,
        hp: 30,
        attack: 15,
        defense: 8,
        xp_reward: 25,
        gold_reward: 15,
        drops: &[("ghost_orchid", 15)],
    },
    Monster {
        key: "dark_mage",
        name: "Dark Mage",
        description: "A corrupted mage who turned to forbidden magic.",
        region: Region::DarkForest,
        hp: 25,
        attack: 20,
        defense: 5,
        xp_reward: 35,
        gold_reward: 25,
        drops: &[("mana_potion", 20), ("scroll_fireball", 5)],
    },
    Monster {
        key: "forest_troll",
        name: "Forest Troll",
        description: "A massive regenerating brute covered in moss.",
        region: Region::DarkForest,
        hp: 50,
        attack: 18,
        defense: 12,
        xp_reward: 45,
        gold_reward: 30,
        drops: &[("health_potion", 25)],
    },

    // Golden Forest (level 4-5)
    Monster {
        key: "light_elemental",
        name: "Wild Light Elemental",
        description: "A being of pure light that has gone feral.",
        region: Region::GoldenForest,
        hp: 45,
        attack: 25,
        defense: 15,
        xp_reward: 60,
        gold_reward: 40,
        drops: &[("magic_crystal", 10)],
    },
    Monster {
        key: "corrupted_treant",
        name: "Corrupted Treant",
        description: "An ancient tree spirit twisted by dark magic.",
        region: Region::GoldenForest,
        hp: 70,
        attack: 22,
        defense: 20,
        xp_reward: 80,
        gold_reward: 50,
        drops: &[("golden_leaf", 20)],
    },
    Monster {
        key: "rival_mage",
        name: "Rival Mage",
        description: "Another mage seeking the title of Arch-Mage. They attack first!",
        region: Region::GoldenForest,
        hp: 55,
        attack: 30,
        defense: 12,
        xp_reward: 100,
        gold_reward: 75,
        drops: &[("scroll_teleport", 5), ("mana_potion", 30)],
    },

    // Dragon Castle (level 6-7)
    Monster {
        key: "castle_guardian",
        name: "Animated Armor",
        description: "A suit of armor brought to life by ancient magic.",
        region: Region::DragonCastle,
        hp: 80,
        attack: 30,
        defense: 25,
        xp_reward: 120,
        gold_reward: 80,
        drops: &[("mage_robe", 5)],
    },
    Monster {
        key: "fire_demon",
        name: "Fire Demon",
        description: "A demon born from the dragon's flame.",
        region: Region::DragonCastle,
        hp: 100,
        attack: 40,
        defense: 20,
        xp_reward: 150,
        gold_reward: 100,
        drops: &[("scroll_inferno", 3)],
    },
    Monster {
        key: "shadow_archmage",
        name: "Shadow Arch-Mage",
        description: "The corrupted ghost of a former Arch-Mage. Immensely powerful.",
        region: Region::DragonCastle,
        hp: 120,
        attack: 50,
        defense: 25,
        xp_reward: 200,
        gold_reward: 150,
        drops: &[("archmage_robe", 10), ("enchanted_staff", 10)],
    },
];

pub fn get_monster(key: &str) -> Option<&'static Monster> {
    MONSTERS.iter().find(|m| m.key == key)
}

pub fn get_monsters_for_region(region: Region) -> Vec<&'static Monster> {
    MONSTERS.iter().filter(|m| m.region == region).collect()
}

// ============================================================================
// CHARACTER RANKS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MageRank {
    Apprentice = 1,
    Initiate = 2,
    Acolyte = 3,
    Mage = 4,
    Sorcerer = 5,
    Wizard = 6,
    ArchMage = 7,
}

impl MageRank {
    pub fn from_level(level: u8) -> Self {
        match level {
            1 => MageRank::Apprentice,
            2 => MageRank::Initiate,
            3 => MageRank::Acolyte,
            4 => MageRank::Mage,
            5 => MageRank::Sorcerer,
            6 => MageRank::Wizard,
            _ => MageRank::ArchMage,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MageRank::Apprentice => "Apprentice",
            MageRank::Initiate => "Initiate",
            MageRank::Acolyte => "Acolyte",
            MageRank::Mage => "Mage",
            MageRank::Sorcerer => "Sorcerer",
            MageRank::Wizard => "Wizard",
            MageRank::ArchMage => "Arch-Mage of Legends",
        }
    }

    pub fn xp_required(&self) -> u64 {
        match self {
            MageRank::Apprentice => 0,
            MageRank::Initiate => 100,
            MageRank::Acolyte => 300,
            MageRank::Mage => 700,
            MageRank::Sorcerer => 1500,
            MageRank::Wizard => 3000,
            MageRank::ArchMage => 6000,
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get room exits as a formatted string
pub fn format_exits(room: &Room) -> String {
    room.exits.iter()
        .map(|(dir, _)| *dir)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_room() {
        assert!(get_room("village_square").is_some());
        assert!(get_room("nonexistent").is_none());
    }

    #[test]
    fn test_get_item() {
        assert!(get_item("pine_cone").is_some());
        assert!(get_item("nonexistent").is_none());
    }

    #[test]
    fn test_get_spell() {
        assert!(get_spell("fireball").is_some());
        assert!(get_spell("nonexistent").is_none());
    }

    #[test]
    fn test_spell_incantation() {
        let spell = get_spell_by_incantation("ignis sphaera");
        assert!(spell.is_some());
        assert_eq!(spell.unwrap().key, "fireball");
    }

    #[test]
    fn test_mage_rank() {
        assert_eq!(MageRank::from_level(1).name(), "Apprentice");
        assert_eq!(MageRank::from_level(7).name(), "Arch-Mage of Legends");
    }

    #[test]
    fn test_monsters_for_region() {
        let village_monsters = get_monsters_for_region(Region::Village);
        assert!(!village_monsters.is_empty());
        assert!(village_monsters.iter().all(|m| m.region == Region::Village));
    }
}
