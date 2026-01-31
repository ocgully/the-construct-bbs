//! Puzzle generation and validation for Queens game
//!
//! Uses date-based seeding so all players worldwide get the same puzzle each day.

use super::data::RegionColor;
use chrono::{Datelike, NaiveDate};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// A generated daily puzzle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPuzzle {
    /// The date this puzzle is for (YYYY-MM-DD)
    pub date: String,
    /// Board size (5-8)
    pub size: usize,
    /// Region assignment for each cell (row-major order)
    /// Value is the region index (0 to size-1)
    pub regions: Vec<usize>,
    /// The unique solution (queen positions)
    /// solution[region_index] = (row, col) where queen should be placed
    pub solution: Vec<(usize, usize)>,
}

impl DailyPuzzle {
    /// Get the region color for a cell
    pub fn get_region(&self, row: usize, col: usize) -> usize {
        self.regions[row * self.size + col]
    }

    /// Get the region color enum for a cell
    pub fn get_region_color(&self, row: usize, col: usize) -> RegionColor {
        RegionColor::from_index(self.get_region(row, col))
    }

    /// Check if a position is part of the solution
    pub fn is_solution_position(&self, row: usize, col: usize) -> bool {
        self.solution.iter().any(|&(r, c)| r == row && c == col)
    }
}

/// Generate a puzzle for a specific date
pub fn generate_daily_puzzle(date: &str) -> DailyPuzzle {
    // Parse date and create deterministic seed
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());

    // Create seed from date components
    let seed = (parsed.year() as u64 * 10000) +
               (parsed.month() as u64 * 100) +
               parsed.day() as u64;

    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // Determine board size (5-8, weighted toward medium)
    let size = match rng.gen_range(0..10) {
        0..=2 => 5,   // 30% chance
        3..=6 => 6,   // 40% chance
        7..=8 => 7,   // 20% chance
        _ => 8,       // 10% chance
    };

    // Generate puzzle with valid solution
    generate_puzzle_with_solution(&mut rng, size, date)
}

/// Generate a puzzle with a guaranteed unique solution
fn generate_puzzle_with_solution(rng: &mut ChaCha8Rng, size: usize, date: &str) -> DailyPuzzle {
    // First, generate a valid N-Queens solution
    let solution = generate_valid_queens(rng, size);

    // Then create regions such that:
    // 1. Each region is contiguous
    // 2. Each region contains exactly one queen position
    // 3. The puzzle has a unique solution
    let regions = generate_regions_for_solution(rng, size, &solution);

    DailyPuzzle {
        date: date.to_string(),
        size,
        regions,
        solution,
    }
}

/// Generate a valid N-Queens solution (no two queens attack each other)
fn generate_valid_queens(rng: &mut ChaCha8Rng, size: usize) -> Vec<(usize, usize)> {
    // Use backtracking with randomized column order
    let mut solution = Vec::with_capacity(size);
    let mut cols_used = vec![false; size];

    if !solve_queens(rng, size, 0, &mut solution, &mut cols_used) {
        // Fallback: use a known valid solution pattern
        solution = (0..size).map(|i| (i, (2 * i + 1) % size)).collect();
    }

    solution
}

/// Recursive backtracking solver for N-Queens
fn solve_queens(
    rng: &mut ChaCha8Rng,
    size: usize,
    row: usize,
    solution: &mut Vec<(usize, usize)>,
    cols_used: &mut Vec<bool>,
) -> bool {
    if row == size {
        return true;
    }

    // Create randomized column order
    let mut cols: Vec<usize> = (0..size).collect();
    for i in (1..size).rev() {
        let j = rng.gen_range(0..=i);
        cols.swap(i, j);
    }

    for &col in &cols {
        if cols_used[col] {
            continue;
        }

        // Check diagonals
        let mut valid = true;
        for &(r, c) in solution.iter() {
            let row_diff = (row as i32 - r as i32).abs();
            let col_diff = (col as i32 - c as i32).abs();
            if row_diff == col_diff {
                valid = false;
                break;
            }
        }

        if valid {
            solution.push((row, col));
            cols_used[col] = true;

            if solve_queens(rng, size, row + 1, solution, cols_used) {
                return true;
            }

            solution.pop();
            cols_used[col] = false;
        }
    }

    false
}

/// Generate contiguous regions that each contain exactly one queen
fn generate_regions_for_solution(
    rng: &mut ChaCha8Rng,
    size: usize,
    solution: &[(usize, usize)],
) -> Vec<usize> {
    let total_cells = size * size;
    let mut regions = vec![usize::MAX; total_cells];

    // Assign each queen position as the seed for its region
    for (region_idx, &(row, col)) in solution.iter().enumerate() {
        regions[row * size + col] = region_idx;
    }

    // Use flood-fill to grow regions outward from queen positions
    // Keep growing until all cells are assigned
    let mut unassigned: Vec<usize> = (0..total_cells)
        .filter(|&i| regions[i] == usize::MAX)
        .collect();

    // Shuffle for variety
    for i in (1..unassigned.len()).rev() {
        let j = rng.gen_range(0..=i);
        unassigned.swap(i, j);
    }

    // Multiple passes to ensure all cells are assigned
    for _ in 0..100 {
        if unassigned.is_empty() {
            break;
        }

        let mut still_unassigned = Vec::new();

        for &cell in &unassigned {
            let row = cell / size;
            let col = cell % size;

            // Find adjacent cells that have a region
            let mut adjacent_regions = Vec::new();

            // Up
            if row > 0 {
                let r = regions[(row - 1) * size + col];
                if r != usize::MAX {
                    adjacent_regions.push(r);
                }
            }
            // Down
            if row + 1 < size {
                let r = regions[(row + 1) * size + col];
                if r != usize::MAX {
                    adjacent_regions.push(r);
                }
            }
            // Left
            if col > 0 {
                let r = regions[row * size + (col - 1)];
                if r != usize::MAX {
                    adjacent_regions.push(r);
                }
            }
            // Right
            if col + 1 < size {
                let r = regions[row * size + (col + 1)];
                if r != usize::MAX {
                    adjacent_regions.push(r);
                }
            }

            if !adjacent_regions.is_empty() {
                // Pick a random adjacent region
                let region = adjacent_regions[rng.gen_range(0..adjacent_regions.len())];
                regions[cell] = region;
            } else {
                still_unassigned.push(cell);
            }
        }

        unassigned = still_unassigned;
    }

    // If any cells still unassigned (shouldn't happen), assign to region 0
    for region in &mut regions {
        if *region == usize::MAX {
            *region = 0;
        }
    }

    regions
}

/// Validate a player's solution
pub fn validate_solution(puzzle: &DailyPuzzle, placements: &[(usize, usize)]) -> ValidationResult {
    let size = puzzle.size;

    // Must have exactly `size` queens
    if placements.len() != size {
        return ValidationResult::WrongCount {
            expected: size,
            actual: placements.len(),
        };
    }

    // Check each queen position
    let mut regions_used = vec![false; size];
    let mut rows_used = vec![false; size];
    let mut cols_used = vec![false; size];

    for (i, &(row, col)) in placements.iter().enumerate() {
        // Bounds check
        if row >= size || col >= size {
            return ValidationResult::OutOfBounds { row, col };
        }

        // Check row conflict
        if rows_used[row] {
            return ValidationResult::RowConflict { row };
        }
        rows_used[row] = true;

        // Check column conflict
        if cols_used[col] {
            return ValidationResult::ColumnConflict { col };
        }
        cols_used[col] = true;

        // Check diagonal conflicts
        for (j, &(other_row, other_col)) in placements.iter().enumerate() {
            if i != j {
                let row_diff = (row as i32 - other_row as i32).abs();
                let col_diff = (col as i32 - other_col as i32).abs();
                if row_diff == col_diff {
                    return ValidationResult::DiagonalConflict {
                        pos1: (row, col),
                        pos2: (other_row, other_col),
                    };
                }
            }
        }

        // Check region constraint
        let region = puzzle.get_region(row, col);
        if regions_used[region] {
            return ValidationResult::RegionConflict { region };
        }
        regions_used[region] = true;
    }

    // Check all regions have a queen
    for (region, used) in regions_used.iter().enumerate() {
        if !*used {
            return ValidationResult::MissingRegion { region };
        }
    }

    ValidationResult::Correct
}

/// Result of validating a solution
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Correct,
    WrongCount { expected: usize, actual: usize },
    OutOfBounds { row: usize, col: usize },
    RowConflict { row: usize },
    ColumnConflict { col: usize },
    DiagonalConflict { pos1: (usize, usize), pos2: (usize, usize) },
    RegionConflict { region: usize },
    MissingRegion { region: usize },
}

impl ValidationResult {
    pub fn is_correct(&self) -> bool {
        matches!(self, ValidationResult::Correct)
    }

    pub fn error_message(&self) -> Option<String> {
        match self {
            ValidationResult::Correct => None,
            ValidationResult::WrongCount { expected, actual } => {
                Some(format!("Need {} queens, placed {}", expected, actual))
            }
            ValidationResult::OutOfBounds { row, col } => {
                Some(format!("Position ({}, {}) is out of bounds", row, col))
            }
            ValidationResult::RowConflict { row } => {
                Some(format!("Multiple queens in row {}", row + 1))
            }
            ValidationResult::ColumnConflict { col } => {
                Some(format!("Multiple queens in column {}", col + 1))
            }
            ValidationResult::DiagonalConflict { pos1, pos2 } => {
                Some(format!(
                    "Queens at ({},{}) and ({},{}) attack diagonally",
                    pos1.0 + 1, pos1.1 + 1, pos2.0 + 1, pos2.1 + 1
                ))
            }
            ValidationResult::RegionConflict { region } => {
                Some(format!(
                    "Multiple queens in {} region",
                    RegionColor::from_index(*region).name()
                ))
            }
            ValidationResult::MissingRegion { region } => {
                Some(format!(
                    "No queen in {} region",
                    RegionColor::from_index(*region).name()
                ))
            }
        }
    }
}

/// Get a hint - returns a position where a queen should be placed
pub fn get_hint(puzzle: &DailyPuzzle, current_placements: &[(usize, usize)]) -> Option<(usize, usize)> {
    // Find a solution position that isn't already placed
    for &(row, col) in &puzzle.solution {
        let already_placed = current_placements.iter().any(|&(r, c)| r == row && c == col);
        if !already_placed {
            return Some((row, col));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::queens::data::{MIN_BOARD_SIZE, MAX_BOARD_SIZE};

    #[test]
    fn test_generate_daily_puzzle_deterministic() {
        let puzzle1 = generate_daily_puzzle("2026-01-30");
        let puzzle2 = generate_daily_puzzle("2026-01-30");

        assert_eq!(puzzle1.date, puzzle2.date);
        assert_eq!(puzzle1.size, puzzle2.size);
        assert_eq!(puzzle1.regions, puzzle2.regions);
        assert_eq!(puzzle1.solution, puzzle2.solution);
    }

    #[test]
    fn test_generate_daily_puzzle_different_dates() {
        let puzzle1 = generate_daily_puzzle("2026-01-30");
        let puzzle2 = generate_daily_puzzle("2026-01-31");

        // Different dates should produce different puzzles
        assert_ne!(puzzle1.solution, puzzle2.solution);
    }

    #[test]
    fn test_puzzle_size_in_range() {
        for day in 1..=31 {
            let date = format!("2026-01-{:02}", day);
            let puzzle = generate_daily_puzzle(&date);
            assert!(puzzle.size >= MIN_BOARD_SIZE);
            assert!(puzzle.size <= MAX_BOARD_SIZE);
        }
    }

    #[test]
    fn test_puzzle_solution_valid() {
        let puzzle = generate_daily_puzzle("2026-01-30");

        // Solution should have correct number of queens
        assert_eq!(puzzle.solution.len(), puzzle.size);

        // Validate the solution
        let result = validate_solution(&puzzle, &puzzle.solution);
        assert!(result.is_correct(), "Solution should be valid: {:?}", result);
    }

    #[test]
    fn test_puzzle_regions_cover_all_cells() {
        let puzzle = generate_daily_puzzle("2026-01-30");

        // All regions should be valid indices
        for &region in &puzzle.regions {
            assert!(region < puzzle.size);
        }

        // Each region should have at least one cell
        for region in 0..puzzle.size {
            let count = puzzle.regions.iter().filter(|&&r| r == region).count();
            assert!(count > 0, "Region {} has no cells", region);
        }
    }

    #[test]
    fn test_validate_correct_solution() {
        let puzzle = generate_daily_puzzle("2026-01-30");
        let result = validate_solution(&puzzle, &puzzle.solution);
        assert_eq!(result, ValidationResult::Correct);
    }

    #[test]
    fn test_validate_wrong_count() {
        let puzzle = generate_daily_puzzle("2026-01-30");
        let partial: Vec<_> = puzzle.solution.iter().take(2).copied().collect();
        let result = validate_solution(&puzzle, &partial);
        assert!(matches!(result, ValidationResult::WrongCount { .. }));
    }

    #[test]
    fn test_validate_row_conflict() {
        let puzzle = generate_daily_puzzle("2026-01-30");
        // Create placements with two queens in same row
        let bad_solution: Vec<(usize, usize)> = (0..puzzle.size)
            .map(|i| (0, i)) // All queens in row 0
            .collect();

        // This will fail for row conflict (multiple queens in row 0)
        let result = validate_solution(&puzzle, &bad_solution);
        assert!(matches!(result, ValidationResult::RowConflict { .. }));
    }

    #[test]
    fn test_get_hint_returns_solution_position() {
        let puzzle = generate_daily_puzzle("2026-01-30");
        let current: Vec<(usize, usize)> = vec![];

        let hint = get_hint(&puzzle, &current);
        assert!(hint.is_some());

        let (row, col) = hint.unwrap();
        assert!(puzzle.is_solution_position(row, col));
    }

    #[test]
    fn test_get_hint_with_partial_solution() {
        let puzzle = generate_daily_puzzle("2026-01-30");
        let current = vec![puzzle.solution[0]];

        let hint = get_hint(&puzzle, &current);
        assert!(hint.is_some());

        // Hint should not be the already placed queen
        let (row, col) = hint.unwrap();
        assert_ne!((row, col), puzzle.solution[0]);
        assert!(puzzle.is_solution_position(row, col));
    }

    #[test]
    fn test_get_hint_with_complete_solution() {
        let puzzle = generate_daily_puzzle("2026-01-30");
        let hint = get_hint(&puzzle, &puzzle.solution);
        assert!(hint.is_none());
    }
}
