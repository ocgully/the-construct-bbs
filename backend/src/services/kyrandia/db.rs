//! Database layer for Morningmist
//! Handles saves, leaderboards, world state, and multiplayer data

#![allow(dead_code)]

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct KyrandiaDb {
    pool: SqlitePool,
}

impl KyrandiaDb {
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
        // Player saves
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                became_archmage INTEGER NOT NULL DEFAULT 0
            )
        "#).execute(&self.pool).await?;

        // Game completions / leaderboard
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                level INTEGER NOT NULL,
                became_archmage INTEGER NOT NULL,
                monsters_killed INTEGER NOT NULL,
                gold_earned INTEGER NOT NULL,
                spells_learned INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // World state (for multiplayer features)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS world_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Player positions (for multiplayer visibility)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS player_positions (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                room_key TEXT NOT NULL,
                last_active TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Player inventory tracking (for trading)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS player_inventory (
                user_id INTEGER NOT NULL,
                item_key TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                PRIMARY KEY (user_id, item_key)
            )
        "#).execute(&self.pool).await?;

        // Romance relationships (for player-player romance)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS romances (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id_1 INTEGER NOT NULL,
                user_id_2 INTEGER NOT NULL,
                affection_1 INTEGER NOT NULL DEFAULT 0,
                affection_2 INTEGER NOT NULL DEFAULT 0,
                stage TEXT NOT NULL DEFAULT 'stranger',
                married INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(user_id_1, user_id_2)
            )
        "#).execute(&self.pool).await?;

        // Messages between players
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_user_id INTEGER NOT NULL,
                to_user_id INTEGER NOT NULL,
                message TEXT NOT NULL,
                read INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Trade requests
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS trade_requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_user_id INTEGER NOT NULL,
                to_user_id INTEGER NOT NULL,
                offer_items TEXT NOT NULL,
                request_items TEXT NOT NULL,
                gold_offer INTEGER NOT NULL DEFAULT 0,
                gold_request INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Create indices
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_level
            ON completions(level DESC, became_archmage DESC)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_player_positions_room
            ON player_positions(room_key)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // SAVE MANAGEMENT
    // ========================================================================

    pub async fn save_game(&self, user_id: i64, handle: &str, state_json: &str, level: u8, became_archmage: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, state_json, last_saved, level, became_archmage)
             VALUES (?, ?, ?, datetime('now', '-5 hours'), ?, ?)"
        )
        .bind(user_id)
        .bind(handle)
        .bind(state_json)
        .bind(level as i32)
        .bind(became_archmage as i32)
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

    // ========================================================================
    // LEADERBOARD
    // ========================================================================

    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        level: u8,
        became_archmage: bool,
        monsters_killed: u32,
        gold_earned: i64,
        spells_learned: usize,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, level, became_archmage, monsters_killed, gold_earned, spells_learned, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(level as i32)
        .bind(became_archmage as i32)
        .bind(monsters_killed as i32)
        .bind(gold_earned)
        .bind(spells_learned as i32)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i32, i32, i32, i64, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY became_archmage DESC, level DESC, monsters_killed DESC) as rank,
                handle,
                level,
                became_archmage,
                monsters_killed,
                gold_earned,
                completed_at
            FROM completions
            ORDER BY became_archmage DESC, level DESC, monsters_killed DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, level, became, monsters, gold, completed)| {
            LeaderboardEntry {
                rank,
                handle,
                level: level as u8,
                became_archmage: became != 0,
                monsters_killed: monsters as u32,
                gold_earned: gold,
                completed_at: completed,
            }
        }).collect())
    }

    // ========================================================================
    // MULTIPLAYER FEATURES
    // ========================================================================

    /// Update player position (for showing other players in room)
    pub async fn update_player_position(&self, user_id: i64, handle: &str, room_key: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO player_positions (user_id, handle, room_key, last_active)
             VALUES (?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(room_key)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get players in a room (active in last 5 minutes)
    pub async fn get_players_in_room(&self, room_key: &str, exclude_user_id: i64) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let rows: Vec<(i64, String)> = sqlx::query_as(
            r#"
            SELECT user_id, handle
            FROM player_positions
            WHERE room_key = ?
            AND user_id != ?
            AND last_active > datetime('now', '-5 hours', '-5 minutes')
            "#
        )
        .bind(room_key)
        .bind(exclude_user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Send a message to another player
    pub async fn send_message(&self, from_user_id: i64, to_user_id: i64, message: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO messages (from_user_id, to_user_id, message, created_at)
             VALUES (?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(from_user_id)
        .bind(to_user_id)
        .bind(message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get unread messages for a player
    pub async fn get_unread_messages(&self, user_id: i64) -> Result<Vec<(i64, String, String)>, sqlx::Error> {
        let rows: Vec<(i64, String, String)> = sqlx::query_as(
            r#"
            SELECT m.from_user_id, p.handle, m.message
            FROM messages m
            JOIN player_positions p ON m.from_user_id = p.user_id
            WHERE m.to_user_id = ? AND m.read = 0
            ORDER BY m.created_at ASC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        // Mark as read
        sqlx::query("UPDATE messages SET read = 1 WHERE to_user_id = ? AND read = 0")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(rows)
    }

    // ========================================================================
    // WORLD STATE
    // ========================================================================

    /// Set a world state value
    pub async fn set_world_state(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO world_state (key, value, updated_at)
             VALUES (?, ?, datetime('now', '-5 hours'))"
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get a world state value
    pub async fn get_world_state(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT value FROM world_state WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Check if any player has become Arch-Mage (for game-wide events)
    pub async fn get_current_archmage(&self) -> Result<Option<String>, sqlx::Error> {
        self.get_world_state("current_archmage").await
    }

    /// Set the current Arch-Mage
    pub async fn set_current_archmage(&self, handle: &str) -> Result<(), sqlx::Error> {
        self.set_world_state("current_archmage", handle).await
    }
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub level: u8,
    pub became_archmage: bool,
    pub monsters_killed: u32,
    pub gold_earned: i64,
    pub completed_at: String,
}
