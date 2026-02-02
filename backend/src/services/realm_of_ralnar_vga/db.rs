//! Database operations for Realm of Ralnar VGA

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::Path;

/// Database for Realm of Ralnar VGA save data
pub struct RalnarVgaDb {
    pool: SqlitePool,
}

impl RalnarVgaDb {
    /// Create or connect to database
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, sqlx::Error> {
        let db_url = format!("sqlite:{}?mode=rwc", path.as_ref().display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                state_json TEXT NOT NULL,
                saved_at INTEGER NOT NULL DEFAULT (unixepoch())
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                ending_type TEXT NOT NULL,
                play_time INTEGER NOT NULL,
                completed_at INTEGER NOT NULL DEFAULT (unixepoch())
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Save game state
    pub async fn save_game(
        &self,
        user_id: i64,
        handle: &str,
        state_json: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO saves (user_id, handle, state_json, saved_at)
            VALUES (?, ?, ?, unixepoch())
            "#,
        )
        .bind(user_id)
        .bind(handle)
        .bind(state_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load game state
    pub async fn load_game(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM saves WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(json,)| json))
    }

    /// Delete save
    pub async fn delete_save(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM saves WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Record game completion
    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        ending_type: &str,
        play_time: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO completions (user_id, handle, ending_type, play_time)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(handle)
        .bind(ending_type)
        .bind(play_time)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
