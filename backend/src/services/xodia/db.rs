//! Database layer for Xodia
//!
//! Handles persistence for characters, world state, and conversation history.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct XodiaDb {
    pool: SqlitePool,
}

#[allow(dead_code)]
impl XodiaDb {
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

        // Enable WAL mode for better concurrency
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await?;

        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // Character saves - one active character per user
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS characters (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                character_name TEXT NOT NULL,
                class TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                experience INTEGER NOT NULL DEFAULT 0,
                health INTEGER NOT NULL,
                max_health INTEGER NOT NULL,
                mana INTEGER NOT NULL,
                max_mana INTEGER NOT NULL,
                current_room_id TEXT NOT NULL,
                current_region TEXT NOT NULL,
                gold INTEGER NOT NULL DEFAULT 50,
                state_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_played TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // World state - shared across all players
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS world_state (
                id INTEGER PRIMARY KEY DEFAULT 1,
                state_json TEXT NOT NULL,
                last_updated TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Rooms - persistent world locations
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS rooms (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                region TEXT NOT NULL,
                exits_json TEXT NOT NULL,
                npcs_json TEXT NOT NULL,
                items_json TEXT NOT NULL,
                discovered_by_json TEXT NOT NULL DEFAULT '[]',
                created_by INTEGER,
                is_generated INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // NPCs - persistent NPC instances
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS npcs (
                instance_id TEXT PRIMARY KEY,
                template_key TEXT NOT NULL,
                name TEXT NOT NULL,
                current_room TEXT NOT NULL,
                health INTEGER NOT NULL,
                max_health INTEGER NOT NULL,
                is_alive INTEGER NOT NULL DEFAULT 1,
                is_hostile INTEGER NOT NULL DEFAULT 0,
                dialogue_state INTEGER NOT NULL DEFAULT 0,
                inventory_json TEXT NOT NULL DEFAULT '[]',
                memory_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Event log - track significant events for narrative consistency
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                room_id TEXT NOT NULL,
                actor_id INTEGER,
                actor_name TEXT NOT NULL,
                action TEXT NOT NULL,
                target TEXT,
                outcome TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Conversation history - for LLM context
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                npc_id TEXT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                tokens_used INTEGER DEFAULT 0,
                timestamp TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Leaderboard entries
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                character_name TEXT NOT NULL,
                class TEXT NOT NULL,
                level INTEGER NOT NULL,
                rooms_discovered INTEGER NOT NULL,
                monsters_slain INTEGER NOT NULL,
                main_quest_completed INTEGER NOT NULL DEFAULT 0,
                playtime_seconds INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // System configuration
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Create indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_events_room ON events(room_id)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_conversations_user ON conversations(user_id)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_level ON completions(level DESC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // Character Operations
    // ========================================================================

    pub async fn save_character(
        &self,
        user_id: i64,
        handle: &str,
        state_json: &str,
        character_name: &str,
        class: &str,
        level: u32,
        experience: u64,
        health: i32,
        max_health: i32,
        mana: i32,
        max_mana: i32,
        current_room_id: &str,
        current_region: &str,
        gold: i64,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query(r#"
            INSERT INTO characters (
                user_id, handle, character_name, class, level, experience,
                health, max_health, mana, max_mana,
                current_room_id, current_region, gold, state_json,
                created_at, last_played
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE SET
                handle = excluded.handle,
                character_name = excluded.character_name,
                class = excluded.class,
                level = excluded.level,
                experience = excluded.experience,
                health = excluded.health,
                max_health = excluded.max_health,
                mana = excluded.mana,
                max_mana = excluded.max_mana,
                current_room_id = excluded.current_room_id,
                current_region = excluded.current_region,
                gold = excluded.gold,
                state_json = excluded.state_json,
                last_played = excluded.last_played
        "#)
        .bind(user_id)
        .bind(handle)
        .bind(character_name)
        .bind(class)
        .bind(level)
        .bind(experience as i64)
        .bind(health)
        .bind(max_health)
        .bind(mana)
        .bind(max_mana)
        .bind(current_room_id)
        .bind(current_region)
        .bind(gold)
        .bind(state_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_character(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM characters WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
    }

    pub async fn delete_character(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM characters WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn has_character(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM characters WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    // ========================================================================
    // World State Operations
    // ========================================================================

    pub async fn save_world_state(&self, state_json: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query(r#"
            INSERT INTO world_state (id, state_json, last_updated)
            VALUES (1, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                state_json = excluded.state_json,
                last_updated = excluded.last_updated
        "#)
        .bind(state_json)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_world_state(&self) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM world_state WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
    }

    // ========================================================================
    // Event Log Operations
    // ========================================================================

    pub async fn log_event(
        &self,
        room_id: &str,
        actor_id: Option<i64>,
        actor_name: &str,
        action: &str,
        target: Option<&str>,
        outcome: &str,
    ) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let result = sqlx::query(r#"
            INSERT INTO events (room_id, actor_id, actor_name, action, target, outcome, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(room_id)
        .bind(actor_id)
        .bind(actor_name)
        .bind(action)
        .bind(target)
        .bind(outcome)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_recent_events(&self, room_id: &str, limit: i64) -> Result<Vec<EventRecord>, sqlx::Error> {
        let rows: Vec<(String, Option<i64>, String, String, Option<String>, String, String)> = sqlx::query_as(r#"
            SELECT room_id, actor_id, actor_name, action, target, outcome, timestamp
            FROM events
            WHERE room_id = ?
            ORDER BY timestamp DESC
            LIMIT ?
        "#)
        .bind(room_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(room_id, actor_id, actor_name, action, target, outcome, timestamp)| {
            EventRecord {
                room_id,
                actor_id,
                actor_name,
                action,
                target,
                outcome,
                timestamp,
            }
        }).collect())
    }

    // ========================================================================
    // Conversation History Operations
    // ========================================================================

    pub async fn log_conversation(
        &self,
        user_id: i64,
        npc_id: Option<&str>,
        role: &str,
        content: &str,
        tokens_used: u32,
    ) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let result = sqlx::query(r#"
            INSERT INTO conversations (user_id, npc_id, role, content, tokens_used, timestamp)
            VALUES (?, ?, ?, ?, ?, ?)
        "#)
        .bind(user_id)
        .bind(npc_id)
        .bind(role)
        .bind(content)
        .bind(tokens_used)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_conversation_history(
        &self,
        user_id: i64,
        npc_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ConversationRecord>, sqlx::Error> {
        let rows: Vec<(String, String, i32, String)> = if let Some(npc) = npc_id {
            sqlx::query_as(r#"
                SELECT role, content, tokens_used, timestamp
                FROM conversations
                WHERE user_id = ? AND npc_id = ?
                ORDER BY timestamp DESC
                LIMIT ?
            "#)
            .bind(user_id)
            .bind(npc)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(r#"
                SELECT role, content, tokens_used, timestamp
                FROM conversations
                WHERE user_id = ? AND npc_id IS NULL
                ORDER BY timestamp DESC
                LIMIT ?
            "#)
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|(role, content, tokens, timestamp)| {
            ConversationRecord {
                role,
                content,
                tokens_used: tokens as u32,
                timestamp,
            }
        }).collect())
    }

    // ========================================================================
    // Configuration Operations
    // ========================================================================

    pub async fn get_config(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT value FROM config WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        sqlx::query(r#"
            INSERT INTO config (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
        "#)
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if maintenance mode is enabled
    pub async fn is_maintenance_mode(&self) -> Result<bool, sqlx::Error> {
        let value = self.get_config("maintenance_mode").await?;
        Ok(value.as_deref() == Some("true"))
    }

    /// Set maintenance mode
    pub async fn set_maintenance_mode(&self, enabled: bool) -> Result<(), sqlx::Error> {
        self.set_config("maintenance_mode", if enabled { "true" } else { "false" }).await
    }

    // ========================================================================
    // Leaderboard Operations
    // ========================================================================

    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        character_name: &str,
        class: &str,
        level: u32,
        rooms_discovered: u32,
        monsters_slain: u32,
        main_quest_completed: bool,
        playtime_seconds: u64,
    ) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let result = sqlx::query(r#"
            INSERT INTO completions (
                user_id, handle, character_name, class, level,
                rooms_discovered, monsters_slain, main_quest_completed,
                playtime_seconds, completed_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(user_id)
        .bind(handle)
        .bind(character_name)
        .bind(class)
        .bind(level)
        .bind(rooms_discovered)
        .bind(monsters_slain)
        .bind(main_quest_completed)
        .bind(playtime_seconds as i64)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, String, i32, i32, i64, String)> = sqlx::query_as(r#"
            SELECT
                RANK() OVER (ORDER BY level DESC, rooms_discovered DESC) as rank,
                handle,
                character_name,
                class,
                level,
                rooms_discovered,
                playtime_seconds,
                completed_at
            FROM completions
            ORDER BY level DESC, rooms_discovered DESC
            LIMIT ?
        "#)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, character_name, class, level, rooms, playtime, completed)| {
            LeaderboardEntry {
                rank,
                handle,
                character_name,
                class,
                level: level as u32,
                rooms_discovered: rooms as u32,
                playtime_seconds: playtime as u64,
                completed_at: completed,
            }
        }).collect())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventRecord {
    pub room_id: String,
    pub actor_id: Option<i64>,
    pub actor_name: String,
    pub action: String,
    pub target: Option<String>,
    pub outcome: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConversationRecord {
    pub role: String,
    pub content: String,
    pub tokens_used: u32,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub character_name: String,
    pub class: String,
    pub level: u32,
    pub rooms_discovered: u32,
    pub playtime_seconds: u64,
    pub completed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> XodiaDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_xodia.db");
        XodiaDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_db_creation() {
        let _db = create_test_db().await;
    }

    #[tokio::test]
    async fn test_character_save_load() {
        let db = create_test_db().await;

        // Save character
        db.save_character(
            1,
            "TestUser",
            r#"{"character_name": "Hero"}"#,
            "Hero",
            "Warrior",
            5,
            500,
            100,
            100,
            20,
            20,
            "misthollow_square",
            "misthollow",
            150,
        ).await.unwrap();

        // Load character
        let loaded = db.load_character(1).await.unwrap();
        assert!(loaded.is_some());
        assert!(loaded.unwrap().contains("Hero"));

        // Check exists
        assert!(db.has_character(1).await.unwrap());
        assert!(!db.has_character(2).await.unwrap());

        // Delete
        db.delete_character(1).await.unwrap();
        assert!(!db.has_character(1).await.unwrap());
    }

    #[tokio::test]
    async fn test_world_state() {
        let db = create_test_db().await;

        // Save world state
        db.save_world_state(r#"{"rooms": {}}"#).await.unwrap();

        // Load world state
        let loaded = db.load_world_state().await.unwrap();
        assert!(loaded.is_some());
        assert!(loaded.unwrap().contains("rooms"));
    }

    #[tokio::test]
    async fn test_event_logging() {
        let db = create_test_db().await;

        // Log event
        let id = db.log_event(
            "misthollow_square",
            Some(1),
            "Hero",
            "attack",
            Some("goblin"),
            "Hit for 10 damage",
        ).await.unwrap();

        assert!(id > 0);

        // Get recent events
        let events = db.get_recent_events("misthollow_square", 10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "attack");
    }

    #[tokio::test]
    async fn test_conversation_history() {
        let db = create_test_db().await;

        // Log conversation
        db.log_conversation(1, Some("elder_mira"), "user", "Hello!", 10).await.unwrap();
        db.log_conversation(1, Some("elder_mira"), "assistant", "Welcome, Seeker.", 15).await.unwrap();

        // Get history
        let history = db.get_conversation_history(1, Some("elder_mira"), 10).await.unwrap();
        assert_eq!(history.len(), 2);
    }

    #[tokio::test]
    async fn test_config() {
        let db = create_test_db().await;

        // Set config
        db.set_config("test_key", "test_value").await.unwrap();

        // Get config
        let value = db.get_config("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Nonexistent key
        let missing = db.get_config("missing_key").await.unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_maintenance_mode() {
        let db = create_test_db().await;

        // Default is off
        assert!(!db.is_maintenance_mode().await.unwrap());

        // Enable
        db.set_maintenance_mode(true).await.unwrap();
        assert!(db.is_maintenance_mode().await.unwrap());

        // Disable
        db.set_maintenance_mode(false).await.unwrap();
        assert!(!db.is_maintenance_mode().await.unwrap());
    }

    #[tokio::test]
    async fn test_leaderboard() {
        let db = create_test_db().await;

        // Record completion
        db.record_completion(
            1, "User1", "Hero1", "Warrior", 10, 25, 50, true, 3600
        ).await.unwrap();

        db.record_completion(
            2, "User2", "Hero2", "Mage", 8, 20, 30, false, 2400
        ).await.unwrap();

        // Get leaderboard
        let entries = db.get_leaderboard(10).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, 10); // Higher level first
        assert_eq!(entries[1].level, 8);
    }
}
