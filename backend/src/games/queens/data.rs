//! Static data for Queens game - colors and display constants

use crate::terminal::Color;

/// Region color identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionColor {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
    Cyan,
    Orange,
    Pink,
}

impl RegionColor {
    /// Get display character for region
    pub fn char(&self) -> char {
        match self {
            RegionColor::Red => 'R',
            RegionColor::Blue => 'B',
            RegionColor::Green => 'G',
            RegionColor::Yellow => 'Y',
            RegionColor::Purple => 'P',
            RegionColor::Cyan => 'C',
            RegionColor::Orange => 'O',
            RegionColor::Pink => 'K',
        }
    }

    /// Get ANSI color for region
    pub fn ansi_color(&self) -> Color {
        match self {
            RegionColor::Red => Color::LightRed,
            RegionColor::Blue => Color::LightBlue,
            RegionColor::Green => Color::LightGreen,
            RegionColor::Yellow => Color::Yellow,
            RegionColor::Purple => Color::LightMagenta,
            RegionColor::Cyan => Color::LightCyan,
            RegionColor::Orange => Color::Brown,
            RegionColor::Pink => Color::Magenta,
        }
    }

    /// Get full name for display
    pub fn name(&self) -> &'static str {
        match self {
            RegionColor::Red => "Red",
            RegionColor::Blue => "Blue",
            RegionColor::Green => "Green",
            RegionColor::Yellow => "Yellow",
            RegionColor::Purple => "Purple",
            RegionColor::Cyan => "Cyan",
            RegionColor::Orange => "Orange",
            RegionColor::Pink => "Pink",
        }
    }

    /// Get from index (0-7)
    pub fn from_index(idx: usize) -> Self {
        match idx % 8 {
            0 => RegionColor::Red,
            1 => RegionColor::Blue,
            2 => RegionColor::Green,
            3 => RegionColor::Yellow,
            4 => RegionColor::Purple,
            5 => RegionColor::Cyan,
            6 => RegionColor::Orange,
            _ => RegionColor::Pink,
        }
    }
}

/// All available region colors
pub const REGION_COLORS: [RegionColor; 8] = [
    RegionColor::Red,
    RegionColor::Blue,
    RegionColor::Green,
    RegionColor::Yellow,
    RegionColor::Purple,
    RegionColor::Cyan,
    RegionColor::Orange,
    RegionColor::Pink,
];

/// Queen symbol for display
pub const QUEEN_SYMBOL: char = '\u{2655}'; // White chess queen

/// Empty cell symbol
pub const EMPTY_SYMBOL: char = ' ';

/// Board size range
pub const MIN_BOARD_SIZE: usize = 5;
pub const MAX_BOARD_SIZE: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_color_chars() {
        assert_eq!(RegionColor::Red.char(), 'R');
        assert_eq!(RegionColor::Blue.char(), 'B');
        assert_eq!(RegionColor::Purple.char(), 'P');
    }

    #[test]
    fn test_region_color_from_index() {
        assert_eq!(RegionColor::from_index(0), RegionColor::Red);
        assert_eq!(RegionColor::from_index(7), RegionColor::Pink);
        assert_eq!(RegionColor::from_index(8), RegionColor::Red); // Wraps
    }

    #[test]
    fn test_all_colors_unique_chars() {
        let chars: Vec<char> = REGION_COLORS.iter().map(|c| c.char()).collect();
        for (i, c) in chars.iter().enumerate() {
            for (j, other) in chars.iter().enumerate() {
                if i != j {
                    assert_ne!(c, other, "Colors {} and {} have same char", i, j);
                }
            }
        }
    }
}
