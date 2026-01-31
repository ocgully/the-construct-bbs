//! Sudoku game screen and flow state machine

use super::state::GameState;
use super::generator::{generate_puzzle, get_eastern_date};

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Welcome/intro screen
    Intro,
    /// Main puzzle screen
    Playing,
    /// Puzzle completed
    Completed,
    /// Already played today
    AlreadyPlayed,
    /// Leaderboard/stats view
    Stats,
    /// Help screen
    Help,
    /// Quit confirmation
    ConfirmQuit,
}

/// Actions returned by SudokuFlow for session to process
#[derive(Debug, Clone)]
pub enum SudokuAction {
    /// Continue - no output needed
    Continue,
    /// Show screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state (triggers DB save)
    SaveGame,
    /// Puzzle completed
    PuzzleComplete { time_seconds: u32, errors: u32 },
    /// Player quit to main menu
    Quit,
}

/// Sudoku game flow state machine
pub struct SudokuFlow {
    pub state: Option<GameState>,
    pub screen: GameScreen,
    input_buffer: String,
    /// Whether player already completed today's puzzle
    pub already_completed: bool,
    /// Player's completion time if already completed
    pub completion_time: Option<u32>,
    /// Player's current streak
    pub current_streak: u32,
    /// Player's longest streak
    pub longest_streak: u32,
}

impl SudokuFlow {
    /// Create a new game flow for a new puzzle
    pub fn new() -> Self {
        Self {
            state: None,
            screen: GameScreen::Intro,
            input_buffer: String::new(),
            already_completed: false,
            completion_time: None,
            current_streak: 0,
            longest_streak: 0,
        }
    }

    /// Create flow with stats (for already played scenario)
    pub fn with_stats(already_completed: bool, completion_time: Option<u32>, current_streak: u32, longest_streak: u32) -> Self {
        Self {
            state: None,
            screen: if already_completed { GameScreen::AlreadyPlayed } else { GameScreen::Intro },
            input_buffer: String::new(),
            already_completed,
            completion_time,
            current_streak,
            longest_streak,
        }
    }

    /// Initialize a new puzzle for today
    pub fn start_puzzle(&mut self) {
        let date = get_eastern_date();
        let daily = generate_puzzle(&date);
        self.state = Some(GameState::new(&date, &daily.puzzle, &daily.solution));
        self.screen = GameScreen::Playing;
    }

    /// Resume a saved puzzle
    pub fn resume_puzzle(&mut self, state: GameState) {
        self.state = Some(state);
        self.screen = GameScreen::Playing;
    }

    /// Get current screen for rendering
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get current game state
    pub fn game_state(&self) -> Option<&GameState> {
        self.state.as_ref()
    }

    /// Get mutable game state
    pub fn game_state_mut(&mut self) -> Option<&mut GameState> {
        self.state.as_mut()
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> SudokuAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return SudokuAction::Echo("\x08 \x08".to_string());
            }
            return SudokuAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return SudokuAction::Continue;
        }

        // For most screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input for quantity entry, etc.
        if self.input_buffer.len() < 20 {
            self.input_buffer.push(ch);
            return SudokuAction::Echo(ch.to_string());
        }

        SudokuAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::Playing
                | GameScreen::Completed
                | GameScreen::AlreadyPlayed
                | GameScreen::Stats
                | GameScreen::Help
                | GameScreen::ConfirmQuit
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> SudokuAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::Playing => self.handle_playing(&input),
            GameScreen::Completed => self.handle_completed(&input),
            GameScreen::AlreadyPlayed => self.handle_already_played(&input),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Help => self.handle_help(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> SudokuAction {
        if self.already_completed {
            self.screen = GameScreen::AlreadyPlayed;
        } else {
            self.start_puzzle();
        }
        SudokuAction::SaveGame
    }

    fn handle_playing(&mut self, input: &str) -> SudokuAction {
        let ch = input.chars().next().unwrap_or(' ');

        // Number entry
        if let Some(num) = ch.to_digit(10) {
            if num >= 1 && num <= 9 {
                if let Some(state) = &mut self.state {
                    state.enter_number(num as u8);

                    // Check for completion
                    if state.completed {
                        self.screen = GameScreen::Completed;
                        return SudokuAction::PuzzleComplete {
                            time_seconds: state.elapsed_seconds,
                            errors: state.error_count,
                        };
                    }
                }
                return SudokuAction::SaveGame;
            }
        }

        // Clear cell
        if ch == '0' || ch == ' ' || input == "C" || input == "DELETE" {
            if let Some(state) = &mut self.state {
                state.clear_cell();
            }
            return SudokuAction::SaveGame;
        }

        // Arrow keys / WASD movement
        match input {
            "W" | "K" => {
                if let Some(state) = &mut self.state {
                    state.move_cursor(-1, 0);
                }
                return SudokuAction::SaveGame;
            }
            "S" | "J" => {
                if let Some(state) = &mut self.state {
                    state.move_cursor(1, 0);
                }
                return SudokuAction::SaveGame;
            }
            "A" | "H" => {
                if let Some(state) = &mut self.state {
                    state.move_cursor(0, -1);
                }
                return SudokuAction::SaveGame;
            }
            "D" | "L" => {
                if let Some(state) = &mut self.state {
                    state.move_cursor(0, 1);
                }
                return SudokuAction::SaveGame;
            }
            _ => {}
        }

        // Pencil mode toggle
        if input == "P" {
            if let Some(state) = &mut self.state {
                state.toggle_pencil_mode();
            }
            return SudokuAction::SaveGame;
        }

        // Help
        if input == "?" {
            self.screen = GameScreen::Help;
            return SudokuAction::SaveGame;
        }

        // Stats
        if input == "T" {
            self.screen = GameScreen::Stats;
            return SudokuAction::SaveGame;
        }

        // Quit
        if input == "Q" {
            self.screen = GameScreen::ConfirmQuit;
            return SudokuAction::SaveGame;
        }

        SudokuAction::Continue
    }

    fn handle_completed(&mut self, _input: &str) -> SudokuAction {
        // Any key returns to BBS
        SudokuAction::Quit
    }

    fn handle_already_played(&mut self, input: &str) -> SudokuAction {
        match input {
            "T" => {
                self.screen = GameScreen::Stats;
                SudokuAction::SaveGame
            }
            _ => SudokuAction::Quit,
        }
    }

    fn handle_stats(&mut self, _input: &str) -> SudokuAction {
        // Any key returns to previous screen
        if self.already_completed {
            self.screen = GameScreen::AlreadyPlayed;
        } else if self.state.is_some() {
            self.screen = GameScreen::Playing;
        } else {
            return SudokuAction::Quit;
        }
        SudokuAction::SaveGame
    }

    fn handle_help(&mut self, _input: &str) -> SudokuAction {
        // Any key returns to playing
        if self.state.is_some() {
            self.screen = GameScreen::Playing;
            SudokuAction::SaveGame
        } else {
            SudokuAction::Quit
        }
    }

    fn handle_confirm_quit(&mut self, input: &str) -> SudokuAction {
        match input {
            "Y" => SudokuAction::Quit,
            _ => {
                if self.state.is_some() {
                    self.screen = GameScreen::Playing;
                    SudokuAction::SaveGame
                } else {
                    SudokuAction::Quit
                }
            }
        }
    }

    /// Update elapsed time
    pub fn tick_timer(&mut self, seconds: u32) {
        if let Some(state) = &mut self.state {
            if !state.completed {
                state.elapsed_seconds += seconds;
            }
        }
    }
}

impl Default for SudokuFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = SudokuFlow::new();
        assert!(matches!(flow.screen, GameScreen::Intro));
        assert!(flow.state.is_none());
    }

    #[test]
    fn test_start_puzzle() {
        let mut flow = SudokuFlow::new();
        flow.start_puzzle();
        assert!(matches!(flow.screen, GameScreen::Playing));
        assert!(flow.state.is_some());
    }

    #[test]
    fn test_already_completed_flow() {
        let flow = SudokuFlow::with_stats(true, Some(300), 5, 10);
        assert!(matches!(flow.screen, GameScreen::AlreadyPlayed));
        assert!(flow.already_completed);
        assert_eq!(flow.completion_time, Some(300));
    }

    #[test]
    fn test_number_entry() {
        let mut flow = SudokuFlow::new();
        flow.start_puzzle();

        // Move to an empty cell first
        if let Some(state) = &mut flow.state {
            state.cursor = (0, 2); // Assuming this is empty in most puzzles
        }

        let action = flow.handle_char('5');
        assert!(matches!(action, SudokuAction::SaveGame | SudokuAction::PuzzleComplete { .. }));
    }

    #[test]
    fn test_cursor_movement() {
        let mut flow = SudokuFlow::new();
        flow.start_puzzle();

        let initial_cursor = flow.state.as_ref().map(|s| s.cursor);

        flow.handle_char('D'); // Move right
        let new_cursor = flow.state.as_ref().map(|s| s.cursor);

        assert_ne!(initial_cursor, new_cursor);
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = SudokuFlow::new();
        flow.start_puzzle();

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::Playing));
    }

    #[test]
    fn test_help_screen() {
        let mut flow = SudokuFlow::new();
        flow.start_puzzle();

        flow.handle_char('?');
        assert!(matches!(flow.screen, GameScreen::Help));

        flow.handle_char(' ');
        assert!(matches!(flow.screen, GameScreen::Playing));
    }

    #[test]
    fn test_pencil_mode_toggle() {
        let mut flow = SudokuFlow::new();
        flow.start_puzzle();

        let initial_mode = flow.state.as_ref().map(|s| s.pencil_mode);
        flow.handle_char('P');
        let new_mode = flow.state.as_ref().map(|s| s.pencil_mode);

        assert_ne!(initial_mode, new_mode);
    }
}
