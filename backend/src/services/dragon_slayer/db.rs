//! Dragon Slayer database layer
//! Manages player saves, character data, romance, and leaderboards

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct DragonSlayerDb {
    pool: SqlitePool,
}

impl DragonSlayerDb {
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
        // Main save table - stores serialized GameState
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                char_name TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                experience INTEGER NOT NULL DEFAULT 0,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Characters table for cross-player features (romance, PvP)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS characters (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                char_name TEXT NOT NULL,
                sex TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                experience INTEGER NOT NULL DEFAULT 0,
                hp_current INTEGER NOT NULL,
                hp_max INTEGER NOT NULL,
                strength INTEGER NOT NULL,
                defense INTEGER NOT NULL,
                weapon TEXT NOT NULL,
                armor TEXT NOT NULL,
                gold_pocket INTEGER NOT NULL DEFAULT 0,
                gold_bank INTEGER NOT NULL DEFAULT 0,
                kills INTEGER NOT NULL DEFAULT 0,
                deaths INTEGER NOT NULL DEFAULT 0,
                dragon_kills INTEGER NOT NULL DEFAULT 0,
                spouse_user_id INTEGER,
                last_active TEXT NOT NULL,
                is_online INTEGER NOT NULL DEFAULT 0
            )
        "#).execute(&self.pool).await?;

        // Romance table for player-to-player relationships
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS romance (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user1_id INTEGER NOT NULL,
                user2_id INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                married_at TEXT,
                divorced_at TEXT
            )
        "#).execute(&self.pool).await?;

        // Completions table for leaderboard
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                char_name TEXT NOT NULL,
                dragon_kills INTEGER NOT NULL,
                final_level INTEGER NOT NULL,
                final_experience INTEGER NOT NULL,
                total_kills INTEGER NOT NULL,
                total_deaths INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Daily state for turn tracking (separate from main save for efficiency)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS daily_state (
                user_id INTEGER PRIMARY KEY,
                date TEXT NOT NULL,
                forest_fights_used INTEGER NOT NULL DEFAULT 0,
                player_fights_used INTEGER NOT NULL DEFAULT 0,
                flirts_used INTEGER NOT NULL DEFAULT 0
            )
        "#).execute(&self.pool).await?;

        // IGM module state storage
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS igm_state (
                user_id INTEGER NOT NULL,
                module_id TEXT NOT NULL,
                state_key TEXT NOT NULL,
                state_value TEXT NOT NULL,
                PRIMARY KEY (user_id, module_id, state_key)
            )
        "#).execute(&self.pool).await?;

        // Create indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_dragon_kills
            ON completions(dragon_kills DESC)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_characters_level
            ON characters(level DESC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // SAVE/LOAD OPERATIONS
    // ========================================================================

    pub async fn save_game(&self, user_id: i64, handle: &str, char_name: &str, level: u8, experience: i64, state_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, char_name, level, experience, state_json, last_saved)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(char_name)
        .bind(level as i32)
        .bind(experience)
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
    // CHARACTER SYNC (for PvP and romance)
    // ========================================================================

    #[allow(dead_code)]
    pub async fn sync_character(&self, user_id: i64, handle: &str, char_name: &str, sex: &str, level: u8, experience: i64, hp_current: u32, hp_max: u32, strength: u32, defense: u32, weapon: &str, armor: &str, gold_pocket: i64, gold_bank: i64, kills: u32, deaths: u32, dragon_kills: u32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO characters (user_id, handle, char_name, sex, level, experience, hp_current, hp_max, strength, defense, weapon, armor, gold_pocket, gold_bank, kills, deaths, dragon_kills, last_active, is_online)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'), 1)"
        )
        .bind(user_id)
        .bind(handle)
        .bind(char_name)
        .bind(sex)
        .bind(level as i32)
        .bind(experience)
        .bind(hp_current as i32)
        .bind(hp_max as i32)
        .bind(strength as i32)
        .bind(defense as i32)
        .bind(weapon)
        .bind(armor)
        .bind(gold_pocket)
        .bind(gold_bank)
        .bind(kills as i32)
        .bind(deaths as i32)
        .bind(dragon_kills as i32)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get characters available for PvP attack
    #[allow(dead_code)]
    pub async fn get_attackable_characters(&self, attacker_level: u8, exclude_user_id: i64, limit: i64) -> Result<Vec<CharacterInfo>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i32, i32, i32)> = sqlx::query_as(
            r#"
            SELECT user_id, handle, char_name, level, hp_current, gold_pocket
            FROM characters
            WHERE user_id != ? AND level <= ? AND is_online = 0
            ORDER BY level DESC
            LIMIT ?
            "#
        )
        .bind(exclude_user_id)
        .bind(attacker_level as i32)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(user_id, handle, char_name, level, hp, gold)| {
            CharacterInfo {
                user_id,
                handle,
                char_name,
                level: level as u8,
                hp: hp as u32,
                gold: gold as i64,
            }
        }).collect())
    }

    // ========================================================================
    // LEADERBOARD OPERATIONS
    // ========================================================================

    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        char_name: &str,
        dragon_kills: u32,
        final_level: u8,
        final_experience: i64,
        total_kills: u32,
        total_deaths: u32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, char_name, dragon_kills, final_level, final_experience, total_kills, total_deaths, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(char_name)
        .bind(dragon_kills as i32)
        .bind(final_level as i32)
        .bind(final_experience)
        .bind(total_kills as i32)
        .bind(total_deaths as i32)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i32, i32, i32, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY dragon_kills DESC, final_level DESC) as rank,
                handle,
                char_name,
                dragon_kills,
                final_level,
                total_kills,
                completed_at
            FROM completions
            ORDER BY dragon_kills DESC, final_level DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, char_name, dragons, level, kills, date)| {
            LeaderboardEntry {
                rank,
                handle,
                char_name,
                dragon_kills: dragons as u32,
                final_level: level as u8,
                total_kills: kills as u32,
                completed_at: date,
            }
        }).collect())
    }

    // ========================================================================
    // DAILY STATE
    // ========================================================================

    #[allow(dead_code)]
    pub async fn get_daily_state(&self, user_id: i64, date: &str) -> Result<Option<DailyState>, sqlx::Error> {
        let row: Option<(i32, i32, i32)> = sqlx::query_as(
            "SELECT forest_fights_used, player_fights_used, flirts_used FROM daily_state WHERE user_id = ? AND date = ?"
        )
        .bind(user_id)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(forest, player, flirts)| DailyState {
            forest_fights_used: forest as u32,
            player_fights_used: player as u32,
            flirts_used: flirts as u8,
        }))
    }

    #[allow(dead_code)]
    pub async fn update_daily_state(&self, user_id: i64, date: &str, forest_fights: u32, player_fights: u32, flirts: u8) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO daily_state (user_id, date, forest_fights_used, player_fights_used, flirts_used)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(date)
        .bind(forest_fights as i32)
        .bind(player_fights as i32)
        .bind(flirts as i32)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ========================================================================
    // IGM STATE
    // ========================================================================

    #[allow(dead_code)]
    pub async fn get_igm_state(&self, user_id: i64, module_id: &str, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_value FROM igm_state WHERE user_id = ? AND module_id = ? AND state_key = ?"
        )
        .bind(user_id)
        .bind(module_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    #[allow(dead_code)]
    pub async fn set_igm_state(&self, user_id: i64, module_id: &str, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO igm_state (user_id, module_id, state_key, state_value)
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

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub char_name: String,
    pub dragon_kills: u32,
    pub final_level: u8,
    pub total_kills: u32,
    pub completed_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CharacterInfo {
    pub user_id: i64,
    pub handle: String,
    pub char_name: String,
    pub level: u8,
    pub hp: u32,
    pub gold: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DailyState {
    pub forest_fights_used: u32,
    pub player_fights_used: u32,
    pub flirts_used: u8,
}
