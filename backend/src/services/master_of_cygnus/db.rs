//! Database layer for Master of Cygnus

#![allow(dead_code)]

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use chrono::{DateTime, Utc};

pub struct MocDb {
    pool: SqlitePool,
}

impl MocDb {
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
        // Games table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                state_json TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'waiting',
                turn_number INTEGER NOT NULL DEFAULT 0,
                turn_deadline TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Player game associations
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS player_games (
                user_id INTEGER NOT NULL,
                game_id INTEGER NOT NULL,
                empire_id INTEGER NOT NULL,
                is_ai INTEGER NOT NULL DEFAULT 0,
                forfeited INTEGER NOT NULL DEFAULT 0,
                timeout_count INTEGER NOT NULL DEFAULT 0,
                last_active TEXT NOT NULL,
                PRIMARY KEY (user_id, game_id)
            )
        "#).execute(&self.pool).await?;

        // Indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_games_status
            ON games(status)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_player_games_user
            ON player_games(user_id)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    /// Save a game
    pub async fn save_game(&self, game_id: i64, name: &str, state_json: &str, status: &str, turn_number: i64, turn_deadline: Option<DateTime<Utc>>) -> Result<(), sqlx::Error> {
        let deadline_str = turn_deadline.map(|d| d.to_rfc3339());
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO games (id, name, state_json, status, turn_number, turn_deadline, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                state_json = excluded.state_json,
                status = excluded.status,
                turn_number = excluded.turn_number,
                turn_deadline = excluded.turn_deadline,
                updated_at = excluded.updated_at
            "#
        )
        .bind(game_id)
        .bind(name)
        .bind(state_json)
        .bind(status)
        .bind(turn_number)
        .bind(deadline_str)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load a game by ID
    pub async fn load_game(&self, game_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM games WHERE id = ?"
        )
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
    }

    /// Get games a user is participating in
    pub async fn get_user_games(&self, user_id: i64) -> Result<Vec<GameSummary>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i64)> = sqlx::query_as(
            r#"
            SELECT g.id, g.name, g.status, g.turn_number
            FROM games g
            JOIN player_games pg ON pg.game_id = g.id
            WHERE pg.user_id = ? AND g.status != 'completed'
            ORDER BY g.updated_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, name, status, turn)| GameSummary {
            id,
            name,
            status,
            turn_number: turn,
        }).collect())
    }

    /// Get available games to join
    pub async fn get_open_games(&self) -> Result<Vec<OpenGame>, sqlx::Error> {
        let rows: Vec<(i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT g.id, g.name,
                   (SELECT COUNT(*) FROM player_games WHERE game_id = g.id) as player_count
            FROM games g
            WHERE g.status = 'waiting'
            ORDER BY g.created_at DESC
            LIMIT 20
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, name, players)| OpenGame {
            id,
            name,
            player_count: players as u32,
        }).collect())
    }

    /// Register a player in a game
    pub async fn join_game(&self, user_id: i64, game_id: i64, empire_id: u32) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO player_games
            (user_id, game_id, empire_id, is_ai, forfeited, timeout_count, last_active)
            VALUES (?, ?, ?, 0, 0, 0, ?)
            "#
        )
        .bind(user_id)
        .bind(game_id)
        .bind(empire_id as i64)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update player activity timestamp
    pub async fn update_activity(&self, user_id: i64, game_id: i64) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE player_games SET last_active = ? WHERE user_id = ? AND game_id = ?"
        )
        .bind(now)
        .bind(user_id)
        .bind(game_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark a player as forfeited
    pub async fn forfeit_player(&self, user_id: i64, game_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE player_games SET forfeited = 1, is_ai = 1 WHERE user_id = ? AND game_id = ?"
        )
        .bind(user_id)
        .bind(game_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment timeout count for a player
    pub async fn increment_timeout(&self, user_id: i64, game_id: i64) -> Result<u32, sqlx::Error> {
        sqlx::query(
            "UPDATE player_games SET timeout_count = timeout_count + 1 WHERE user_id = ? AND game_id = ?"
        )
        .bind(user_id)
        .bind(game_id)
        .execute(&self.pool)
        .await?;

        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT timeout_count FROM player_games WHERE user_id = ? AND game_id = ?"
        )
        .bind(user_id)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0 as u32).unwrap_or(0))
    }

    /// Delete a game
    pub async fn delete_game(&self, game_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM player_games WHERE game_id = ?")
            .bind(game_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM games WHERE id = ?")
            .bind(game_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get games that need turn processing (past deadline)
    pub async fn get_games_past_deadline(&self) -> Result<Vec<i64>, sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        let rows: Vec<(i64,)> = sqlx::query_as(
            r#"
            SELECT id FROM games
            WHERE status = 'in_progress' AND turn_deadline < ?
            "#
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}

#[derive(Debug, Clone)]
pub struct GameSummary {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub turn_number: i64,
}

#[derive(Debug, Clone)]
pub struct OpenGame {
    pub id: i64,
    pub name: String,
    pub player_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_db_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_moc.db");
        let db = MocDb::new(&db_path).await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_save_load_game() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_moc.db");
        let db = MocDb::new(&db_path).await.unwrap();

        let game_json = r#"{"test": "data"}"#;
        db.save_game(1, "Test Game", game_json, "waiting", 0, None).await.unwrap();

        let loaded = db.load_game(1).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), game_json);
    }

    #[tokio::test]
    async fn test_player_game_association() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_moc.db");
        let db = MocDb::new(&db_path).await.unwrap();

        db.save_game(1, "Test Game", "{}", "waiting", 0, None).await.unwrap();
        db.join_game(100, 1, 0).await.unwrap();

        let games = db.get_user_games(100).await.unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Test Game");
    }
}
