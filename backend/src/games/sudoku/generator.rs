//! Sudoku puzzle generator with deterministic daily seeding
//!
//! Generates puzzles that are:
//! - Deterministic based on date (same puzzle for all players)
//! - Guaranteed to have a unique solution
//! - Approximately medium difficulty (25-35 clues)

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// A daily puzzle with puzzle and solution
#[derive(Debug, Clone)]
pub struct DailyPuzzle {
    pub date: String,
    /// The puzzle (0 = empty)
    pub puzzle: [[u8; 9]; 9],
    /// The solution
    pub solution: [[u8; 9]; 9],
    /// Number of clues (given numbers)
    pub clue_count: u32,
}

/// Generate a daily puzzle based on the date string
///
/// The date should be in "YYYY-MM-DD" format.
/// Uses the date to seed the RNG for deterministic generation.
pub fn generate_puzzle(date: &str) -> DailyPuzzle {
    // Create a seed from the date
    let seed = date_to_seed(date);
    let mut rng = StdRng::seed_from_u64(seed);

    // Generate a complete valid Sudoku grid
    let mut grid = [[0u8; 9]; 9];
    fill_grid(&mut grid, &mut rng);

    // Store the solution
    let solution = grid;

    // Remove numbers to create the puzzle
    // Target approximately 25-35 clues (medium difficulty)
    let target_clues = rng.gen_range(25..=35);
    let puzzle = create_puzzle(&solution, target_clues, &mut rng);

    let clue_count = puzzle
        .iter()
        .flat_map(|row| row.iter())
        .filter(|&&x| x != 0)
        .count() as u32;

    DailyPuzzle {
        date: date.to_string(),
        puzzle,
        solution,
        clue_count,
    }
}

/// Convert a date string to a u64 seed
fn date_to_seed(date: &str) -> u64 {
    // Simple hash: multiply each char by position and sum
    let mut seed: u64 = 0;
    for (i, c) in date.chars().enumerate() {
        seed = seed.wrapping_mul(31).wrapping_add((c as u64).wrapping_mul((i + 1) as u64));
    }
    // Add some fixed salt for uniqueness
    seed.wrapping_add(0xDEAD_BEEF_CAFE_BABE)
}

/// Fill a grid with a valid complete Sudoku solution
fn fill_grid(grid: &mut [[u8; 9]; 9], rng: &mut StdRng) -> bool {
    // Find next empty cell
    let mut empty = None;
    'outer: for row in 0..9 {
        for col in 0..9 {
            if grid[row][col] == 0 {
                empty = Some((row, col));
                break 'outer;
            }
        }
    }

    let (row, col) = match empty {
        Some(pos) => pos,
        None => return true, // Grid is complete
    };

    // Try numbers 1-9 in random order
    let mut nums: Vec<u8> = (1..=9).collect();
    shuffle(&mut nums, rng);

    for num in nums {
        if is_valid_placement(grid, row, col, num) {
            grid[row][col] = num;
            if fill_grid(grid, rng) {
                return true;
            }
            grid[row][col] = 0;
        }
    }

    false
}

/// Check if a number can be placed at position without conflicts
fn is_valid_placement(grid: &[[u8; 9]; 9], row: usize, col: usize, num: u8) -> bool {
    // Check row
    for c in 0..9 {
        if grid[row][c] == num {
            return false;
        }
    }

    // Check column
    for r in 0..9 {
        if grid[r][col] == num {
            return false;
        }
    }

    // Check 3x3 box
    let box_row = (row / 3) * 3;
    let box_col = (col / 3) * 3;
    for r in box_row..box_row + 3 {
        for c in box_col..box_col + 3 {
            if grid[r][c] == num {
                return false;
            }
        }
    }

    true
}

/// Shuffle a slice using Fisher-Yates
fn shuffle<T>(slice: &mut [T], rng: &mut StdRng) {
    for i in (1..slice.len()).rev() {
        let j = rng.gen_range(0..=i);
        slice.swap(i, j);
    }
}

/// Create a puzzle by removing numbers from a complete grid
/// while maintaining unique solvability (approximately)
fn create_puzzle(solution: &[[u8; 9]; 9], target_clues: u32, rng: &mut StdRng) -> [[u8; 9]; 9] {
    let mut puzzle = *solution;
    let total_cells = 81;
    let mut cells_to_remove = total_cells - target_clues as usize;

    // Create list of positions to potentially remove
    let mut positions: Vec<(usize, usize)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .collect();
    shuffle(&mut positions, rng);

    for (row, col) in positions {
        if cells_to_remove == 0 {
            break;
        }

        let value = puzzle[row][col];
        if value == 0 {
            continue;
        }

        // Try removing this cell
        puzzle[row][col] = 0;

        // Check if puzzle still has unique solution
        // For performance, we do a simple check instead of full solving
        if has_unique_solution(&puzzle) {
            cells_to_remove -= 1;
        } else {
            // Restore the cell
            puzzle[row][col] = value;
        }
    }

    puzzle
}

/// Check if a puzzle has a unique solution
/// Uses constraint propagation for efficiency
fn has_unique_solution(puzzle: &[[u8; 9]; 9]) -> bool {
    let mut solution_count = 0;
    let mut grid = *puzzle;
    count_solutions(&mut grid, &mut solution_count, 2);
    solution_count == 1
}

/// Count solutions up to a limit using backtracking
fn count_solutions(grid: &mut [[u8; 9]; 9], count: &mut usize, limit: usize) {
    if *count >= limit {
        return;
    }

    // Find next empty cell
    let mut empty = None;
    'outer: for row in 0..9 {
        for col in 0..9 {
            if grid[row][col] == 0 {
                empty = Some((row, col));
                break 'outer;
            }
        }
    }

    let (row, col) = match empty {
        Some(pos) => pos,
        None => {
            *count += 1;
            return;
        }
    };

    for num in 1..=9 {
        if *count >= limit {
            return;
        }
        if is_valid_placement(grid, row, col, num) {
            grid[row][col] = num;
            count_solutions(grid, count, limit);
            grid[row][col] = 0;
        }
    }
}

/// Get today's date in Eastern time as a string
pub fn get_eastern_date() -> String {
    use chrono::Utc;
    use chrono_tz::US::Eastern;

    let now_utc = Utc::now();
    let now_eastern = now_utc.with_timezone(&Eastern);
    now_eastern.format("%Y-%m-%d").to_string()
}

/// Check if a date is valid for puzzle generation
pub fn is_valid_date(date: &str) -> bool {
    // Basic format check YYYY-MM-DD
    if date.len() != 10 {
        return false;
    }
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    // Year must be 4 digits, month and day must be 2 digits each
    if parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    // All must be numeric
    if let (Ok(year), Ok(month), Ok(day)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) {
        // Basic range validation
        year >= 1900 && year <= 9999 && month >= 1 && month <= 12 && day >= 1 && day <= 31
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_to_seed() {
        let seed1 = date_to_seed("2026-01-30");
        let seed2 = date_to_seed("2026-01-30");
        let seed3 = date_to_seed("2026-01-31");

        assert_eq!(seed1, seed2); // Same date = same seed
        assert_ne!(seed1, seed3); // Different date = different seed
    }

    #[test]
    fn test_generate_puzzle_deterministic() {
        let puzzle1 = generate_puzzle("2026-01-30");
        let puzzle2 = generate_puzzle("2026-01-30");

        assert_eq!(puzzle1.puzzle, puzzle2.puzzle);
        assert_eq!(puzzle1.solution, puzzle2.solution);
    }

    #[test]
    fn test_generate_puzzle_different_days() {
        let puzzle1 = generate_puzzle("2026-01-30");
        let puzzle2 = generate_puzzle("2026-01-31");

        assert_ne!(puzzle1.puzzle, puzzle2.puzzle);
    }

    #[test]
    fn test_puzzle_has_solution() {
        let puzzle = generate_puzzle("2026-01-30");

        // Verify solution is valid
        for row in 0..9 {
            for col in 0..9 {
                let val = puzzle.solution[row][col];
                assert!(val >= 1 && val <= 9);
            }
        }

        // Verify no row conflicts
        for row in 0..9 {
            let mut seen = [false; 10];
            for col in 0..9 {
                let val = puzzle.solution[row][col] as usize;
                assert!(!seen[val]);
                seen[val] = true;
            }
        }

        // Verify no column conflicts
        for col in 0..9 {
            let mut seen = [false; 10];
            for row in 0..9 {
                let val = puzzle.solution[row][col] as usize;
                assert!(!seen[val]);
                seen[val] = true;
            }
        }

        // Verify no box conflicts
        for box_row in 0..3 {
            for box_col in 0..3 {
                let mut seen = [false; 10];
                for r in 0..3 {
                    for c in 0..3 {
                        let val = puzzle.solution[box_row * 3 + r][box_col * 3 + c] as usize;
                        assert!(!seen[val]);
                        seen[val] = true;
                    }
                }
            }
        }
    }

    #[test]
    fn test_puzzle_clue_count() {
        let puzzle = generate_puzzle("2026-01-30");

        // Should have 25-35 clues (medium difficulty)
        assert!(puzzle.clue_count >= 25 && puzzle.clue_count <= 35);

        // Verify clue count matches actual clues
        let actual_clues = puzzle
            .puzzle
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&x| x != 0)
            .count() as u32;
        assert_eq!(puzzle.clue_count, actual_clues);
    }

    #[test]
    fn test_puzzle_clues_match_solution() {
        let puzzle = generate_puzzle("2026-01-30");

        for row in 0..9 {
            for col in 0..9 {
                if puzzle.puzzle[row][col] != 0 {
                    assert_eq!(puzzle.puzzle[row][col], puzzle.solution[row][col]);
                }
            }
        }
    }

    #[test]
    fn test_is_valid_date() {
        assert!(is_valid_date("2026-01-30"));
        assert!(is_valid_date("2024-12-31"));
        assert!(!is_valid_date("2026-1-30")); // Wrong format
        assert!(!is_valid_date("2026/01/30")); // Wrong separator
        assert!(!is_valid_date("01-30-2026")); // Wrong order
    }

    #[test]
    fn test_unique_solution() {
        let puzzle = generate_puzzle("2026-01-30");
        assert!(has_unique_solution(&puzzle.puzzle));
    }

    #[test]
    fn test_is_valid_placement() {
        let mut grid = [[0u8; 9]; 9];
        grid[0][0] = 5;

        // Same row conflict
        assert!(!is_valid_placement(&grid, 0, 5, 5));
        // Same column conflict
        assert!(!is_valid_placement(&grid, 5, 0, 5));
        // Same box conflict
        assert!(!is_valid_placement(&grid, 1, 1, 5));
        // No conflict
        assert!(is_valid_placement(&grid, 3, 3, 5));
    }
}
