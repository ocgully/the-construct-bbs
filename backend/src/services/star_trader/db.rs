//! Star Trader - Database Layer
//!
//! SQLite database for galaxy persistence, player saves, and corporations.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct StarTraderDb {
    pool: SqlitePool,
}

impl StarTraderDb {
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
        // Galaxy table - stores the generated galaxy
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS galaxies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                seed INTEGER NOT NULL,
                size INTEGER NOT NULL,
                galaxy_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_tick TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // Active player saves
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                galaxy_id INTEGER NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (galaxy_id) REFERENCES galaxies(id)
            )
        "#).execute(&self.pool).await?;

        // Corporations
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS corporations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                tag TEXT NOT NULL UNIQUE,
                ceo_id INTEGER NOT NULL,
                corp_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // Leaderboard / completions
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS leaderboard (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                credits INTEGER NOT NULL,
                experience INTEGER NOT NULL,
                kills INTEGER NOT NULL,
                sectors_explored INTEGER NOT NULL,
                rank TEXT NOT NULL,
                recorded_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // Index for leaderboard
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_leaderboard_credits
            ON leaderboard(credits DESC)
        "#).execute(&self.pool).await?;

        // Index for corporations
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_corps_name
            ON corporations(name)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========== Galaxy Operations ==========

    /// Create or get the main galaxy
    pub async fn get_or_create_galaxy(&self, seed: u64, size: u32) -> Result<(i64, String), sqlx::Error> {
        // Try to get existing galaxy
        let existing: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, galaxy_json FROM galaxies ORDER BY id LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, json)) = existing {
            return Ok((id, json));
        }

        // Create new galaxy
        use crate::games::star_trader::galaxy::Galaxy;
        let galaxy = Galaxy::generate(seed, size);
        let galaxy_json = serde_json::to_string(&galaxy)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let result = sqlx::query(
            "INSERT INTO galaxies (seed, size, galaxy_json) VALUES (?, ?, ?)"
        )
        .bind(seed as i64)
        .bind(size as i64)
        .bind(&galaxy_json)
        .execute(&self.pool)
        .await?;

        Ok((result.last_insert_rowid(), galaxy_json))
    }

    /// Update galaxy state
    pub async fn save_galaxy(&self, galaxy_id: i64, galaxy_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE galaxies SET galaxy_json = ?, last_tick = datetime('now') WHERE id = ?"
        )
        .bind(galaxy_json)
        .bind(galaxy_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ========== Player Save Operations ==========

    pub async fn save_game(&self, user_id: i64, handle: &str, galaxy_id: i64, state_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, galaxy_id, state_json, last_saved)
             VALUES (?, ?, ?, ?, datetime('now'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(galaxy_id)
        .bind(state_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_game(&self, user_id: i64) -> Result<Option<(i64, String)>, sqlx::Error> {
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT galaxy_id, state_json FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
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

    // ========== Corporation Operations ==========

    pub async fn create_corporation(&self, name: &str, tag: &str, ceo_id: i64, corp_json: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO corporations (name, tag, ceo_id, corp_json) VALUES (?, ?, ?, ?)"
        )
        .bind(name)
        .bind(tag)
        .bind(ceo_id)
        .bind(corp_json)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn load_corporation(&self, corp_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT corp_json FROM corporations WHERE id = ?"
        )
        .bind(corp_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn save_corporation(&self, corp_id: i64, corp_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE corporations SET corp_json = ? WHERE id = ?"
        )
        .bind(corp_json)
        .bind(corp_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_corporation_by_name(&self, name: &str) -> Result<Option<(i64, String)>, sqlx::Error> {
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, corp_json FROM corporations WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_corporations(&self, limit: i64) -> Result<Vec<(i64, String, String)>, sqlx::Error> {
        let rows: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT id, name, tag FROM corporations ORDER BY name LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // ========== Leaderboard Operations ==========

    pub async fn record_score(
        &self,
        user_id: i64,
        handle: &str,
        credits: i64,
        experience: i64,
        kills: u32,
        sectors_explored: u32,
        rank: &str,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO leaderboard (user_id, handle, credits, experience, kills, sectors_explored, rank)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(handle)
        .bind(credits)
        .bind(experience)
        .bind(kills as i64)
        .bind(sectors_explored as i64)
        .bind(rank)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64, i64, i64, String, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY credits DESC) as rank,
                handle,
                credits,
                experience,
                kills,
                sectors_explored,
                rank as fed_rank,
                recorded_at
            FROM leaderboard
            ORDER BY credits DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, credits, experience, kills, sectors, fed_rank, recorded)| {
            LeaderboardEntry {
                rank,
                handle,
                credits,
                experience,
                kills: kills as u32,
                sectors_explored: sectors as u32,
                federation_rank: fed_rank,
                recorded_at: recorded,
            }
        }).collect())
    }

    // ========== Active Players ==========

    pub async fn get_active_players(&self) -> Result<Vec<(i64, String, String)>, sqlx::Error> {
        // Get players who have been active in the last 24 hours
        let rows: Vec<(i64, String, String)> = sqlx::query_as(
            r#"
            SELECT user_id, handle, last_saved
            FROM saves
            WHERE datetime(last_saved) > datetime('now', '-1 day')
            ORDER BY last_saved DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub credits: i64,
    pub experience: i64,
    pub kills: u32,
    pub sectors_explored: u32,
    pub federation_rank: String,
    pub recorded_at: String,
}
