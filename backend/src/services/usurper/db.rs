//! Database layer for Usurper
//!
//! Manages player saves, completions, leaderboards, and multiplayer state.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct UsurperDb {
    pool: SqlitePool,
}

impl UsurperDb {
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
        // Main player saves
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Game completions (for leaderboard)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                character_name TEXT NOT NULL,
                final_level INTEGER NOT NULL,
                deepest_dungeon INTEGER NOT NULL,
                monsters_killed INTEGER NOT NULL,
                total_gold INTEGER NOT NULL,
                supreme_defeated INTEGER NOT NULL,
                is_king INTEGER NOT NULL,
                godhood_level INTEGER NOT NULL,
                days_played INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_level
            ON completions(final_level DESC, deepest_dungeon DESC)
        "#).execute(&self.pool).await?;

        // Players table (for multiplayer features)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS players (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                character_name TEXT NOT NULL,
                level INTEGER NOT NULL,
                class TEXT NOT NULL,
                is_king INTEGER NOT NULL DEFAULT 0,
                godhood_level INTEGER NOT NULL DEFAULT 0,
                clan_id TEXT,
                last_active TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Clans table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS clans (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                leader_user_id INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Romance relationships
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS romances (
                user_id_1 INTEGER NOT NULL,
                user_id_2 INTEGER NOT NULL,
                relationship_level INTEGER NOT NULL,
                started_at TEXT NOT NULL,
                PRIMARY KEY (user_id_1, user_id_2)
            )
        "#).execute(&self.pool).await?;

        // PvP records
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS pvp_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                attacker_id INTEGER NOT NULL,
                defender_id INTEGER NOT NULL,
                attacker_level INTEGER NOT NULL,
                defender_level INTEGER NOT NULL,
                attacker_won INTEGER NOT NULL,
                xp_penalty INTEGER NOT NULL,
                fought_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Substances inventory (for persistence between sessions)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS substance_inventory (
                user_id INTEGER NOT NULL,
                substance_key TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                PRIMARY KEY (user_id, substance_key)
            )
        "#).execute(&self.pool).await?;

        // IGM custom data storage
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS igm_data (
                user_id INTEGER NOT NULL,
                module_id TEXT NOT NULL,
                data_key TEXT NOT NULL,
                data_value TEXT NOT NULL,
                PRIMARY KEY (user_id, module_id, data_key)
            )
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // SAVE/LOAD OPERATIONS
    // ========================================================================

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

    // ========================================================================
    // COMPLETION RECORDS
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        character_name: &str,
        final_level: u32,
        deepest_dungeon: u32,
        monsters_killed: u64,
        total_gold: u64,
        supreme_defeated: bool,
        is_king: bool,
        godhood_level: u32,
        days_played: u32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (
                user_id, handle, character_name, final_level, deepest_dungeon,
                monsters_killed, total_gold, supreme_defeated, is_king,
                godhood_level, days_played, completed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(character_name)
        .bind(final_level as i64)
        .bind(deepest_dungeon as i64)
        .bind(monsters_killed as i64)
        .bind(total_gold as i64)
        .bind(supreme_defeated as i64)
        .bind(is_king as i64)
        .bind(godhood_level as i64)
        .bind(days_played as i64)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i64, i64, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY final_level DESC, deepest_dungeon DESC) as rank,
                handle,
                character_name,
                final_level,
                deepest_dungeon,
                monsters_killed,
                supreme_defeated,
                completed_at
            FROM completions
            ORDER BY final_level DESC, deepest_dungeon DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, name, level, dungeon, kills, supreme, date)| {
            LeaderboardEntry {
                rank,
                handle,
                character_name: name,
                final_level: level as u32,
                deepest_dungeon: dungeon as u32,
                monsters_killed: kills as u64,
                supreme_defeated: supreme != 0,
                completed_at: date,
            }
        }).collect())
    }

    // ========================================================================
    // MULTIPLAYER OPERATIONS
    // ========================================================================

    pub async fn update_player_status(
        &self,
        user_id: i64,
        handle: &str,
        character_name: &str,
        level: u32,
        class: &str,
        is_king: bool,
        godhood_level: u32,
        clan_id: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO players (
                user_id, handle, character_name, level, class,
                is_king, godhood_level, clan_id, last_active
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(character_name)
        .bind(level as i64)
        .bind(class)
        .bind(is_king as i64)
        .bind(godhood_level as i64)
        .bind(clan_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_active_players(&self, limit: i64) -> Result<Vec<PlayerInfo>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i64, String, i64, i64)> = sqlx::query_as(
            "SELECT user_id, handle, character_name, level, class, is_king, godhood_level
             FROM players
             WHERE last_active > datetime('now', '-1 day', '-5 hours')
             ORDER BY level DESC
             LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, handle, name, level, class, king, god)| {
            PlayerInfo {
                user_id: id,
                handle,
                character_name: name,
                level: level as u32,
                class,
                is_king: king != 0,
                godhood_level: god as u32,
            }
        }).collect())
    }

    pub async fn get_current_king(&self) -> Result<Option<PlayerInfo>, sqlx::Error> {
        let row: Option<(i64, String, String, i64, String, i64, i64)> = sqlx::query_as(
            "SELECT user_id, handle, character_name, level, class, is_king, godhood_level
             FROM players
             WHERE is_king = 1
             LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, handle, name, level, class, _, god)| {
            PlayerInfo {
                user_id: id,
                handle,
                character_name: name,
                level: level as u32,
                class,
                is_king: true,
                godhood_level: god as u32,
            }
        }))
    }

    // ========================================================================
    // PVP OPERATIONS
    // ========================================================================

    pub async fn record_pvp_fight(
        &self,
        attacker_id: i64,
        defender_id: i64,
        attacker_level: u32,
        defender_level: u32,
        attacker_won: bool,
        xp_penalty: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO pvp_records (
                attacker_id, defender_id, attacker_level, defender_level,
                attacker_won, xp_penalty, fought_at
            ) VALUES (?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(attacker_id)
        .bind(defender_id)
        .bind(attacker_level as i64)
        .bind(defender_level as i64)
        .bind(attacker_won as i64)
        .bind(xp_penalty as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ========================================================================
    // IGM DATA OPERATIONS
    // ========================================================================

    pub async fn get_igm_data(
        &self,
        user_id: i64,
        module_id: &str,
        key: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT data_value FROM igm_data
             WHERE user_id = ? AND module_id = ? AND data_key = ?"
        )
        .bind(user_id)
        .bind(module_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn set_igm_data(
        &self,
        user_id: i64,
        module_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO igm_data (user_id, module_id, data_key, data_value)
             VALUES (?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(module_id)
        .bind(key)
        .bind(value)
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
    pub character_name: String,
    pub final_level: u32,
    pub deepest_dungeon: u32,
    pub monsters_killed: u64,
    pub supreme_defeated: bool,
    pub completed_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlayerInfo {
    pub user_id: i64,
    pub handle: String,
    pub character_name: String,
    pub level: u32,
    pub class: String,
    pub is_king: bool,
    pub godhood_level: u32,
}
