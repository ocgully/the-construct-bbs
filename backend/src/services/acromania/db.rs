//! Database layer for Acromania
//!
//! Stores game history, player statistics, and leaderboards.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct AcroDb {
    pool: SqlitePool,
}

#[allow(dead_code)]
impl AcroDb {
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
        // Games table - stores completed games
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
                ended_at TEXT,
                rounds_played INTEGER NOT NULL DEFAULT 0,
                player_count INTEGER NOT NULL DEFAULT 0,
                winner_user_id INTEGER,
                winner_handle TEXT,
                winner_score INTEGER
            )
        "#).execute(&self.pool).await?;

        // Game players - links players to games
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS game_players (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                final_score INTEGER NOT NULL DEFAULT 0,
                final_rank INTEGER NOT NULL DEFAULT 0,
                submissions_made INTEGER NOT NULL DEFAULT 0,
                votes_cast INTEGER NOT NULL DEFAULT 0,
                votes_received INTEGER NOT NULL DEFAULT 0,
                rounds_won INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (game_id) REFERENCES games(id)
            )
        "#).execute(&self.pool).await?;

        // Player statistics - aggregated stats across all games
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS player_stats (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                games_played INTEGER NOT NULL DEFAULT 0,
                games_won INTEGER NOT NULL DEFAULT 0,
                total_score INTEGER NOT NULL DEFAULT 0,
                highest_score INTEGER NOT NULL DEFAULT 0,
                total_submissions INTEGER NOT NULL DEFAULT 0,
                total_votes_received INTEGER NOT NULL DEFAULT 0,
                total_rounds_won INTEGER NOT NULL DEFAULT 0,
                last_played TEXT NOT NULL DEFAULT (datetime('now', '-5 hours'))
            )
        "#).execute(&self.pool).await?;

        // Rounds table - stores round history for analysis
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS rounds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                round_number INTEGER NOT NULL,
                acronym TEXT NOT NULL,
                category TEXT,
                winning_submission TEXT,
                winning_user_id INTEGER,
                FOREIGN KEY (game_id) REFERENCES games(id)
            )
        "#).execute(&self.pool).await?;

        // Submissions table - stores all submissions for analysis/moderation
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS submissions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                round_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                text TEXT NOT NULL,
                votes_received INTEGER NOT NULL DEFAULT 0,
                points_earned INTEGER NOT NULL DEFAULT 0,
                submission_time_ms INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (round_id) REFERENCES rounds(id)
            )
        "#).execute(&self.pool).await?;

        // Indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_game_players_user
            ON game_players(user_id)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_player_stats_score
            ON player_stats(highest_score DESC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    /// Record a completed game
    pub async fn record_game(
        &self,
        rounds_played: u32,
        player_count: u32,
        winner_user_id: i64,
        winner_handle: &str,
        winner_score: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO games (ended_at, rounds_played, player_count, winner_user_id, winner_handle, winner_score)
            VALUES (datetime('now', '-5 hours'), ?, ?, ?, ?, ?)
            "#
        )
        .bind(rounds_played as i64)
        .bind(player_count as i64)
        .bind(winner_user_id)
        .bind(winner_handle)
        .bind(winner_score)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Record a player's game participation
    pub async fn record_game_player(
        &self,
        game_id: i64,
        user_id: i64,
        handle: &str,
        final_score: i64,
        final_rank: u32,
        submissions_made: u32,
        votes_cast: u32,
        votes_received: u32,
        rounds_won: u32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO game_players
            (game_id, user_id, handle, final_score, final_rank, submissions_made, votes_cast, votes_received, rounds_won)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(game_id)
        .bind(user_id)
        .bind(handle)
        .bind(final_score)
        .bind(final_rank as i64)
        .bind(submissions_made as i64)
        .bind(votes_cast as i64)
        .bind(votes_received as i64)
        .bind(rounds_won as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update player statistics
    pub async fn update_player_stats(
        &self,
        user_id: i64,
        handle: &str,
        score: i64,
        won: bool,
        submissions: u32,
        votes_received: u32,
        rounds_won: u32,
    ) -> Result<(), sqlx::Error> {
        // Upsert player stats
        sqlx::query(
            r#"
            INSERT INTO player_stats
            (user_id, handle, games_played, games_won, total_score, highest_score, total_submissions, total_votes_received, total_rounds_won, last_played)
            VALUES (?, ?, 1, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))
            ON CONFLICT(user_id) DO UPDATE SET
                handle = excluded.handle,
                games_played = games_played + 1,
                games_won = games_won + ?,
                total_score = total_score + ?,
                highest_score = MAX(highest_score, ?),
                total_submissions = total_submissions + ?,
                total_votes_received = total_votes_received + ?,
                total_rounds_won = total_rounds_won + ?,
                last_played = datetime('now', '-5 hours')
            "#
        )
        .bind(user_id)
        .bind(handle)
        .bind(if won { 1i64 } else { 0i64 })
        .bind(score)
        .bind(score)
        .bind(submissions as i64)
        .bind(votes_received as i64)
        .bind(rounds_won as i64)
        // ON CONFLICT update bindings
        .bind(if won { 1i64 } else { 0i64 })
        .bind(score)
        .bind(score)
        .bind(submissions as i64)
        .bind(votes_received as i64)
        .bind(rounds_won as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get leaderboard by highest single-game score
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT handle, highest_score, games_played, games_won
            FROM player_stats
            ORDER BY highest_score DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(handle, score, games, wins)| {
            LeaderboardEntry {
                handle,
                highest_score: score,
                games_played: games as u32,
                games_won: wins as u32,
            }
        }).collect())
    }

    /// Get player's personal stats
    pub async fn get_player_stats(&self, user_id: i64) -> Result<Option<PlayerStats>, sqlx::Error> {
        let row: Option<(String, i64, i64, i64, i64, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT handle, games_played, games_won, total_score, highest_score,
                   total_submissions, total_votes_received, total_rounds_won
            FROM player_stats
            WHERE user_id = ?
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(handle, games, wins, total, highest, subs, votes, rounds)| {
            PlayerStats {
                handle,
                games_played: games as u32,
                games_won: wins as u32,
                total_score: total,
                highest_score: highest,
                total_submissions: subs as u32,
                total_votes_received: votes as u32,
                total_rounds_won: rounds as u32,
            }
        }))
    }

    /// Record a round (for analysis)
    pub async fn record_round(
        &self,
        game_id: i64,
        round_number: u32,
        acronym: &str,
        category: Option<&str>,
        winning_submission: Option<&str>,
        winning_user_id: Option<i64>,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO rounds (game_id, round_number, acronym, category, winning_submission, winning_user_id)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(game_id)
        .bind(round_number as i64)
        .bind(acronym)
        .bind(category)
        .bind(winning_submission)
        .bind(winning_user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get recent games (for display)
    pub async fn get_recent_games(&self, limit: i64) -> Result<Vec<RecentGame>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT id, ended_at, player_count, winner_handle, winner_score
            FROM games
            WHERE ended_at IS NOT NULL
            ORDER BY ended_at DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, ended_at, players, winner, score)| {
            RecentGame {
                game_id: id,
                ended_at,
                player_count: players as u32,
                winner_handle: winner,
                winner_score: score,
            }
        }).collect())
    }
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub handle: String,
    pub highest_score: i64,
    pub games_played: u32,
    pub games_won: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlayerStats {
    pub handle: String,
    pub games_played: u32,
    pub games_won: u32,
    pub total_score: i64,
    pub highest_score: i64,
    pub total_submissions: u32,
    pub total_votes_received: u32,
    pub total_rounds_won: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecentGame {
    pub game_id: i64,
    pub ended_at: String,
    pub player_count: u32,
    pub winner_handle: String,
    pub winner_score: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> AcroDb {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_acro.db");
        AcroDb::new(&path).await.unwrap()
    }

    #[tokio::test]
    async fn test_db_creation() {
        let _db = create_test_db().await;
    }

    #[tokio::test]
    async fn test_record_and_get_stats() {
        let db = create_test_db().await;

        // Record game
        let game_id = db.record_game(10, 4, 1, "Winner", 1500).await.unwrap();
        assert!(game_id > 0);

        // Record player
        db.record_game_player(game_id, 1, "Winner", 1500, 1, 10, 9, 15, 4).await.unwrap();

        // Update stats
        db.update_player_stats(1, "Winner", 1500, true, 10, 15, 4).await.unwrap();

        // Get stats
        let stats = db.get_player_stats(1).await.unwrap();
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.games_played, 1);
        assert_eq!(stats.games_won, 1);
        assert_eq!(stats.highest_score, 1500);
    }

    #[tokio::test]
    async fn test_leaderboard() {
        let db = create_test_db().await;

        // Add some players
        db.update_player_stats(1, "Alice", 1000, true, 5, 10, 2).await.unwrap();
        db.update_player_stats(2, "Bob", 1500, true, 5, 12, 3).await.unwrap();
        db.update_player_stats(3, "Carol", 800, false, 5, 8, 1).await.unwrap();

        let leaderboard = db.get_leaderboard(10).await.unwrap();
        assert_eq!(leaderboard.len(), 3);
        assert_eq!(leaderboard[0].handle, "Bob"); // Highest score first
        assert_eq!(leaderboard[1].handle, "Alice");
        assert_eq!(leaderboard[2].handle, "Carol");
    }
}
