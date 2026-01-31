//! Chess game state management

use serde::{Serialize, Deserialize};
use super::board::Board;
use super::moves::Move;

/// Player color assignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerColor {
    White,
    Black,
}

impl PlayerColor {
    pub fn to_board_color(&self) -> super::board::Color {
        match self {
            PlayerColor::White => super::board::Color::White,
            PlayerColor::Black => super::board::Color::Black,
        }
    }

    pub fn opposite(&self) -> PlayerColor {
        match self {
            PlayerColor::White => PlayerColor::Black,
            PlayerColor::Black => PlayerColor::White,
        }
    }
}

/// Game status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    /// Waiting for an opponent to join
    WaitingForOpponent,
    /// Game in progress, it's the specified color's turn
    InProgress,
    /// Game ended in checkmate, specified color wins
    Checkmate { winner: PlayerColor },
    /// Game ended in stalemate
    Stalemate,
    /// Game ended by resignation, specified color wins
    Resigned { winner: PlayerColor },
    /// Game ended by timeout (3-day rule), specified color wins
    Timeout { winner: PlayerColor },
    /// Game ended in draw by agreement
    DrawAgreed,
    /// Game ended due to 50-move rule
    Draw50Moves,
    /// Game ended due to threefold repetition
    DrawRepetition,
    /// Game ended due to insufficient material
    DrawInsufficientMaterial,
}

impl GameStatus {
    pub fn is_game_over(&self) -> bool {
        !matches!(self, GameStatus::WaitingForOpponent | GameStatus::InProgress)
    }

    pub fn winner(&self) -> Option<PlayerColor> {
        match self {
            GameStatus::Checkmate { winner } => Some(*winner),
            GameStatus::Resigned { winner } => Some(*winner),
            GameStatus::Timeout { winner } => Some(*winner),
            _ => None,
        }
    }
}

/// Matchmaking mode for creating/joining games
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchmakingMode {
    /// Open game - anyone can join
    Open,
    /// ELO-based matching (within range)
    EloMatched { min_elo: i32, max_elo: i32 },
    /// Direct challenge to specific user
    Challenge { target_user_id: i64, target_handle: String },
}

/// A move record with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRecord {
    pub move_notation: String,  // e.g., "e2e4"
    pub fen_after: String,      // Board state after move
    pub timestamp: String,      // ISO timestamp
}

/// Complete game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Unique game ID (set by database)
    pub game_id: Option<i64>,

    /// Current board state
    pub board: Board,

    /// White player's user ID
    pub white_user_id: i64,
    /// White player's handle
    pub white_handle: String,
    /// White player's ELO at game start
    pub white_elo: i32,

    /// Black player's user ID (None if waiting for opponent)
    pub black_user_id: Option<i64>,
    /// Black player's handle
    pub black_handle: Option<String>,
    /// Black player's ELO at game start
    pub black_elo: Option<i32>,

    /// Current game status
    pub status: GameStatus,

    /// Move history
    pub moves: Vec<MoveRecord>,

    /// Matchmaking mode
    pub matchmaking: MatchmakingMode,

    /// Timestamp of last move (for timeout tracking)
    pub last_move_time: String,

    /// Timestamp when game was created
    pub created_at: String,

    /// Whether white has offered a draw
    pub white_draw_offer: bool,
    /// Whether black has offered a draw
    pub black_draw_offer: bool,
}

impl GameState {
    /// Create a new game with the first player as white
    pub fn new(user_id: i64, handle: &str, elo: i32, matchmaking: MatchmakingMode) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        GameState {
            game_id: None,
            board: Board::new(),
            white_user_id: user_id,
            white_handle: handle.to_string(),
            white_elo: elo,
            black_user_id: None,
            black_handle: None,
            black_elo: None,
            status: GameStatus::WaitingForOpponent,
            moves: Vec::new(),
            matchmaking,
            last_move_time: now.clone(),
            created_at: now,
            white_draw_offer: false,
            black_draw_offer: false,
        }
    }

    /// Join the game as black
    pub fn join(&mut self, user_id: i64, handle: &str, elo: i32) {
        self.black_user_id = Some(user_id);
        self.black_handle = Some(handle.to_string());
        self.black_elo = Some(elo);
        self.status = GameStatus::InProgress;
        self.last_move_time = chrono::Utc::now().to_rfc3339();
    }

    /// Get which color is to move
    pub fn to_move(&self) -> PlayerColor {
        match self.board.side_to_move {
            super::board::Color::White => PlayerColor::White,
            super::board::Color::Black => PlayerColor::Black,
        }
    }

    /// Get a player's color in this game
    pub fn player_color(&self, user_id: i64) -> Option<PlayerColor> {
        if user_id == self.white_user_id {
            Some(PlayerColor::White)
        } else if self.black_user_id == Some(user_id) {
            Some(PlayerColor::Black)
        } else {
            None
        }
    }

    /// Check if it's the specified player's turn
    pub fn is_player_turn(&self, user_id: i64) -> bool {
        match self.status {
            GameStatus::InProgress => {
                let player_color = self.player_color(user_id);
                player_color == Some(self.to_move())
            }
            _ => false,
        }
    }

    /// Get the opponent's user ID
    pub fn opponent_id(&self, user_id: i64) -> Option<i64> {
        if user_id == self.white_user_id {
            self.black_user_id
        } else if self.black_user_id == Some(user_id) {
            Some(self.white_user_id)
        } else {
            None
        }
    }

    /// Make a move (validates and applies)
    pub fn make_move(&mut self, mv: Move) -> Result<super::moves::MoveResult, String> {
        if self.status != GameStatus::InProgress {
            return Err("Game is not in progress".to_string());
        }

        let result = super::moves::make_move(&mut self.board, mv)
            .ok_or_else(|| "Invalid move".to_string())?;

        // Record the move
        self.moves.push(MoveRecord {
            move_notation: mv.to_algebraic(),
            fen_after: self.board.to_fen(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        self.last_move_time = chrono::Utc::now().to_rfc3339();

        // Clear draw offers after a move
        self.white_draw_offer = false;
        self.black_draw_offer = false;

        // Update game status
        if result.is_checkmate {
            self.status = GameStatus::Checkmate {
                winner: self.to_move().opposite(), // The player who just moved won
            };
        } else if result.is_stalemate {
            self.status = GameStatus::Stalemate;
        } else if self.board.halfmove_clock >= 100 {
            self.status = GameStatus::Draw50Moves;
        }

        Ok(result)
    }

    /// Resign the game
    pub fn resign(&mut self, user_id: i64) -> Result<(), String> {
        let player_color = self.player_color(user_id)
            .ok_or("Not a player in this game")?;

        self.status = GameStatus::Resigned {
            winner: player_color.opposite(),
        };
        Ok(())
    }

    /// Offer a draw
    pub fn offer_draw(&mut self, user_id: i64) -> Result<bool, String> {
        let player_color = self.player_color(user_id)
            .ok_or("Not a player in this game")?;

        match player_color {
            PlayerColor::White => self.white_draw_offer = true,
            PlayerColor::Black => self.black_draw_offer = true,
        }

        // Check if both players have offered draw
        if self.white_draw_offer && self.black_draw_offer {
            self.status = GameStatus::DrawAgreed;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Accept a draw offer
    pub fn accept_draw(&mut self, user_id: i64) -> Result<(), String> {
        let player_color = self.player_color(user_id)
            .ok_or("Not a player in this game")?;

        // Check if opponent has offered
        let opponent_offered = match player_color {
            PlayerColor::White => self.black_draw_offer,
            PlayerColor::Black => self.white_draw_offer,
        };

        if !opponent_offered {
            return Err("No draw offer from opponent".to_string());
        }

        self.status = GameStatus::DrawAgreed;
        Ok(())
    }

    /// Get the game notation (PGN-style move list)
    pub fn get_move_list(&self) -> String {
        let mut result = String::new();
        for (i, mv) in self.moves.iter().enumerate() {
            if i % 2 == 0 {
                result.push_str(&format!("{}. ", i / 2 + 1));
            }
            result.push_str(&mv.move_notation);
            result.push(' ');
        }
        result.trim().to_string()
    }

    /// Check if the game has timed out (3 days without a move)
    pub fn check_timeout(&mut self) -> bool {
        if self.status != GameStatus::InProgress {
            return false;
        }

        let last_move = chrono::DateTime::parse_from_rfc3339(&self.last_move_time)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let now = chrono::Utc::now();
        let duration = now - last_move;

        if duration.num_hours() >= 72 {
            // Player whose turn it is loses
            self.status = GameStatus::Timeout {
                winner: self.to_move().opposite(),
            };
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::moves::Move;

    #[test]
    fn test_new_game() {
        let game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        assert_eq!(game.status, GameStatus::WaitingForOpponent);
        assert_eq!(game.white_user_id, 1);
        assert!(game.black_user_id.is_none());
    }

    #[test]
    fn test_join_game() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        assert_eq!(game.status, GameStatus::InProgress);
        assert_eq!(game.black_user_id, Some(2));
        assert_eq!(game.black_handle.as_deref(), Some("Player2"));
    }

    #[test]
    fn test_player_color() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        assert_eq!(game.player_color(1), Some(PlayerColor::White));
        assert_eq!(game.player_color(2), Some(PlayerColor::Black));
        assert_eq!(game.player_color(3), None);
    }

    #[test]
    fn test_make_move() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        let mv = Move::from_algebraic("e2e4").unwrap();
        let result = game.make_move(mv);

        assert!(result.is_ok());
        assert_eq!(game.moves.len(), 1);
        assert_eq!(game.to_move(), PlayerColor::Black);
    }

    #[test]
    fn test_resign() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        game.resign(1).unwrap();

        assert_eq!(game.status, GameStatus::Resigned { winner: PlayerColor::Black });
    }

    #[test]
    fn test_draw_offer() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        // White offers draw
        let accepted = game.offer_draw(1).unwrap();
        assert!(!accepted);
        assert!(game.white_draw_offer);

        // Black accepts
        game.accept_draw(2).unwrap();
        assert_eq!(game.status, GameStatus::DrawAgreed);
    }
}
