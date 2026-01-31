//! Database layer for Depths of Diablo
//!
//! Handles persistence of game state, meta-progression, and leaderboards.

#![allow(dead_code)]

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct DiabloDb {
    pool: SqlitePool,
}

impl DiabloDb {
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
        // Active saves (one per user)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Meta progression (persistent between runs)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS meta_progress (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                meta_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Run completions (for leaderboards)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                floor_reached INTEGER NOT NULL,
                soul_essence INTEGER NOT NULL,
                class TEXT NOT NULL,
                victory INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Character runs (history of all runs)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                run_seed INTEGER NOT NULL,
                class TEXT NOT NULL,
                floor_reached INTEGER NOT NULL,
                kills INTEGER NOT NULL,
                gold_earned INTEGER NOT NULL,
                soul_essence INTEGER NOT NULL,
                victory INTEGER NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Active lobbies (for multiplayer matchmaking)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS lobbies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_type TEXT NOT NULL,
                invite_code TEXT,
                seed INTEGER NOT NULL,
                host_user_id INTEGER NOT NULL,
                max_players INTEGER NOT NULL DEFAULT 4,
                state TEXT NOT NULL DEFAULT 'waiting',
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Lobby players
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS lobby_players (
                lobby_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                is_ready INTEGER NOT NULL DEFAULT 0,
                is_host INTEGER NOT NULL DEFAULT 0,
                selected_class TEXT,
                joined_at TEXT NOT NULL,
                PRIMARY KEY (lobby_id, user_id),
                FOREIGN KEY (lobby_id) REFERENCES lobbies(id) ON DELETE CASCADE
            )
        "#).execute(&self.pool).await?;

        // Active co-op sessions
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS coop_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                lobby_id INTEGER,
                seed INTEGER NOT NULL,
                current_floor INTEGER NOT NULL DEFAULT 1,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                FOREIGN KEY (lobby_id) REFERENCES lobbies(id)
            )
        "#).execute(&self.pool).await?;

        // Indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_floor
            ON completions(floor_reached DESC, soul_essence DESC)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_victory
            ON completions(victory DESC, floor_reached DESC)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_lobbies_state
            ON lobbies(state, match_type)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_lobby_players_user
            ON lobby_players(user_id)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    /// Save game state
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

    /// Save meta progression
    pub async fn save_meta(&self, user_id: i64, handle: &str, meta_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO meta_progress (user_id, handle, meta_json, updated_at)
             VALUES (?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(meta_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load meta progression
    pub async fn load_meta(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT meta_json FROM meta_progress WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Record run completion
    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        floor_reached: u32,
        soul_essence: i64,
        class: &str,
        victory: bool,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, floor_reached, soul_essence, class, victory, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(floor_reached as i64)
        .bind(soul_essence)
        .bind(class)
        .bind(victory as i64)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Record run history
    pub async fn record_run(
        &self,
        user_id: i64,
        run_seed: u64,
        class: &str,
        floor_reached: u32,
        kills: u64,
        gold_earned: i64,
        soul_essence: i64,
        victory: bool,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO runs (user_id, run_seed, class, floor_reached, kills, gold_earned, soul_essence, victory, started_at, ended_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'), datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(run_seed as i64)
        .bind(class)
        .bind(floor_reached as i64)
        .bind(kills as i64)
        .bind(gold_earned)
        .bind(soul_essence)
        .bind(victory as i64)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get leaderboard (by floor reached, then soul essence)
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY floor_reached DESC, soul_essence DESC) as rank,
                handle,
                floor_reached,
                soul_essence,
                class,
                victory
            FROM completions
            ORDER BY floor_reached DESC, soul_essence DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, floor, essence, class, victory)| {
            LeaderboardEntry {
                rank,
                handle,
                floor_reached: floor as u32,
                soul_essence: essence,
                class,
                victory: victory != 0,
            }
        }).collect())
    }

    /// Get victories leaderboard
    pub async fn get_victories_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY soul_essence DESC) as rank,
                handle,
                floor_reached,
                soul_essence,
                class,
                victory
            FROM completions
            WHERE victory = 1
            ORDER BY soul_essence DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, floor, essence, class, victory)| {
            LeaderboardEntry {
                rank,
                handle,
                floor_reached: floor as u32,
                soul_essence: essence,
                class,
                victory: victory != 0,
            }
        }).collect())
    }

    /// Get user's best run
    pub async fn get_user_best(&self, user_id: i64) -> Result<Option<LeaderboardEntry>, sqlx::Error> {
        let row: Option<(String, i64, i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT handle, floor_reached, soul_essence, class, victory
            FROM completions
            WHERE user_id = ?
            ORDER BY floor_reached DESC, soul_essence DESC
            LIMIT 1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(handle, floor, essence, class, victory)| {
            LeaderboardEntry {
                rank: 0,
                handle,
                floor_reached: floor as u32,
                soul_essence: essence,
                class,
                victory: victory != 0,
            }
        }))
    }

    /// Get user run count
    pub async fn get_user_run_count(&self, user_id: i64) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM runs WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    // ===== Lobby Management =====

    /// Create a new lobby
    pub async fn create_lobby(
        &self,
        match_type: &str,
        invite_code: Option<&str>,
        seed: u64,
        host_user_id: i64,
        host_handle: &str,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO lobbies (match_type, invite_code, seed, host_user_id, created_at)
             VALUES (?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(match_type)
        .bind(invite_code)
        .bind(seed as i64)
        .bind(host_user_id)
        .execute(&self.pool)
        .await?;

        let lobby_id = result.last_insert_rowid();

        // Add host as first player
        sqlx::query(
            "INSERT INTO lobby_players (lobby_id, user_id, handle, is_ready, is_host, joined_at)
             VALUES (?, ?, ?, 1, 1, datetime('now', '-5 hours'))"
        )
        .bind(lobby_id)
        .bind(host_user_id)
        .bind(host_handle)
        .execute(&self.pool)
        .await?;

        Ok(lobby_id)
    }

    /// Join a lobby
    pub async fn join_lobby(
        &self,
        lobby_id: i64,
        user_id: i64,
        handle: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO lobby_players (lobby_id, user_id, handle, is_ready, is_host, joined_at)
             VALUES (?, ?, ?, 0, 0, datetime('now', '-5 hours'))"
        )
        .bind(lobby_id)
        .bind(user_id)
        .bind(handle)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Leave a lobby
    pub async fn leave_lobby(&self, lobby_id: i64, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM lobby_players WHERE lobby_id = ? AND user_id = ?"
        )
        .bind(lobby_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a lobby and all its players
    pub async fn delete_lobby(&self, lobby_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM lobby_players WHERE lobby_id = ?")
            .bind(lobby_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM lobbies WHERE id = ?")
            .bind(lobby_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Find lobby by invite code
    pub async fn find_lobby_by_invite(&self, invite_code: &str) -> Result<Option<LobbyInfo>, sqlx::Error> {
        let row: Option<(i64, String, String, i64, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT l.id, l.match_type, l.invite_code, l.seed, l.host_user_id, l.max_players, l.state
            FROM lobbies l
            WHERE l.invite_code = ? AND l.state = 'waiting'
            "#
        )
        .bind(invite_code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, match_type, invite_code, seed, host_id, max_players, state)| {
            LobbyInfo {
                id,
                match_type,
                invite_code: Some(invite_code),
                seed: seed as u64,
                host_user_id: host_id,
                max_players: max_players as usize,
                state,
            }
        }))
    }

    /// Find available public lobbies
    pub async fn find_public_lobbies(&self, limit: i64) -> Result<Vec<LobbyInfo>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT l.id, l.match_type, l.seed, l.host_user_id, l.max_players, l.state
            FROM lobbies l
            WHERE l.match_type = 'public' AND l.state = 'waiting'
            ORDER BY l.created_at DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, match_type, seed, host_id, max_players, state)| {
            LobbyInfo {
                id,
                match_type,
                invite_code: None,
                seed: seed as u64,
                host_user_id: host_id,
                max_players: max_players as usize,
                state,
            }
        }).collect())
    }

    /// Get lobby player count
    pub async fn get_lobby_player_count(&self, lobby_id: i64) -> Result<usize, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM lobby_players WHERE lobby_id = ?"
        )
        .bind(lobby_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0 as usize)
    }

    /// Set player ready status
    pub async fn set_player_ready(
        &self,
        lobby_id: i64,
        user_id: i64,
        is_ready: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE lobby_players SET is_ready = ? WHERE lobby_id = ? AND user_id = ?"
        )
        .bind(is_ready as i64)
        .bind(lobby_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Set player class selection
    pub async fn set_player_class(
        &self,
        lobby_id: i64,
        user_id: i64,
        class: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE lobby_players SET selected_class = ? WHERE lobby_id = ? AND user_id = ?"
        )
        .bind(class)
        .bind(lobby_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update lobby state
    pub async fn update_lobby_state(&self, lobby_id: i64, state: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE lobbies SET state = ? WHERE id = ?")
            .bind(state)
            .bind(lobby_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get user's current lobby
    pub async fn get_user_lobby(&self, user_id: i64) -> Result<Option<i64>, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT lp.lobby_id FROM lobby_players lp
            JOIN lobbies l ON l.id = lp.lobby_id
            WHERE lp.user_id = ? AND l.state IN ('waiting', 'countdown')
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Create a co-op session from a lobby
    pub async fn create_coop_session(
        &self,
        lobby_id: i64,
        seed: u64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO coop_sessions (lobby_id, seed, current_floor, started_at)
             VALUES (?, ?, 1, datetime('now', '-5 hours'))"
        )
        .bind(lobby_id)
        .bind(seed as i64)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// End a co-op session
    pub async fn end_coop_session(&self, session_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE coop_sessions SET ended_at = datetime('now', '-5 hours') WHERE id = ?"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Clean up old lobbies (older than 24 hours)
    pub async fn cleanup_old_lobbies(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM lobbies WHERE created_at < datetime('now', '-5 hours', '-24 hours')"
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[derive(Debug, Clone)]
pub struct LobbyInfo {
    pub id: i64,
    pub match_type: String,
    pub invite_code: Option<String>,
    pub seed: u64,
    pub host_user_id: i64,
    pub max_players: usize,
    pub state: String,
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub floor_reached: u32,
    pub soul_essence: i64,
    pub class: String,
    pub victory: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_db_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        // Should be able to save and load
        db.save_game(1, "Test", r#"{"test": true}"#).await.unwrap();
        let loaded = db.load_game(1).await.unwrap();
        assert!(loaded.is_some());
        assert!(loaded.unwrap().contains("test"));
    }

    #[tokio::test]
    async fn test_meta_persistence() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        db.save_meta(1, "Test", r#"{"soul_essence": 100}"#).await.unwrap();
        let loaded = db.load_meta(1).await.unwrap();
        assert!(loaded.is_some());
        assert!(loaded.unwrap().contains("100"));
    }

    #[tokio::test]
    async fn test_leaderboard() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        // Record some completions
        db.record_completion(1, "Player1", 10, 500, "Warrior", false).await.unwrap();
        db.record_completion(2, "Player2", 15, 800, "Rogue", true).await.unwrap();
        db.record_completion(3, "Player3", 5, 200, "Sorcerer", false).await.unwrap();

        let leaderboard = db.get_leaderboard(10).await.unwrap();
        assert_eq!(leaderboard.len(), 3);
        assert_eq!(leaderboard[0].handle, "Player2"); // Highest floor
        assert_eq!(leaderboard[0].floor_reached, 15);
    }

    #[tokio::test]
    async fn test_delete_save() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        db.save_game(1, "Test", r#"{"test": true}"#).await.unwrap();
        assert!(db.load_game(1).await.unwrap().is_some());

        db.delete_save(1).await.unwrap();
        assert!(db.load_game(1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_create_lobby() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        let lobby_id = db.create_lobby("public", None, 12345, 1, "Host").await.unwrap();
        assert!(lobby_id > 0);

        let count = db.get_lobby_player_count(lobby_id).await.unwrap();
        assert_eq!(count, 1); // Just the host
    }

    #[tokio::test]
    async fn test_join_lobby() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        let lobby_id = db.create_lobby("public", None, 12345, 1, "Host").await.unwrap();
        db.join_lobby(lobby_id, 2, "Player2").await.unwrap();

        let count = db.get_lobby_player_count(lobby_id).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_find_public_lobbies() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        db.create_lobby("public", None, 12345, 1, "Host1").await.unwrap();
        db.create_lobby("public", None, 12346, 2, "Host2").await.unwrap();
        db.create_lobby("private", Some("ABC123"), 12347, 3, "Host3").await.unwrap();

        let lobbies = db.find_public_lobbies(10).await.unwrap();
        assert_eq!(lobbies.len(), 2); // Only public ones
    }

    #[tokio::test]
    async fn test_find_lobby_by_invite() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        db.create_lobby("private", Some("XYZ789"), 12345, 1, "Host").await.unwrap();

        let lobby = db.find_lobby_by_invite("XYZ789").await.unwrap();
        assert!(lobby.is_some());
        assert_eq!(lobby.unwrap().invite_code, Some("XYZ789".to_string()));

        // Non-existent code
        let no_lobby = db.find_lobby_by_invite("NOTEXIST").await.unwrap();
        assert!(no_lobby.is_none());
    }

    #[tokio::test]
    async fn test_player_ready_status() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        let lobby_id = db.create_lobby("public", None, 12345, 1, "Host").await.unwrap();
        db.join_lobby(lobby_id, 2, "Player2").await.unwrap();

        // Set player 2 ready
        db.set_player_ready(lobby_id, 2, true).await.unwrap();

        // Set player 2 class
        db.set_player_class(lobby_id, 2, "Warrior").await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_lobby() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        let lobby_id = db.create_lobby("public", None, 12345, 1, "Host").await.unwrap();
        db.join_lobby(lobby_id, 2, "Player2").await.unwrap();

        let count = db.get_lobby_player_count(lobby_id).await.unwrap();
        assert_eq!(count, 2);

        db.delete_lobby(lobby_id).await.unwrap();

        let count_after = db.get_lobby_player_count(lobby_id).await.unwrap();
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn test_get_user_lobby() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        let lobby_id = db.create_lobby("public", None, 12345, 1, "Host").await.unwrap();

        let user_lobby = db.get_user_lobby(1).await.unwrap();
        assert_eq!(user_lobby, Some(lobby_id));

        let no_lobby = db.get_user_lobby(999).await.unwrap();
        assert!(no_lobby.is_none());
    }

    #[tokio::test]
    async fn test_coop_session() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        let lobby_id = db.create_lobby("public", None, 12345, 1, "Host").await.unwrap();
        let session_id = db.create_coop_session(lobby_id, 12345).await.unwrap();
        assert!(session_id > 0);

        db.end_coop_session(session_id).await.unwrap();
    }
}
