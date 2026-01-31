//! Chess game screen and flow management

use super::state::{GameState, MatchmakingMode, PlayerColor};
use super::moves::Move;

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Main lobby - view games, create game, join game
    Lobby,
    /// Creating a new game - select matchmaking type
    CreateGame,
    /// Selecting ELO range for matchmaking
    SelectEloRange { min: i32, max: i32, editing_min: bool },
    /// Entering handle for direct challenge
    ChallengePlayer { input: String },
    /// Viewing and playing an active game
    InGame { game_id: i64 },
    /// Entering a move
    EnterMove { game_id: i64, input: String },
    /// Confirming resignation
    ConfirmResign { game_id: i64 },
    /// Viewing leaderboard
    Leaderboard,
    /// Viewing completed game history
    History,
    /// Confirm quit to main menu
    ConfirmQuit,
}

/// Actions returned by ChessFlow for session.rs to handle
#[derive(Debug, Clone)]
pub enum ChessAction {
    /// Continue - no output needed
    Continue,
    /// Show screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Request to load/refresh lobby data
    RefreshLobby,
    /// Request to create a new game with matchmaking mode
    CreateGame { matchmaking: MatchmakingMode },
    /// Request to join an open game
    JoinGame { game_id: i64 },
    /// Request to load a game for viewing/playing
    LoadGame { game_id: i64 },
    /// Request to make a move
    MakeMove { game_id: i64, mv: Move },
    /// Request to resign the game
    Resign { game_id: i64 },
    /// Request to offer a draw
    OfferDraw { game_id: i64 },
    /// Request to accept a draw
    AcceptDraw { game_id: i64 },
    /// Request to load leaderboard
    LoadLeaderboard,
    /// Request to challenge a specific player
    ChallengePlayer { handle: String },
    /// Save game state
    SaveGame,
    /// Game is over
    GameOver { game_id: i64 },
    /// Player quit to main menu
    Quit,
}

/// Chess game flow state machine
pub struct ChessFlow {
    /// Current screen
    pub screen: GameScreen,
    /// Current user ID
    pub user_id: i64,
    /// Current user handle
    pub handle: String,
    /// Current user ELO
    pub elo: i32,
    /// Currently loaded game (if in game screen)
    pub current_game: Option<GameState>,
    /// Open games list (game_id, handle, elo, type)
    pub open_games: Vec<(i64, String, i32, String)>,
    /// Active games for this user (game_id, opponent, is_your_turn, created)
    pub active_games: Vec<(i64, String, bool, String)>,
    /// Incoming challenges (game_id, handle, elo)
    pub incoming_challenges: Vec<(i64, String, i32)>,
    /// Outgoing challenges (game_id, handle)
    pub outgoing_challenges: Vec<(i64, String)>,
    /// Leaderboard entries (user_id, handle, elo, wins, losses)
    pub leaderboard: Vec<(i64, String, i32, i32, i32)>,
    /// Input buffer
    input_buffer: String,
    /// Last message to display
    pub last_message: Option<String>,
}

impl ChessFlow {
    /// Create new chess flow for a user
    pub fn new(user_id: i64, handle: &str, elo: i32) -> Self {
        ChessFlow {
            screen: GameScreen::Lobby,
            user_id,
            handle: handle.to_string(),
            elo,
            current_game: None,
            open_games: Vec::new(),
            active_games: Vec::new(),
            incoming_challenges: Vec::new(),
            outgoing_challenges: Vec::new(),
            leaderboard: Vec::new(),
            input_buffer: String::new(),
            last_message: None,
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> ChessAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return ChessAction::Echo("\x08 \x08".to_string());
            }
            return ChessAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars (except enter/backspace handled above)
        if ch.is_control() {
            return ChessAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input
        if self.input_buffer.len() < 20 {
            self.input_buffer.push(ch);
            return ChessAction::Echo(ch.to_string());
        }

        ChessAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Lobby
                | GameScreen::CreateGame
                | GameScreen::ConfirmResign { .. }
                | GameScreen::Leaderboard
                | GameScreen::ConfirmQuit
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> ChessAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Lobby => self.handle_lobby(&input),
            GameScreen::CreateGame => self.handle_create_game(&input),
            GameScreen::SelectEloRange { min, max, editing_min } => {
                let min = *min;
                let max = *max;
                let editing_min = *editing_min;
                self.handle_elo_range(&input, min, max, editing_min)
            }
            GameScreen::ChallengePlayer { input: current } => {
                let current = current.clone();
                self.handle_challenge_input(&input, &current)
            }
            GameScreen::InGame { game_id } => {
                let game_id = *game_id;
                self.handle_in_game(&input, game_id)
            }
            GameScreen::EnterMove { game_id, input: current } => {
                let game_id = *game_id;
                let current = current.clone();
                self.handle_enter_move(&input, game_id, &current)
            }
            GameScreen::ConfirmResign { game_id } => {
                let game_id = *game_id;
                self.handle_confirm_resign(&input, game_id)
            }
            GameScreen::Leaderboard => self.handle_leaderboard(&input),
            GameScreen::History => self.handle_history(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_lobby(&mut self, input: &str) -> ChessAction {
        self.last_message = None;

        match input {
            "N" => {
                // New game
                self.screen = GameScreen::CreateGame;
                ChessAction::RefreshLobby
            }
            "L" => {
                // Leaderboard
                self.screen = GameScreen::Leaderboard;
                ChessAction::LoadLeaderboard
            }
            "R" => {
                // Refresh lobby
                ChessAction::RefreshLobby
            }
            "Q" | "X" => {
                // Quit
                self.screen = GameScreen::ConfirmQuit;
                ChessAction::Continue
            }
            _ => {
                // Check if joining an open game
                if let Ok(idx) = input.parse::<usize>() {
                    if idx >= 1 && idx <= self.open_games.len() {
                        let (game_id, _, _, _) = &self.open_games[idx - 1];
                        return ChessAction::JoinGame { game_id: *game_id };
                    }
                }

                // Check if selecting an active game
                if input.starts_with('G') {
                    if let Ok(idx) = input[1..].parse::<usize>() {
                        if idx >= 1 && idx <= self.active_games.len() {
                            let (game_id, _, _, _) = &self.active_games[idx - 1];
                            self.screen = GameScreen::InGame { game_id: *game_id };
                            return ChessAction::LoadGame { game_id: *game_id };
                        }
                    }
                }

                // Check if accepting a challenge (A, B, C, etc.)
                if input.len() == 1 {
                    let ch = input.chars().next().unwrap();
                    if ch >= 'A' && ch <= 'Z' {
                        let idx = (ch as u8 - b'A') as usize;
                        if idx < self.incoming_challenges.len() {
                            let (game_id, _, _) = &self.incoming_challenges[idx];
                            return ChessAction::JoinGame { game_id: *game_id };
                        }
                    }
                }

                ChessAction::Continue
            }
        }
    }

    fn handle_create_game(&mut self, input: &str) -> ChessAction {
        match input {
            "1" | "O" => {
                // Open game
                ChessAction::CreateGame {
                    matchmaking: MatchmakingMode::Open,
                }
            }
            "2" | "E" => {
                // ELO matched
                self.screen = GameScreen::SelectEloRange {
                    min: self.elo - 200,
                    max: self.elo + 200,
                    editing_min: true,
                };
                ChessAction::Continue
            }
            "3" | "C" => {
                // Challenge player
                self.screen = GameScreen::ChallengePlayer { input: String::new() };
                ChessAction::Continue
            }
            "Q" | "B" => {
                // Back to lobby
                self.screen = GameScreen::Lobby;
                ChessAction::RefreshLobby
            }
            _ => ChessAction::Continue,
        }
    }

    fn handle_elo_range(&mut self, input: &str, mut min: i32, mut max: i32, editing_min: bool) -> ChessAction {
        match input {
            "Q" | "B" => {
                self.screen = GameScreen::CreateGame;
                return ChessAction::Continue;
            }
            "" => {
                // Submit with current values
                return ChessAction::CreateGame {
                    matchmaking: MatchmakingMode::EloMatched { min_elo: min, max_elo: max },
                };
            }
            _ => {
                // Try to parse as number
                if let Ok(val) = input.parse::<i32>() {
                    if editing_min {
                        min = val;
                        self.screen = GameScreen::SelectEloRange { min, max, editing_min: false };
                    } else {
                        max = val;
                        // Create the game
                        return ChessAction::CreateGame {
                            matchmaking: MatchmakingMode::EloMatched { min_elo: min, max_elo: max },
                        };
                    }
                }
            }
        }
        ChessAction::Continue
    }

    fn handle_challenge_input(&mut self, input: &str, current: &str) -> ChessAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::CreateGame;
            return ChessAction::Continue;
        }

        // Input is the handle to challenge
        let handle = if current.is_empty() {
            input.to_string()
        } else {
            format!("{}{}", current, input)
        };

        if !handle.is_empty() {
            ChessAction::ChallengePlayer { handle }
        } else {
            ChessAction::Continue
        }
    }

    fn handle_in_game(&mut self, input: &str, game_id: i64) -> ChessAction {
        match input {
            "M" => {
                // Enter move
                self.screen = GameScreen::EnterMove { game_id, input: String::new() };
                ChessAction::Continue
            }
            "D" => {
                // Offer/accept draw
                if let Some(ref game) = self.current_game {
                    let opponent_offered = match game.player_color(self.user_id) {
                        Some(PlayerColor::White) => game.black_draw_offer,
                        Some(PlayerColor::Black) => game.white_draw_offer,
                        None => false,
                    };
                    if opponent_offered {
                        return ChessAction::AcceptDraw { game_id };
                    } else {
                        return ChessAction::OfferDraw { game_id };
                    }
                }
                ChessAction::Continue
            }
            "R" => {
                // Resign
                self.screen = GameScreen::ConfirmResign { game_id };
                ChessAction::Continue
            }
            "Q" | "B" => {
                // Back to lobby
                self.current_game = None;
                self.screen = GameScreen::Lobby;
                ChessAction::RefreshLobby
            }
            _ => {
                // Try to parse as move directly (e.g., "E2E4")
                if input.len() >= 4 {
                    if let Some(mv) = Move::from_algebraic(input) {
                        return ChessAction::MakeMove { game_id, mv };
                    }
                }
                ChessAction::Continue
            }
        }
    }

    fn handle_enter_move(&mut self, input: &str, game_id: i64, current: &str) -> ChessAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::InGame { game_id };
            return ChessAction::Continue;
        }

        // Build the move string
        let move_str = format!("{}{}", current, input);

        if move_str.len() >= 4 {
            if let Some(mv) = Move::from_algebraic(&move_str) {
                self.screen = GameScreen::InGame { game_id };
                return ChessAction::MakeMove { game_id, mv };
            } else {
                self.last_message = Some("Invalid move format. Use e2e4 format.".to_string());
                self.screen = GameScreen::EnterMove { game_id, input: String::new() };
            }
        } else {
            self.screen = GameScreen::EnterMove { game_id, input: move_str };
        }

        ChessAction::Continue
    }

    fn handle_confirm_resign(&mut self, input: &str, game_id: i64) -> ChessAction {
        match input {
            "Y" => {
                self.screen = GameScreen::InGame { game_id };
                ChessAction::Resign { game_id }
            }
            _ => {
                self.screen = GameScreen::InGame { game_id };
                ChessAction::LoadGame { game_id }
            }
        }
    }

    fn handle_leaderboard(&mut self, _input: &str) -> ChessAction {
        self.screen = GameScreen::Lobby;
        ChessAction::RefreshLobby
    }

    fn handle_history(&mut self, _input: &str) -> ChessAction {
        self.screen = GameScreen::Lobby;
        ChessAction::RefreshLobby
    }

    fn handle_confirm_quit(&mut self, input: &str) -> ChessAction {
        match input {
            "Y" => ChessAction::Quit,
            _ => {
                self.screen = GameScreen::Lobby;
                ChessAction::RefreshLobby
            }
        }
    }

    /// Set the current game being viewed
    pub fn set_current_game(&mut self, game: GameState) {
        let game_id = game.game_id.unwrap_or(0);
        self.current_game = Some(game);
        self.screen = GameScreen::InGame { game_id };
    }

    /// Update lobby data
    pub fn update_lobby(
        &mut self,
        open_games: Vec<(i64, String, i32, String)>,
        active_games: Vec<(i64, String, bool, String)>,
        incoming: Vec<(i64, String, i32)>,
        outgoing: Vec<(i64, String)>,
    ) {
        self.open_games = open_games;
        self.active_games = active_games;
        self.incoming_challenges = incoming;
        self.outgoing_challenges = outgoing;
    }

    /// Set leaderboard data
    pub fn set_leaderboard(&mut self, entries: Vec<(i64, String, i32, i32, i32)>) {
        self.leaderboard = entries;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = ChessFlow::new(1, "TestPlayer", 1200);
        assert_eq!(*flow.current_screen(), GameScreen::Lobby);
        assert_eq!(flow.user_id, 1);
        assert_eq!(flow.elo, 1200);
    }

    #[test]
    fn test_lobby_navigation() {
        let mut flow = ChessFlow::new(1, "TestPlayer", 1200);

        // Press N to create new game
        let action = flow.handle_char('N');
        assert!(matches!(action, ChessAction::RefreshLobby));
        assert_eq!(*flow.current_screen(), GameScreen::CreateGame);

        // Press Q to go back
        let action = flow.handle_char('Q');
        assert!(matches!(action, ChessAction::RefreshLobby));
        assert_eq!(*flow.current_screen(), GameScreen::Lobby);
    }

    #[test]
    fn test_create_open_game() {
        let mut flow = ChessFlow::new(1, "TestPlayer", 1200);
        flow.screen = GameScreen::CreateGame;

        let action = flow.handle_char('1');
        assert!(matches!(action, ChessAction::CreateGame { matchmaking: MatchmakingMode::Open }));
    }

    #[test]
    fn test_join_game() {
        let mut flow = ChessFlow::new(1, "TestPlayer", 1200);
        flow.open_games = vec![(42, "Opponent".to_string(), 1300, "Open".to_string())];

        let action = flow.handle_char('1');
        assert!(matches!(action, ChessAction::JoinGame { game_id: 42 }));
    }

    #[test]
    fn test_enter_move() {
        let mut flow = ChessFlow::new(1, "TestPlayer", 1200);
        flow.screen = GameScreen::InGame { game_id: 1 };

        // Enter a move directly
        for ch in "e2e4".chars() {
            let _ = flow.handle_char(ch);
        }
        let action = flow.handle_char('\r');

        assert!(matches!(action, ChessAction::MakeMove { game_id: 1, .. }));
    }
}
