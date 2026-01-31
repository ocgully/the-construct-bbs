//! Static game data for Tanks
//!
//! Defines weapons, power-ups, and game constants.

/// Maximum players in a game
pub const MAX_PLAYERS: usize = 8;

/// Minimum players to start
pub const MIN_PLAYERS: usize = 2;

/// Game field dimensions (80x24 terminal, leave room for UI)
pub const FIELD_WIDTH: usize = 80;
pub const FIELD_HEIGHT: usize = 20;

/// Physics constants
pub const GRAVITY: f64 = 0.15;
pub const MAX_POWER: u32 = 100;
pub const MIN_POWER: u32 = 10;
pub const MAX_ANGLE: i32 = 180;
pub const MIN_ANGLE: i32 = 0;

/// Turn time limit in seconds
pub const TURN_TIME_SECS: u64 = 30;

/// Time between turns for results display
pub const RESULT_TIME_SECS: u64 = 3;

/// Wind strength range (-10 to +10)
pub const MAX_WIND: i32 = 10;

/// Tank starting health
pub const TANK_MAX_HEALTH: u32 = 100;

/// Explosion radius for standard shell
pub const STANDARD_EXPLOSION_RADIUS: u32 = 3;

/// Weapon definitions
#[derive(Debug, Clone, PartialEq)]
pub struct WeaponDef {
    pub key: &'static str,
    pub name: &'static str,
    pub damage: u32,
    pub explosion_radius: u32,
    pub ammo: Option<u32>,  // None = unlimited
    pub description: &'static str,
}

/// Available weapons
pub static WEAPONS: &[WeaponDef] = &[
    WeaponDef {
        key: "standard",
        name: "Standard Shell",
        damage: 25,
        explosion_radius: 3,
        ammo: None,
        description: "Basic artillery shell",
    },
    WeaponDef {
        key: "baby_missile",
        name: "Baby Missile",
        damage: 15,
        explosion_radius: 2,
        ammo: Some(5),
        description: "Small but accurate",
    },
    WeaponDef {
        key: "heavy_shell",
        name: "Heavy Shell",
        damage: 40,
        explosion_radius: 5,
        ammo: Some(3),
        description: "Massive damage, huge crater",
    },
    WeaponDef {
        key: "napalm",
        name: "Napalm",
        damage: 20,
        explosion_radius: 6,
        ammo: Some(2),
        description: "Burns wide area",
    },
    WeaponDef {
        key: "dirt_bomb",
        name: "Dirt Bomb",
        damage: 5,
        explosion_radius: 4,
        ammo: Some(3),
        description: "Adds terrain instead of destroying",
    },
    WeaponDef {
        key: "nuke",
        name: "Tactical Nuke",
        damage: 80,
        explosion_radius: 10,
        ammo: Some(1),
        description: "The big one",
    },
];

/// Get weapon definition by key
pub fn get_weapon(key: &str) -> Option<&'static WeaponDef> {
    WEAPONS.iter().find(|w| w.key == key)
}

/// Power-up items that can appear on the field
#[derive(Debug, Clone, PartialEq)]
pub struct PowerUp {
    pub key: &'static str,
    pub name: &'static str,
    pub effect: PowerUpEffect,
    pub symbol: char,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PowerUpEffect {
    Health(u32),
    Ammo { weapon: &'static str, count: u32 },
    Shield(u32),
    Fuel(u32),
}

pub static POWERUPS: &[PowerUp] = &[
    PowerUp {
        key: "repair",
        name: "Repair Kit",
        effect: PowerUpEffect::Health(25),
        symbol: '+',
    },
    PowerUp {
        key: "ammo",
        name: "Ammo Crate",
        effect: PowerUpEffect::Ammo { weapon: "heavy_shell", count: 2 },
        symbol: 'A',
    },
    PowerUp {
        key: "shield",
        name: "Shield",
        effect: PowerUpEffect::Shield(30),
        symbol: 'S',
    },
    PowerUp {
        key: "fuel",
        name: "Fuel Can",
        effect: PowerUpEffect::Fuel(20),
        symbol: 'F',
    },
];

/// Military-themed color palette for ANSI rendering
/// Using earthy/military tones to differentiate from other games
pub mod colors {
    use crate::terminal::Color;

    pub const OLIVE: Color = Color::Green;       // Olive drab for UI
    pub const KHAKI: Color = Color::Brown;       // Khaki for terrain
    pub const STEEL: Color = Color::LightGray;   // Steel for tanks
    pub const ALERT: Color = Color::LightRed;    // Alert/damage
    pub const AMMO: Color = Color::Yellow;       // Ammo/power indicators
    pub const RADAR: Color = Color::LightGreen;  // Radar/targeting
    pub const HEADER: Color = Color::LightCyan;  // Headers
    pub const EXPLOSION: Color = Color::LightRed; // Explosions
    pub const SMOKE: Color = Color::DarkGray;    // Smoke trails
    pub const SKY: Color = Color::Blue;          // Sky background
}

/// Tank color assignments by player index
pub fn tank_color(player_index: usize) -> crate::terminal::Color {
    use crate::terminal::Color;
    match player_index % 8 {
        0 => Color::LightGreen,
        1 => Color::LightRed,
        2 => Color::LightCyan,
        3 => Color::Yellow,
        4 => Color::LightMagenta,
        5 => Color::White,
        6 => Color::LightBlue,
        _ => Color::LightGray,
    }
}

/// ASCII art for tank (facing right)
pub const TANK_RIGHT: &[&str] = &[
    " _",
    "(_)===D",
    "[oOo]",
];

/// ASCII art for tank (facing left)
pub const TANK_LEFT: &[&str] = &[
    "   _",
    "D===(_)",
    " [oOo]",
];

/// Sound effect hints (for period-appropriate beeps)
pub mod sounds {
    pub const FIRE: &str = "\x07";         // Bell character for firing
    pub const EXPLOSION: &str = "\x07\x07"; // Double bell for explosion
    pub const HIT: &str = "\x07";          // Hit confirmation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_lookup() {
        let standard = get_weapon("standard");
        assert!(standard.is_some());
        assert_eq!(standard.unwrap().damage, 25);
        assert!(standard.unwrap().ammo.is_none());

        let nuke = get_weapon("nuke");
        assert!(nuke.is_some());
        assert_eq!(nuke.unwrap().ammo, Some(1));
    }

    #[test]
    fn test_tank_colors() {
        use crate::terminal::Color;
        assert_eq!(tank_color(0), Color::LightGreen);
        assert_eq!(tank_color(8), Color::LightGreen); // Wraps around
    }

    #[test]
    fn test_all_weapons_defined() {
        assert!(WEAPONS.len() >= 4);
        for weapon in WEAPONS {
            assert!(!weapon.key.is_empty());
            assert!(!weapon.name.is_empty());
            assert!(weapon.damage > 0 || weapon.key == "dirt_bomb");
            assert!(weapon.explosion_radius > 0);
        }
    }
}
