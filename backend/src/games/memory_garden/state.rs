//! Memory Garden state types
//!
//! Unlike games, Memory Garden doesn't have per-user save state.
//! Instead, it maintains view state for navigation and the shared
//! memories database.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Maximum length for memory content
pub const MAX_MEMORY_LENGTH: usize = 280;

/// Maximum flags a user can submit per day
pub const MAX_FLAGS_PER_DAY: u32 = 3;

/// Edit window in seconds (1 hour)
pub const EDIT_WINDOW_SECONDS: i64 = 3600;

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: i64,
    pub user_id: Option<i64>,  // None for system-generated
    pub handle: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_system_generated: bool,
    pub milestone_type: Option<MilestoneType>,
    pub is_flagged: bool,
    pub flag_count: u32,
}

impl Memory {
    /// Check if this memory can still be edited (within 1 hour of creation)
    pub fn can_edit(&self, now: DateTime<Utc>) -> bool {
        if self.is_system_generated {
            return false;
        }
        let age = now.signed_duration_since(self.created_at);
        age.num_seconds() <= EDIT_WINDOW_SECONDS
    }

    /// Check if the given user owns this memory
    pub fn is_owned_by(&self, user_id: i64) -> bool {
        self.user_id == Some(user_id)
    }

    /// Format the memory's date for display
    pub fn display_date(&self) -> String {
        self.created_at.format("%B %d, %Y").to_string()
    }

    /// Format short date for listings
    pub fn short_date(&self) -> String {
        self.created_at.format("%m/%d/%y").to_string()
    }
}

/// Types of system-generated milestone memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneType {
    /// Total registered users milestone
    Users,
    /// Total sessions milestone
    Sessions,
    /// Total time spent milestone (hours)
    Time,
    /// BBS birth (the first memory)
    Birth,
}

impl MilestoneType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MilestoneType::Users => "users",
            MilestoneType::Sessions => "sessions",
            MilestoneType::Time => "time",
            MilestoneType::Birth => "birth",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "users" => Some(MilestoneType::Users),
            "sessions" => Some(MilestoneType::Sessions),
            "time" => Some(MilestoneType::Time),
            "birth" => Some(MilestoneType::Birth),
            _ => None,
        }
    }
}

/// A flag report on a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFlag {
    pub id: i64,
    pub memory_id: i64,
    pub reporter_id: i64,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
    pub resolution: Option<FlagResolution>,
}

/// How a flag was resolved by sysop
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlagResolution {
    /// Memory was removed
    Removed,
    /// Flag was dismissed (memory OK)
    Dismissed,
}

/// View state for Memory Garden navigation
/// This is transient per-session, not persisted
#[derive(Debug, Clone)]
pub struct ViewState {
    /// Current page (0-indexed)
    pub current_page: usize,
    /// Items per page
    pub page_size: usize,
    /// Total memories available
    pub total_memories: usize,
    /// Currently selected memory index (if any)
    pub selected_index: Option<usize>,
    /// Filter: specific date to view
    pub filter_date: Option<String>,
    /// Filter: show only own memories
    pub show_own_only: bool,
    /// Message to display (cleared after showing)
    pub message: Option<String>,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            current_page: 0,
            page_size: 10,
            total_memories: 0,
            selected_index: None,
            filter_date: None,
            show_own_only: false,
            message: None,
        }
    }

    /// Total number of pages
    pub fn total_pages(&self) -> usize {
        if self.total_memories == 0 {
            1
        } else {
            (self.total_memories + self.page_size - 1) / self.page_size
        }
    }

    /// Can go to previous page?
    pub fn has_prev_page(&self) -> bool {
        self.current_page > 0
    }

    /// Can go to next page?
    pub fn has_next_page(&self) -> bool {
        self.current_page + 1 < self.total_pages()
    }

    /// Go to previous page
    pub fn prev_page(&mut self) {
        if self.has_prev_page() {
            self.current_page -= 1;
        }
    }

    /// Go to next page
    pub fn next_page(&mut self) {
        if self.has_next_page() {
            self.current_page += 1;
        }
    }

    /// Reset to first page
    pub fn reset_page(&mut self) {
        self.current_page = 0;
        self.selected_index = None;
    }

    /// Set a temporary message
    pub fn set_message(&mut self, msg: &str) {
        self.message = Some(msg.to_string());
    }

    /// Take and clear the message
    pub fn take_message(&mut self) -> Option<String> {
        self.message.take()
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new()
    }
}

/// BBS statistics for milestone tracking
#[derive(Debug, Clone, Default)]
pub struct BbsStats {
    pub total_users: i64,
    pub total_sessions: i64,
    pub total_time_seconds: i64,
    pub last_user_milestone: i64,
    pub last_session_milestone: i64,
    pub last_time_milestone: i64,
}

impl BbsStats {
    /// Check if we've crossed a new milestone threshold
    /// Milestones at: 10, 100, 1000, 10000, 100000, 1000000
    pub fn check_milestone(current: i64, last_milestone: i64) -> Option<i64> {
        if current < 10 {
            return None;
        }

        // Calculate the milestone threshold we just crossed
        let current_power = (current as f64).log10().floor() as i64;
        let threshold = 10i64.pow(current_power as u32);

        if threshold > last_milestone && current >= threshold {
            Some(threshold)
        } else {
            None
        }
    }

    /// Format milestone number for display
    pub fn format_milestone(value: i64) -> String {
        if value >= 1_000_000 {
            format!("{}M", value / 1_000_000)
        } else if value >= 1_000 {
            format!("{}K", value / 1_000)
        } else {
            value.to_string()
        }
    }

    /// Format time in hours
    pub fn format_hours(seconds: i64) -> String {
        let hours = seconds / 3600;
        BbsStats::format_milestone(hours)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_memory_can_edit_within_window() {
        let now = Utc::now();
        let memory = Memory {
            id: 1,
            user_id: Some(1),
            handle: Some("test".to_string()),
            content: "Test memory".to_string(),
            created_at: now - Duration::minutes(30),
            updated_at: None,
            is_system_generated: false,
            milestone_type: None,
            is_flagged: false,
            flag_count: 0,
        };

        assert!(memory.can_edit(now));
    }

    #[test]
    fn test_memory_cannot_edit_after_window() {
        let now = Utc::now();
        let memory = Memory {
            id: 1,
            user_id: Some(1),
            handle: Some("test".to_string()),
            content: "Test memory".to_string(),
            created_at: now - Duration::hours(2),
            updated_at: None,
            is_system_generated: false,
            milestone_type: None,
            is_flagged: false,
            flag_count: 0,
        };

        assert!(!memory.can_edit(now));
    }

    #[test]
    fn test_system_memory_cannot_edit() {
        let now = Utc::now();
        let memory = Memory {
            id: 1,
            user_id: None,
            handle: None,
            content: "System memory".to_string(),
            created_at: now,
            updated_at: None,
            is_system_generated: true,
            milestone_type: Some(MilestoneType::Birth),
            is_flagged: false,
            flag_count: 0,
        };

        assert!(!memory.can_edit(now));
    }

    #[test]
    fn test_view_state_pagination() {
        let mut state = ViewState::new();
        state.total_memories = 25;
        state.page_size = 10;

        assert_eq!(state.total_pages(), 3);
        assert!(!state.has_prev_page());
        assert!(state.has_next_page());

        state.next_page();
        assert_eq!(state.current_page, 1);
        assert!(state.has_prev_page());
        assert!(state.has_next_page());

        state.next_page();
        assert_eq!(state.current_page, 2);
        assert!(!state.has_next_page());

        state.prev_page();
        assert_eq!(state.current_page, 1);
    }

    #[test]
    fn test_milestone_detection() {
        // No milestone below 10
        assert_eq!(BbsStats::check_milestone(5, 0), None);
        assert_eq!(BbsStats::check_milestone(9, 0), None);

        // First milestone at 10
        assert_eq!(BbsStats::check_milestone(10, 0), Some(10));
        assert_eq!(BbsStats::check_milestone(15, 0), Some(10));

        // Already passed 10, no new milestone
        assert_eq!(BbsStats::check_milestone(15, 10), None);

        // Milestone at 100
        assert_eq!(BbsStats::check_milestone(100, 10), Some(100));
        assert_eq!(BbsStats::check_milestone(150, 10), Some(100));

        // Milestone at 1000
        assert_eq!(BbsStats::check_milestone(1000, 100), Some(1000));

        // Large milestones
        assert_eq!(BbsStats::check_milestone(10000, 1000), Some(10000));
        assert_eq!(BbsStats::check_milestone(100000, 10000), Some(100000));
        assert_eq!(BbsStats::check_milestone(1000000, 100000), Some(1000000));
    }

    #[test]
    fn test_format_milestone() {
        assert_eq!(BbsStats::format_milestone(10), "10");
        assert_eq!(BbsStats::format_milestone(100), "100");
        assert_eq!(BbsStats::format_milestone(1000), "1K");
        assert_eq!(BbsStats::format_milestone(10000), "10K");
        assert_eq!(BbsStats::format_milestone(100000), "100K");
        assert_eq!(BbsStats::format_milestone(1000000), "1M");
    }

    #[test]
    fn test_milestone_type_roundtrip() {
        for mt in [MilestoneType::Users, MilestoneType::Sessions, MilestoneType::Time, MilestoneType::Birth] {
            assert_eq!(MilestoneType::from_str(mt.as_str()), Some(mt));
        }
    }
}
