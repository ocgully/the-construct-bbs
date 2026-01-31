//! Memory Garden ANSI rendering
//!
//! Uses a garden/nature visual theme with greens, earth tones, and floral accents.
//! The aesthetic is peaceful and contemplative, reflecting the journaling nature.

use crate::terminal::{AnsiWriter, Color};
use super::state::{Memory, MilestoneType, MAX_MEMORY_LENGTH};
use super::screen::{GardenFlow, GardenScreen};

// ============================================================================
// VISUAL CONSTANTS - Garden Theme
// ============================================================================

/// Primary accent color - soft green (growth, life)
const ACCENT_PRIMARY: Color = Color::LightGreen;
/// Secondary accent - warm gold (memories, nostalgia)
const ACCENT_SECONDARY: Color = Color::Yellow;
/// Text color for content
const TEXT_COLOR: Color = Color::White;
/// Muted text for metadata
const MUTED_COLOR: Color = Color::LightGray;
/// Border/decoration color
const BORDER_COLOR: Color = Color::Green;
/// System memory indicator
const SYSTEM_COLOR: Color = Color::LightCyan;
/// Error/flag color
const ALERT_COLOR: Color = Color::LightRed;

// Box drawing characters for garden-style borders
const VINE_TOP: &str = "~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~.~";
const VINE_BOT: &str = "~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~'~";

// ============================================================================
// HEADER RENDERING
// ============================================================================

/// Render the Memory Garden header with ASCII art title
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(ACCENT_PRIMARY);
    w.writeln("");
    w.writeln("   ███╗   ███╗███████╗███╗   ███╗ ██████╗ ██████╗ ██╗   ██╗");
    w.writeln("   ████╗ ████║██╔════╝████╗ ████║██╔═══██╗██╔══██╗╚██╗ ██╔╝");
    w.writeln("   ██╔████╔██║█████╗  ██╔████╔██║██║   ██║██████╔╝ ╚████╔╝ ");
    w.writeln("   ██║╚██╔╝██║██╔══╝  ██║╚██╔╝██║██║   ██║██╔══██╗  ╚██╔╝  ");
    w.writeln("   ██║ ╚═╝ ██║███████╗██║ ╚═╝ ██║╚██████╔╝██║  ██║   ██║   ");
    w.writeln("   ╚═╝     ╚═╝╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("            ██████╗  █████╗ ██████╗ ██████╗ ███████╗███╗   ██╗");
    w.writeln("           ██╔════╝ ██╔══██╗██╔══██╗██╔══██╗██╔════╝████╗  ██║");
    w.writeln("           ██║  ███╗███████║██████╔╝██║  ██║█████╗  ██╔██╗ ██║");
    w.writeln("           ██║   ██║██╔══██║██╔══██╗██║  ██║██╔══╝  ██║╚██╗██║");
    w.writeln("           ╚██████╔╝██║  ██║██║  ██║██████╔╝███████╗██║ ╚████║");
    w.writeln("            ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═══╝");
    w.reset_color();
}

/// Render decorative vine border
fn render_vine_border(w: &mut AnsiWriter, top: bool) {
    w.set_fg(BORDER_COLOR);
    if top {
        w.writeln(VINE_TOP);
    } else {
        w.writeln(VINE_BOT);
    }
    w.reset_color();
}

/// Render a message if present in view state
fn render_message(w: &mut AnsiWriter, message: Option<&String>) {
    if let Some(msg) = message {
        w.writeln("");
        w.set_fg(ACCENT_SECONDARY);
        w.bold();
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }
}

// ============================================================================
// PUBLIC RENDER FUNCTIONS
// ============================================================================

/// Main render function - dispatches to appropriate screen renderer
pub fn render_screen(flow: &GardenFlow) -> String {
    let mut w = AnsiWriter::new();

    match &flow.screen {
        GardenScreen::Welcome => render_welcome(&mut w, flow),
        GardenScreen::Browse => render_browse(&mut w, flow),
        GardenScreen::ViewMemory { .. } => render_view_memory(&mut w, flow),
        GardenScreen::NewMemory => render_new_memory(&mut w, flow),
        GardenScreen::EditMemory { .. } => render_edit_memory(&mut w, flow),
        GardenScreen::ConfirmDelete { .. } => render_confirm_delete(&mut w, flow),
        GardenScreen::FlagMemory { .. } => render_flag_memory(&mut w, flow),
        GardenScreen::MyMemories => render_my_memories(&mut w, flow),
        GardenScreen::DateFilter => render_date_filter(&mut w, flow),
        GardenScreen::ConfirmQuit => render_confirm_quit(&mut w),
    }

    w.flush()
}

/// Render the welcome screen with random recent memories
fn render_welcome(w: &mut AnsiWriter, flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    w.writeln("");
    w.set_fg(TEXT_COLOR);
    w.writeln("  Welcome to the Memory Garden, a place where thoughts bloom");
    w.writeln("  and memories grow. Each day, you may plant one memory for");
    w.writeln("  others to discover.");
    w.writeln("");

    // Show random memories
    if !flow.welcome_memories.is_empty() {
        w.set_fg(ACCENT_PRIMARY);
        w.bold();
        w.writeln("  Recent blooms from the garden:");
        w.reset_color();
        w.writeln("");

        for memory in &flow.welcome_memories {
            render_memory_compact(w, memory, false);
        }
    }

    render_message(w, flow.view_state.message.as_ref());

    w.writeln("");
    render_vine_border(w, false);
    w.writeln("");

    // Menu options
    w.set_fg(ACCENT_PRIMARY);
    w.write_str("    [B] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Browse the Garden");

    if !flow.posted_today {
        w.set_fg(ACCENT_PRIMARY);
        w.write_str("    [N] ");
        w.set_fg(TEXT_COLOR);
        w.writeln("Plant a New Memory");
    } else {
        w.set_fg(MUTED_COLOR);
        w.writeln("    [N] Plant a New Memory (already planted today)");
    }

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("    [M] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("My Memories");

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("    [D] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Browse by Date");

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("    [Q] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Leave the Garden");

    w.writeln("");
    w.reset_color();
    w.write_str("  > ");
}

/// Render the browse screen with paginated memories
fn render_browse(w: &mut AnsiWriter, flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    // Show current filter if any
    if let Some(date) = &flow.view_state.filter_date {
        w.set_fg(ACCENT_SECONDARY);
        w.writeln(&format!("  Viewing memories from: {}", date));
    } else if flow.view_state.show_own_only {
        w.set_fg(ACCENT_SECONDARY);
        w.writeln("  Viewing: Your memories");
    } else {
        w.set_fg(MUTED_COLOR);
        w.writeln("  Viewing: All memories (newest first)");
    }
    w.reset_color();

    // Page info
    w.set_fg(MUTED_COLOR);
    w.writeln(&format!(
        "  Page {} of {} ({} memories total)",
        flow.view_state.current_page + 1,
        flow.view_state.total_pages(),
        flow.view_state.total_memories
    ));
    w.reset_color();
    w.writeln("");

    render_message(w, flow.view_state.message.as_ref());

    // Memory list
    if flow.memories.is_empty() {
        w.set_fg(MUTED_COLOR);
        w.writeln("  The garden is empty. Be the first to plant a memory!");
    } else {
        for (i, memory) in flow.memories.iter().enumerate() {
            render_memory_list_item(w, memory, i + 1);
        }
    }

    w.writeln("");
    render_vine_border(w, false);
    w.writeln("");

    // Navigation
    w.set_fg(MUTED_COLOR);
    w.write_str("  [1-9] View  ");

    if flow.view_state.has_prev_page() {
        w.set_fg(ACCENT_PRIMARY);
        w.write_str("[P]rev  ");
    }
    if flow.view_state.has_next_page() {
        w.set_fg(ACCENT_PRIMARY);
        w.write_str("[N]ext  ");
    }

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("[W]rite  [M]ine  [D]ate  ");

    if flow.view_state.filter_date.is_some() || flow.view_state.show_own_only {
        w.write_str("[C]lear  ");
    }

    w.write_str("[Q]uit");
    w.reset_color();

    w.writeln("");
    w.writeln("");
    w.write_str("  > ");
}

/// Render single memory list item
fn render_memory_list_item(w: &mut AnsiWriter, memory: &Memory, num: usize) {
    // Line 1: Number, date, author
    w.set_fg(ACCENT_PRIMARY);
    w.write_str(&format!("  [{:1}] ", num));

    w.set_fg(MUTED_COLOR);
    w.write_str(&format!("{} ", memory.short_date()));

    if memory.is_system_generated {
        w.set_fg(SYSTEM_COLOR);
        w.write_str("[GARDEN] ");
    } else if let Some(handle) = &memory.handle {
        w.set_fg(ACCENT_SECONDARY);
        w.write_str(&format!("{} ", handle));
    }

    // Truncated content
    w.set_fg(TEXT_COLOR);
    let content = truncate_str(&memory.content, 55);
    w.writeln(&content);
}

/// Render memory in compact format for welcome screen
fn render_memory_compact(w: &mut AnsiWriter, memory: &Memory, _show_num: bool) {
    w.set_fg(BORDER_COLOR);
    w.write_str("    ");

    // Content (wrapped if needed)
    w.set_fg(TEXT_COLOR);
    w.write_str("\"");
    let content = truncate_str(&memory.content, 60);
    w.write_str(&content);
    w.write_str("\"");
    w.writeln("");

    // Attribution
    w.write_str("      ");
    w.set_fg(MUTED_COLOR);
    w.write_str("- ");

    if memory.is_system_generated {
        w.set_fg(SYSTEM_COLOR);
        w.write_str("The Garden");
    } else if let Some(handle) = &memory.handle {
        w.set_fg(ACCENT_SECONDARY);
        w.write_str(handle);
    }

    w.set_fg(MUTED_COLOR);
    w.writeln(&format!(", {}", memory.display_date()));
    w.writeln("");
}

/// Render detailed memory view
fn render_view_memory(w: &mut AnsiWriter, flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    if let Some(memory) = &flow.current_memory {
        w.writeln("");

        // Date header
        w.set_fg(ACCENT_SECONDARY);
        w.bold();
        w.writeln(&format!("  {}", memory.display_date()));
        w.reset_color();
        w.writeln("");

        // Author
        if memory.is_system_generated {
            w.set_fg(SYSTEM_COLOR);
            w.write_str("  Planted by: The Garden");
            if let Some(milestone) = &memory.milestone_type {
                w.set_fg(MUTED_COLOR);
                let milestone_text = match milestone {
                    MilestoneType::Birth => " (the beginning)",
                    MilestoneType::Users => " (user milestone)",
                    MilestoneType::Sessions => " (session milestone)",
                    MilestoneType::Time => " (time milestone)",
                };
                w.write_str(milestone_text);
            }
            w.writeln("");
        } else if let Some(handle) = &memory.handle {
            w.set_fg(ACCENT_SECONDARY);
            w.writeln(&format!("  Planted by: {}", handle));
        }
        w.writeln("");

        // Content - word wrap at 70 chars
        w.set_fg(TEXT_COLOR);
        for line in word_wrap(&memory.content, 70) {
            w.writeln(&format!("  {}", line));
        }

        // Edit timestamp if edited
        if memory.updated_at.is_some() {
            w.writeln("");
            w.set_fg(MUTED_COLOR);
            w.writeln("  (edited)");
        }

        render_message(w, flow.view_state.message.as_ref());

        w.writeln("");
        render_vine_border(w, false);
        w.writeln("");

        // Actions
        let is_own = memory.is_owned_by(flow.user_id);
        let now = chrono::Utc::now();
        let can_edit = memory.can_edit(now);

        if is_own {
            if can_edit {
                w.set_fg(ACCENT_PRIMARY);
                w.write_str("  [E]dit  ");
            }
            w.set_fg(ALERT_COLOR);
            w.write_str("[X] Delete  ");
        } else if !memory.is_system_generated {
            w.set_fg(ALERT_COLOR);
            if flow.flags_remaining > 0 {
                w.write_str(&format!("[F]lag ({} left)  ", flow.flags_remaining));
            } else {
                w.set_fg(MUTED_COLOR);
                w.write_str("[F]lag (none left)  ");
            }
        }

        w.set_fg(ACCENT_PRIMARY);
        w.writeln("[B]ack");
    } else {
        w.writeln("");
        w.set_fg(ALERT_COLOR);
        w.writeln("  Memory not found.");
        w.writeln("");
        w.set_fg(ACCENT_PRIMARY);
        w.writeln("  [B]ack");
    }

    w.writeln("");
    w.reset_color();
    w.write_str("  > ");
}

/// Render new memory input screen
fn render_new_memory(w: &mut AnsiWriter, flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    w.writeln("");
    w.set_fg(ACCENT_PRIMARY);
    w.bold();
    w.writeln("  Plant a Memory");
    w.reset_color();
    w.writeln("");

    w.set_fg(TEXT_COLOR);
    w.writeln("  Share a thought, a moment, or a feeling.");
    w.writeln("  Your words will grow here for others to discover.");
    w.writeln("");

    w.set_fg(MUTED_COLOR);
    w.writeln(&format!("  Maximum {} characters. Press Enter when done.", MAX_MEMORY_LENGTH));
    w.writeln("  Leave empty and press Enter to cancel.");
    w.writeln("");

    render_message(w, flow.view_state.message.as_ref());

    w.writeln("");
    render_vine_border(w, false);
    w.writeln("");

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("  Your memory: ");
    w.reset_color();
}

/// Render edit memory screen
fn render_edit_memory(w: &mut AnsiWriter, flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.bold();
    w.writeln("  Edit Memory");
    w.reset_color();
    w.writeln("");

    if let Some(memory) = &flow.current_memory {
        w.set_fg(MUTED_COLOR);
        w.writeln("  Current content:");
        w.set_fg(TEXT_COLOR);
        for line in word_wrap(&memory.content, 70) {
            w.writeln(&format!("    {}", line));
        }
    }

    w.writeln("");
    w.set_fg(MUTED_COLOR);
    w.writeln(&format!("  Maximum {} characters. Press Enter when done.", MAX_MEMORY_LENGTH));
    w.writeln("  Leave empty and press Enter to cancel.");
    w.writeln("");

    render_message(w, flow.view_state.message.as_ref());

    w.writeln("");
    render_vine_border(w, false);
    w.writeln("");

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("  New content: ");
    w.reset_color();
}

/// Render delete confirmation
fn render_confirm_delete(w: &mut AnsiWriter, _flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    w.writeln("");
    w.set_fg(ALERT_COLOR);
    w.bold();
    w.writeln("  Remove Memory");
    w.reset_color();
    w.writeln("");

    w.set_fg(TEXT_COLOR);
    w.writeln("  Are you sure you want to remove this memory?");
    w.writeln("  This cannot be undone.");
    w.writeln("");

    render_vine_border(w, false);
    w.writeln("");

    w.set_fg(ALERT_COLOR);
    w.write_str("  [Y]es, remove it  ");
    w.set_fg(ACCENT_PRIMARY);
    w.writeln("[N]o, keep it");
    w.writeln("");

    w.reset_color();
    w.write_str("  > ");
}

/// Render flag memory screen
fn render_flag_memory(w: &mut AnsiWriter, flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    w.writeln("");
    w.set_fg(ALERT_COLOR);
    w.bold();
    w.writeln("  Flag Memory for Review");
    w.reset_color();
    w.writeln("");

    w.set_fg(TEXT_COLOR);
    w.writeln("  You are flagging this memory for Sysop review.");
    w.writeln("  Flagged content will be hidden until reviewed.");
    w.writeln("");

    w.set_fg(MUTED_COLOR);
    w.writeln(&format!("  You have {} flags remaining today.", flow.flags_remaining));
    w.writeln("");
    w.writeln("  (Optional) Enter a reason, or press Enter to flag without reason:");
    w.writeln("");

    render_vine_border(w, false);
    w.writeln("");

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("  Reason: ");
    w.reset_color();
}

/// Render my memories screen (filtered browse)
fn render_my_memories(w: &mut AnsiWriter, flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    w.set_fg(ACCENT_SECONDARY);
    w.bold();
    w.writeln("  My Memories");
    w.reset_color();

    // Page info
    w.set_fg(MUTED_COLOR);
    w.writeln(&format!(
        "  Page {} of {} ({} memories)",
        flow.view_state.current_page + 1,
        flow.view_state.total_pages(),
        flow.view_state.total_memories
    ));
    w.reset_color();
    w.writeln("");

    render_message(w, flow.view_state.message.as_ref());

    // Memory list
    if flow.memories.is_empty() {
        w.set_fg(MUTED_COLOR);
        w.writeln("  You haven't planted any memories yet.");
        w.writeln("  Share your first thought in the garden!");
    } else {
        for (i, memory) in flow.memories.iter().enumerate() {
            render_memory_list_item(w, memory, i + 1);
        }
    }

    w.writeln("");
    render_vine_border(w, false);
    w.writeln("");

    // Navigation
    w.set_fg(MUTED_COLOR);
    w.write_str("  [1-9] View  ");

    if flow.view_state.has_prev_page() {
        w.set_fg(ACCENT_PRIMARY);
        w.write_str("[P]rev  ");
    }
    if flow.view_state.has_next_page() {
        w.set_fg(ACCENT_PRIMARY);
        w.write_str("[N]ext  ");
    }

    w.set_fg(ACCENT_PRIMARY);
    w.writeln("[B]ack to all  [Q]uit");
    w.reset_color();

    w.writeln("");
    w.write_str("  > ");
}

/// Render date filter input
fn render_date_filter(w: &mut AnsiWriter, flow: &GardenFlow) {
    render_header(w);
    render_vine_border(w, true);

    w.writeln("");
    w.set_fg(ACCENT_PRIMARY);
    w.bold();
    w.writeln("  Browse by Date");
    w.reset_color();
    w.writeln("");

    w.set_fg(TEXT_COLOR);
    w.writeln("  Enter a date to view memories from that day.");
    w.writeln("");

    w.set_fg(MUTED_COLOR);
    w.writeln("  Format: YYYY-MM-DD or MM/DD/YYYY");
    w.writeln("  Example: 2026-01-30 or 01/30/2026");
    w.writeln("");
    w.writeln("  Leave empty and press Enter to cancel.");
    w.writeln("");

    render_message(w, flow.view_state.message.as_ref());

    render_vine_border(w, false);
    w.writeln("");

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("  Date: ");
    w.reset_color();
}

/// Render quit confirmation
fn render_confirm_quit(w: &mut AnsiWriter) {
    w.clear_screen();

    w.set_fg(ACCENT_PRIMARY);
    w.bold();
    w.writeln("");
    w.writeln("  Leave the Garden?");
    w.reset_color();
    w.writeln("");

    w.set_fg(TEXT_COLOR);
    w.writeln("  Your memories will remain here, growing quietly");
    w.writeln("  until you return.");
    w.writeln("");

    w.set_fg(ACCENT_PRIMARY);
    w.write_str("  [Y]es, leave  ");
    w.write_str("[N]o, stay");
    w.reset_color();
    w.writeln("");
    w.writeln("");
    w.write_str("  > ");
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Truncate string to max length, adding "..." if truncated
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Word wrap text to specified width
fn word_wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_str_short() {
        assert_eq!(truncate_str("Hello", 10), "Hello");
    }

    #[test]
    fn test_truncate_str_exact() {
        assert_eq!(truncate_str("Hello", 5), "Hello");
    }

    #[test]
    fn test_truncate_str_long() {
        assert_eq!(truncate_str("Hello World", 8), "Hello...");
    }

    #[test]
    fn test_word_wrap_short() {
        let lines = word_wrap("Hello world", 20);
        assert_eq!(lines, vec!["Hello world"]);
    }

    #[test]
    fn test_word_wrap_long() {
        let lines = word_wrap("The quick brown fox jumps over the lazy dog", 20);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].len() <= 20);
        assert!(lines[1].len() <= 20);
    }

    #[test]
    fn test_render_screen_welcome() {
        let flow = GardenFlow::new(1, "test", false);
        let output = render_screen(&flow);
        // Check for menu options (these are in plain text)
        assert!(output.contains("Browse the Garden"));
        assert!(output.contains("Plant a New Memory"));
        assert!(output.contains("My Memories"));
    }

    #[test]
    fn test_render_does_not_panic() {
        let mut flow = GardenFlow::new(1, "test", false);

        // Test all screens
        flow.screen = GardenScreen::Welcome;
        let _ = render_screen(&flow);

        flow.screen = GardenScreen::Browse;
        let _ = render_screen(&flow);

        flow.screen = GardenScreen::NewMemory;
        let _ = render_screen(&flow);

        flow.screen = GardenScreen::ConfirmQuit;
        let _ = render_screen(&flow);
    }
}
