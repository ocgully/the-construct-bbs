//! Database for Fortress game
//!
//! Stores fortress saves and completions.

#![allow(dead_code)]

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct FortressDb {
    pool: SqlitePool,
}

impl FortressDb {
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

        // Completed games for leaderboard
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                fortress_name TEXT NOT NULL,
                final_wealth INTEGER NOT NULL,
                years_survived INTEGER NOT NULL,
                peak_population INTEGER NOT NULL,
                invasions_repelled INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_wealth
            ON completions(final_wealth DESC)
        "#).execute(&self.pool).await?;

        // Fortress metadata for multiplayer world (future)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS fortresses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                world_x INTEGER NOT NULL,
                world_y INTEGER NOT NULL,
                wealth INTEGER NOT NULL DEFAULT 0,
                population INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                last_active TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Dwarves table for detailed tracking (optional)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS dwarves (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                fortress_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                profession TEXT NOT NULL,
                skills_json TEXT NOT NULL,
                status TEXT NOT NULL,
                born_at TEXT NOT NULL,
                died_at TEXT,
                FOREIGN KEY (fortress_id) REFERENCES fortresses(id)
            )
        "#).execute(&self.pool).await?;

        Ok(())
    }

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

    pub async fn load_game(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn delete_save(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM saves WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

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

    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        fortress_name: &str,
        final_wealth: i64,
        years_survived: i64,
        peak_population: i64,
        invasions_repelled: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, fortress_name, final_wealth, years_survived, peak_population, invasions_repelled, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(fortress_name)
        .bind(final_wealth)
        .bind(years_survived)
        .bind(peak_population)
        .bind(invasions_repelled)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i64, i64, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY final_wealth DESC) as rank,
                handle,
                fortress_name,
                final_wealth,
                years_survived,
                peak_population,
                invasions_repelled,
                completed_at
            FROM completions
            ORDER BY final_wealth DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, fortress_name, wealth, years, pop, invasions, completed)| {
            LeaderboardEntry {
                rank,
                handle,
                fortress_name,
                final_wealth: wealth,
                years_survived: years,
                peak_population: pop,
                invasions_repelled: invasions,
                completed_at: completed,
            }
        }).collect())
    }

    /// Get active fortresses for multiplayer world display
    #[allow(dead_code)]
    pub async fn get_active_fortresses(&self, limit: i64) -> Result<Vec<FortressInfo>, sqlx::Error> {
        let rows: Vec<(i64, i64, String, i64, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT id, user_id, name, world_x, world_y, wealth, population
            FROM fortresses
            WHERE datetime(last_active) > datetime('now', '-7 days')
            ORDER BY wealth DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, user_id, name, x, y, wealth, pop)| {
            FortressInfo {
                id,
                user_id,
                name,
                world_x: x,
                world_y: y,
                wealth,
                population: pop,
            }
        }).collect())
    }

    /// Update fortress world position and stats
    #[allow(dead_code)]
    pub async fn update_fortress_stats(&self, fortress_id: i64, wealth: i64, population: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE fortresses SET wealth = ?, population = ?, last_active = datetime('now', '-5 hours') WHERE id = ?"
        )
        .bind(wealth)
        .bind(population)
        .bind(fortress_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub fortress_name: String,
    pub final_wealth: i64,
    pub years_survived: i64,
    pub peak_population: i64,
    pub invasions_repelled: i64,
    pub completed_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FortressInfo {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub world_x: i64,
    pub world_y: i64,
    pub wealth: i64,
    pub population: i64,
}
