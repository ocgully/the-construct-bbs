//! Chess database layer
//!
//! Schema:
//! - players: ELO ratings and stats per user
//! - games: Active and completed games
//! - moves: Move history per game
//! - config: Sysop configurable settings

#![allow(dead_code)]

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use crate::games::chess::{GameState, MatchmakingMode};

/// Default starting ELO
pub const DEFAULT_ELO: i32 = 1200;

/// Default max concurrent games (sysop configurable)
pub const DEFAULT_MAX_CONCURRENT: i32 = 5;

/// Max allowed concurrent games
pub const MAX_CONCURRENT_LIMIT: i32 = 20;

pub struct ChessDb {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct PlayerRating {
    pub user_id: i64,
    pub handle: String,
    pub elo: i32,
    pub wins: i32,
    pub losses: i32,
    pub draws: i32,
    pub games_played: i32,
}

#[derive(Debug, Clone)]
pub struct GameRecord {
    pub game_id: i64,
    pub white_user_id: i64,
    pub white_handle: String,
    pub white_elo: i32,
    pub black_user_id: Option<i64>,
    pub black_handle: Option<String>,
    pub black_elo: Option<i32>,
    pub status: String,
    pub fen: String,
    pub matchmaking_type: String,
    pub last_move_time: String,
    pub created_at: String,
}

impl ChessDb {
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Enable WAL mode for concurrency
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await?;

        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // Player ratings table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS players (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                elo INTEGER NOT NULL DEFAULT 1200,
                wins INTEGER NOT NULL DEFAULT 0,
                losses INTEGER NOT NULL DEFAULT 0,
                draws INTEGER NOT NULL DEFAULT 0,
                games_played INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours'))
            )
        "#).execute(&self.pool).await?;

        // Games table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS games (
                game_id INTEGER PRIMARY KEY AUTOINCREMENT,
                white_user_id INTEGER NOT NULL,
                white_handle TEXT NOT NULL,
                white_elo INTEGER NOT NULL,
                black_user_id INTEGER,
                black_handle TEXT,
                black_elo INTEGER,
                status TEXT NOT NULL DEFAULT 'waiting',
                fen TEXT NOT NULL,
                matchmaking_type TEXT NOT NULL DEFAULT 'open',
                challenge_target_id INTEGER,
                challenge_target_handle TEXT,
                white_draw_offer INTEGER NOT NULL DEFAULT 0,
                black_draw_offer INTEGER NOT NULL DEFAULT 0,
                winner_user_id INTEGER,
                last_move_time TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
                created_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
                completed_at TEXT,
                FOREIGN KEY (white_user_id) REFERENCES players(user_id),
                FOREIGN KEY (black_user_id) REFERENCES players(user_id)
            )
        "#).execute(&self.pool).await?;

        // Moves table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS moves (
                move_id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                move_number INTEGER NOT NULL,
                move_notation TEXT NOT NULL,
                fen_after TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
                FOREIGN KEY (game_id) REFERENCES games(game_id)
            )
        "#).execute(&self.pool).await?;

        // Config table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_games_status ON games(status)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_games_white ON games(white_user_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_games_black ON games(black_user_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_moves_game ON moves(game_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_players_elo ON players(elo DESC)")
            .execute(&self.pool).await?;

        // Set default config
        sqlx::query(
            "INSERT OR IGNORE INTO config (key, value) VALUES ('max_concurrent_games', ?)"
        )
        .bind(DEFAULT_MAX_CONCURRENT.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get or create player rating
    pub async fn get_or_create_player(&self, user_id: i64, handle: &str) -> Result<PlayerRating, sqlx::Error> {
        // Try to get existing
        let existing: Option<(i64, String, i32, i32, i32, i32, i32)> = sqlx::query_as(
            "SELECT user_id, handle, elo, wins, losses, draws, games_played FROM players WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((uid, h, elo, wins, losses, draws, games)) = existing {
            Ok(PlayerRating {
                user_id: uid,
                handle: h,
                elo,
                wins,
                losses,
                draws,
                games_played: games,
            })
        } else {
            // Create new player
            sqlx::query(
                "INSERT INTO players (user_id, handle, elo) VALUES (?, ?, ?)"
            )
            .bind(user_id)
            .bind(handle)
            .bind(DEFAULT_ELO)
            .execute(&self.pool)
            .await?;

            Ok(PlayerRating {
                user_id,
                handle: handle.to_string(),
                elo: DEFAULT_ELO,
                wins: 0,
                losses: 0,
                draws: 0,
                games_played: 0,
            })
        }
    }

    /// Update player rating after game
    pub async fn update_player_rating(
        &self,
        user_id: i64,
        new_elo: i32,
        win: bool,
        loss: bool,
        draw: bool,
    ) -> Result<(), sqlx::Error> {
        let wins_delta = if win { 1 } else { 0 };
        let losses_delta = if loss { 1 } else { 0 };
        let draws_delta = if draw { 1 } else { 0 };

        sqlx::query(
            r#"UPDATE players SET
                elo = ?,
                wins = wins + ?,
                losses = losses + ?,
                draws = draws + ?,
                games_played = games_played + 1,
                updated_at = datetime('now', '-5 hours')
            WHERE user_id = ?"#
        )
        .bind(new_elo)
        .bind(wins_delta)
        .bind(losses_delta)
        .bind(draws_delta)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a new game
    pub async fn create_game(&self, game: &GameState) -> Result<i64, sqlx::Error> {
        let matchmaking_type = match &game.matchmaking {
            MatchmakingMode::Open => "open".to_string(),
            MatchmakingMode::EloMatched { min_elo, max_elo } => format!("elo:{}-{}", min_elo, max_elo),
            MatchmakingMode::Challenge { target_user_id, target_handle } => {
                format!("challenge:{}:{}", target_user_id, target_handle)
            }
        };

        let (challenge_target_id, challenge_target_handle) = match &game.matchmaking {
            MatchmakingMode::Challenge { target_user_id, target_handle } => {
                (Some(*target_user_id), Some(target_handle.clone()))
            }
            _ => (None, None),
        };

        let result = sqlx::query(
            r#"INSERT INTO games (
                white_user_id, white_handle, white_elo,
                status, fen, matchmaking_type,
                challenge_target_id, challenge_target_handle
            ) VALUES (?, ?, ?, 'waiting', ?, ?, ?, ?)"#
        )
        .bind(game.white_user_id)
        .bind(&game.white_handle)
        .bind(game.white_elo)
        .bind(game.board.to_fen())
        .bind(&matchmaking_type)
        .bind(challenge_target_id)
        .bind(challenge_target_handle)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Join a game as black
    pub async fn join_game(&self, game_id: i64, user_id: i64, handle: &str, elo: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE games SET
                black_user_id = ?,
                black_handle = ?,
                black_elo = ?,
                status = 'in_progress',
                last_move_time = datetime('now', '-5 hours')
            WHERE game_id = ? AND status = 'waiting'"#
        )
        .bind(user_id)
        .bind(handle)
        .bind(elo)
        .bind(game_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a game by ID
    pub async fn get_game(&self, game_id: i64) -> Result<Option<GameRecord>, sqlx::Error> {
        let row: Option<(i64, i64, String, i32, Option<i64>, Option<String>, Option<i32>, String, String, String, String, String)> = sqlx::query_as(
            r#"SELECT
                game_id, white_user_id, white_handle, white_elo,
                black_user_id, black_handle, black_elo,
                status, fen, matchmaking_type, last_move_time, created_at
            FROM games WHERE game_id = ?"#
        )
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(gid, wuid, wh, we, buid, bh, be, status, fen, mt, lmt, ca)| GameRecord {
            game_id: gid,
            white_user_id: wuid,
            white_handle: wh,
            white_elo: we,
            black_user_id: buid,
            black_handle: bh,
            black_elo: be,
            status,
            fen,
            matchmaking_type: mt,
            last_move_time: lmt,
            created_at: ca,
        }))
    }

    /// Get all moves for a game
    pub async fn get_moves(&self, game_id: i64) -> Result<Vec<(i32, String, String)>, sqlx::Error> {
        let rows: Vec<(i32, String, String)> = sqlx::query_as(
            "SELECT move_number, move_notation, fen_after FROM moves WHERE game_id = ? ORDER BY move_number"
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Record a move
    pub async fn record_move(&self, game_id: i64, move_number: i32, notation: &str, fen_after: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO moves (game_id, move_number, move_notation, fen_after) VALUES (?, ?, ?, ?)"
        )
        .bind(game_id)
        .bind(move_number)
        .bind(notation)
        .bind(fen_after)
        .execute(&self.pool)
        .await?;

        // Update game's FEN and last move time
        sqlx::query(
            r#"UPDATE games SET
                fen = ?,
                last_move_time = datetime('now', '-5 hours'),
                white_draw_offer = 0,
                black_draw_offer = 0
            WHERE game_id = ?"#
        )
        .bind(fen_after)
        .bind(game_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update game status
    pub async fn update_game_status(&self, game_id: i64, status: &str, winner_id: Option<i64>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE games SET
                status = ?,
                winner_user_id = ?,
                completed_at = datetime('now', '-5 hours')
            WHERE game_id = ?"#
        )
        .bind(status)
        .bind(winner_id)
        .bind(game_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Set draw offer
    pub async fn set_draw_offer(&self, game_id: i64, by_white: bool) -> Result<(), sqlx::Error> {
        if by_white {
            sqlx::query("UPDATE games SET white_draw_offer = 1 WHERE game_id = ?")
                .bind(game_id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("UPDATE games SET black_draw_offer = 1 WHERE game_id = ?")
                .bind(game_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    /// Get open games (waiting for opponent)
    pub async fn get_open_games(&self, exclude_user_id: i64) -> Result<Vec<(i64, String, i32, String)>, sqlx::Error> {
        let rows: Vec<(i64, String, i32, String)> = sqlx::query_as(
            r#"SELECT game_id, white_handle, white_elo, matchmaking_type
            FROM games
            WHERE status = 'waiting'
                AND white_user_id != ?
                AND (challenge_target_id IS NULL OR challenge_target_id = ?)
            ORDER BY created_at DESC
            LIMIT 20"#
        )
        .bind(exclude_user_id)
        .bind(exclude_user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get active games for a user
    pub async fn get_active_games(&self, user_id: i64) -> Result<Vec<(i64, String, bool, String)>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i64, Option<i64>, String, String)> = sqlx::query_as(
            r#"SELECT
                game_id, white_handle, black_handle, white_user_id, black_user_id, fen, created_at
            FROM games
            WHERE status = 'in_progress'
                AND (white_user_id = ? OR black_user_id = ?)
            ORDER BY last_move_time DESC"#
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for (game_id, white_handle, black_handle, white_uid, black_uid, fen, created) in rows {
            // Determine opponent and whose turn
            let opponent = if white_uid == user_id {
                black_handle
            } else {
                white_handle
            };

            // Parse FEN to determine whose turn
            let is_white_turn = fen.split_whitespace().nth(1) == Some("w");
            let is_your_turn = (white_uid == user_id && is_white_turn) ||
                              (black_uid == Some(user_id) && !is_white_turn);

            result.push((game_id, opponent, is_your_turn, created));
        }

        Ok(result)
    }

    /// Get incoming challenges for a user
    pub async fn get_incoming_challenges(&self, user_id: i64) -> Result<Vec<(i64, String, i32)>, sqlx::Error> {
        let rows: Vec<(i64, String, i32)> = sqlx::query_as(
            r#"SELECT game_id, white_handle, white_elo
            FROM games
            WHERE status = 'waiting'
                AND challenge_target_id = ?"#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get outgoing challenges for a user
    pub async fn get_outgoing_challenges(&self, user_id: i64) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let rows: Vec<(i64, String)> = sqlx::query_as(
            r#"SELECT game_id, challenge_target_handle
            FROM games
            WHERE status = 'waiting'
                AND white_user_id = ?
                AND challenge_target_id IS NOT NULL"#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|(id, h)| Some((id, h))).collect())
    }

    /// Count active games for a user
    pub async fn count_active_games(&self, user_id: i64) -> Result<i32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM games
            WHERE (status = 'waiting' OR status = 'in_progress')
                AND (white_user_id = ? OR black_user_id = ?)"#
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    /// Get max concurrent games from config
    pub async fn get_max_concurrent(&self) -> Result<i32, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT value FROM config WHERE key = 'max_concurrent_games'"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|(v,)| v.parse().ok()).unwrap_or(DEFAULT_MAX_CONCURRENT))
    }

    /// Get leaderboard
    pub async fn get_leaderboard(&self, limit: i32) -> Result<Vec<(i64, String, i32, i32, i32)>, sqlx::Error> {
        let rows: Vec<(i64, String, i32, i32, i32)> = sqlx::query_as(
            r#"SELECT user_id, handle, elo, wins, losses
            FROM players
            WHERE games_played > 0
            ORDER BY elo DESC
            LIMIT ?"#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Check for timed out games (3 days without move) and forfeit them
    pub async fn check_timeouts(&self) -> Result<Vec<(i64, i64)>, sqlx::Error> {
        // Find games that have timed out
        let rows: Vec<(i64, i64, Option<i64>, String)> = sqlx::query_as(
            r#"SELECT game_id, white_user_id, black_user_id, fen
            FROM games
            WHERE status = 'in_progress'
                AND datetime(last_move_time, '+3 days') < datetime('now', '-5 hours')"#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut forfeits = Vec::new();

        for (game_id, white_uid, black_uid, fen) in rows {
            // Determine whose turn it was (they lose)
            let is_white_turn = fen.split_whitespace().nth(1) == Some("w");
            let winner_id = if is_white_turn {
                black_uid // White timed out, black wins
            } else {
                Some(white_uid) // Black timed out, white wins
            };

            if let Some(winner) = winner_id {
                self.update_game_status(game_id, "timeout", Some(winner)).await?;
                forfeits.push((game_id, winner));
            }
        }

        Ok(forfeits)
    }

    /// Delete a game (for canceling challenges before they're accepted)
    pub async fn delete_game(&self, game_id: i64, user_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM games WHERE game_id = ? AND white_user_id = ? AND status = 'waiting'"
        )
        .bind(game_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn setup_test_db() -> ChessDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_chess.db");
        ChessDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_player() {
        let db = setup_test_db().await;
        let player = db.get_or_create_player(1, "TestPlayer").await.unwrap();

        assert_eq!(player.user_id, 1);
        assert_eq!(player.handle, "TestPlayer");
        assert_eq!(player.elo, DEFAULT_ELO);
        assert_eq!(player.wins, 0);
    }

    #[tokio::test]
    async fn test_create_game() {
        let db = setup_test_db().await;
        let _ = db.get_or_create_player(1, "Player1").await.unwrap();

        let game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        let game_id = db.create_game(&game).await.unwrap();

        assert!(game_id > 0);

        let record = db.get_game(game_id).await.unwrap().unwrap();
        assert_eq!(record.white_user_id, 1);
        assert_eq!(record.status, "waiting");
    }

    #[tokio::test]
    async fn test_join_game() {
        let db = setup_test_db().await;
        let _ = db.get_or_create_player(1, "Player1").await.unwrap();
        let _ = db.get_or_create_player(2, "Player2").await.unwrap();

        let game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        let game_id = db.create_game(&game).await.unwrap();

        db.join_game(game_id, 2, "Player2", 1300).await.unwrap();

        let record = db.get_game(game_id).await.unwrap().unwrap();
        assert_eq!(record.black_user_id, Some(2));
        assert_eq!(record.status, "in_progress");
    }

    #[tokio::test]
    async fn test_record_move() {
        let db = setup_test_db().await;
        // Create players first (needed for foreign key constraints)
        let _ = db.get_or_create_player(1, "Player1").await.unwrap();
        let _ = db.get_or_create_player(2, "Player2").await.unwrap();

        let game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        let game_id = db.create_game(&game).await.unwrap();
        db.join_game(game_id, 2, "Player2", 1300).await.unwrap();

        db.record_move(game_id, 1, "e2e4", "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").await.unwrap();

        let moves = db.get_moves(game_id).await.unwrap();
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].1, "e2e4");
    }

    #[tokio::test]
    async fn test_leaderboard() {
        let db = setup_test_db().await;

        // Create some players with games
        let _ = db.get_or_create_player(1, "Player1").await.unwrap();
        let _ = db.get_or_create_player(2, "Player2").await.unwrap();

        db.update_player_rating(1, 1400, true, false, false).await.unwrap();
        db.update_player_rating(2, 1100, false, true, false).await.unwrap();

        let leaderboard = db.get_leaderboard(10).await.unwrap();
        assert_eq!(leaderboard.len(), 2);
        assert_eq!(leaderboard[0].2, 1400); // Highest ELO first
    }
}
