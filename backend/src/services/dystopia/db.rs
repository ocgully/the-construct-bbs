//! Dystopia database - provinces, kingdoms, ages, attacks

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct DystopiaDb {
    pool: SqlitePool,
}

impl DystopiaDb {
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
        // Ages table - tracks game rounds
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS ages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ends_at TEXT NOT NULL,
                winner_kingdom_id INTEGER,
                is_active INTEGER NOT NULL DEFAULT 1
            )
        "#).execute(&self.pool).await?;

        // Kingdoms table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS kingdoms (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                age_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                motto TEXT,
                ruler_user_id INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (age_id) REFERENCES ages(id)
            )
        "#).execute(&self.pool).await?;

        // Provinces table - active game saves
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS provinces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                age_id INTEGER NOT NULL,
                kingdom_id INTEGER,
                state_json TEXT NOT NULL,
                last_tick INTEGER NOT NULL DEFAULT 0,
                last_saved TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (age_id) REFERENCES ages(id),
                FOREIGN KEY (kingdom_id) REFERENCES kingdoms(id)
            )
        "#).execute(&self.pool).await?;

        // Kingdom membership (max 10 per kingdom)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS kingdom_members (
                kingdom_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                province_id INTEGER NOT NULL,
                joined_at TEXT NOT NULL,
                is_ruler INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (kingdom_id, user_id),
                FOREIGN KEY (kingdom_id) REFERENCES kingdoms(id),
                FOREIGN KEY (province_id) REFERENCES provinces(id)
            )
        "#).execute(&self.pool).await?;

        // Attack history
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS attacks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                age_id INTEGER NOT NULL,
                attacker_id INTEGER NOT NULL,
                defender_id INTEGER NOT NULL,
                attack_type TEXT NOT NULL,
                success INTEGER NOT NULL,
                land_captured INTEGER NOT NULL DEFAULT 0,
                gold_stolen INTEGER NOT NULL DEFAULT 0,
                attacker_losses TEXT,
                defender_losses TEXT,
                attacked_at TEXT NOT NULL,
                FOREIGN KEY (age_id) REFERENCES ages(id),
                FOREIGN KEY (attacker_id) REFERENCES provinces(id),
                FOREIGN KEY (defender_id) REFERENCES provinces(id)
            )
        "#).execute(&self.pool).await?;

        // Completions for leaderboard (end of age scores)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                age_id INTEGER NOT NULL,
                final_networth INTEGER NOT NULL,
                final_land INTEGER NOT NULL,
                attacks_won INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Create indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_provinces_user
            ON provinces(user_id, age_id, is_active)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_networth
            ON completions(final_networth DESC)
        "#).execute(&self.pool).await?;

        // Ensure there's an active age
        let active_age: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM ages WHERE is_active = 1 LIMIT 1"
        ).fetch_optional(&self.pool).await?;

        if active_age.is_none() {
            // Create initial age (4 weeks default duration)
            sqlx::query(
                "INSERT INTO ages (name, started_at, ends_at, is_active)
                 VALUES ('Age of Shadows', datetime('now'), datetime('now', '+28 days'), 1)"
            ).execute(&self.pool).await?;
        }

        Ok(())
    }

    /// Get the current active age ID
    pub async fn get_active_age_id(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT id FROM ages WHERE is_active = 1 ORDER BY id DESC LIMIT 1"
        ).fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    /// Save or update a province
    pub async fn save_province(
        &self,
        user_id: i64,
        handle: &str,
        age_id: i64,
        state_json: &str,
        last_tick: i64,
    ) -> Result<i64, sqlx::Error> {
        // Check for existing province in this age
        let existing: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM provinces WHERE user_id = ? AND age_id = ? AND is_active = 1"
        )
        .bind(user_id)
        .bind(age_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((province_id,)) = existing {
            // Update existing
            sqlx::query(
                "UPDATE provinces SET state_json = ?, last_tick = ?, last_saved = datetime('now')
                 WHERE id = ?"
            )
            .bind(state_json)
            .bind(last_tick)
            .bind(province_id)
            .execute(&self.pool)
            .await?;
            Ok(province_id)
        } else {
            // Insert new
            let result = sqlx::query(
                "INSERT INTO provinces (user_id, handle, age_id, state_json, last_tick, last_saved)
                 VALUES (?, ?, ?, ?, ?, datetime('now'))"
            )
            .bind(user_id)
            .bind(handle)
            .bind(age_id)
            .bind(state_json)
            .bind(last_tick)
            .execute(&self.pool)
            .await?;
            Ok(result.last_insert_rowid())
        }
    }

    /// Load a province for a user in the current age
    pub async fn load_province(&self, user_id: i64, age_id: i64) -> Result<Option<(i64, String, i64)>, sqlx::Error> {
        let row: Option<(i64, String, i64)> = sqlx::query_as(
            "SELECT id, state_json, last_tick FROM provinces
             WHERE user_id = ? AND age_id = ? AND is_active = 1"
        )
        .bind(user_id)
        .bind(age_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// Check if user has a province in the current age
    #[allow(dead_code)] // Reserved for future UI implementation
    pub async fn has_province(&self, user_id: i64, age_id: i64) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM provinces WHERE user_id = ? AND age_id = ? AND is_active = 1"
        )
        .bind(user_id)
        .bind(age_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    /// Create a new kingdom
    #[allow(dead_code)] // Reserved for kingdom UI implementation
    pub async fn create_kingdom(
        &self,
        age_id: i64,
        name: &str,
        motto: Option<&str>,
        ruler_user_id: i64,
        ruler_province_id: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO kingdoms (age_id, name, motto, ruler_user_id, created_at)
             VALUES (?, ?, ?, ?, datetime('now'))"
        )
        .bind(age_id)
        .bind(name)
        .bind(motto)
        .bind(ruler_user_id)
        .execute(&self.pool)
        .await?;

        let kingdom_id = result.last_insert_rowid();

        // Add ruler as first member
        sqlx::query(
            "INSERT INTO kingdom_members (kingdom_id, user_id, province_id, joined_at, is_ruler)
             VALUES (?, ?, ?, datetime('now'), 1)"
        )
        .bind(kingdom_id)
        .bind(ruler_user_id)
        .bind(ruler_province_id)
        .execute(&self.pool)
        .await?;

        // Update province with kingdom
        sqlx::query("UPDATE provinces SET kingdom_id = ? WHERE id = ?")
            .bind(kingdom_id)
            .bind(ruler_province_id)
            .execute(&self.pool)
            .await?;

        Ok(kingdom_id)
    }

    /// Get kingdom member count
    #[allow(dead_code)] // Reserved for kingdom UI implementation
    pub async fn get_kingdom_member_count(&self, kingdom_id: i64) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM kingdom_members WHERE kingdom_id = ?"
        )
        .bind(kingdom_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    /// Join a kingdom (max 10 members)
    #[allow(dead_code)] // Reserved for kingdom UI implementation
    pub async fn join_kingdom(
        &self,
        kingdom_id: i64,
        user_id: i64,
        province_id: i64,
    ) -> Result<bool, sqlx::Error> {
        // Check member count
        let count = self.get_kingdom_member_count(kingdom_id).await?;
        if count >= 10 {
            return Ok(false);
        }

        sqlx::query(
            "INSERT INTO kingdom_members (kingdom_id, user_id, province_id, joined_at, is_ruler)
             VALUES (?, ?, ?, datetime('now'), 0)"
        )
        .bind(kingdom_id)
        .bind(user_id)
        .bind(province_id)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE provinces SET kingdom_id = ? WHERE id = ?")
            .bind(kingdom_id)
            .bind(province_id)
            .execute(&self.pool)
            .await?;

        Ok(true)
    }

    /// Record an attack
    #[allow(dead_code)] // Reserved for combat UI implementation
    pub async fn record_attack(
        &self,
        age_id: i64,
        attacker_id: i64,
        defender_id: i64,
        attack_type: &str,
        success: bool,
        land_captured: u32,
        gold_stolen: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO attacks (age_id, attacker_id, defender_id, attack_type, success, land_captured, gold_stolen, attacked_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(age_id)
        .bind(attacker_id)
        .bind(defender_id)
        .bind(attack_type)
        .bind(success as i32)
        .bind(land_captured)
        .bind(gold_stolen)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Record age completion for leaderboard
    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        age_id: i64,
        final_networth: i64,
        final_land: u32,
        attacks_won: u32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, age_id, final_networth, final_land, attacks_won, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(age_id)
        .bind(final_networth)
        .bind(final_land)
        .bind(attacks_won)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get leaderboard (top provinces by networth)
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY final_networth DESC) as rank,
                handle,
                final_networth,
                final_land,
                attacks_won,
                completed_at
            FROM completions
            ORDER BY final_networth DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, networth, land, attacks, completed)| {
            LeaderboardEntry {
                rank,
                handle,
                final_networth: networth,
                final_land: land as u32,
                attacks_won: attacks as u32,
                completed_at: completed,
            }
        }).collect())
    }

    /// Get provinces for ranking in current age
    #[allow(dead_code)] // Reserved for leaderboard UI implementation
    pub async fn get_province_rankings(&self, age_id: i64, limit: i64) -> Result<Vec<ProvinceRanking>, sqlx::Error> {
        // This would need a more complex query to calculate networth from state_json
        // For now, return empty - in production would use JSON functions or pre-calculated field
        let rows: Vec<(i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT id, handle, last_tick
            FROM provinces
            WHERE age_id = ? AND is_active = 1
            ORDER BY last_tick DESC
            LIMIT ?
            "#
        )
        .bind(age_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().enumerate().map(|(i, (id, handle, _))| {
            ProvinceRanking {
                rank: (i + 1) as i64,
                province_id: id,
                handle,
                networth: 0, // Would calculate from state
                land: 0,
            }
        }).collect())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used for leaderboard display
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub final_networth: i64,
    pub final_land: u32,
    pub attacks_won: u32,
    pub completed_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for rankings UI
pub struct ProvinceRanking {
    pub rank: i64,
    pub province_id: i64,
    pub handle: String,
    pub networth: i64,
    pub land: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_db_init() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_dystopia.db");

        let db = DystopiaDb::new(&db_path).await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_active_age() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_dystopia.db");
        let db = DystopiaDb::new(&db_path).await.unwrap();

        let age_id = db.get_active_age_id().await;
        assert!(age_id.is_ok());
        assert!(age_id.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_save_load_province() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_dystopia.db");
        let db = DystopiaDb::new(&db_path).await.unwrap();

        let age_id = db.get_active_age_id().await.unwrap();
        let user_id = 1;
        let handle = "TestUser";
        let state_json = r#"{"name":"Test","race":"human"}"#;

        // Save
        let province_id = db.save_province(user_id, handle, age_id, state_json, 0).await.unwrap();
        assert!(province_id > 0);

        // Load
        let loaded = db.load_province(user_id, age_id).await.unwrap();
        assert!(loaded.is_some());
        let (id, json, tick) = loaded.unwrap();
        assert_eq!(id, province_id);
        assert_eq!(json, state_json);
        assert_eq!(tick, 0);
    }
}
