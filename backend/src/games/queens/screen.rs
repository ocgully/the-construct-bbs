//! Screen and flow state machine for Queens game

use super::puzzle::{DailyPuzzle, ValidationResult, generate_daily_puzzle, validate_solution, get_hint};
use super::state::{GameState, PlayerStats};

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Welcome/intro screen
    Intro,
    /// Main puzzle board
    Playing,
    /// Puzzle completed successfully
    Victory,
    /// Already played today
    AlreadyPlayed,
    /// Viewing stats
    Stats,
    /// Leaderboard
    Leaderboard,
    /// Confirm quit
    ConfirmQuit,
    /// Help screen
    Help,
}

/// Actions returned by QueensFlow for session to process
#[derive(Debug, Clone)]
pub enum QueensAction {
    /// Continue - no output needed
    Continue,
    /// Show screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state (triggers DB save)
    SaveGame,
    /// Record completion
    RecordCompletion { time_seconds: u32, hints_used: u32 },
    /// Player quit to main menu
    Quit,
}

/// Queens game flow state machine
pub struct QueensFlow {
    pub state: GameState,
    pub stats: PlayerStats,
    pub puzzle: DailyPuzzle,
    pub screen: GameScreen,
    input_buffer: String,
}

impl QueensFlow {
    /// Create new game flow for today's puzzle
    pub fn new(date: &str, stats: PlayerStats) -> Self {
        let puzzle = generate_daily_puzzle(date);
        let state = GameState::new(date);

        // Check if already played today
        let screen = if stats.has_played_today(date) {
            GameScreen::AlreadyPlayed
        } else {
            GameScreen::Intro
        };

        Self {
            state,
            stats,
            puzzle,
            screen,
            input_buffer: String::new(),
        }
    }

    /// Resume from saved state
    pub fn from_state(state: GameState, stats: PlayerStats) -> Self {
        let puzzle = generate_daily_puzzle(&state.puzzle_date);

        let screen = if state.completed {
            GameScreen::Victory
        } else if stats.has_played_today(&state.puzzle_date) {
            GameScreen::AlreadyPlayed
        } else {
            GameScreen::Intro
        };

        Self {
            state,
            stats,
            puzzle,
            screen,
            input_buffer: String::new(),
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get current game state
    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// Get player stats
    pub fn player_stats(&self) -> &PlayerStats {
        &self.stats
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> QueensAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return QueensAction::Echo("\x08 \x08".to_string());
            }
            return QueensAction::Continue;
        }

        // Enter processing for coordinate input
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars (except arrow keys which come as escape sequences)
        if ch.is_control() {
            return QueensAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input for coordinate entry
        if self.input_buffer.len() < 10 {
            self.input_buffer.push(ch);
            return QueensAction::Echo(ch.to_string());
        }

        QueensAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::Playing
                | GameScreen::Victory
                | GameScreen::AlreadyPlayed
                | GameScreen::Stats
                | GameScreen::Leaderboard
                | GameScreen::ConfirmQuit
                | GameScreen::Help
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> QueensAction {
        let raw_input = std::mem::take(&mut self.input_buffer);
        let input = raw_input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            // For Playing screen, pass both raw and trimmed input
            // so we can detect space character (which trims to empty string)
            GameScreen::Playing => self.handle_playing(&input, &raw_input),
            GameScreen::Victory => self.handle_victory(&input),
            GameScreen::AlreadyPlayed => self.handle_already_played(&input),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Leaderboard => self.handle_leaderboard(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
            GameScreen::Help => self.handle_help(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> QueensAction {
        self.screen = GameScreen::Playing;
        QueensAction::SaveGame
    }

    fn handle_playing(&mut self, input: &str, raw_input: &str) -> QueensAction {
        // Clear last message
        self.state.last_message = None;

        // Check for space character first (before trimming removes it)
        // raw_input contains the original input before trimming
        if raw_input == " " {
            self.state.toggle_queen();
            return QueensAction::SaveGame;
        }

        // Movement keys (WASD and HJKL vim-style, plus arrow key sequences)
        match input {
            // Up
            "W" | "K" | "8" => {
                self.state.move_cursor(-1, 0, self.puzzle.size);
                return QueensAction::SaveGame;
            }
            // Down
            "S" | "J" | "2" => {
                self.state.move_cursor(1, 0, self.puzzle.size);
                return QueensAction::SaveGame;
            }
            // Left
            "A" | "H" | "4" => {
                self.state.move_cursor(0, -1, self.puzzle.size);
                return QueensAction::SaveGame;
            }
            // Right
            "D" | "L" | "6" => {
                self.state.move_cursor(0, 1, self.puzzle.size);
                return QueensAction::SaveGame;
            }
            // Place/remove queen at cursor
            "E" | "5" | "." => {
                self.state.toggle_queen();
                return QueensAction::SaveGame;
            }
            // Submit solution (Enter key - empty input after trimming)
            "" => {
                return self.check_solution();
            }
            // Get hint
            "?" | "T" => {
                return self.use_hint();
            }
            // Show stats
            "I" => {
                self.screen = GameScreen::Stats;
                return QueensAction::SaveGame;
            }
            // Show help
            "F1" | "/" => {
                self.screen = GameScreen::Help;
                return QueensAction::SaveGame;
            }
            // Quit
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
                return QueensAction::SaveGame;
            }
            // Clear all queens
            "C" => {
                self.state.placements.clear();
                self.state.last_message = Some("Board cleared.".to_string());
                return QueensAction::SaveGame;
            }
            _ => {
                // Try to parse as coordinate (e.g., "A1", "B3")
                if let Some((row, col)) = parse_coordinate(input, self.puzzle.size) {
                    self.state.cursor = (row, col);
                    self.state.toggle_queen();
                    return QueensAction::SaveGame;
                }
            }
        }

        QueensAction::Continue
    }

    fn check_solution(&mut self) -> QueensAction {
        let result = validate_solution(&self.puzzle, &self.state.placements);

        match result {
            ValidationResult::Correct => {
                self.state.mark_completed();
                self.screen = GameScreen::Victory;

                // Record completion in stats
                let time = self.state.completion_time.unwrap_or(0);
                let hints = self.state.hints_used;

                QueensAction::RecordCompletion {
                    time_seconds: time,
                    hints_used: hints,
                }
            }
            _ => {
                self.state.last_message = result.error_message();
                QueensAction::SaveGame
            }
        }
    }

    fn use_hint(&mut self) -> QueensAction {
        if let Some((row, col)) = get_hint(&self.puzzle, &self.state.placements) {
            self.state.placements.push((row, col));
            self.state.hints_used += 1;
            self.state.cursor = (row, col);
            self.state.last_message = Some(format!(
                "Hint: Queen placed at {}{}",
                (b'A' + row as u8) as char,
                col + 1
            ));
        } else {
            self.state.last_message = Some("No more hints available.".to_string());
        }
        QueensAction::SaveGame
    }

    fn handle_victory(&mut self, input: &str) -> QueensAction {
        match input {
            "L" => {
                self.screen = GameScreen::Leaderboard;
                QueensAction::SaveGame
            }
            "S" => {
                self.screen = GameScreen::Stats;
                QueensAction::SaveGame
            }
            _ => {
                // Any other key returns to BBS
                QueensAction::Quit
            }
        }
    }

    fn handle_already_played(&mut self, input: &str) -> QueensAction {
        match input {
            "L" => {
                self.screen = GameScreen::Leaderboard;
                QueensAction::SaveGame
            }
            "S" => {
                self.screen = GameScreen::Stats;
                QueensAction::SaveGame
            }
            _ => QueensAction::Quit,
        }
    }

    fn handle_stats(&mut self, _input: &str) -> QueensAction {
        // Any key returns to previous screen
        if self.state.completed {
            self.screen = GameScreen::Victory;
        } else if self.stats.has_played_today(&self.state.puzzle_date) {
            self.screen = GameScreen::AlreadyPlayed;
        } else {
            self.screen = GameScreen::Playing;
        }
        QueensAction::SaveGame
    }

    fn handle_leaderboard(&mut self, _input: &str) -> QueensAction {
        // Any key returns to previous screen
        if self.state.completed {
            self.screen = GameScreen::Victory;
        } else if self.stats.has_played_today(&self.state.puzzle_date) {
            self.screen = GameScreen::AlreadyPlayed;
        } else {
            self.screen = GameScreen::Playing;
        }
        QueensAction::SaveGame
    }

    fn handle_confirm_quit(&mut self, input: &str) -> QueensAction {
        match input {
            "Y" => QueensAction::Quit,
            _ => {
                self.screen = GameScreen::Playing;
                QueensAction::SaveGame
            }
        }
    }

    fn handle_help(&mut self, _input: &str) -> QueensAction {
        self.screen = GameScreen::Playing;
        QueensAction::SaveGame
    }
}

/// Parse a coordinate like "A1", "B3", etc.
/// Returns (row, col) if valid, None otherwise
fn parse_coordinate(input: &str, board_size: usize) -> Option<(usize, usize)> {
    if input.len() < 2 {
        return None;
    }

    let chars: Vec<char> = input.chars().collect();

    // First char should be A-H (row)
    let row_char = chars[0].to_ascii_uppercase();
    if row_char < 'A' || row_char > 'H' {
        return None;
    }
    let row = (row_char as usize) - ('A' as usize);

    // Rest should be number 1-8 (column)
    let col_str: String = chars[1..].iter().collect();
    let col = col_str.parse::<usize>().ok()?;
    if col < 1 || col > 8 {
        return None;
    }
    let col = col - 1; // Convert to 0-indexed

    // Check bounds
    if row >= board_size || col >= board_size {
        return None;
    }

    Some((row, col))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow_intro_screen() {
        let stats = PlayerStats::new();
        let flow = QueensFlow::new("2026-01-30", stats);
        assert_eq!(*flow.current_screen(), GameScreen::Intro);
    }

    #[test]
    fn test_new_flow_already_played() {
        let mut stats = PlayerStats::new();
        stats.record_completion("2026-01-30", 120, 0);

        let flow = QueensFlow::new("2026-01-30", stats);
        assert_eq!(*flow.current_screen(), GameScreen::AlreadyPlayed);
    }

    #[test]
    fn test_intro_to_playing() {
        let stats = PlayerStats::new();
        let mut flow = QueensFlow::new("2026-01-30", stats);

        // Press any key to start
        flow.handle_char('\r');

        assert_eq!(*flow.current_screen(), GameScreen::Playing);
    }

    #[test]
    fn test_cursor_movement() {
        let stats = PlayerStats::new();
        let mut flow = QueensFlow::new("2026-01-30", stats);
        flow.screen = GameScreen::Playing;

        let initial = flow.state.cursor;

        // Move down
        flow.handle_char('s');
        assert_eq!(flow.state.cursor.0, initial.0 + 1);
    }

    #[test]
    fn test_parse_coordinate() {
        assert_eq!(parse_coordinate("A1", 8), Some((0, 0)));
        assert_eq!(parse_coordinate("B3", 8), Some((1, 2)));
        assert_eq!(parse_coordinate("H8", 8), Some((7, 7)));
        assert_eq!(parse_coordinate("a1", 8), Some((0, 0))); // Case insensitive

        assert_eq!(parse_coordinate("I1", 8), None); // Out of range
        assert_eq!(parse_coordinate("A9", 8), None); // Out of range
        assert_eq!(parse_coordinate("A", 8), None);  // Incomplete
        assert_eq!(parse_coordinate("1", 8), None);  // Wrong format
    }

    #[test]
    fn test_toggle_queen() {
        let stats = PlayerStats::new();
        let mut flow = QueensFlow::new("2026-01-30", stats);
        flow.screen = GameScreen::Playing;
        flow.state.cursor = (2, 3);

        assert!(!flow.state.has_queen(2, 3));

        // Toggle on
        flow.handle_char(' ');
        assert!(flow.state.has_queen(2, 3));

        // Toggle off
        flow.handle_char(' ');
        assert!(!flow.state.has_queen(2, 3));
    }

    #[test]
    fn test_hint_adds_queen() {
        let stats = PlayerStats::new();
        let mut flow = QueensFlow::new("2026-01-30", stats);
        flow.screen = GameScreen::Playing;

        let initial_count = flow.state.placements.len();
        flow.handle_char('?');

        assert_eq!(flow.state.placements.len(), initial_count + 1);
        assert_eq!(flow.state.hints_used, 1);
    }

    #[test]
    fn test_clear_board() {
        let stats = PlayerStats::new();
        let mut flow = QueensFlow::new("2026-01-30", stats);
        flow.screen = GameScreen::Playing;

        // Add some queens
        flow.state.placements.push((0, 0));
        flow.state.placements.push((1, 2));

        // Clear
        flow.handle_char('c');

        assert!(flow.state.placements.is_empty());
    }

    #[test]
    fn test_correct_solution_leads_to_victory() {
        let stats = PlayerStats::new();
        let mut flow = QueensFlow::new("2026-01-30", stats);
        flow.screen = GameScreen::Playing;

        // Set the correct solution
        flow.state.placements = flow.puzzle.solution.clone();

        // Submit
        let action = flow.check_solution();

        assert!(matches!(action, QueensAction::RecordCompletion { .. }));
        assert_eq!(flow.screen, GameScreen::Victory);
        assert!(flow.state.completed);
    }

    #[test]
    fn test_wrong_solution_shows_error() {
        let stats = PlayerStats::new();
        let mut flow = QueensFlow::new("2026-01-30", stats);
        flow.screen = GameScreen::Playing;

        // Wrong number of queens
        flow.state.placements = vec![(0, 0), (1, 1)]; // Only 2 queens

        let action = flow.check_solution();

        assert!(matches!(action, QueensAction::SaveGame));
        assert_eq!(flow.screen, GameScreen::Playing);
        assert!(flow.state.last_message.is_some());
    }
}
