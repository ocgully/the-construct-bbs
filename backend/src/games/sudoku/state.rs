//! Sudoku game state - player progress on current puzzle

use serde::{Deserialize, Serialize};

/// State of a single cell in the puzzle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellState {
    /// Original clue (cannot be modified)
    Given(u8),
    /// Player-entered value
    Entered(u8),
    /// Empty cell
    Empty,
}

impl CellState {
    /// Get the value if present
    pub fn value(&self) -> Option<u8> {
        match self {
            CellState::Given(v) | CellState::Entered(v) => Some(*v),
            CellState::Empty => None,
        }
    }

    /// Check if this is a given (immutable) cell
    pub fn is_given(&self) -> bool {
        matches!(self, CellState::Given(_))
    }
}

/// Player's current puzzle state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// The puzzle date (YYYY-MM-DD format)
    pub puzzle_date: String,
    /// 9x9 grid state (row-major order)
    pub grid: [[CellState; 9]; 9],
    /// The solution for validation
    pub solution: [[u8; 9]; 9],
    /// Pencil marks for each cell (candidates)
    pub pencil_marks: [[u16; 9]; 9], // bit field: bit N = candidate N+1
    /// Current cursor position (row, col)
    pub cursor: (usize, usize),
    /// Whether pencil mode is active
    pub pencil_mode: bool,
    /// Time spent in seconds
    pub elapsed_seconds: u32,
    /// Whether puzzle is completed
    pub completed: bool,
    /// Number of errors made
    pub error_count: u32,
    /// Last error position (for highlighting)
    pub last_error: Option<(usize, usize)>,
    /// Message to display
    pub last_message: Option<String>,
}

impl GameState {
    /// Create a new game state from a puzzle
    pub fn new(puzzle_date: &str, puzzle: &[[u8; 9]; 9], solution: &[[u8; 9]; 9]) -> Self {
        let mut grid = [[CellState::Empty; 9]; 9];

        for row in 0..9 {
            for col in 0..9 {
                if puzzle[row][col] != 0 {
                    grid[row][col] = CellState::Given(puzzle[row][col]);
                }
            }
        }

        Self {
            puzzle_date: puzzle_date.to_string(),
            grid,
            solution: *solution,
            pencil_marks: [[0u16; 9]; 9],
            cursor: (0, 0),
            pencil_mode: false,
            elapsed_seconds: 0,
            completed: false,
            error_count: 0,
            last_error: None,
            last_message: None,
        }
    }

    /// Move cursor in a direction
    pub fn move_cursor(&mut self, dr: i32, dc: i32) {
        let (row, col) = self.cursor;
        let new_row = (row as i32 + dr).clamp(0, 8) as usize;
        let new_col = (col as i32 + dc).clamp(0, 8) as usize;
        self.cursor = (new_row, new_col);
        self.last_error = None; // Clear error highlight on move
    }

    /// Enter a number at cursor position
    pub fn enter_number(&mut self, num: u8) -> bool {
        let (row, col) = self.cursor;

        // Can't modify given cells
        if self.grid[row][col].is_given() {
            self.last_message = Some("Cannot modify given numbers.".to_string());
            return false;
        }

        if self.pencil_mode {
            // Toggle pencil mark
            self.toggle_pencil_mark(row, col, num);
            self.last_message = None;
            true
        } else {
            // Enter the number
            self.grid[row][col] = CellState::Entered(num);
            self.pencil_marks[row][col] = 0; // Clear pencil marks when entering

            // Validate against solution
            if self.solution[row][col] != num {
                self.error_count += 1;
                self.last_error = Some((row, col));
                self.last_message = Some("Incorrect!".to_string());
                false
            } else {
                self.last_error = None;
                self.last_message = None;

                // Check if puzzle is complete
                if self.is_complete() {
                    self.completed = true;
                    self.last_message = Some("Congratulations! Puzzle completed!".to_string());
                }
                true
            }
        }
    }

    /// Clear the cell at cursor position
    pub fn clear_cell(&mut self) {
        let (row, col) = self.cursor;

        if self.grid[row][col].is_given() {
            self.last_message = Some("Cannot clear given numbers.".to_string());
            return;
        }

        self.grid[row][col] = CellState::Empty;
        self.pencil_marks[row][col] = 0;
        self.last_error = None;
        self.last_message = None;
    }

    /// Toggle a pencil mark
    fn toggle_pencil_mark(&mut self, row: usize, col: usize, num: u8) {
        if num < 1 || num > 9 {
            return;
        }
        let bit = 1u16 << (num - 1);
        self.pencil_marks[row][col] ^= bit;
    }

    /// Check if a pencil mark is set
    pub fn has_pencil_mark(&self, row: usize, col: usize, num: u8) -> bool {
        if num < 1 || num > 9 {
            return false;
        }
        let bit = 1u16 << (num - 1);
        (self.pencil_marks[row][col] & bit) != 0
    }

    /// Get all pencil marks for a cell as a vector
    pub fn get_pencil_marks(&self, row: usize, col: usize) -> Vec<u8> {
        let mut marks = Vec::new();
        for n in 1..=9 {
            if self.has_pencil_mark(row, col, n) {
                marks.push(n);
            }
        }
        marks
    }

    /// Toggle pencil mode
    pub fn toggle_pencil_mode(&mut self) {
        self.pencil_mode = !self.pencil_mode;
        self.last_message = Some(if self.pencil_mode {
            "Pencil mode ON".to_string()
        } else {
            "Pencil mode OFF".to_string()
        });
    }

    /// Check if puzzle is complete (all cells filled correctly)
    pub fn is_complete(&self) -> bool {
        for row in 0..9 {
            for col in 0..9 {
                match self.grid[row][col] {
                    CellState::Empty => return false,
                    CellState::Given(v) | CellState::Entered(v) => {
                        if v != self.solution[row][col] {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// Check if there's a conflict with the value at (row, col)
    pub fn has_conflict(&self, row: usize, col: usize) -> bool {
        let value = match self.grid[row][col].value() {
            Some(v) => v,
            None => return false,
        };

        // Check row
        for c in 0..9 {
            if c != col {
                if self.grid[row][c].value() == Some(value) {
                    return true;
                }
            }
        }

        // Check column
        for r in 0..9 {
            if r != row {
                if self.grid[r][col].value() == Some(value) {
                    return true;
                }
            }
        }

        // Check 3x3 box
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        for r in box_row..box_row + 3 {
            for c in box_col..box_col + 3 {
                if r != row || c != col {
                    if self.grid[r][c].value() == Some(value) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Count filled cells
    pub fn filled_count(&self) -> usize {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.value().is_some())
            .count()
    }
}

/// Player statistics (persisted in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub user_id: i64,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub last_played_date: Option<String>,
    pub games_completed: u32,
    pub best_time_seconds: Option<u32>,
    pub total_time_seconds: u64,
    pub pause_days_used: u32,
    pub pause_days_week_start: Option<String>,
}

impl PlayerStats {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            current_streak: 0,
            longest_streak: 0,
            last_played_date: None,
            games_completed: 0,
            best_time_seconds: None,
            total_time_seconds: 0,
            pause_days_used: 0,
            pause_days_week_start: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_puzzle() -> [[u8; 9]; 9] {
        [
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ]
    }

    fn sample_solution() -> [[u8; 9]; 9] {
        [
            [5, 3, 4, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ]
    }

    #[test]
    fn test_new_game_state() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let state = GameState::new("2026-01-30", &puzzle, &solution);

        assert_eq!(state.puzzle_date, "2026-01-30");
        assert!(!state.completed);
        assert_eq!(state.cursor, (0, 0));
        assert!(!state.pencil_mode);
    }

    #[test]
    fn test_given_cells() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let state = GameState::new("2026-01-30", &puzzle, &solution);

        // Check given cells
        assert!(state.grid[0][0].is_given());
        assert_eq!(state.grid[0][0].value(), Some(5));

        // Check empty cells
        assert!(!state.grid[0][2].is_given());
        assert_eq!(state.grid[0][2].value(), None);
    }

    #[test]
    fn test_enter_number() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let mut state = GameState::new("2026-01-30", &puzzle, &solution);

        // Move to an empty cell
        state.cursor = (0, 2);

        // Enter correct number
        let result = state.enter_number(4);
        assert!(result);
        assert_eq!(state.grid[0][2].value(), Some(4));

        // Enter incorrect number
        state.cursor = (0, 3);
        let result = state.enter_number(1); // Wrong, should be 6
        assert!(!result);
        assert_eq!(state.error_count, 1);
    }

    #[test]
    fn test_cannot_modify_given() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let mut state = GameState::new("2026-01-30", &puzzle, &solution);

        state.cursor = (0, 0); // Given cell with 5
        let result = state.enter_number(9);
        assert!(!result);
        assert_eq!(state.grid[0][0].value(), Some(5)); // Unchanged
    }

    #[test]
    fn test_pencil_marks() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let mut state = GameState::new("2026-01-30", &puzzle, &solution);

        state.cursor = (0, 2);
        state.pencil_mode = true;

        // Add pencil marks
        state.enter_number(4);
        state.enter_number(6);
        state.enter_number(2);

        assert!(state.has_pencil_mark(0, 2, 4));
        assert!(state.has_pencil_mark(0, 2, 6));
        assert!(state.has_pencil_mark(0, 2, 2));
        assert!(!state.has_pencil_mark(0, 2, 5));

        // Toggle off
        state.enter_number(6);
        assert!(!state.has_pencil_mark(0, 2, 6));

        let marks = state.get_pencil_marks(0, 2);
        assert_eq!(marks, vec![2, 4]);
    }

    #[test]
    fn test_clear_cell() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let mut state = GameState::new("2026-01-30", &puzzle, &solution);

        state.cursor = (0, 2);
        state.enter_number(4);
        assert_eq!(state.grid[0][2].value(), Some(4));

        state.clear_cell();
        assert_eq!(state.grid[0][2].value(), None);
    }

    #[test]
    fn test_cursor_movement() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let mut state = GameState::new("2026-01-30", &puzzle, &solution);

        state.move_cursor(1, 0); // Down
        assert_eq!(state.cursor, (1, 0));

        state.move_cursor(0, 1); // Right
        assert_eq!(state.cursor, (1, 1));

        // Test boundary
        state.cursor = (0, 0);
        state.move_cursor(-1, 0); // Up at top
        assert_eq!(state.cursor, (0, 0));

        state.cursor = (8, 8);
        state.move_cursor(1, 1); // Down-right at corner
        assert_eq!(state.cursor, (8, 8));
    }

    #[test]
    fn test_conflict_detection() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let mut state = GameState::new("2026-01-30", &puzzle, &solution);

        // Enter a conflicting number in a row
        state.cursor = (0, 2);
        state.grid[0][2] = CellState::Entered(5); // Conflicts with (0,0) which has 5

        assert!(state.has_conflict(0, 2));
    }

    #[test]
    fn test_completion() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let mut state = GameState::new("2026-01-30", &puzzle, &solution);

        // Fill in all the solution values
        for row in 0..9 {
            for col in 0..9 {
                if !state.grid[row][col].is_given() {
                    state.grid[row][col] = CellState::Entered(solution[row][col]);
                }
            }
        }

        assert!(state.is_complete());
    }

    #[test]
    fn test_state_serialization() {
        let puzzle = sample_puzzle();
        let solution = sample_solution();
        let state = GameState::new("2026-01-30", &puzzle, &solution);

        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.puzzle_date, restored.puzzle_date);
        assert_eq!(state.cursor, restored.cursor);
        assert_eq!(state.completed, restored.completed);
    }
}
