//! World map and location data for Last Dream
//! Overworld, towns, dungeons, and transportation

use serde::{Deserialize, Serialize};

// ============================================================================
// MAP TILES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Grass,
    Forest,
    Mountain,
    Water,
    Desert,
    Snow,
    Town,
    Castle,
    Dungeon,
    Bridge,
    Dock,
    Cave,
}

impl Tile {
    pub fn char(&self) -> char {
        match self {
            Tile::Grass => '.',
            Tile::Forest => '#',
            Tile::Mountain => '^',
            Tile::Water => '~',
            Tile::Desert => ':',
            Tile::Snow => '*',
            Tile::Town => 'O',
            Tile::Castle => 'C',
            Tile::Dungeon => 'X',
            Tile::Bridge => '=',
            Tile::Dock => 'D',
            Tile::Cave => 'U',
        }
    }

    pub fn walkable(&self, transport: Transportation) -> bool {
        match (self, transport) {
            // Walking
            (Tile::Water, Transportation::Walking) => false,
            (Tile::Mountain, Transportation::Walking) => false,

            // Ship
            (Tile::Water, Transportation::Ship) => true,
            (Tile::Dock, Transportation::Ship) => true,
            (_, Transportation::Ship) => false,

            // Airship
            (Tile::Mountain, Transportation::Airship) => false,
            (_, Transportation::Airship) => true,

            // All land tiles
            _ => true,
        }
    }

    pub fn encounter_rate(&self) -> u8 {
        match self {
            Tile::Grass => 8,
            Tile::Forest => 15,
            Tile::Desert => 12,
            Tile::Snow => 10,
            Tile::Water => 5, // Ship encounters
            _ => 0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tile::Grass => "Grassland",
            Tile::Forest => "Forest",
            Tile::Mountain => "Mountain",
            Tile::Water => "Ocean",
            Tile::Desert => "Desert",
            Tile::Snow => "Snowfield",
            Tile::Town => "Town",
            Tile::Castle => "Castle",
            Tile::Dungeon => "Dungeon",
            Tile::Bridge => "Bridge",
            Tile::Dock => "Dock",
            Tile::Cave => "Cave",
        }
    }
}

// ============================================================================
// TRANSPORTATION
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Transportation {
    #[default]
    Walking,
    Ship,
    Airship,
}

impl Transportation {
    pub fn name(&self) -> &'static str {
        match self {
            Transportation::Walking => "Walking",
            Transportation::Ship => "Ship",
            Transportation::Airship => "Airship",
        }
    }
}

// ============================================================================
// WORLD MAP
// ============================================================================

/// The overworld map
pub struct WorldMap {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
}

impl WorldMap {
    /// Create a new world map
    pub fn new() -> Self {
        // 40x20 world map
        const WIDTH: usize = 40;
        const HEIGHT: usize = 20;

        // Map layout (simplified for ASCII representation)
        let map_str = concat!(
            "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~",
            "~~~~...###..~~~..##....~~~...##...~~~~~",
            "~~~....#O#..~~~.###....~~~..###...~~~~~",
            "~~.....###.D~~~~##.....~~X.####....~~~~",
            "~~......#...~~~~~~~....~~~~~~~~....~~~~",
            "~~O..........~~~~~~~~..~~~~~~~~~...~~~~",
            "~~.....^^^^^.~~~~~~~..~~~~~~~~~~...~~~~",
            "~~....^^^^^^..~~~~~~..~~~~~~~~~~..~~~~~",
            "~~~...^C^^^^...~~~~..~~~~~~~~~~~~.~~~~~",
            "~~~....^^^^....~~~D.~~~~~~~~~~~~~~~~~~~",
            "~~~~.....O.....~~~..~~~~~~~~~~~...~~~~~",
            "~~~~~.........~~~~..~~.........O.~~~~~~",
            "~~~~~~....X...~~~~..~~.....^^^...~~~~~~",
            "~~~~~~~.......~~~~D~~~....^^^^...~~~~~~",
            "~~~~~~~~......~~~~.~~~....^^^X..~~~~~~~",
            "~~~~~~~~~.O...~~~~.~~~.....C...~~~~~~~~",
            "~~~~~~~~~~....~~~~.~~~~........~~~~~~~~",
            "~~~~~~~~~~~...~~~~.~~~~~......~~~~~~~~~",
            "~~~~~~~~~~~~..~~~~~.~~~~~~...~~~~~~~~~~",
            "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~"
        );

        let tiles: Vec<Tile> = map_str.chars().map(|c| match c {
            '.' => Tile::Grass,
            '#' => Tile::Forest,
            '^' => Tile::Mountain,
            '~' => Tile::Water,
            ':' => Tile::Desert,
            '*' => Tile::Snow,
            'O' => Tile::Town,
            'C' => Tile::Castle,
            'X' => Tile::Dungeon,
            '=' => Tile::Bridge,
            'D' => Tile::Dock,
            'U' => Tile::Cave,
            _ => Tile::Water,
        }).collect();

        Self {
            width: WIDTH,
            height: HEIGHT,
            tiles,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Tile {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x]
        } else {
            Tile::Water
        }
    }

    /// Check if position is walkable with given transportation
    pub fn can_walk(&self, x: usize, y: usize, transport: Transportation) -> bool {
        self.get(x, y).walkable(transport)
    }

    /// Get location name at position (if any)
    pub fn location_at(&self, x: usize, y: usize) -> Option<&'static Location> {
        LOCATIONS.iter().find(|loc| loc.world_x == x && loc.world_y == y)
    }
}

impl Default for WorldMap {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// LOCATIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    Town,
    Castle,
    Dungeon,
    Cave,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub key: &'static str,
    pub name: &'static str,
    pub location_type: LocationType,
    pub world_x: usize,
    pub world_y: usize,
    pub area_level: u8,
    pub has_inn: bool,
    pub has_shop: bool,
    pub has_save_point: bool,
    pub description: &'static str,
    pub story_flag_required: Option<&'static str>,
}

pub static LOCATIONS: &[Location] = &[
    // Starting area
    Location {
        key: "cornelia",
        name: "Cornelia Village",
        location_type: LocationType::Town,
        world_x: 2,
        world_y: 5,
        area_level: 1,
        has_inn: true,
        has_shop: true,
        has_save_point: true,
        description: "A peaceful village where your journey begins.",
        story_flag_required: None,
    },
    Location {
        key: "cornelia_castle",
        name: "Cornelia Castle",
        location_type: LocationType::Castle,
        world_x: 7,
        world_y: 8,
        area_level: 3,
        has_inn: false,
        has_shop: false,
        has_save_point: true,
        description: "Home of the King and the Crystal of Earth.",
        story_flag_required: None,
    },
    Location {
        key: "chaos_shrine",
        name: "Chaos Shrine",
        location_type: LocationType::Dungeon,
        world_x: 6,
        world_y: 2,
        area_level: 5,
        has_inn: false,
        has_shop: false,
        has_save_point: false,
        description: "An ancient temple where darkness gathers.",
        story_flag_required: None,
    },

    // Second continent
    Location {
        key: "pravoka",
        name: "Pravoka Port",
        location_type: LocationType::Town,
        world_x: 10,
        world_y: 10,
        area_level: 8,
        has_inn: true,
        has_shop: true,
        has_save_point: true,
        description: "A bustling port town. Pirates trouble the seas.",
        story_flag_required: Some("ship_obtained"),
    },
    Location {
        key: "elfheim",
        name: "Elfheim",
        location_type: LocationType::Town,
        world_x: 29,
        world_y: 11,
        area_level: 12,
        has_inn: true,
        has_shop: true,
        has_save_point: true,
        description: "Home of the elves and ancient magic.",
        story_flag_required: Some("ship_obtained"),
    },
    Location {
        key: "marsh_cave",
        name: "Marsh Cave",
        location_type: LocationType::Dungeon,
        world_x: 22,
        world_y: 3,
        area_level: 10,
        has_inn: false,
        has_shop: false,
        has_save_point: false,
        description: "A dank cave filled with undead.",
        story_flag_required: Some("ship_obtained"),
    },

    // Third continent - Fire
    Location {
        key: "melmond",
        name: "Melmond",
        location_type: LocationType::Town,
        world_x: 6,
        world_y: 15,
        area_level: 15,
        has_inn: true,
        has_shop: true,
        has_save_point: true,
        description: "A town troubled by the rotting earth.",
        story_flag_required: Some("earth_crystal_lit"),
    },
    Location {
        key: "volcano",
        name: "Gurgu Volcano",
        location_type: LocationType::Dungeon,
        world_x: 30,
        world_y: 14,
        area_level: 20,
        has_inn: false,
        has_shop: false,
        has_save_point: false,
        description: "An active volcano hiding the Fire Crystal.",
        story_flag_required: Some("earth_crystal_lit"),
    },

    // Final areas
    Location {
        key: "onrac",
        name: "Onrac",
        location_type: LocationType::Town,
        world_x: 28,
        world_y: 15,
        area_level: 25,
        has_inn: true,
        has_shop: true,
        has_save_point: true,
        description: "A mysterious underwater city.",
        story_flag_required: Some("fire_crystal_lit"),
    },
    Location {
        key: "mirage_tower",
        name: "Mirage Tower",
        location_type: LocationType::Dungeon,
        world_x: 30,
        world_y: 12,
        area_level: 30,
        has_inn: false,
        has_shop: false,
        has_save_point: false,
        description: "A tower that leads to the floating fortress.",
        story_flag_required: Some("airship_obtained"),
    },
    Location {
        key: "sky_fortress",
        name: "Sky Fortress",
        location_type: LocationType::Castle,
        world_x: 32,
        world_y: 15,
        area_level: 35,
        has_inn: false,
        has_shop: false,
        has_save_point: true,
        description: "The floating fortress of the Wind Fiend.",
        story_flag_required: Some("airship_obtained"),
    },
    Location {
        key: "the_rift",
        name: "The Rift",
        location_type: LocationType::Dungeon,
        world_x: 20,
        world_y: 10,
        area_level: 40,
        has_inn: false,
        has_shop: false,
        has_save_point: true,
        description: "The boundary between reality and the Void.",
        story_flag_required: Some("four_crystals_lit"),
    },
];

pub fn get_location(key: &str) -> Option<&'static Location> {
    LOCATIONS.iter().find(|l| l.key == key)
}

// ============================================================================
// DUNGEON FLOORS
// ============================================================================

#[derive(Debug, Clone)]
pub struct DungeonFloor {
    pub dungeon: String,
    pub floor: u8,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<DungeonTile>,
    pub encounters: Vec<&'static str>,
    pub boss: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonTile {
    Floor,
    Wall,
    Door,
    Chest,
    Stairs,
    BossRoom,
    SavePoint,
    Entrance,
}

impl DungeonTile {
    pub fn char(&self) -> char {
        match self {
            DungeonTile::Floor => '.',
            DungeonTile::Wall => '#',
            DungeonTile::Door => '+',
            DungeonTile::Chest => '$',
            DungeonTile::Stairs => '>',
            DungeonTile::BossRoom => 'B',
            DungeonTile::SavePoint => 'S',
            DungeonTile::Entrance => '<',
        }
    }

    pub fn walkable(&self) -> bool {
        !matches!(self, DungeonTile::Wall)
    }
}

/// Generate a simple dungeon floor
pub fn generate_dungeon_floor(dungeon_key: &str, floor: u8) -> DungeonFloor {
    // Simple 20x10 dungeon layout
    const WIDTH: usize = 20;
    const HEIGHT: usize = 10;

    let base_layout = concat!(
        "####################",
        "#.<.....+....$.....#",
        "#.###...#...###....#",
        "#.#.....#.....#....#",
        "#.#..$.######.#..$.#",
        "#.#...........#....#",
        "#.###...#...###....#",
        "#.......#..........#",
        "#.......#.......>..#",
        "####################"
    );

    let tiles: Vec<DungeonTile> = base_layout.chars().map(|c| match c {
        '.' => DungeonTile::Floor,
        '#' => DungeonTile::Wall,
        '+' => DungeonTile::Door,
        '$' => DungeonTile::Chest,
        '>' => DungeonTile::Stairs,
        'B' => DungeonTile::BossRoom,
        'S' => DungeonTile::SavePoint,
        '<' => DungeonTile::Entrance,
        _ => DungeonTile::Wall,
    }).collect();

    // Get encounters based on dungeon
    let encounters = match dungeon_key {
        "chaos_shrine" => vec!["skeleton", "bat", "goblin"],
        "marsh_cave" => vec!["zombie", "ghost", "skeleton"],
        "volcano" => vec!["demon", "wyvern"],
        "mirage_tower" => vec!["dark_knight", "dragon"],
        "the_rift" => vec!["demon", "lich", "dragon"],
        _ => vec!["goblin", "slime"],
    };

    // Boss on final floor
    let boss = if floor == 5 {
        match dungeon_key {
            "chaos_shrine" => Some("earth_fiend"),
            "volcano" => Some("fire_fiend"),
            "mirage_tower" => Some("wind_fiend"),
            "the_rift" => Some("void_lord"),
            _ => None,
        }
    } else {
        None
    };

    DungeonFloor {
        dungeon: dungeon_key.to_string(),
        floor,
        width: WIDTH,
        height: HEIGHT,
        tiles,
        encounters,
        boss,
    }
}

// ============================================================================
// POSITION
// ============================================================================

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

// ============================================================================
// STORY FLAGS
// ============================================================================

/// Story progression flags
pub static STORY_FLAGS: &[&str] = &[
    // Transportation
    "ship_obtained",
    "airship_obtained",
    "canoe_obtained",

    // Crystals
    "earth_crystal_lit",
    "fire_crystal_lit",
    "water_crystal_lit",
    "wind_crystal_lit",
    "four_crystals_lit",

    // Story events
    "intro_complete",
    "pirate_defeated",
    "elf_prince_saved",
    "vampire_defeated",
    "sage_met",
    "void_opened",
    "game_complete",

    // Simulation hints seen (tracking for rarity)
    "hint_1_seen",
    "hint_2_seen",
];

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_map() {
        let map = WorldMap::new();
        assert_eq!(map.width, 40);
        assert_eq!(map.height, 20);
    }

    #[test]
    fn test_tile_walkability() {
        assert!(Tile::Grass.walkable(Transportation::Walking));
        assert!(!Tile::Water.walkable(Transportation::Walking));
        assert!(Tile::Water.walkable(Transportation::Ship));
        assert!(!Tile::Mountain.walkable(Transportation::Airship));
    }

    #[test]
    fn test_location_lookup() {
        assert!(get_location("cornelia").is_some());
        assert!(get_location("nonexistent").is_none());
    }

    #[test]
    fn test_starting_town() {
        let cornelia = get_location("cornelia").unwrap();
        assert!(cornelia.has_inn);
        assert!(cornelia.has_shop);
        assert_eq!(cornelia.area_level, 1);
    }

    #[test]
    fn test_dungeon_generation() {
        let floor = generate_dungeon_floor("chaos_shrine", 1);
        assert_eq!(floor.width, 20);
        assert_eq!(floor.height, 10);
        assert!(!floor.encounters.is_empty());
    }

    #[test]
    fn test_dungeon_tiles() {
        assert!(DungeonTile::Floor.walkable());
        assert!(!DungeonTile::Wall.walkable());
    }
}
