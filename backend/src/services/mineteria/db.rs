//! Database layer for Mineteria
//!
//! Handles persistent storage for worlds, player saves, and leaderboards.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct MineteriaDb {
    pool: SqlitePool,
}

impl MineteriaDb {
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
        // Player saves table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                world_seed INTEGER NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Worlds table (for persistent world data)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS worlds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                seed INTEGER NOT NULL,
                name TEXT,
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Modified chunks table (stores player modifications)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS chunks (
                world_id INTEGER NOT NULL,
                chunk_x INTEGER NOT NULL,
                chunk_y INTEGER NOT NULL,
                data BLOB NOT NULL,
                modified_at TEXT NOT NULL,
                PRIMARY KEY (world_id, chunk_x, chunk_y)
            )
        "#).execute(&self.pool).await?;

        // Completions/Leaderboard
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                score INTEGER NOT NULL,
                days_survived INTEGER NOT NULL,
                level_reached INTEGER NOT NULL,
                blocks_mined INTEGER NOT NULL,
                monsters_killed INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_score
            ON completions(score DESC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    /// Save game state
    pub async fn save_game(
        &self,
        user_id: i64,
        handle: &str,
        world_seed: u64,
        state_json: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, world_seed, state_json, last_saved)
             VALUES (?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(world_seed as i64)
        .bind(state_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load game state
    pub async fn load_game(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Delete save
    pub async fn delete_save(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM saves WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Check if save exists
    #[allow(dead_code)]
    pub async fn has_save(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    /// Record game completion
    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        score: i64,
        days_survived: i64,
        level_reached: i64,
        blocks_mined: i64,
        monsters_killed: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, score, days_survived, level_reached,
             blocks_mined, monsters_killed, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(score)
        .bind(days_survived)
        .bind(level_reached)
        .bind(blocks_mined)
        .bind(monsters_killed)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get leaderboard
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64, i64, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY score DESC) as rank,
                handle,
                score,
                days_survived,
                level_reached,
                blocks_mined,
                monsters_killed,
                completed_at
            FROM completions
            ORDER BY score DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, score, days, level, blocks, monsters, completed)| {
            LeaderboardEntry {
                rank,
                handle,
                score,
                days_survived: days,
                level_reached: level,
                blocks_mined: blocks,
                monsters_killed: monsters,
                completed_at: completed,
            }
        }).collect())
    }

    /// Create or get world for user
    pub async fn get_or_create_world(&self, user_id: i64, seed: u64) -> Result<i64, sqlx::Error> {
        // Check if world exists
        let existing: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM worlds WHERE user_id = ? AND seed = ?"
        )
        .bind(user_id)
        .bind(seed as i64)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id,)) = existing {
            return Ok(id);
        }

        // Create new world
        let result = sqlx::query(
            "INSERT INTO worlds (user_id, seed, created_at) VALUES (?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(seed as i64)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Save chunk data
    #[allow(dead_code)]
    pub async fn save_chunk(
        &self,
        world_id: i64,
        chunk_x: i32,
        chunk_y: i32,
        data: &[u8],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO chunks (world_id, chunk_x, chunk_y, data, modified_at)
             VALUES (?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(world_id)
        .bind(chunk_x)
        .bind(chunk_y)
        .bind(data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load chunk data
    #[allow(dead_code)]
    pub async fn load_chunk(
        &self,
        world_id: i64,
        chunk_x: i32,
        chunk_y: i32,
    ) -> Result<Option<Vec<u8>>, sqlx::Error> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM chunks WHERE world_id = ? AND chunk_x = ? AND chunk_y = ?"
        )
        .bind(world_id)
        .bind(chunk_x)
        .bind(chunk_y)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub score: i64,
    pub days_survived: i64,
    pub level_reached: i64,
    pub blocks_mined: i64,
    pub monsters_killed: i64,
    pub completed_at: String,
}
