//! Static game data for Acromania
//!
//! Contains categories, letter weights, and acronym generation logic.

use rand::prelude::*;
use rand::seq::SliceRandom;

/// A category theme for acronym rounds
#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

/// Available categories for themed rounds
pub static CATEGORIES: &[Category] = &[
    Category {
        key: "open",
        name: "Open",
        description: "Anything goes!",
    },
    Category {
        key: "movies",
        name: "Movies & TV",
        description: "Film and television titles or quotes",
    },
    Category {
        key: "food",
        name: "Food & Drink",
        description: "Culinary creations and beverages",
    },
    Category {
        key: "tech",
        name: "Technology",
        description: "Gadgets, software, and tech jargon",
    },
    Category {
        key: "excuses",
        name: "Excuses",
        description: "Reasons why you can't do something",
    },
    Category {
        key: "pickup",
        name: "Pickup Lines",
        description: "Cheesy romantic approaches",
    },
    Category {
        key: "warning",
        name: "Warning Signs",
        description: "Cautionary notices and alerts",
    },
    Category {
        key: "bands",
        name: "Band Names",
        description: "Names for musical groups",
    },
    Category {
        key: "books",
        name: "Book Titles",
        description: "Names for books or chapters",
    },
    Category {
        key: "conspiracy",
        name: "Conspiracy Theories",
        description: "Wild and outlandish theories",
    },
];

/// Letter weights based on English frequency
/// Lower weight = less likely to appear (avoid QXZJ heavy acronyms)
static LETTER_WEIGHTS: [(char, u32); 26] = [
    ('A', 82),  ('B', 15),  ('C', 28),  ('D', 43),  ('E', 127),
    ('F', 22),  ('G', 20),  ('H', 61),  ('I', 70),  ('J', 2),
    ('K', 8),   ('L', 40),  ('M', 24),  ('N', 67),  ('O', 75),
    ('P', 19),  ('Q', 1),   ('R', 60),  ('S', 63),  ('T', 91),
    ('U', 28),  ('V', 10),  ('W', 24),  ('X', 2),   ('Y', 20),
    ('Z', 1),
];

/// Vowels for ensuring pronounceability
static VOWELS: [char; 5] = ['A', 'E', 'I', 'O', 'U'];

/// Get the acronym length for a given round (1-10)
pub fn acronym_length_for_round(round: u32) -> usize {
    match round {
        1..=3 => 3,   // Rounds 1-3: 3 letters
        4..=6 => 4,   // Rounds 4-6: 4 letters
        7..=9 => 5,   // Rounds 7-9: 5 letters
        10 => 6,       // Round 10 (Face-Off): 6-7 letters
        _ => 3,
    }
}

/// Generate a random acronym of the specified length
/// Ensures at least one vowel for 4+ letter acronyms
pub fn generate_acronym(length: usize) -> String {
    let mut rng = thread_rng();
    let mut acronym = String::with_capacity(length);

    // Build weighted letter pool
    let mut pool: Vec<char> = Vec::new();
    for (letter, weight) in LETTER_WEIGHTS.iter() {
        for _ in 0..*weight {
            pool.push(*letter);
        }
    }

    // Generate letters
    for _ in 0..length {
        if let Some(&letter) = pool.choose(&mut rng) {
            acronym.push(letter);
        }
    }

    // Ensure at least one vowel for longer acronyms (4+)
    if length >= 4 && !acronym.chars().any(|c| VOWELS.contains(&c)) {
        // Replace a random consonant with a vowel
        let pos = rng.gen_range(0..length);
        let vowel = VOWELS.choose(&mut rng).unwrap_or(&'A');
        let mut chars: Vec<char> = acronym.chars().collect();
        chars[pos] = *vowel;
        acronym = chars.into_iter().collect();
    }

    acronym
}

/// Get a random category (can be None for open round)
pub fn random_category() -> Option<&'static Category> {
    let mut rng = thread_rng();
    // 40% chance of open round
    if rng.gen_bool(0.4) {
        None
    } else {
        // Skip "open" category when selecting themed
        let themed: Vec<_> = CATEGORIES.iter().skip(1).collect();
        themed.choose(&mut rng).copied()
    }
}

/// Get category by key
#[allow(dead_code)]
pub fn get_category(key: &str) -> Option<&'static Category> {
    CATEGORIES.iter().find(|c| c.key == key)
}

/// Format acronym for display (with dots between letters)
pub fn format_acronym(acronym: &str) -> String {
    acronym.chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acronym_length_for_round() {
        assert_eq!(acronym_length_for_round(1), 3);
        assert_eq!(acronym_length_for_round(3), 3);
        assert_eq!(acronym_length_for_round(4), 4);
        assert_eq!(acronym_length_for_round(6), 4);
        assert_eq!(acronym_length_for_round(7), 5);
        assert_eq!(acronym_length_for_round(9), 5);
        assert_eq!(acronym_length_for_round(10), 6);
    }

    #[test]
    fn test_generate_acronym_length() {
        for len in 3..=7 {
            let acronym = generate_acronym(len);
            assert_eq!(acronym.len(), len);
        }
    }

    #[test]
    fn test_generate_acronym_has_vowel_for_long() {
        // Generate many 5-letter acronyms and verify they have vowels
        for _ in 0..100 {
            let acronym = generate_acronym(5);
            assert!(
                acronym.chars().any(|c| VOWELS.contains(&c)),
                "Acronym {} should have a vowel", acronym
            );
        }
    }

    #[test]
    fn test_format_acronym() {
        assert_eq!(format_acronym("ABC"), "A.B.C");
        assert_eq!(format_acronym("WTFL"), "W.T.F.L");
    }

    #[test]
    fn test_get_category() {
        let movies = get_category("movies");
        assert!(movies.is_some());
        assert_eq!(movies.unwrap().name, "Movies & TV");

        let unknown = get_category("nonexistent");
        assert!(unknown.is_none());
    }
}
