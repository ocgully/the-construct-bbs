//! Memory Garden screen state machine
//!
//! Manages navigation and input handling for the Memory Garden feature.

use super::state::{Memory, ViewState, MAX_MEMORY_LENGTH};
use chrono::Utc;

/// Current screen in Memory Garden
#[derive(Debug, Clone, PartialEq)]
pub enum GardenScreen {
    /// Welcome screen with random recent memories
    Welcome,
    /// Main browsing view (paginated list)
    Browse,
    /// Viewing a single memory in detail
    ViewMemory { memory_id: i64 },
    /// Creating a new memory
    NewMemory,
    /// Editing own memory (within time window)
    EditMemory { memory_id: i64 },
    /// Confirm delete own memory
    ConfirmDelete { memory_id: i64 },
    /// Flag a memory for review
    FlagMemory { memory_id: i64 },
    /// View own memories only
    MyMemories,
    /// View memories from a specific date
    DateFilter,
    /// Confirm exit
    ConfirmQuit,
}

/// Actions returned by the flow for session handling
#[derive(Debug, Clone)]
pub enum GardenAction {
    /// Continue - no special action needed
    Continue,
    /// Render screen output
    Render(String),
    /// Echo characters back
    Echo(String),
    /// Exit to main menu
    Quit,
    /// Save a new memory (content provided)
    SaveNewMemory { content: String },
    /// Update existing memory
    UpdateMemory { memory_id: i64, content: String },
    /// Delete a memory
    DeleteMemory { memory_id: i64 },
    /// Flag a memory
    FlagMemory { memory_id: i64, reason: Option<String> },
    /// Load memories for display
    LoadMemories { page: usize, filter_date: Option<String>, own_only: bool },
    /// Load a single memory
    LoadMemory { memory_id: i64 },
    /// Load random recent memories for welcome screen
    LoadRandomMemories { count: usize },
    /// Check if user can post today
    CheckCanPost,
    /// Check flags remaining today
    CheckFlagsRemaining,
}

/// Memory Garden flow state machine
pub struct GardenFlow {
    pub screen: GardenScreen,
    pub view_state: ViewState,
    /// Current user's ID
    pub user_id: i64,
    /// Current user's handle
    pub handle: String,
    /// Is user a sysop?
    pub is_sysop: bool,
    /// Cached memories for current view
    pub memories: Vec<Memory>,
    /// Currently viewed memory (for detail views)
    pub current_memory: Option<Memory>,
    /// Input buffer for text entry
    input_buffer: String,
    /// Has user already posted today?
    pub posted_today: bool,
    /// Flags remaining today
    pub flags_remaining: u32,
    /// Random memories for welcome screen
    pub welcome_memories: Vec<Memory>,
}

impl GardenFlow {
    /// Create a new flow for a user
    pub fn new(user_id: i64, handle: &str, is_sysop: bool) -> Self {
        Self {
            screen: GardenScreen::Welcome,
            view_state: ViewState::new(),
            user_id,
            handle: handle.to_string(),
            is_sysop,
            memories: Vec::new(),
            current_memory: None,
            input_buffer: String::new(),
            posted_today: false,
            flags_remaining: 3,
            welcome_memories: Vec::new(),
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GardenScreen {
        &self.screen
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> GardenAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return GardenAction::Echo("\x08 \x08".to_string());
            }
            return GardenAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return GardenAction::Continue;
        }

        // For text entry screens, buffer input
        if self.is_text_entry_screen() {
            if self.input_buffer.len() < MAX_MEMORY_LENGTH {
                self.input_buffer.push(ch);
                return GardenAction::Echo(ch.to_string());
            }
            return GardenAction::Continue;
        }

        // For menu screens, process single key immediately
        self.input_buffer.clear();
        self.input_buffer.push(ch);
        self.process_input()
    }

    /// Check if current screen is text entry
    fn is_text_entry_screen(&self) -> bool {
        matches!(
            self.screen,
            GardenScreen::NewMemory
                | GardenScreen::EditMemory { .. }
                | GardenScreen::FlagMemory { .. }
                | GardenScreen::DateFilter
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> GardenAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim();

        match &self.screen {
            GardenScreen::Welcome => self.handle_welcome(input),
            GardenScreen::Browse => self.handle_browse(input),
            GardenScreen::ViewMemory { memory_id } => self.handle_view_memory(input, *memory_id),
            GardenScreen::NewMemory => self.handle_new_memory(input),
            GardenScreen::EditMemory { memory_id } => self.handle_edit_memory(input, *memory_id),
            GardenScreen::ConfirmDelete { memory_id } => self.handle_confirm_delete(input, *memory_id),
            GardenScreen::FlagMemory { memory_id } => self.handle_flag_memory(input, *memory_id),
            GardenScreen::MyMemories => self.handle_my_memories(input),
            GardenScreen::DateFilter => self.handle_date_filter(input),
            GardenScreen::ConfirmQuit => self.handle_confirm_quit(input),
        }
    }

    fn handle_welcome(&mut self, input: &str) -> GardenAction {
        let upper = input.to_uppercase();
        match upper.as_str() {
            "B" | "" => {
                self.screen = GardenScreen::Browse;
                self.view_state.reset_page();
                GardenAction::LoadMemories {
                    page: 0,
                    filter_date: None,
                    own_only: false,
                }
            }
            "N" => {
                if self.posted_today {
                    self.view_state.set_message("You have already planted a memory today.");
                    GardenAction::Continue
                } else {
                    self.screen = GardenScreen::NewMemory;
                    GardenAction::Continue
                }
            }
            "M" => {
                self.screen = GardenScreen::MyMemories;
                self.view_state.reset_page();
                self.view_state.show_own_only = true;
                GardenAction::LoadMemories {
                    page: 0,
                    filter_date: None,
                    own_only: true,
                }
            }
            "D" => {
                self.screen = GardenScreen::DateFilter;
                GardenAction::Continue
            }
            "Q" => {
                self.screen = GardenScreen::ConfirmQuit;
                GardenAction::Continue
            }
            _ => GardenAction::Continue,
        }
    }

    fn handle_browse(&mut self, input: &str) -> GardenAction {
        let upper = input.to_uppercase();

        // Number selection for memory detail
        if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= self.memories.len() {
                let memory = &self.memories[num - 1];
                let memory_id = memory.id;
                self.screen = GardenScreen::ViewMemory { memory_id };
                return GardenAction::LoadMemory { memory_id };
            }
        }

        match upper.as_str() {
            "N" | ">" => {
                if self.view_state.has_next_page() {
                    self.view_state.next_page();
                    return GardenAction::LoadMemories {
                        page: self.view_state.current_page,
                        filter_date: self.view_state.filter_date.clone(),
                        own_only: self.view_state.show_own_only,
                    };
                }
                GardenAction::Continue
            }
            "P" | "<" => {
                if self.view_state.has_prev_page() {
                    self.view_state.prev_page();
                    return GardenAction::LoadMemories {
                        page: self.view_state.current_page,
                        filter_date: self.view_state.filter_date.clone(),
                        own_only: self.view_state.show_own_only,
                    };
                }
                GardenAction::Continue
            }
            "W" => {
                // New memory (plant)
                if self.posted_today {
                    self.view_state.set_message("You have already planted a memory today.");
                    GardenAction::Continue
                } else {
                    self.screen = GardenScreen::NewMemory;
                    GardenAction::Continue
                }
            }
            "M" => {
                self.screen = GardenScreen::MyMemories;
                self.view_state.reset_page();
                self.view_state.show_own_only = true;
                GardenAction::LoadMemories {
                    page: 0,
                    filter_date: None,
                    own_only: true,
                }
            }
            "D" => {
                self.screen = GardenScreen::DateFilter;
                GardenAction::Continue
            }
            "C" => {
                // Clear filter
                self.view_state.filter_date = None;
                self.view_state.show_own_only = false;
                self.view_state.reset_page();
                self.screen = GardenScreen::Browse;
                GardenAction::LoadMemories {
                    page: 0,
                    filter_date: None,
                    own_only: false,
                }
            }
            "Q" => {
                self.screen = GardenScreen::ConfirmQuit;
                GardenAction::Continue
            }
            _ => GardenAction::Continue,
        }
    }

    fn handle_view_memory(&mut self, input: &str, memory_id: i64) -> GardenAction {
        let upper = input.to_uppercase();

        // Check if we have the memory loaded
        let memory = self.current_memory.as_ref();
        let is_own = memory.map(|m| m.is_owned_by(self.user_id)).unwrap_or(false);
        let can_edit = memory.map(|m| m.can_edit(Utc::now())).unwrap_or(false);

        match upper.as_str() {
            "E" if is_own && can_edit => {
                self.screen = GardenScreen::EditMemory { memory_id };
                if let Some(m) = &self.current_memory {
                    self.input_buffer = m.content.clone();
                }
                GardenAction::Continue
            }
            "X" if is_own => {
                self.screen = GardenScreen::ConfirmDelete { memory_id };
                GardenAction::Continue
            }
            "F" if !is_own => {
                if self.flags_remaining == 0 {
                    self.view_state.set_message("You have used all your flags for today.");
                    GardenAction::Continue
                } else {
                    self.screen = GardenScreen::FlagMemory { memory_id };
                    GardenAction::Continue
                }
            }
            "B" | "Q" | "" => {
                self.screen = GardenScreen::Browse;
                self.current_memory = None;
                GardenAction::LoadMemories {
                    page: self.view_state.current_page,
                    filter_date: self.view_state.filter_date.clone(),
                    own_only: self.view_state.show_own_only,
                }
            }
            _ => GardenAction::Continue,
        }
    }

    fn handle_new_memory(&mut self, input: &str) -> GardenAction {
        let content = input.trim();

        if content.is_empty() {
            // Cancel - go back
            self.screen = GardenScreen::Browse;
            return GardenAction::LoadMemories {
                page: self.view_state.current_page,
                filter_date: self.view_state.filter_date.clone(),
                own_only: self.view_state.show_own_only,
            };
        }

        if content.len() > MAX_MEMORY_LENGTH {
            self.view_state.set_message(&format!(
                "Memory too long ({}/{}). Please shorten it.",
                content.len(),
                MAX_MEMORY_LENGTH
            ));
            return GardenAction::Continue;
        }

        // Save the memory
        self.posted_today = true;
        self.view_state.set_message("Your memory has been planted in the garden.");
        self.screen = GardenScreen::Browse;
        GardenAction::SaveNewMemory {
            content: content.to_string(),
        }
    }

    fn handle_edit_memory(&mut self, input: &str, memory_id: i64) -> GardenAction {
        let content = input.trim();

        if content.is_empty() {
            // Cancel edit
            self.screen = GardenScreen::ViewMemory { memory_id };
            return GardenAction::LoadMemory { memory_id };
        }

        if content.len() > MAX_MEMORY_LENGTH {
            self.view_state.set_message(&format!(
                "Memory too long ({}/{}). Please shorten it.",
                content.len(),
                MAX_MEMORY_LENGTH
            ));
            return GardenAction::Continue;
        }

        self.view_state.set_message("Your memory has been updated.");
        self.screen = GardenScreen::Browse;
        GardenAction::UpdateMemory {
            memory_id,
            content: content.to_string(),
        }
    }

    fn handle_confirm_delete(&mut self, input: &str, memory_id: i64) -> GardenAction {
        let upper = input.to_uppercase();
        match upper.as_str() {
            "Y" => {
                self.view_state.set_message("Memory has been removed from the garden.");
                self.screen = GardenScreen::Browse;
                self.current_memory = None;
                GardenAction::DeleteMemory { memory_id }
            }
            _ => {
                self.screen = GardenScreen::ViewMemory { memory_id };
                GardenAction::Continue
            }
        }
    }

    fn handle_flag_memory(&mut self, input: &str, memory_id: i64) -> GardenAction {
        // Input is optional reason
        let reason = if input.trim().is_empty() {
            None
        } else {
            Some(input.trim().to_string())
        };

        self.flags_remaining = self.flags_remaining.saturating_sub(1);
        self.view_state.set_message("Memory has been flagged for Sysop review.");
        self.screen = GardenScreen::Browse;
        self.current_memory = None;
        GardenAction::FlagMemory { memory_id, reason }
    }

    fn handle_my_memories(&mut self, input: &str) -> GardenAction {
        // Same as browse but filtered
        let upper = input.to_uppercase();

        // Number selection
        if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= self.memories.len() {
                let memory = &self.memories[num - 1];
                let memory_id = memory.id;
                self.screen = GardenScreen::ViewMemory { memory_id };
                return GardenAction::LoadMemory { memory_id };
            }
        }

        match upper.as_str() {
            "N" | ">" => {
                if self.view_state.has_next_page() {
                    self.view_state.next_page();
                    return GardenAction::LoadMemories {
                        page: self.view_state.current_page,
                        filter_date: None,
                        own_only: true,
                    };
                }
                GardenAction::Continue
            }
            "P" | "<" => {
                if self.view_state.has_prev_page() {
                    self.view_state.prev_page();
                    return GardenAction::LoadMemories {
                        page: self.view_state.current_page,
                        filter_date: None,
                        own_only: true,
                    };
                }
                GardenAction::Continue
            }
            "B" | "C" => {
                // Back to all memories
                self.view_state.show_own_only = false;
                self.view_state.reset_page();
                self.screen = GardenScreen::Browse;
                GardenAction::LoadMemories {
                    page: 0,
                    filter_date: None,
                    own_only: false,
                }
            }
            "Q" => {
                self.screen = GardenScreen::ConfirmQuit;
                GardenAction::Continue
            }
            _ => GardenAction::Continue,
        }
    }

    fn handle_date_filter(&mut self, input: &str) -> GardenAction {
        let date_str = input.trim();

        if date_str.is_empty() {
            // Cancel
            self.screen = GardenScreen::Browse;
            return GardenAction::LoadMemories {
                page: self.view_state.current_page,
                filter_date: self.view_state.filter_date.clone(),
                own_only: self.view_state.show_own_only,
            };
        }

        // Validate date format (YYYY-MM-DD or MM/DD/YYYY)
        let normalized = normalize_date(date_str);
        if normalized.is_none() {
            self.view_state.set_message("Invalid date format. Use YYYY-MM-DD or MM/DD/YYYY.");
            return GardenAction::Continue;
        }

        self.view_state.filter_date = normalized.clone();
        self.view_state.reset_page();
        self.screen = GardenScreen::Browse;
        GardenAction::LoadMemories {
            page: 0,
            filter_date: normalized,
            own_only: false,
        }
    }

    fn handle_confirm_quit(&mut self, input: &str) -> GardenAction {
        let upper = input.to_uppercase();
        match upper.as_str() {
            "Y" => GardenAction::Quit,
            _ => {
                self.screen = GardenScreen::Welcome;
                GardenAction::LoadRandomMemories { count: 5 }
            }
        }
    }

    /// Update memories after loading
    pub fn set_memories(&mut self, memories: Vec<Memory>, total: usize) {
        self.memories = memories;
        self.view_state.total_memories = total;
    }

    /// Update welcome memories
    pub fn set_welcome_memories(&mut self, memories: Vec<Memory>) {
        self.welcome_memories = memories;
    }

    /// Update current memory for detail view
    pub fn set_current_memory(&mut self, memory: Option<Memory>) {
        self.current_memory = memory;
    }

    /// Update posted today status
    pub fn set_posted_today(&mut self, posted: bool) {
        self.posted_today = posted;
    }

    /// Update flags remaining
    pub fn set_flags_remaining(&mut self, count: u32) {
        self.flags_remaining = count;
    }
}

/// Normalize date string to YYYY-MM-DD format
fn normalize_date(input: &str) -> Option<String> {
    // Try YYYY-MM-DD
    if input.len() == 10 && input.chars().nth(4) == Some('-') && input.chars().nth(7) == Some('-') {
        let parts: Vec<&str> = input.split('-').collect();
        if parts.len() == 3 {
            let year: u16 = parts[0].parse().ok()?;
            let month: u8 = parts[1].parse().ok()?;
            let day: u8 = parts[2].parse().ok()?;
            if year >= 2020 && year <= 2100 && month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                return Some(format!("{:04}-{:02}-{:02}", year, month, day));
            }
        }
    }

    // Try MM/DD/YYYY
    if input.contains('/') {
        let parts: Vec<&str> = input.split('/').collect();
        if parts.len() == 3 {
            let month: u8 = parts[0].parse().ok()?;
            let day: u8 = parts[1].parse().ok()?;
            let year: u16 = parts[2].parse().ok()?;
            if year >= 2020 && year <= 2100 && month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                return Some(format!("{:04}-{:02}-{:02}", year, month, day));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow_starts_at_welcome() {
        let flow = GardenFlow::new(1, "test", false);
        assert!(matches!(flow.screen, GardenScreen::Welcome));
    }

    #[test]
    fn test_navigate_to_browse() {
        let mut flow = GardenFlow::new(1, "test", false);
        let action = flow.handle_char('B');
        assert!(matches!(flow.screen, GardenScreen::Browse));
        assert!(matches!(action, GardenAction::LoadMemories { .. }));
    }

    #[test]
    fn test_cannot_post_twice_today() {
        let mut flow = GardenFlow::new(1, "test", false);
        flow.posted_today = true;
        flow.screen = GardenScreen::Welcome;

        let action = flow.handle_char('N');
        // Should not change screen
        assert!(matches!(flow.screen, GardenScreen::Welcome));
        assert!(matches!(action, GardenAction::Continue));
    }

    #[test]
    fn test_pagination() {
        let mut flow = GardenFlow::new(1, "test", false);
        flow.screen = GardenScreen::Browse;
        flow.view_state.total_memories = 25;
        flow.view_state.page_size = 10;

        // Go to next page
        flow.handle_char('N');
        assert_eq!(flow.view_state.current_page, 1);

        // Go back
        flow.handle_char('P');
        assert_eq!(flow.view_state.current_page, 0);
    }

    #[test]
    fn test_normalize_date_iso() {
        assert_eq!(normalize_date("2026-01-30"), Some("2026-01-30".to_string()));
        assert_eq!(normalize_date("2026-12-31"), Some("2026-12-31".to_string()));
        assert_eq!(normalize_date("invalid"), None);
    }

    #[test]
    fn test_normalize_date_us() {
        assert_eq!(normalize_date("01/30/2026"), Some("2026-01-30".to_string()));
        assert_eq!(normalize_date("12/31/2026"), Some("2026-12-31".to_string()));
    }

    #[test]
    fn test_text_entry_buffer() {
        let mut flow = GardenFlow::new(1, "test", false);
        flow.screen = GardenScreen::NewMemory;

        // Type some text
        for ch in "Hello".chars() {
            flow.handle_char(ch);
        }
        assert_eq!(flow.input_buffer, "Hello");

        // Backspace
        flow.handle_char('\x7f');
        assert_eq!(flow.input_buffer, "Hell");
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = GardenFlow::new(1, "test", false);
        flow.screen = GardenScreen::ConfirmQuit;

        let action = flow.handle_char('Y');
        assert!(matches!(action, GardenAction::Quit));
    }
}
