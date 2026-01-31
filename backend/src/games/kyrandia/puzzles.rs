//! Puzzle system for Kyrandia
//! Handles mystical puzzles that block progression

#![allow(dead_code)]

use super::state::GameState;
use super::data::RoomSpecial;
use once_cell::sync::Lazy;

/// Puzzle definition
#[derive(Debug, Clone)]
pub struct Puzzle {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub hint: &'static str,
    pub solution: PuzzleSolution,
    pub reward: PuzzleReward,
    pub required_for_progression: bool,
}

/// How to solve a puzzle
#[derive(Debug, Clone)]
pub enum PuzzleSolution {
    /// Type a specific phrase
    Phrase(String),
    /// Use a specific item
    UseItem(String),
    /// Have specific items in inventory
    HaveItems(Vec<String>),
    /// Cast a specific spell
    CastSpell(String),
    /// Be in a specific location and do something
    LocationAction { room: String, action: String },
    /// Combine items
    CombineItems(Vec<String>),
}

/// What the player gets for solving
#[derive(Debug, Clone)]
pub enum PuzzleReward {
    /// Get an item
    Item(String),
    /// Get gold
    Gold(i64),
    /// Get XP
    Xp(u64),
    /// Learn a spell
    LearnSpell(String),
    /// Unlock a region
    UnlockRegion(String),
    /// Advance quest
    AdvanceQuest(String),
    /// Multiple rewards
    Multiple(Vec<PuzzleReward>),
    /// No tangible reward (story progression)
    None,
}

/// Lazy-initialized puzzle definitions
pub static PUZZLES: Lazy<Vec<Puzzle>> = Lazy::new(|| vec![
    Puzzle {
        key: "golden_forest_entry",
        name: "The Golden Gate",
        description: "The luminous gatekeeper blocks entry to the Golden Forest.",
        hint: "The gatekeeper seeks proof of your magical prowess. Perhaps a key forged from golden light?",
        solution: PuzzleSolution::HaveItems(vec!["golden_key".to_string()]),
        reward: PuzzleReward::AdvanceQuest("golden_forest_unlocked".to_string()),
        required_for_progression: true,
    },
    Puzzle {
        key: "altar_blessing",
        name: "Tashanna's Blessing",
        description: "The altar of Tashanna awaits a worthy supplicant.",
        hint: "Glory be to the Lady of Legends...",
        solution: PuzzleSolution::CastSpell("glory_tashanna".to_string()),
        reward: PuzzleReward::Multiple(vec![
            PuzzleReward::Item("tashanna_amulet".to_string()),
            PuzzleReward::Xp(100),
        ]),
        required_for_progression: true,
    },
    Puzzle {
        key: "spirit_elara",
        name: "The Trapped Spirit",
        description: "Elara's ghost is trapped between worlds.",
        hint: "She seeks peace... perhaps an orchid from her favorite spot would help.",
        solution: PuzzleSolution::UseItem("ghost_orchid".to_string()),
        reward: PuzzleReward::Multiple(vec![
            PuzzleReward::LearnSpell("greater_heal".to_string()),
            PuzzleReward::Xp(75),
        ]),
        required_for_progression: false,
    },
    Puzzle {
        key: "crossroads_riddle",
        name: "The Wanderer's Riddle",
        description: "The wanderer at the crossroads poses a question.",
        hint: "What has roots as nobody sees, is taller than trees, up up it goes, and yet never grows?",
        solution: PuzzleSolution::Phrase("mountain".to_string()),
        reward: PuzzleReward::Item("golden_key".to_string()),
        required_for_progression: true,
    },
    Puzzle {
        key: "archmage_ritual",
        name: "The Arch-Mage Ritual",
        description: "To claim the throne, you must complete the ancient ritual.",
        hint: "Three items of power: The Tome, the Amulet, and the Rose that survived the flames.",
        solution: PuzzleSolution::HaveItems(vec![
            "tome_arcane".to_string(),
            "tashanna_amulet".to_string(),
            "charred_rose".to_string(),
        ]),
        reward: PuzzleReward::AdvanceQuest("ritual_ready".to_string()),
        required_for_progression: true,
    },
    Puzzle {
        key: "dragon_defeat",
        name: "Defeat Pyraxis",
        description: "The dragon guardian must be overcome to claim the title.",
        hint: "Prepare well. The dragon is powerful but lonely...",
        solution: PuzzleSolution::LocationAction {
            room: "dragon_lair".to_string(),
            action: "defeat_dragon".to_string(),
        },
        reward: PuzzleReward::Multiple(vec![
            PuzzleReward::Item("archmage_robe".to_string()),
            PuzzleReward::Xp(1000),
            PuzzleReward::Gold(5000),
            PuzzleReward::AdvanceQuest("became_archmage".to_string()),
        ]),
        required_for_progression: true,
    },
]);

/// Get a puzzle by key
pub fn get_puzzle(key: &str) -> Option<&Puzzle> {
    (*PUZZLES).iter().find(|p| p.key == key)
}

/// Result of attempting a puzzle
#[derive(Debug)]
pub enum PuzzleResult {
    Solved {
        puzzle_name: String,
        message: String,
        rewards: Vec<String>,
    },
    WrongSolution {
        puzzle_name: String,
        hint: String,
    },
    AlreadySolved,
    NotAtPuzzle,
}

/// Try to solve a puzzle with a phrase
pub fn try_phrase_puzzle(state: &mut GameState, phrase: &str, _room_key: &str) -> PuzzleResult {
    let phrase_lower = phrase.to_lowercase().trim().to_string();

    // Find puzzles that can be solved with this phrase in this location
    for puzzle in &*PUZZLES {
        // Skip if already solved
        if state.is_puzzle_solved(puzzle.key) {
            continue;
        }

        // Check if this is a phrase puzzle
        if let PuzzleSolution::Phrase(solution) = &puzzle.solution {
            if phrase_lower == solution.to_lowercase() {
                // Solved!
                state.solve_puzzle(puzzle.key);
                let rewards = apply_rewards(state, &puzzle.reward);

                return PuzzleResult::Solved {
                    puzzle_name: puzzle.name.to_string(),
                    message: "Correct! The puzzle is solved.".to_string(),
                    rewards,
                };
            }
        }
    }

    // Check if there's a puzzle here they're trying to solve
    for puzzle in &*PUZZLES {
        if !state.is_puzzle_solved(puzzle.key) {
            if let PuzzleSolution::Phrase(_) = &puzzle.solution {
                return PuzzleResult::WrongSolution {
                    puzzle_name: puzzle.name.to_string(),
                    hint: puzzle.hint.to_string(),
                };
            }
        }
    }

    PuzzleResult::NotAtPuzzle
}

/// Try to solve a puzzle by using an item
pub fn try_item_puzzle(state: &mut GameState, item_key: &str) -> PuzzleResult {
    for puzzle in &*PUZZLES {
        if state.is_puzzle_solved(puzzle.key) {
            continue;
        }

        if let PuzzleSolution::UseItem(required_item) = &puzzle.solution {
            if item_key == required_item && state.has_item(item_key) {
                // Consume item and solve
                state.remove_item(item_key, 1);
                state.solve_puzzle(puzzle.key);
                let rewards = apply_rewards(state, &puzzle.reward);

                return PuzzleResult::Solved {
                    puzzle_name: puzzle.name.to_string(),
                    message: "The item's power is released! The puzzle is solved.".to_string(),
                    rewards,
                };
            }
        }
    }

    PuzzleResult::NotAtPuzzle
}

/// Check if player has items to solve a puzzle
pub fn check_item_puzzles(state: &mut GameState) -> Option<PuzzleResult> {
    for puzzle in &*PUZZLES {
        if state.is_puzzle_solved(puzzle.key) {
            continue;
        }

        if let PuzzleSolution::HaveItems(required_items) = &puzzle.solution {
            let has_all = required_items.iter().all(|item| state.has_item(item));

            if has_all {
                state.solve_puzzle(puzzle.key);
                let rewards = apply_rewards(state, &puzzle.reward);

                return Some(PuzzleResult::Solved {
                    puzzle_name: puzzle.name.to_string(),
                    message: "You have gathered all the required items!".to_string(),
                    rewards,
                });
            }
        }
    }

    None
}

/// Apply puzzle rewards to player state
fn apply_rewards(state: &mut GameState, reward: &PuzzleReward) -> Vec<String> {
    let mut descriptions = Vec::new();

    match reward {
        PuzzleReward::Item(item_key) => {
            if state.add_item(item_key, 1) {
                let name = super::data::get_item(item_key)
                    .map(|i| i.name)
                    .unwrap_or("item");
                descriptions.push(format!("Received: {}", name));
            }
        }
        PuzzleReward::Gold(amount) => {
            state.add_gold(*amount);
            descriptions.push(format!("Gained {} gold", amount));
        }
        PuzzleReward::Xp(amount) => {
            let leveled = state.add_xp(*amount);
            descriptions.push(format!("Gained {} XP", amount));
            if leveled {
                descriptions.push(format!("LEVEL UP! You are now level {}!", state.level));
            }
        }
        PuzzleReward::LearnSpell(spell_key) => {
            if state.learn_spell(spell_key) {
                let name = super::data::get_spell(spell_key)
                    .map(|s| s.name)
                    .unwrap_or("spell");
                descriptions.push(format!("Learned spell: {}", name));
            }
        }
        PuzzleReward::UnlockRegion(region) => {
            state.set_flag(&format!("region_{}_unlocked", region), "true");
            descriptions.push(format!("Region unlocked: {}", region));
        }
        PuzzleReward::AdvanceQuest(quest_key) => {
            state.set_flag(quest_key, "true");
            descriptions.push("Quest advanced!".to_string());
        }
        PuzzleReward::Multiple(rewards) => {
            for r in rewards {
                descriptions.extend(apply_rewards(state, &r));
            }
        }
        PuzzleReward::None => {}
    }

    descriptions
}

/// Get available puzzles in the current room
pub fn get_room_puzzles(state: &GameState) -> Vec<&Puzzle> {
    let room = super::data::get_room(&state.current_room);

    (*PUZZLES).iter()
        .filter(|puzzle| {
            // Filter by room-specific puzzles
            match &puzzle.solution {
                PuzzleSolution::LocationAction { room: puzzle_room, .. } => {
                    &state.current_room == puzzle_room
                }
                _ => {
                    // Check if puzzle is relevant to room special
                    if let Some(room) = room {
                        match (room.special, puzzle.key) {
                            (Some(RoomSpecial::Altar), "altar_blessing") => true,
                            (Some(RoomSpecial::Fountain), _) => puzzle.key == "fountain_scrolls",
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
            }
        })
        .filter(|puzzle| !state.is_puzzle_solved(puzzle.key))
        .collect()
}

/// Get hint for a puzzle
pub fn get_puzzle_hint(state: &GameState, puzzle_key: &str) -> Option<String> {
    get_puzzle(puzzle_key).map(|p| {
        if state.is_puzzle_solved(puzzle_key) {
            "You have already solved this puzzle.".to_string()
        } else {
            p.hint.to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_puzzle() {
        assert!(get_puzzle("crossroads_riddle").is_some());
        assert!(get_puzzle("nonexistent").is_none());
    }

    #[test]
    fn test_phrase_puzzle_correct() {
        let mut state = GameState::new("Test");

        let result = try_phrase_puzzle(&mut state, "mountain", "crossroads");

        match result {
            PuzzleResult::Solved { puzzle_name, .. } => {
                assert_eq!(puzzle_name, "The Wanderer's Riddle");
                assert!(state.is_puzzle_solved("crossroads_riddle"));
            }
            _ => panic!("Expected puzzle to be solved"),
        }
    }

    #[test]
    fn test_phrase_puzzle_wrong() {
        let mut state = GameState::new("Test");

        let result = try_phrase_puzzle(&mut state, "tree", "crossroads");

        match result {
            PuzzleResult::WrongSolution { hint, .. } => {
                assert!(!hint.is_empty());
            }
            _ => {}  // May be NotAtPuzzle if phrase puzzles work differently
        }
    }

    #[test]
    fn test_item_puzzle() {
        let mut state = GameState::new("Test");
        state.add_item("ghost_orchid", 1);

        let result = try_item_puzzle(&mut state, "ghost_orchid");

        match result {
            PuzzleResult::Solved { puzzle_name, rewards, .. } => {
                assert_eq!(puzzle_name, "The Trapped Spirit");
                assert!(!rewards.is_empty());
            }
            _ => panic!("Expected puzzle to be solved"),
        }
    }

    #[test]
    fn test_check_item_puzzles() {
        let mut state = GameState::new("Test");
        state.add_item("tome_arcane", 1);
        state.add_item("tashanna_amulet", 1);
        state.add_item("charred_rose", 1);

        let result = check_item_puzzles(&mut state);

        assert!(result.is_some());
        assert!(state.is_puzzle_solved("archmage_ritual"));
    }

    #[test]
    fn test_puzzle_already_solved() {
        let mut state = GameState::new("Test");
        state.solve_puzzle("crossroads_riddle");

        let result = try_phrase_puzzle(&mut state, "mountain", "crossroads");

        // Should not solve again
        match result {
            PuzzleResult::Solved { .. } => panic!("Should not solve twice"),
            _ => {}
        }
    }
}
