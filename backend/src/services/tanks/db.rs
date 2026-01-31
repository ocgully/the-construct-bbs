//! Database layer for Tanks
//!
//! Stores game history, player statistics, and leaderboards.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct TanksDb {
    pool: SqlitePool,
}

impl TanksDb {
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
                player_count INTEGER NOT NULL DEFAULT 0,
                rounds_played INTEGER NOT NULL DEFAULT 0,
                winner_user_id INTEGER,
                winner_handle TEXT,
                terrain_seed INTEGER
            )
        "#).execute(&self.pool).await?;

        // Game players - links players to games
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS game_players (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                final_rank INTEGER NOT NULL DEFAULT 0,
                kills INTEGER NOT NULL DEFAULT 0,
                damage_dealt INTEGER NOT NULL DEFAULT 0,
                shots_fired INTEGER NOT NULL DEFAULT 0,
                shots_hit INTEGER NOT NULL DEFAULT 0,
                survived BOOLEAN NOT NULL DEFAULT 0,
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
                total_kills INTEGER NOT NULL DEFAULT 0,
                total_damage INTEGER NOT NULL DEFAULT 0,
                total_shots INTEGER NOT NULL DEFAULT 0,
                total_hits INTEGER NOT NULL DEFAULT 0,
                last_played TEXT NOT NULL DEFAULT (datetime('now', '-5 hours'))
            )
        "#).execute(&self.pool).await?;

        // Indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_game_players_user
            ON game_players(user_id)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_player_stats_wins
            ON player_stats(games_won DESC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    /// Record a completed game
    pub async fn record_game(
        &self,
        player_count: u32,
        rounds_played: u32,
        winner_user_id: i64,
        winner_handle: &str,
        terrain_seed: u64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO games (ended_at, player_count, rounds_played, winner_user_id, winner_handle, terrain_seed)
            VALUES (datetime('now', '-5 hours'), ?, ?, ?, ?, ?)
            "#
        )
        .bind(player_count as i64)
        .bind(rounds_played as i64)
        .bind(winner_user_id)
        .bind(winner_handle)
        .bind(terrain_seed as i64)
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
        final_rank: u32,
        kills: u32,
        damage_dealt: u32,
        shots_fired: u32,
        shots_hit: u32,
        survived: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO game_players
            (game_id, user_id, handle, final_rank, kills, damage_dealt, shots_fired, shots_hit, survived)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(game_id)
        .bind(user_id)
        .bind(handle)
        .bind(final_rank as i64)
        .bind(kills as i64)
        .bind(damage_dealt as i64)
        .bind(shots_fired as i64)
        .bind(shots_hit as i64)
        .bind(survived)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update player statistics
    pub async fn update_player_stats(
        &self,
        user_id: i64,
        handle: &str,
        won: bool,
        kills: u32,
        damage: u32,
        shots: u32,
        hits: u32,
    ) -> Result<(), sqlx::Error> {
        // Upsert player stats
        sqlx::query(
            r#"
            INSERT INTO player_stats
            (user_id, handle, games_played, games_won, total_kills, total_damage, total_shots, total_hits, last_played)
            VALUES (?, ?, 1, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))
            ON CONFLICT(user_id) DO UPDATE SET
                handle = excluded.handle,
                games_played = games_played + 1,
                games_won = games_won + ?,
                total_kills = total_kills + ?,
                total_damage = total_damage + ?,
                total_shots = total_shots + ?,
                total_hits = total_hits + ?,
                last_played = datetime('now', '-5 hours')
            "#
        )
        .bind(user_id)
        .bind(handle)
        .bind(if won { 1i64 } else { 0i64 })
        .bind(kills as i64)
        .bind(damage as i64)
        .bind(shots as i64)
        .bind(hits as i64)
        // ON CONFLICT update bindings
        .bind(if won { 1i64 } else { 0i64 })
        .bind(kills as i64)
        .bind(damage as i64)
        .bind(shots as i64)
        .bind(hits as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get leaderboard by wins
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(String, i64, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT handle, games_won, total_kills, total_shots, total_hits
            FROM player_stats
            ORDER BY games_won DESC, total_kills DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(handle, wins, kills, shots, hits)| {
            let accuracy = if shots > 0 { ((hits * 100) / shots) as u32 } else { 0 };
            LeaderboardEntry {
                handle,
                wins: wins as u32,
                kills: kills as u32,
                accuracy,
            }
        }).collect())
    }

    /// Get player's personal stats
    pub async fn get_player_stats(&self, user_id: i64) -> Result<Option<PlayerStats>, sqlx::Error> {
        let row: Option<(String, i64, i64, i64, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT handle, games_played, games_won, total_kills, total_damage, total_shots, total_hits
            FROM player_stats
            WHERE user_id = ?
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(handle, games, wins, kills, damage, shots, hits)| {
            let accuracy = if shots > 0 { ((hits * 100) / shots) as u32 } else { 0 };
            PlayerStats {
                handle,
                games_played: games as u32,
                games_won: wins as u32,
                total_kills: kills as u32,
                total_damage: damage as u32,
                accuracy,
            }
        }))
    }

    /// Get recent games (for display)
    pub async fn get_recent_games(&self, limit: i64) -> Result<Vec<RecentGame>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT id, ended_at, player_count, winner_handle, rounds_played
            FROM games
            WHERE ended_at IS NOT NULL
            ORDER BY ended_at DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, ended_at, players, winner, rounds)| {
            RecentGame {
                game_id: id,
                ended_at,
                player_count: players as u32,
                winner_handle: winner,
                rounds_played: rounds as u32,
            }
        }).collect())
    }
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub handle: String,
    pub wins: u32,
    pub kills: u32,
    pub accuracy: u32,
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub handle: String,
    pub games_played: u32,
    pub games_won: u32,
    pub total_kills: u32,
    pub total_damage: u32,
    pub accuracy: u32,
}

#[derive(Debug, Clone)]
pub struct RecentGame {
    pub game_id: i64,
    pub ended_at: String,
    pub player_count: u32,
    pub winner_handle: String,
    pub rounds_played: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> TanksDb {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_tanks.db");
        TanksDb::new(&path).await.unwrap()
    }

    #[tokio::test]
    async fn test_db_creation() {
        let _db = create_test_db().await;
    }

    #[tokio::test]
    async fn test_record_and_get_stats() {
        let db = create_test_db().await;

        // Record game
        let game_id = db.record_game(4, 8, 1, "Winner", 12345).await.unwrap();
        assert!(game_id > 0);

        // Record player
        db.record_game_player(game_id, 1, "Winner", 1, 3, 150, 10, 6, true).await.unwrap();

        // Update stats
        db.update_player_stats(1, "Winner", true, 3, 150, 10, 6).await.unwrap();

        // Get stats
        let stats = db.get_player_stats(1).await.unwrap();
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.games_played, 1);
        assert_eq!(stats.games_won, 1);
        assert_eq!(stats.total_kills, 3);
        assert_eq!(stats.accuracy, 60); // 6/10 = 60%
    }

    #[tokio::test]
    async fn test_leaderboard() {
        let db = create_test_db().await;

        // Add some players
        db.update_player_stats(1, "Alice", true, 5, 100, 10, 8).await.unwrap();
        db.update_player_stats(2, "Bob", true, 3, 80, 10, 5).await.unwrap();
        db.update_player_stats(2, "Bob", true, 4, 90, 10, 6).await.unwrap(); // Bob played twice
        db.update_player_stats(3, "Carol", false, 1, 50, 10, 3).await.unwrap();

        let leaderboard = db.get_leaderboard(10).await.unwrap();
        assert_eq!(leaderboard.len(), 3);
        assert_eq!(leaderboard[0].handle, "Bob"); // 2 wins
        assert_eq!(leaderboard[0].wins, 2);
        assert_eq!(leaderboard[1].handle, "Alice"); // 1 win, more kills
    }

    #[tokio::test]
    async fn test_recent_games() {
        let db = create_test_db().await;

        db.record_game(4, 8, 1, "Winner1", 111).await.unwrap();
        db.record_game(3, 5, 2, "Winner2", 222).await.unwrap();

        let games = db.get_recent_games(10).await.unwrap();
        assert_eq!(games.len(), 2);
        // Both games recorded in same second, so order depends on ID
        // Verify both games are present
        let handles: Vec<&str> = games.iter().map(|g| g.winner_handle.as_str()).collect();
        assert!(handles.contains(&"Winner1"));
        assert!(handles.contains(&"Winner2"));
    }
}
