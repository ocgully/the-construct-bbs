//! Memory Garden Database Layer
//!
//! Manages persistence for memories, flags, and BBS statistics.
//! Uses a separate SQLite database: data/memory_garden.db

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use chrono::{DateTime, Utc};
use crate::games::memory_garden::{Memory, MilestoneType, FlagResolution};
use crate::games::memory_garden::state::BbsStats;

pub struct MemoryGardenDb {
    pool: SqlitePool,
}

impl MemoryGardenDb {
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
        db.ensure_birth_memory().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // Memories table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER,
                handle TEXT,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT,
                is_system_generated INTEGER NOT NULL DEFAULT 0,
                milestone_type TEXT,
                is_flagged INTEGER NOT NULL DEFAULT 0,
                flag_count INTEGER NOT NULL DEFAULT 0,
                is_deleted INTEGER NOT NULL DEFAULT 0
            )
        "#).execute(&self.pool).await?;

        // Indexes for efficient querying
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_memories_created_at
            ON memories(created_at DESC)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_memories_user_id
            ON memories(user_id)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_memories_date
            ON memories(date(created_at))
        "#).execute(&self.pool).await?;

        // Flags table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memory_flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                memory_id INTEGER NOT NULL,
                reporter_id INTEGER NOT NULL,
                reason TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                resolved INTEGER NOT NULL DEFAULT 0,
                resolution TEXT,
                resolved_at TEXT,
                resolved_by INTEGER,
                FOREIGN KEY (memory_id) REFERENCES memories(id)
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_flags_memory_id
            ON memory_flags(memory_id)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_flags_reporter_date
            ON memory_flags(reporter_id, date(created_at))
        "#).execute(&self.pool).await?;

        // BBS Stats table (single row)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS bbs_stats (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                total_users INTEGER NOT NULL DEFAULT 0,
                total_sessions INTEGER NOT NULL DEFAULT 0,
                total_time_seconds INTEGER NOT NULL DEFAULT 0,
                last_user_milestone INTEGER NOT NULL DEFAULT 0,
                last_session_milestone INTEGER NOT NULL DEFAULT 0,
                last_time_milestone INTEGER NOT NULL DEFAULT 0
            )
        "#).execute(&self.pool).await?;

        // Initialize stats row if not exists
        sqlx::query(r#"
            INSERT OR IGNORE INTO bbs_stats (id) VALUES (1)
        "#).execute(&self.pool).await?;

        // Daily posts tracking (for 1 post per day limit)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS daily_posts (
                user_id INTEGER NOT NULL,
                post_date TEXT NOT NULL,
                PRIMARY KEY (user_id, post_date)
            )
        "#).execute(&self.pool).await?;

        // Daily flags tracking (for 3 flags per day limit)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS daily_flags (
                user_id INTEGER NOT NULL,
                flag_date TEXT NOT NULL,
                flag_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (user_id, flag_date)
            )
        "#).execute(&self.pool).await?;

        Ok(())
    }

    /// Ensure the birth memory exists (first memory, dated 1/25/2026)
    async fn ensure_birth_memory(&self) -> Result<(), sqlx::Error> {
        let exists: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM memories WHERE milestone_type = 'birth' LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        if exists.is_none() {
            sqlx::query(r#"
                INSERT INTO memories (content, created_at, is_system_generated, milestone_type)
                VALUES ('I was born.', '2026-01-25 00:00:00', 1, 'birth')
            "#)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    // ========================================================================
    // MEMORY CRUD OPERATIONS
    // ========================================================================

    /// Create a new memory
    pub async fn create_memory(
        &self,
        user_id: i64,
        handle: &str,
        content: &str,
    ) -> Result<i64, sqlx::Error> {
        // Use Eastern timezone for date tracking
        let now = Utc::now();
        let today = now.format("%Y-%m-%d").to_string();

        // Record daily post
        sqlx::query(
            "INSERT OR REPLACE INTO daily_posts (user_id, post_date) VALUES (?, ?)"
        )
        .bind(user_id)
        .bind(&today)
        .execute(&self.pool)
        .await?;

        // Create memory
        let result = sqlx::query(
            r#"INSERT INTO memories (user_id, handle, content, created_at)
               VALUES (?, ?, ?, datetime('now'))"#
        )
        .bind(user_id)
        .bind(handle)
        .bind(content)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Create a system-generated milestone memory
    pub async fn create_milestone_memory(
        &self,
        content: &str,
        milestone_type: MilestoneType,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"INSERT INTO memories (content, created_at, is_system_generated, milestone_type)
               VALUES (?, datetime('now'), 1, ?)"#
        )
        .bind(content)
        .bind(milestone_type.as_str())
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Update a memory (only content can be changed)
    pub async fn update_memory(
        &self,
        memory_id: i64,
        user_id: i64,
        content: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE memories
               SET content = ?, updated_at = datetime('now')
               WHERE id = ? AND user_id = ? AND is_system_generated = 0"#
        )
        .bind(content)
        .bind(memory_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a memory (soft delete - mark as deleted)
    pub async fn delete_memory(
        &self,
        memory_id: i64,
        user_id: i64,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE memories SET is_deleted = 1 WHERE id = ? AND user_id = ?"
        )
        .bind(memory_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get a single memory by ID
    pub async fn get_memory(&self, memory_id: i64) -> Result<Option<Memory>, sqlx::Error> {
        let row: Option<MemoryRow> = sqlx::query_as(
            r#"SELECT id, user_id, handle, content, created_at, updated_at,
                      is_system_generated, milestone_type, is_flagged, flag_count
               FROM memories
               WHERE id = ? AND is_deleted = 0"#
        )
        .bind(memory_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_memory()))
    }

    /// Get paginated memories (newest first)
    pub async fn get_memories(
        &self,
        page: usize,
        page_size: usize,
        filter_date: Option<&str>,
        user_id_filter: Option<i64>,
    ) -> Result<(Vec<Memory>, usize), sqlx::Error> {
        let offset = page * page_size;

        // Build query based on filters
        let (query, count_query) = if let Some(_date) = filter_date {
            (
                format!(
                    r#"SELECT id, user_id, handle, content, created_at, updated_at,
                              is_system_generated, milestone_type, is_flagged, flag_count
                       FROM memories
                       WHERE is_deleted = 0 AND is_flagged = 0 AND date(created_at) = ?
                       ORDER BY created_at DESC
                       LIMIT ? OFFSET ?"#
                ),
                format!(
                    "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND is_flagged = 0 AND date(created_at) = ?"
                ),
            )
        } else if let Some(_uid) = user_id_filter {
            (
                format!(
                    r#"SELECT id, user_id, handle, content, created_at, updated_at,
                              is_system_generated, milestone_type, is_flagged, flag_count
                       FROM memories
                       WHERE is_deleted = 0 AND user_id = ?
                       ORDER BY created_at DESC
                       LIMIT ? OFFSET ?"#
                ),
                format!(
                    "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND user_id = ?"
                ),
            )
        } else {
            (
                format!(
                    r#"SELECT id, user_id, handle, content, created_at, updated_at,
                              is_system_generated, milestone_type, is_flagged, flag_count
                       FROM memories
                       WHERE is_deleted = 0 AND is_flagged = 0
                       ORDER BY created_at DESC
                       LIMIT ? OFFSET ?"#
                ),
                format!(
                    "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND is_flagged = 0"
                ),
            )
        };

        // Execute queries based on filter type
        let rows: Vec<MemoryRow>;
        let total: i64;

        if let Some(date) = filter_date {
            rows = sqlx::query_as(&query)
                .bind(date)
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?;

            let count_row: (i64,) = sqlx::query_as(&count_query)
                .bind(date)
                .fetch_one(&self.pool)
                .await?;
            total = count_row.0;
        } else if let Some(uid) = user_id_filter {
            rows = sqlx::query_as(&query)
                .bind(uid)
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?;

            let count_row: (i64,) = sqlx::query_as(&count_query)
                .bind(uid)
                .fetch_one(&self.pool)
                .await?;
            total = count_row.0;
        } else {
            rows = sqlx::query_as(&query)
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?;

            let count_row: (i64,) = sqlx::query_as(&count_query)
                .fetch_one(&self.pool)
                .await?;
            total = count_row.0;
        }

        let memories = rows.into_iter().map(|r| r.into_memory()).collect();
        Ok((memories, total as usize))
    }

    /// Get random recent memories for welcome screen
    pub async fn get_random_memories(&self, count: usize) -> Result<Vec<Memory>, sqlx::Error> {
        let rows: Vec<MemoryRow> = sqlx::query_as(
            r#"SELECT id, user_id, handle, content, created_at, updated_at,
                      is_system_generated, milestone_type, is_flagged, flag_count
               FROM memories
               WHERE is_deleted = 0 AND is_flagged = 0
               ORDER BY RANDOM()
               LIMIT ?"#
        )
        .bind(count as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_memory()).collect())
    }

    // ========================================================================
    // FLAG OPERATIONS
    // ========================================================================

    /// Flag a memory
    pub async fn flag_memory(
        &self,
        memory_id: i64,
        reporter_id: i64,
        reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        // Update daily flags count
        sqlx::query(
            r#"INSERT INTO daily_flags (user_id, flag_date, flag_count)
               VALUES (?, ?, 1)
               ON CONFLICT(user_id, flag_date) DO UPDATE SET flag_count = flag_count + 1"#
        )
        .bind(reporter_id)
        .bind(&today)
        .execute(&self.pool)
        .await?;

        // Create flag record
        sqlx::query(
            r#"INSERT INTO memory_flags (memory_id, reporter_id, reason, created_at)
               VALUES (?, ?, ?, datetime('now'))"#
        )
        .bind(memory_id)
        .bind(reporter_id)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        // Update memory flag count and status
        sqlx::query(
            r#"UPDATE memories
               SET flag_count = flag_count + 1,
                   is_flagged = CASE WHEN flag_count >= 2 THEN 1 ELSE is_flagged END
               WHERE id = ?"#
        )
        .bind(memory_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get flags remaining for user today
    pub async fn get_flags_remaining(&self, user_id: i64) -> Result<u32, sqlx::Error> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT flag_count FROM daily_flags WHERE user_id = ? AND flag_date = ?"
        )
        .bind(user_id)
        .bind(&today)
        .fetch_optional(&self.pool)
        .await?;

        let used = row.map(|r| r.0 as u32).unwrap_or(0);
        Ok(3u32.saturating_sub(used))
    }

    /// Get flagged memories for sysop review
    #[allow(dead_code)]
    pub async fn get_flagged_memories(&self) -> Result<Vec<Memory>, sqlx::Error> {
        let rows: Vec<MemoryRow> = sqlx::query_as(
            r#"SELECT id, user_id, handle, content, created_at, updated_at,
                      is_system_generated, milestone_type, is_flagged, flag_count
               FROM memories
               WHERE is_deleted = 0 AND is_flagged = 1
               ORDER BY flag_count DESC, created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_memory()).collect())
    }

    /// Resolve a flag (sysop action)
    #[allow(dead_code)]
    pub async fn resolve_flag(
        &self,
        memory_id: i64,
        resolution: FlagResolution,
        sysop_id: i64,
    ) -> Result<(), sqlx::Error> {
        let resolution_str = match resolution {
            FlagResolution::Removed => "removed",
            FlagResolution::Dismissed => "dismissed",
        };

        // Update flags
        sqlx::query(
            r#"UPDATE memory_flags
               SET resolved = 1, resolution = ?, resolved_at = datetime('now'), resolved_by = ?
               WHERE memory_id = ? AND resolved = 0"#
        )
        .bind(resolution_str)
        .bind(sysop_id)
        .bind(memory_id)
        .execute(&self.pool)
        .await?;

        // Update memory based on resolution
        match resolution {
            FlagResolution::Removed => {
                sqlx::query("UPDATE memories SET is_deleted = 1 WHERE id = ?")
                    .bind(memory_id)
                    .execute(&self.pool)
                    .await?;
            }
            FlagResolution::Dismissed => {
                sqlx::query("UPDATE memories SET is_flagged = 0, flag_count = 0 WHERE id = ?")
                    .bind(memory_id)
                    .execute(&self.pool)
                    .await?;
            }
        }

        Ok(())
    }

    // ========================================================================
    // DAILY LIMITS
    // ========================================================================

    /// Check if user has already posted today
    pub async fn has_posted_today(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let today = Utc::now().format("%Y-%m-%d").to_string();

        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM daily_posts WHERE user_id = ? AND post_date = ?"
        )
        .bind(user_id)
        .bind(&today)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    // ========================================================================
    // BBS STATS & MILESTONES
    // ========================================================================

    /// Get current BBS stats
    pub async fn get_stats(&self) -> Result<BbsStats, sqlx::Error> {
        let row: (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
            r#"SELECT total_users, total_sessions, total_time_seconds,
                      last_user_milestone, last_session_milestone, last_time_milestone
               FROM bbs_stats WHERE id = 1"#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(BbsStats {
            total_users: row.0,
            total_sessions: row.1,
            total_time_seconds: row.2,
            last_user_milestone: row.3,
            last_session_milestone: row.4,
            last_time_milestone: row.5,
        })
    }

    /// Increment user count and check for milestone
    #[allow(dead_code)]
    pub async fn increment_users(&self) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query("UPDATE bbs_stats SET total_users = total_users + 1 WHERE id = 1")
            .execute(&self.pool)
            .await?;

        let stats = self.get_stats().await?;
        if let Some(milestone) = BbsStats::check_milestone(stats.total_users, stats.last_user_milestone) {
            sqlx::query("UPDATE bbs_stats SET last_user_milestone = ? WHERE id = 1")
                .bind(milestone)
                .execute(&self.pool)
                .await?;

            let content = format!(
                "The garden welcomes its {}th soul.",
                BbsStats::format_milestone(milestone)
            );
            self.create_milestone_memory(&content, MilestoneType::Users).await?;

            return Ok(Some(milestone));
        }

        Ok(None)
    }

    /// Increment session count and check for milestone
    pub async fn increment_sessions(&self) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query("UPDATE bbs_stats SET total_sessions = total_sessions + 1 WHERE id = 1")
            .execute(&self.pool)
            .await?;

        let stats = self.get_stats().await?;
        if let Some(milestone) = BbsStats::check_milestone(stats.total_sessions, stats.last_session_milestone) {
            sqlx::query("UPDATE bbs_stats SET last_session_milestone = ? WHERE id = 1")
                .bind(milestone)
                .execute(&self.pool)
                .await?;

            let content = format!(
                "The garden has been visited {} times.",
                BbsStats::format_milestone(milestone)
            );
            self.create_milestone_memory(&content, MilestoneType::Sessions).await?;

            return Ok(Some(milestone));
        }

        Ok(None)
    }

    /// Add time spent and check for milestone (in seconds)
    #[allow(dead_code)]
    pub async fn add_time(&self, seconds: i64) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query("UPDATE bbs_stats SET total_time_seconds = total_time_seconds + ? WHERE id = 1")
            .bind(seconds)
            .execute(&self.pool)
            .await?;

        let stats = self.get_stats().await?;
        let hours = stats.total_time_seconds / 3600;

        if let Some(milestone) = BbsStats::check_milestone(hours, stats.last_time_milestone) {
            sqlx::query("UPDATE bbs_stats SET last_time_milestone = ? WHERE id = 1")
                .bind(milestone)
                .execute(&self.pool)
                .await?;

            let content = format!(
                "Together, we have spent {} hours in the garden.",
                BbsStats::format_milestone(milestone)
            );
            self.create_milestone_memory(&content, MilestoneType::Time).await?;

            return Ok(Some(milestone));
        }

        Ok(None)
    }
}

/// Internal row type for SQLite queries
#[derive(sqlx::FromRow)]
struct MemoryRow {
    id: i64,
    user_id: Option<i64>,
    handle: Option<String>,
    content: String,
    created_at: String,
    updated_at: Option<String>,
    is_system_generated: i64,
    milestone_type: Option<String>,
    is_flagged: i64,
    flag_count: i64,
}

impl MemoryRow {
    fn into_memory(self) -> Memory {
        Memory {
            id: self.id,
            user_id: self.user_id,
            handle: self.handle,
            content: self.content,
            created_at: parse_datetime(&self.created_at),
            updated_at: self.updated_at.as_ref().map(|s| parse_datetime(s)),
            is_system_generated: self.is_system_generated != 0,
            milestone_type: self.milestone_type.as_ref().and_then(|s| MilestoneType::from_str(s)),
            is_flagged: self.is_flagged != 0,
            flag_count: self.flag_count as u32,
        }
    }
}

/// Parse SQLite datetime string to DateTime<Utc>
fn parse_datetime(s: &str) -> DateTime<Utc> {
    // Try parsing as full datetime first
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return DateTime::from_naive_utc_and_offset(dt, Utc);
    }

    // Try date only
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = d.and_hms_opt(0, 0, 0).unwrap();
        return DateTime::from_naive_utc_and_offset(dt, Utc);
    }

    // Fallback to now
    Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> MemoryGardenDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        MemoryGardenDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_birth_memory_exists() {
        let db = create_test_db().await;
        let memory = db.get_memory(1).await.unwrap();
        assert!(memory.is_some());
        let m = memory.unwrap();
        assert!(m.is_system_generated);
        assert_eq!(m.milestone_type, Some(MilestoneType::Birth));
        assert!(m.content.contains("born"));
    }

    #[tokio::test]
    async fn test_create_memory() {
        let db = create_test_db().await;
        let id = db.create_memory(1, "testuser", "Hello world!").await.unwrap();
        assert!(id > 0);

        let memory = db.get_memory(id).await.unwrap().unwrap();
        assert_eq!(memory.content, "Hello world!");
        assert_eq!(memory.handle, Some("testuser".to_string()));
    }

    #[tokio::test]
    async fn test_update_memory() {
        let db = create_test_db().await;
        let id = db.create_memory(1, "testuser", "Original").await.unwrap();

        let updated = db.update_memory(id, 1, "Updated content").await.unwrap();
        assert!(updated);

        let memory = db.get_memory(id).await.unwrap().unwrap();
        assert_eq!(memory.content, "Updated content");
        assert!(memory.updated_at.is_some());
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let db = create_test_db().await;
        let id = db.create_memory(1, "testuser", "To delete").await.unwrap();

        let deleted = db.delete_memory(id, 1).await.unwrap();
        assert!(deleted);

        // Should not be visible anymore
        let memory = db.get_memory(id).await.unwrap();
        assert!(memory.is_none());
    }

    #[tokio::test]
    async fn test_posted_today() {
        let db = create_test_db().await;

        // Initially not posted
        let posted = db.has_posted_today(1).await.unwrap();
        assert!(!posted);

        // Create a memory
        db.create_memory(1, "testuser", "Today's memory").await.unwrap();

        // Now should show as posted
        let posted = db.has_posted_today(1).await.unwrap();
        assert!(posted);
    }

    #[tokio::test]
    async fn test_flag_memory() {
        let db = create_test_db().await;
        let id = db.create_memory(1, "testuser", "Controversial").await.unwrap();

        // Initial flags remaining
        let remaining = db.get_flags_remaining(2).await.unwrap();
        assert_eq!(remaining, 3);

        // Flag the memory
        db.flag_memory(id, 2, Some("inappropriate")).await.unwrap();

        // Check flags remaining decreased
        let remaining = db.get_flags_remaining(2).await.unwrap();
        assert_eq!(remaining, 2);

        // Check memory has flag count increased
        let memory = db.get_memory(id).await.unwrap().unwrap();
        assert_eq!(memory.flag_count, 1);
    }

    #[tokio::test]
    async fn test_pagination() {
        let db = create_test_db().await;

        // Create 15 memories
        for i in 0..15 {
            db.create_memory(1, "testuser", &format!("Memory {}", i)).await.unwrap();
        }

        // Get first page (10 items)
        let (memories, total) = db.get_memories(0, 10, None, None).await.unwrap();
        assert_eq!(memories.len(), 10);
        assert_eq!(total, 16); // 15 + birth memory

        // Get second page
        let (memories, _) = db.get_memories(1, 10, None, None).await.unwrap();
        assert_eq!(memories.len(), 6);
    }

    #[tokio::test]
    async fn test_random_memories() {
        let db = create_test_db().await;

        for i in 0..10 {
            db.create_memory(1, "testuser", &format!("Memory {}", i)).await.unwrap();
        }

        let random = db.get_random_memories(5).await.unwrap();
        assert_eq!(random.len(), 5);
    }

    #[tokio::test]
    async fn test_milestone_creation() {
        let db = create_test_db().await;

        // Simulate reaching 10 users
        for _ in 0..9 {
            db.increment_users().await.unwrap();
        }
        let milestone = db.increment_users().await.unwrap();
        assert_eq!(milestone, Some(10));

        // Check milestone memory was created
        let (memories, _) = db.get_memories(0, 100, None, None).await.unwrap();
        let milestone_memory = memories.iter().find(|m| {
            m.is_system_generated && m.milestone_type == Some(MilestoneType::Users)
        });
        assert!(milestone_memory.is_some());
    }
}
