//! ASCII representation of tiles for BBS display
//!
//! Converts tile IDs to ASCII characters with colors for text-mode rendering.

use super::map::{Direction, Map, Tile};

/// ANSI color codes for terminal display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Brown, // Often rendered as dark yellow
    Gray,  // Alias for bright black
}

impl Color {
    /// Get ANSI foreground color code
    pub fn fg_code(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[30m",
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow | Color::Brown => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[37m",
            Color::BrightBlack | Color::Gray => "\x1b[90m",
            Color::BrightRed => "\x1b[91m",
            Color::BrightGreen => "\x1b[92m",
            Color::BrightYellow => "\x1b[93m",
            Color::BrightBlue => "\x1b[94m",
            Color::BrightMagenta => "\x1b[95m",
            Color::BrightCyan => "\x1b[96m",
            Color::BrightWhite => "\x1b[97m",
        }
    }

    /// Get ANSI reset code
    pub fn reset() -> &'static str {
        "\x1b[0m"
    }
}

/// Result of converting a tile to ASCII
#[derive(Debug, Clone, Copy)]
pub struct AsciiTile {
    /// The character to display
    pub ch: char,
    /// Foreground color
    pub fg: Color,
    /// Optional background color
    pub bg: Option<Color>,
}

impl AsciiTile {
    pub fn new(ch: char, fg: Color) -> Self {
        Self { ch, fg, bg: None }
    }

    pub fn with_bg(ch: char, fg: Color, bg: Color) -> Self {
        Self { ch, fg, bg: Some(bg) }
    }

    /// Format with ANSI color codes
    pub fn to_ansi_string(&self) -> String {
        if let Some(bg) = self.bg {
            format!(
                "{}{}{}{}",
                self.fg.fg_code(),
                bg.fg_code().replace("[3", "[4").replace("[9", "[10"),
                self.ch,
                Color::reset()
            )
        } else {
            format!("{}{}{}", self.fg.fg_code(), self.ch, Color::reset())
        }
    }

    /// Get just the character (no color)
    pub fn to_plain_char(&self) -> char {
        self.ch
    }
}

/// Convert a tile to its ASCII representation
pub fn tile_to_ascii(tile: &Tile) -> AsciiTile {
    match tile.base_id {
        // Terrain - grass variants
        0..=9 => AsciiTile::new('.', Color::Green),

        // Water
        10..=19 => AsciiTile::new('~', Color::Blue),

        // Mountains
        20..=29 => AsciiTile::new('^', Color::Gray),

        // Trees/Forest
        30..=39 => AsciiTile::new('T', Color::BrightGreen),

        // Desert/Sand
        40..=49 => AsciiTile::new('#', Color::Yellow),

        // Lava
        50..=59 => AsciiTile::with_bg('*', Color::BrightYellow, Color::Red),

        // Snow/Ice
        60..=69 => AsciiTile::new('_', Color::BrightWhite),

        // Swamp
        70..=79 => AsciiTile::new('&', Color::Green),

        // Road/Path
        80..=89 => AsciiTile::new('=', Color::Brown),

        // Bridge
        90..=99 => AsciiTile::new('=', Color::Brown),

        // Castle walls
        100..=109 => AsciiTile::new('[', Color::BrightWhite),

        // Houses
        110..=119 => AsciiTile::new('+', Color::Brown),

        // Bridges
        120..=129 => AsciiTile::new('=', Color::Gray),

        // Towers
        130..=139 => AsciiTile::new('I', Color::BrightWhite),

        // Fences
        140..=149 => AsciiTile::new('-', Color::Brown),

        // Interior floor
        200..=209 => AsciiTile::new(' ', Color::White),

        // Interior walls
        210..=219 => AsciiTile::new('|', Color::Brown),

        // Doors
        220..=229 => AsciiTile::new('+', Color::Brown),

        // Stairs down
        250 => AsciiTile::new('>', Color::BrightYellow),

        // Stairs up
        251 => AsciiTile::new('<', Color::BrightYellow),

        // Chest
        252 => AsciiTile::new('?', Color::BrightCyan),

        // Save point
        253 => AsciiTile::new('!', Color::BrightRed),

        // Shrine/Altar
        254 => AsciiTile::new('*', Color::BrightMagenta),

        // Portal
        255 => AsciiTile::new('O', Color::BrightBlue),

        // Default
        _ => AsciiTile::new('.', Color::Gray),
    }
}

/// Get the ASCII character for the player based on direction
pub fn player_char(direction: Direction) -> char {
    match direction {
        Direction::Up => '^',
        Direction::Down => 'v',
        Direction::Left => '<',
        Direction::Right => '>',
    }
}

/// Render the visible map area as ASCII strings
pub fn render_map_ascii(
    map: &Map,
    player_pos: (u32, u32),
    player_dir: Direction,
    view_width: usize,
    view_height: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let half_w = view_width as i32 / 2;
    let half_h = view_height as i32 / 2;
    let px = player_pos.0 as i32;
    let py = player_pos.1 as i32;

    for dy in -half_h..=half_h {
        let mut line = String::new();
        for dx in -half_w..=half_w {
            let x = px + dx;
            let y = py + dy;

            if dx == 0 && dy == 0 {
                // Player position
                let player_ch = player_char(player_dir);
                line.push(player_ch);
            } else if let Some(npc) = map.get_npc_at(
                if x >= 0 { x as u32 } else { u32::MAX },
                if y >= 0 { y as u32 } else { u32::MAX },
            ) {
                // NPC - show as @
                let _ = npc; // Use the variable
                line.push('@');
            } else if let Some(tile) = map.get_tile(x, y) {
                let ascii = tile_to_ascii(tile);
                line.push(ascii.ch);
            } else {
                line.push(' ');
            }
        }
        lines.push(line);
    }
    lines
}

/// Render the visible map area as ASCII with ANSI colors
pub fn render_map_ascii_color(
    map: &Map,
    player_pos: (u32, u32),
    player_dir: Direction,
    view_width: usize,
    view_height: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let half_w = view_width as i32 / 2;
    let half_h = view_height as i32 / 2;
    let px = player_pos.0 as i32;
    let py = player_pos.1 as i32;

    for dy in -half_h..=half_h {
        let mut line = String::new();
        for dx in -half_w..=half_w {
            let x = px + dx;
            let y = py + dy;

            if dx == 0 && dy == 0 {
                // Player in bright white
                let player_ch = player_char(player_dir);
                line.push_str(Color::BrightWhite.fg_code());
                line.push(player_ch);
                line.push_str(Color::reset());
            } else if let Some(npc) = map.get_npc_at(
                if x >= 0 { x as u32 } else { u32::MAX },
                if y >= 0 { y as u32 } else { u32::MAX },
            ) {
                // NPC in magenta
                let _ = npc;
                line.push_str(Color::Magenta.fg_code());
                line.push('@');
                line.push_str(Color::reset());
            } else if let Some(tile) = map.get_tile(x, y) {
                let ascii = tile_to_ascii(tile);
                line.push_str(&ascii.to_ansi_string());
            } else {
                line.push(' ');
            }
        }
        lines.push(line);
    }
    lines
}

/// Create a simple ASCII mini-map
pub fn render_minimap(
    map: &Map,
    player_pos: (u32, u32),
    width: usize,
    height: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let scale_x = map.width as f32 / width as f32;
    let scale_y = map.height as f32 / height as f32;

    for row in 0..height {
        let mut line = String::new();
        for col in 0..width {
            let world_x = (col as f32 * scale_x) as i32;
            let world_y = (row as f32 * scale_y) as i32;

            // Check if player is at this scaled position
            let player_scaled_x = (player_pos.0 as f32 / scale_x) as usize;
            let player_scaled_y = (player_pos.1 as f32 / scale_y) as usize;

            if col == player_scaled_x && row == player_scaled_y {
                line.push('X');
            } else if let Some(tile) = map.get_tile(world_x, world_y) {
                // Simplified tile representation for minimap
                let ch = match tile.base_id {
                    0..=9 => '.',     // Grass
                    10..=19 => '~',   // Water
                    20..=29 => '^',   // Mountain
                    30..=39 => 'T',   // Forest
                    100..=139 => '#', // Buildings
                    _ => ' ',
                };
                line.push(ch);
            } else {
                line.push(' ');
            }
        }
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::realm_of_ralnar::map::NpcSpawn;

    #[test]
    fn test_tile_to_ascii_grass() {
        let tile = Tile::grass();
        let ascii = tile_to_ascii(&tile);
        assert_eq!(ascii.ch, '.');
        assert_eq!(ascii.fg, Color::Green);
    }

    #[test]
    fn test_tile_to_ascii_water() {
        let tile = Tile::water();
        let ascii = tile_to_ascii(&tile);
        assert_eq!(ascii.ch, '~');
        assert_eq!(ascii.fg, Color::Blue);
    }

    #[test]
    fn test_tile_to_ascii_tree() {
        let tile = Tile::tree();
        let ascii = tile_to_ascii(&tile);
        assert_eq!(ascii.ch, 'T');
        assert_eq!(ascii.fg, Color::BrightGreen);
    }

    #[test]
    fn test_tile_to_ascii_mountain() {
        let tile = Tile::mountain();
        let ascii = tile_to_ascii(&tile);
        assert_eq!(ascii.ch, '^');
        assert_eq!(ascii.fg, Color::Gray);
    }

    #[test]
    fn test_tile_to_ascii_lava() {
        let tile = Tile::lava();
        let ascii = tile_to_ascii(&tile);
        assert_eq!(ascii.ch, '*');
        assert!(ascii.bg.is_some());
    }

    #[test]
    fn test_player_char() {
        assert_eq!(player_char(Direction::Up), '^');
        assert_eq!(player_char(Direction::Down), 'v');
        assert_eq!(player_char(Direction::Left), '<');
        assert_eq!(player_char(Direction::Right), '>');
    }

    #[test]
    fn test_color_fg_code() {
        assert_eq!(Color::Red.fg_code(), "\x1b[31m");
        assert_eq!(Color::Green.fg_code(), "\x1b[32m");
        assert_eq!(Color::BrightWhite.fg_code(), "\x1b[97m");
    }

    #[test]
    fn test_ascii_tile_to_ansi_string() {
        let ascii = AsciiTile::new('.', Color::Green);
        let ansi = ascii.to_ansi_string();
        assert!(ansi.contains("\x1b[32m")); // Green
        assert!(ansi.contains(".")); // Character
        assert!(ansi.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn test_ascii_tile_to_plain_char() {
        let ascii = AsciiTile::new('T', Color::BrightGreen);
        assert_eq!(ascii.to_plain_char(), 'T');
    }

    #[test]
    fn test_render_map_ascii() {
        let map = Map::new("test".to_string(), 20, 20);
        let lines = render_map_ascii(&map, (10, 10), Direction::Down, 5, 5);

        // Should have 5 lines (-2 to +2)
        assert_eq!(lines.len(), 5);

        // Each line should have 5 characters
        for line in &lines {
            assert_eq!(line.len(), 5);
        }

        // Center should be player character
        assert_eq!(lines[2].chars().nth(2), Some('v'));
    }

    #[test]
    fn test_render_map_ascii_with_tree() {
        let mut map = Map::new("test".to_string(), 20, 20);
        map.tiles[10][11] = Tile::tree(); // Right of player (tiles[y][x])

        let lines = render_map_ascii(&map, (10, 10), Direction::Down, 5, 5);

        // Tree should appear as 'T'
        assert_eq!(lines[2].chars().nth(3), Some('T'));
    }

    #[test]
    fn test_render_map_ascii_with_npc() {
        let mut map = Map::new("test".to_string(), 20, 20);
        map.npcs.push(NpcSpawn {
            npc_id: "guard".to_string(),
            x: 11,
            y: 10,
            direction: Direction::Down,
            stationary: true,
        });

        let lines = render_map_ascii(&map, (10, 10), Direction::Down, 5, 5);

        // NPC should appear as '@' to the right of player
        assert_eq!(lines[2].chars().nth(3), Some('@'));
    }

    #[test]
    fn test_render_map_ascii_color() {
        let map = Map::new("test".to_string(), 20, 20);
        let lines = render_map_ascii_color(&map, (10, 10), Direction::Down, 5, 5);

        assert_eq!(lines.len(), 5);

        // Lines should contain ANSI codes
        for line in &lines {
            assert!(line.contains("\x1b["));
        }
    }

    #[test]
    fn test_render_minimap() {
        let map = Map::new("test".to_string(), 100, 100);
        let lines = render_minimap(&map, (50, 50), 10, 5);

        assert_eq!(lines.len(), 5);
        for line in &lines {
            assert_eq!(line.len(), 10);
        }

        // Player marker 'X' should be somewhere in the minimap
        let has_player = lines.iter().any(|line| line.contains('X'));
        assert!(has_player);
    }

    #[test]
    fn test_render_map_at_edge() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.map_type = super::super::map::MapType::Town;

        // Player at corner
        let lines = render_map_ascii(&map, (0, 0), Direction::Down, 5, 5);

        // Should still produce 5 lines
        assert_eq!(lines.len(), 5);

        // Some positions should be empty (outside map)
        let has_space = lines[0].contains(' ');
        assert!(has_space);
    }

    #[test]
    fn test_render_map_overworld_wrap() {
        let mut map = Map::new("world".to_string(), 10, 10);
        map.map_type = super::super::map::MapType::Overworld;

        // Player at corner - overworld should wrap
        let lines = render_map_ascii(&map, (0, 0), Direction::Down, 5, 5);

        assert_eq!(lines.len(), 5);

        // All positions should have content (wrapped tiles)
        for line in &lines {
            assert!(!line.trim().is_empty() || line.contains('.') || line.contains('v'));
        }
    }

    #[test]
    fn test_special_tiles() {
        // Stairs down
        let stairs_down = Tile {
            base_id: 250,
            overlay_id: None,
            attributes: Default::default(),
        };
        assert_eq!(tile_to_ascii(&stairs_down).ch, '>');

        // Stairs up
        let stairs_up = Tile {
            base_id: 251,
            overlay_id: None,
            attributes: Default::default(),
        };
        assert_eq!(tile_to_ascii(&stairs_up).ch, '<');

        // Chest
        let chest = Tile {
            base_id: 252,
            overlay_id: None,
            attributes: Default::default(),
        };
        assert_eq!(tile_to_ascii(&chest).ch, '?');

        // Save point
        let save = Tile {
            base_id: 253,
            overlay_id: None,
            attributes: Default::default(),
        };
        assert_eq!(tile_to_ascii(&save).ch, '!');
    }
}
