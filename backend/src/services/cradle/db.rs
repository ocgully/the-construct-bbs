//! Database operations for Cradle
//!
//! Stores game saves, completions, and leaderboard data in SQLite.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct CradleDb {
    pool: SqlitePool,
}

impl CradleDb {
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
        // Main saves table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Completions/prestige history
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                final_tier INTEGER NOT NULL,
                ascension_points INTEGER NOT NULL,
                total_ticks INTEGER NOT NULL,
                prestige_count INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Leaderboard index
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_tier
            ON completions(final_tier DESC, ascension_points DESC)
        "#).execute(&self.pool).await?;

        // Prestige upgrades tracking (separate from state for cross-session persistence)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS prestige_progress (
                user_id INTEGER PRIMARY KEY,
                total_ascension_points INTEGER NOT NULL DEFAULT 0,
                total_prestiges INTEGER NOT NULL DEFAULT 0,
                highest_tier_ever INTEGER NOT NULL DEFAULT 0,
                madra_multiplier REAL NOT NULL DEFAULT 1.0,
                insight_multiplier REAL NOT NULL DEFAULT 1.0,
                stone_multiplier REAL NOT NULL DEFAULT 1.0,
                unlock_speed_bonus REAL NOT NULL DEFAULT 1.0,
                starting_tier_bonus INTEGER NOT NULL DEFAULT 0,
                last_updated TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        Ok(())
    }

    /// Save current game state
    pub async fn save_game(&self, user_id: i64, handle: &str, state_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, state_json, last_saved)
             VALUES (?, ?, ?, datetime('now', '-5 hours'))"
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

    /// Check if user has a save
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

    /// Record a game completion (prestige/ascension)
    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        final_tier: u8,
        ascension_points: i64,
        total_ticks: i64,
        prestige_count: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, final_tier, ascension_points, total_ticks, prestige_count, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(final_tier as i64)
        .bind(ascension_points)
        .bind(total_ticks)
        .bind(prestige_count)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get leaderboard entries
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY final_tier DESC, ascension_points DESC) as rank,
                handle,
                final_tier,
                ascension_points,
                prestige_count,
                completed_at
            FROM completions
            ORDER BY final_tier DESC, ascension_points DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, tier, points, prestiges, completed)| {
            LeaderboardEntry {
                rank,
                handle,
                final_tier: tier as u8,
                ascension_points: points as u64,
                prestige_count: prestiges as u32,
                completed_at: completed,
            }
        }).collect())
    }

    /// Update prestige progress (persistent across saves)
    pub async fn update_prestige_progress(
        &self,
        user_id: i64,
        total_points: i64,
        total_prestiges: i64,
        highest_tier: i64,
        madra_mult: f64,
        insight_mult: f64,
        stone_mult: f64,
        speed_bonus: f64,
        starting_tier: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO prestige_progress
             (user_id, total_ascension_points, total_prestiges, highest_tier_ever,
              madra_multiplier, insight_multiplier, stone_multiplier,
              unlock_speed_bonus, starting_tier_bonus, last_updated)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(total_points)
        .bind(total_prestiges)
        .bind(highest_tier)
        .bind(madra_mult)
        .bind(insight_mult)
        .bind(stone_mult)
        .bind(speed_bonus)
        .bind(starting_tier)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load prestige progress
    #[allow(dead_code)]
    pub async fn load_prestige_progress(&self, user_id: i64) -> Result<Option<PrestigeProgress>, sqlx::Error> {
        let row: Option<(i64, i64, i64, f64, f64, f64, f64, i64)> = sqlx::query_as(
            "SELECT total_ascension_points, total_prestiges, highest_tier_ever,
                    madra_multiplier, insight_multiplier, stone_multiplier,
                    unlock_speed_bonus, starting_tier_bonus
             FROM prestige_progress WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(points, prestiges, tier, madra, insight, stone, speed, start)| {
            PrestigeProgress {
                total_ascension_points: points as u64,
                total_prestiges: prestiges as u32,
                highest_tier_ever: tier as u8,
                madra_multiplier: madra,
                insight_multiplier: insight,
                stone_multiplier: stone,
                unlock_speed_bonus: speed,
                starting_tier_bonus: start as u8,
            }
        }))
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub final_tier: u8,
    pub ascension_points: u64,
    pub prestige_count: u32,
    pub completed_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PrestigeProgress {
    pub total_ascension_points: u64,
    pub total_prestiges: u32,
    pub highest_tier_ever: u8,
    pub madra_multiplier: f64,
    pub insight_multiplier: f64,
    pub stone_multiplier: f64,
    pub unlock_speed_bonus: f64,
    pub starting_tier_bonus: u8,
}
