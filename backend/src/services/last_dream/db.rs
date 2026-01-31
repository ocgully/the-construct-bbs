//! Last Dream database layer
//! Manages player saves, party data, world state, and completions

#![allow(dead_code)]

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct LastDreamDb {
    pool: SqlitePool,
}

impl LastDreamDb {
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
                state_json TEXT NOT NULL,
                play_time INTEGER NOT NULL DEFAULT 0,
                party_level INTEGER NOT NULL DEFAULT 1,
                crystals_lit INTEGER NOT NULL DEFAULT 0,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Party members table (for leaderboard display)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS party_members (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                slot INTEGER NOT NULL,
                name TEXT NOT NULL,
                class TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                UNIQUE(user_id, slot)
            )
        "#).execute(&self.pool).await?;

        // Completions table for leaderboard
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                party_info TEXT NOT NULL,
                play_time INTEGER NOT NULL,
                crystals_lit INTEGER NOT NULL,
                monsters_defeated INTEGER NOT NULL,
                battles_fought INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Achievements
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS achievements (
                user_id INTEGER NOT NULL,
                achievement_key TEXT NOT NULL,
                unlocked_at TEXT NOT NULL,
                PRIMARY KEY (user_id, achievement_key)
            )
        "#).execute(&self.pool).await?;

        // Create indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_time
            ON completions(play_time ASC)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_saves_level
            ON saves(party_level DESC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // SAVE/LOAD OPERATIONS
    // ========================================================================

    pub async fn save_game(
        &self,
        user_id: i64,
        handle: &str,
        state_json: &str,
        play_time: u64,
        party_level: u8,
        crystals_lit: u8,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, state_json, play_time, party_level, crystals_lit, last_saved)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(state_json)
        .bind(play_time as i64)
        .bind(party_level as i32)
        .bind(crystals_lit as i32)
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

        sqlx::query("DELETE FROM party_members WHERE user_id = ?")
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
    // PARTY MEMBERS
    // ========================================================================

    pub async fn save_party_member(
        &self,
        user_id: i64,
        slot: u8,
        name: &str,
        class: &str,
        level: u8,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO party_members (user_id, slot, name, class, level)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(slot as i32)
        .bind(name)
        .bind(class)
        .bind(level as i32)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_party_members(&self, user_id: i64) -> Result<Vec<PartyMemberInfo>, sqlx::Error> {
        let rows: Vec<(i32, String, String, i32)> = sqlx::query_as(
            "SELECT slot, name, class, level FROM party_members WHERE user_id = ? ORDER BY slot"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(slot, name, class, level)| {
            PartyMemberInfo {
                slot: slot as u8,
                name,
                class,
                level: level as u8,
            }
        }).collect())
    }

    // ========================================================================
    // COMPLETIONS & LEADERBOARD
    // ========================================================================

    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        party_info: &str,
        play_time: u64,
        crystals_lit: u8,
        monsters_defeated: u32,
        battles_fought: u32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, party_info, play_time, crystals_lit, monsters_defeated, battles_fought, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(party_info)
        .bind(play_time as i64)
        .bind(crystals_lit as i32)
        .bind(monsters_defeated as i32)
        .bind(battles_fought as i32)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i64, i32, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY play_time ASC) as rank,
                handle,
                party_info,
                play_time,
                crystals_lit,
                completed_at
            FROM completions
            WHERE crystals_lit = 4
            ORDER BY play_time ASC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, party_info, time, crystals, date)| {
            LeaderboardEntry {
                rank,
                handle,
                party_info,
                play_time: time as u64,
                crystals_lit: crystals as u8,
                completed_at: date,
            }
        }).collect())
    }

    // ========================================================================
    // ACHIEVEMENTS
    // ========================================================================

    #[allow(dead_code)]
    pub async fn unlock_achievement(&self, user_id: i64, achievement: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO achievements (user_id, achievement_key, unlocked_at)
             VALUES (?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(achievement)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    #[allow(dead_code)]
    pub async fn get_achievements(&self, user_id: i64) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT achievement_key FROM achievements WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // ========================================================================
    // STATISTICS
    // ========================================================================

    #[allow(dead_code)]
    pub async fn get_global_stats(&self) -> Result<GlobalStats, sqlx::Error> {
        let row: Option<(i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT
                (SELECT COUNT(*) FROM saves) as active_games,
                (SELECT COUNT(*) FROM completions) as total_completions,
                COALESCE((SELECT SUM(monsters_defeated) FROM completions), 0) as total_monsters
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(active, completions, monsters)| {
            GlobalStats {
                active_games: active as u32,
                total_completions: completions as u32,
                total_monsters_defeated: monsters as u64,
            }
        }).unwrap_or_default())
    }
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub party_info: String,
    pub play_time: u64,
    pub crystals_lit: u8,
    pub completed_at: String,
}

#[derive(Debug, Clone)]
pub struct PartyMemberInfo {
    pub slot: u8,
    pub name: String,
    pub class: String,
    pub level: u8,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalStats {
    pub active_games: u32,
    pub total_completions: u32,
    pub total_monsters_defeated: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_db() -> (LastDreamDb, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_last_dream.db");
        let db = LastDreamDb::new(&db_path).await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let (db, _temp) = create_test_db().await;

        let state_json = r#"{"gold":500,"play_time":0}"#;
        db.save_game(1, "TestUser", state_json, 0, 1, 0).await.unwrap();

        let loaded = db.load_game(1).await.unwrap();
        assert!(loaded.is_some());
        assert!(loaded.unwrap().contains("500"));
    }

    #[tokio::test]
    async fn test_delete_save() {
        let (db, _temp) = create_test_db().await;

        db.save_game(1, "TestUser", "{}", 0, 1, 0).await.unwrap();
        assert!(db.has_save(1).await.unwrap());

        db.delete_save(1).await.unwrap();
        assert!(!db.has_save(1).await.unwrap());
    }

    #[tokio::test]
    async fn test_party_members() {
        let (db, _temp) = create_test_db().await;

        db.save_party_member(1, 0, "Hero", "Warrior", 5).await.unwrap();
        db.save_party_member(1, 1, "Mage", "Mage", 4).await.unwrap();

        let members = db.get_party_members(1).await.unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].name, "Hero");
    }

    #[tokio::test]
    async fn test_completion_recording() {
        let (db, _temp) = create_test_db().await;

        let id = db.record_completion(
            1, "TestUser", "Hero/Mage/Thief/Cleric",
            7200, 4, 500, 100
        ).await.unwrap();

        assert!(id > 0);

        let leaderboard = db.get_leaderboard(10).await.unwrap();
        assert!(!leaderboard.is_empty());
    }

    #[tokio::test]
    async fn test_achievements() {
        let (db, _temp) = create_test_db().await;

        let new = db.unlock_achievement(1, "first_crystal").await.unwrap();
        assert!(new);

        let duplicate = db.unlock_achievement(1, "first_crystal").await.unwrap();
        assert!(!duplicate);

        let achievements = db.get_achievements(1).await.unwrap();
        assert!(achievements.contains(&"first_crystal".to_string()));
    }
}
