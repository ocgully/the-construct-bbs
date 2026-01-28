use crate::db::messages::{InboxEntry, Message};
use crate::terminal::{AnsiWriter, Color};
use std::collections::HashMap;

/// Format an ISO datetime string as "MMM DD" for inbox date column.
///
/// Examples:
///   "2026-01-26 14:15:00" -> "Jan 26"
///   "2026-12-01T09:30:00" -> "Dec 01"
fn format_date_short(iso: &str) -> String {
    let date_part = if iso.len() >= 10 { &iso[..10] } else { iso };
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() == 3 {
        let month = match parts[1] {
            "01" => "Jan",
            "02" => "Feb",
            "03" => "Mar",
            "04" => "Apr",
            "05" => "May",
            "06" => "Jun",
            "07" => "Jul",
            "08" => "Aug",
            "09" => "Sep",
            "10" => "Oct",
            "11" => "Nov",
            "12" => "Dec",
            _ => return iso.to_string(),
        };
        let day = parts[2];
        format!("{} {}", month, day)
    } else {
        iso.to_string()
    }
}

/// Format an ISO datetime string as "Month Day, Year at H:MM AM/PM".
///
/// Falls back to date-only format if time parsing fails.
fn format_datetime(iso: &str) -> String {
    // Parse date portion
    let date_part = if iso.len() >= 10 { &iso[..10] } else { iso };
    let parts: Vec<&str> = date_part.split('-').collect();

    let date_str = if parts.len() == 3 {
        let year = parts[0];
        let month = match parts[1] {
            "01" => "January",
            "02" => "February",
            "03" => "March",
            "04" => "April",
            "05" => "May",
            "06" => "June",
            "07" => "July",
            "08" => "August",
            "09" => "September",
            "10" => "October",
            "11" => "November",
            "12" => "December",
            _ => return iso.to_string(),
        };
        let day = parts[2].trim_start_matches('0');
        if day.is_empty() {
            return iso.to_string();
        }
        format!("{} {}, {}", month, day, year)
    } else {
        return iso.to_string();
    };

    // Try to extract time portion after space or 'T'
    let time_part = if iso.len() > 11 {
        let sep = if iso.contains('T') { 'T' } else { ' ' };
        iso.split(sep).nth(1)
    } else {
        None
    };

    if let Some(time) = time_part {
        let time_parts: Vec<&str> = time.split(':').collect();
        if time_parts.len() >= 2 {
            if let (Ok(hour), Ok(min)) = (
                time_parts[0].parse::<u32>(),
                time_parts[1].parse::<u32>(),
            ) {
                let (h12, ampm) = if hour == 0 {
                    (12, "AM")
                } else if hour < 12 {
                    (hour, "AM")
                } else if hour == 12 {
                    (12, "PM")
                } else {
                    (hour - 12, "PM")
                };
                return format!("{} at {}:{:02} {}", date_str, h12, min, ampm);
            }
        }
    }

    date_str
}

/// Truncate a string to max length, appending "..." if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

// ============================================================================
// COMPOSE STATE MACHINE
// ============================================================================

/// The stages of the compose flow.
#[derive(Debug, Clone, PartialEq)]
pub enum ComposeState {
    PromptTo,
    InputTo,
    PromptSubject,
    InputSubject,
    PromptBody,
    InputBody,
    Sending,
    Done,
    Aborted,
}

/// Actions returned by ComposeFlow to guide session.rs.
#[derive(Debug, Clone, PartialEq)]
pub enum ComposeAction {
    /// Continue without any output.
    Continue,
    /// Echo this string back to user.
    Echo(String),
    /// Show this prompt to user.
    ShowPrompt(String),
    /// Need async DB lookup for recipient handle.
    NeedRecipientLookup(String),
    /// Ready to send message (all fields collected).
    SendMessage {
        recipient_id: i64,
        recipient_handle: String,
        subject: String,
        body: String,
    },
    /// User aborted compose.
    Aborted,
    /// Show help text.
    ShowHelp,
    /// Show body lines with line numbers.
    ShowLines(String),
}

/// Compose flow state machine.
///
/// Handles To -> Subject -> Body flow with character-by-character input.
/// Like LoginFlow and RegistrationFlow, this is a synchronous state machine
/// that returns ComposeAction to let session.rs handle async DB operations.
pub struct ComposeFlow {
    state: ComposeState,
    sender_id: i64,
    sender_handle: String,
    input_buffer: String,
    recipient_id: Option<i64>,
    recipient_handle: Option<String>,
    subject: Option<String>,
    body_lines: Vec<String>,
    is_reply: bool,
}

impl ComposeFlow {
    /// Create a new compose flow starting at To prompt.
    pub fn new(sender_id: i64, sender_handle: String) -> Self {
        Self {
            state: ComposeState::PromptTo,
            sender_id,
            sender_handle,
            input_buffer: String::new(),
            recipient_id: None,
            recipient_handle: None,
            subject: None,
            body_lines: Vec::new(),
            is_reply: false,
        }
    }

    /// Create a reply compose flow with pre-filled recipient and quoted body.
    ///
    /// Subject gets "Re: " prefix (de-duplicated to prevent "Re: Re: Re:").
    /// Body gets quoted with "> " prefix on each line.
    pub fn new_reply(
        sender_id: i64,
        sender_handle: String,
        recipient_id: i64,
        recipient_handle: String,
        original_subject: String,
        original_body: String,
    ) -> Self {
        // De-duplicate "Re: " prefix
        let reply_subject = if original_subject.starts_with("Re: ") {
            original_subject
        } else {
            format!("Re: {}", original_subject)
        };

        // Quote original body with "> " prefix
        let quoted_lines: Vec<String> = original_body
            .lines()
            .map(|line| format!("> {}", line))
            .collect();

        Self {
            state: ComposeState::PromptBody,
            sender_id,
            sender_handle,
            input_buffer: String::new(),
            recipient_id: Some(recipient_id),
            recipient_handle: Some(recipient_handle),
            subject: Some(reply_subject),
            body_lines: quoted_lines,
            is_reply: true,
        }
    }

    /// Get the current prompt text.
    pub fn current_prompt(&self) -> &str {
        match self.state {
            ComposeState::PromptTo => "To: ",
            ComposeState::PromptSubject => "Subject: ",
            ComposeState::PromptBody => "Enter message. /s to send, /a to abort, /h for help:\r\n",
            _ => "",
        }
    }

    /// Get current state (for external inspection if needed).
    pub fn state(&self) -> &ComposeState {
        &self.state
    }

    /// Maximum input length for current state.
    fn max_input_length(&self) -> usize {
        match self.state {
            ComposeState::InputTo => 254,
            ComposeState::InputSubject => 254,
            ComposeState::InputBody => usize::MAX, // No limit on body lines
            _ => 0,
        }
    }

    /// Handle a single character of input and return action.
    ///
    /// Follows LoginFlow/RegistrationFlow pattern:
    /// - Backspace: remove last char, return "\x08 \x08"
    /// - Enter: process line via handle_line()
    /// - Printable: accumulate and echo
    pub fn handle_char(&mut self, ch: char) -> ComposeAction {
        // Backspace: \x7f (DEL) or \x08 (BS)
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return ComposeAction::Echo("\x08 \x08".to_string());
            }
            return ComposeAction::Continue; // Nothing to erase
        }

        // Enter: process completed line
        if ch == '\r' || ch == '\n' {
            return self.handle_line();
        }

        // Ignore other control characters
        if ch.is_control() {
            return ComposeAction::Continue;
        }

        // Enforce max length
        if self.input_buffer.len() >= self.max_input_length() {
            return ComposeAction::Continue;
        }

        // Accumulate printable character
        self.input_buffer.push(ch);
        ComposeAction::Echo(ch.to_string())
    }

    /// Handle a completed line of input (Enter was pressed).
    fn handle_line(&mut self) -> ComposeAction {
        let input = std::mem::take(&mut self.input_buffer);
        let trimmed = input.trim();

        match self.state {
            ComposeState::InputTo => {
                if trimmed.is_empty() {
                    self.state = ComposeState::PromptTo;
                    return ComposeAction::ShowPrompt("To: ".to_string());
                }
                // Request async DB lookup from session
                ComposeAction::NeedRecipientLookup(trimmed.to_string())
            }
            ComposeState::InputSubject => {
                self.subject = Some(trimmed.to_string());
                self.state = ComposeState::PromptBody;
                ComposeAction::ShowPrompt(self.current_prompt().to_string())
            }
            ComposeState::InputBody => {
                // Check for slash commands
                if trimmed.starts_with("/s") || trimmed.starts_with("/S") {
                    // Send message
                    if let (Some(recipient_id), Some(recipient_handle), Some(subject)) = (
                        self.recipient_id,
                        self.recipient_handle.as_ref(),
                        self.subject.as_ref(),
                    ) {
                        self.state = ComposeState::Sending;
                        return ComposeAction::SendMessage {
                            recipient_id,
                            recipient_handle: recipient_handle.clone(),
                            subject: subject.clone(),
                            body: self.get_body(),
                        };
                    }
                    // Should never happen (missing fields), just continue
                    ComposeAction::Continue
                } else if trimmed.starts_with("/a") || trimmed.starts_with("/A") {
                    // Abort
                    self.state = ComposeState::Aborted;
                    ComposeAction::Aborted
                } else if trimmed.starts_with("/h") || trimmed.starts_with("/H") {
                    // Show help
                    ComposeAction::ShowHelp
                } else if trimmed.starts_with("/l") || trimmed.starts_with("/L") {
                    // List lines
                    let formatted = format_body_lines(&self.body_lines);
                    ComposeAction::ShowLines(formatted)
                } else {
                    // Regular body line
                    self.body_lines.push(input); // Use original input (not trimmed) to preserve whitespace
                    ComposeAction::Continue
                }
            }
            _ => ComposeAction::Continue,
        }
    }

    /// Set recipient after successful DB lookup.
    ///
    /// Called by session.rs after NeedRecipientLookup action.
    /// Advances to PromptSubject (or PromptBody if is_reply).
    pub fn set_recipient(&mut self, id: i64, handle: String) {
        self.recipient_id = Some(id);
        self.recipient_handle = Some(handle);

        if self.is_reply {
            self.state = ComposeState::PromptBody;
        } else {
            self.state = ComposeState::PromptSubject;
        }
    }

    /// Handle recipient lookup error.
    ///
    /// Stays in InputTo state so user can try again.
    pub fn set_recipient_error(&mut self) {
        self.state = ComposeState::PromptTo;
    }

    /// Get the composed message body.
    ///
    /// Joins body_lines with "\n" (LF for storage).
    /// Session converts to CRLF for display.
    pub fn get_body(&self) -> String {
        self.body_lines.join("\n")
    }

    /// Transition to input state after prompt is shown.
    ///
    /// Called by session after showing prompt.
    pub fn advance_to_input(&mut self) {
        match self.state {
            ComposeState::PromptTo => self.state = ComposeState::InputTo,
            ComposeState::PromptSubject => self.state = ComposeState::InputSubject,
            ComposeState::PromptBody => self.state = ComposeState::InputBody,
            _ => {}
        }
    }
}

// ============================================================================
// RENDER FUNCTIONS
// ============================================================================

/// Render inbox as ANSI table with CP437 box-drawing.
///
/// Table layout (80 columns total):
/// - #(3) | From(18) | Subject(33) | Date(12) | St(2) = 68 data cols
/// - Borders: 5 vertical bars + 2 outer = 7 border chars
/// - Total: 68 + 7 = 75 (need 80, adjusted below)
///
/// Adjusted: #(3) | From(20) | Subject(32) | Date(12) | St(2) = 69 data + 5 borders + 2 outer = 76
/// Final: #(3) | From(20) | Subject(33) | Date(12) | St(2) = 70 data + 5 borders + 2 outer = 77
/// Corrected: #(3) | From(18) | Subject(33) | Date(12) | St(3) = 69 data + 6 borders = 75 + 2 outer = 77 (need 3 more)
/// Final calc: 3 + 1 + 18 + 1 + 33 + 1 + 12 + 1 + 3 + 1 = 74 inner + 2 outer = 76
/// Add 1 to Subject: 3 + 1 + 18 + 1 + 34 + 1 + 12 + 1 + 3 + 1 = 75 inner + 2 outer = 77
/// Add 1 more to Subject: 3 + 1 + 18 + 1 + 35 + 1 + 12 + 1 + 3 + 1 = 76 inner + 2 outer = 78
/// Add 1 to From: 3 + 1 + 19 + 1 + 35 + 1 + 12 + 1 + 3 + 1 = 77 inner + 2 outer = 79
/// Add 1 to From: 3 + 1 + 20 + 1 + 35 + 1 + 12 + 1 + 3 + 1 = 78 inner + 2 outer = 80 ✓
pub fn render_inbox(
    entries: &[InboxEntry],
    page: i64,
    total_count: i64,
    page_size: i64,
    sender_handles: &HashMap<i64, String>,
) -> String {
    let mut w = AnsiWriter::new();
    let border_color = Color::LightCyan;
    let header_color = Color::Yellow;
    let value_color = Color::White;
    let unread_color = Color::Yellow;
    let read_color = Color::LightGray;

    // Clear screen
    w.clear_screen();

    // Top border (78 inner chars + 2 outer = 80)
    w.set_fg(border_color);
    w.writeln(&format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(78)));

    // Title: "MAIL INBOX" centered with padding
    let title = "MAIL INBOX";
    let title_padding = (78 - title.len()) / 2;
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(title_padding));
    w.set_fg(header_color);
    w.bold();
    w.write_str(title);
    w.reset_color();
    w.write_str(&" ".repeat(78 - title_padding - title.len()));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(78)));

    // Column headers: #(3) | From(20) | Subject(35) | Date(12) | St(3)
    // Total inner: 3 + 1 + 20 + 1 + 35 + 1 + 12 + 1 + 3 = 77
    // Need 78, add 1 space after St
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:^3}", "#"));
    w.reset_color();
    w.set_fg(border_color);
    w.write_str("\u{2502}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:<20}", " From"));
    w.reset_color();
    w.set_fg(border_color);
    w.write_str("\u{2502}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:<35}", " Subject"));
    w.reset_color();
    w.set_fg(border_color);
    w.write_str("\u{2502}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:<12}", " Date"));
    w.reset_color();
    w.set_fg(border_color);
    w.write_str("\u{2502}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:<4}", " St"));
    w.reset_color();
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Header separator
    w.set_fg(border_color);
    w.writeln(&format!(
        "\u{255F}{}\u{253C}{}\u{253C}{}\u{253C}{}\u{253C}{}\u{2562}",
        "\u{2500}".repeat(3),
        "\u{2500}".repeat(20),
        "\u{2500}".repeat(35),
        "\u{2500}".repeat(12),
        "\u{2500}".repeat(4),
    ));

    // Data rows or empty message
    if entries.is_empty() {
        w.set_fg(border_color);
        w.write_str("\u{2551}");
        w.set_fg(read_color);
        w.write_str(&format!("{:^78}", "No messages."));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    } else {
        for (idx, entry) in entries.iter().enumerate() {
            let msg_num = page * page_size + idx as i64 + 1;
            let sender_handle = sender_handles
                .get(&entry.sender_id)
                .map(|s| s.as_str())
                .unwrap_or("(unknown)");
            let subject = truncate(&entry.subject, 35);
            let date = format_date_short(&entry.sent_at);
            let status = if entry.is_read == 0 { "N" } else { " " };

            let num_color = if entry.is_read == 0 {
                unread_color
            } else {
                read_color
            };

            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(num_color);
            if entry.is_read == 0 {
                w.bold();
            }
            w.write_str(&format!("{:>3}", msg_num));
            w.reset_color();
            w.set_fg(border_color);
            w.write_str("\u{2502}");
            w.set_fg(value_color);
            w.write_str(&format!(" {:<19}", truncate(sender_handle, 19)));
            w.set_fg(border_color);
            w.write_str("\u{2502}");
            w.set_fg(value_color);
            w.write_str(&format!(" {:<34}", subject));
            w.set_fg(border_color);
            w.write_str("\u{2502}");
            w.set_fg(value_color);
            w.write_str(&format!(" {:<11}", date));
            w.set_fg(border_color);
            w.write_str("\u{2502}");
            w.set_fg(num_color);
            if entry.is_read == 0 {
                w.bold();
            }
            w.write_str(&format!(" {:<2} ", status));
            w.reset_color();
            w.set_fg(border_color);
            w.writeln("\u{2551}");
        }
    }

    // Bottom border
    w.set_fg(border_color);
    w.writeln(&format!("\u{255A}{}\u{255D}", "\u{2550}".repeat(78)));

    // Footer with page info
    let total_pages = (total_count + page_size - 1) / page_size;
    let footer = if total_count > 0 {
        format!(
            "Page {} of {} ({} message{})",
            page + 1,
            total_pages,
            total_count,
            if total_count == 1 { "" } else { "s" }
        )
    } else {
        "No messages".to_string()
    };
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  {}", footer));

    // Action bar
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [#] ");
    w.set_fg(Color::White);
    w.write_str("Read  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[C] ");
    w.set_fg(Color::White);
    w.write_str("Compose  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[N] ");
    w.set_fg(Color::White);
    w.write_str("Next  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[P] ");
    w.set_fg(Color::White);
    w.write_str("Prev  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[Q] ");
    w.set_fg(Color::White);
    w.writeln("Quit");
    w.reset_color();

    w.flush()
}

/// Render a full message view with CP437 box-drawing.
pub fn render_message(msg: &Message, sender_handle: &str) -> String {
    let mut w = AnsiWriter::new();
    let border_color = Color::LightCyan;
    let label_color = Color::LightGray;
    let value_color = Color::White;
    let from_color = Color::Yellow;

    let inner = 78;

    // Clear screen
    w.clear_screen();

    // Top border
    w.set_fg(border_color);
    w.writeln(&format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(inner)));

    // From field
    let from_label = "From: ";
    let from_content = format!("{}{}", from_label, sender_handle);
    let padding = if from_content.len() < inner - 2 {
        inner - 2 - from_content.len()
    } else {
        0
    };
    w.set_fg(border_color);
    w.write_str("\u{2551}  ");
    w.set_fg(label_color);
    w.write_str(from_label);
    w.set_fg(from_color);
    w.bold();
    w.write_str(sender_handle);
    w.reset_color();
    w.write_str(&" ".repeat(padding));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Date field
    let date_str = format_datetime(&msg.sent_at);
    let date_label = "Date: ";
    let date_content = format!("{}{}", date_label, date_str);
    let padding = if date_content.len() < inner - 2 {
        inner - 2 - date_content.len()
    } else {
        0
    };
    w.set_fg(border_color);
    w.write_str("\u{2551}  ");
    w.set_fg(label_color);
    w.write_str(date_label);
    w.set_fg(label_color);
    w.write_str(&date_str);
    w.reset_color();
    w.write_str(&" ".repeat(padding));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Subject field
    let subj_label = "Subject: ";
    let subj_content = format!("{}{}", subj_label, msg.subject);
    let padding = if subj_content.len() < inner - 2 {
        inner - 2 - subj_content.len()
    } else {
        0
    };
    w.set_fg(border_color);
    w.write_str("\u{2551}  ");
    w.set_fg(label_color);
    w.write_str(subj_label);
    w.set_fg(value_color);
    w.bold();
    w.write_str(&msg.subject);
    w.reset_color();
    w.write_str(&" ".repeat(padding));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2500}".repeat(inner)));

    // Body lines (convert \n to \r\n for terminal display)
    let body_lines: Vec<&str> = msg.body.lines().collect();
    for line in &body_lines {
        let padding = if line.len() < inner - 2 {
            inner - 2 - line.len()
        } else {
            0
        };
        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(value_color);
        w.write_str(&truncate(line, inner - 2));
        w.write_str(&" ".repeat(padding));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Blank line after body if body exists
    if !body_lines.is_empty() {
        w.set_fg(border_color);
        w.write_str("\u{2551}");
        w.write_str(&" ".repeat(inner));
        w.writeln("\u{2551}");
    }

    // Bottom border
    w.set_fg(border_color);
    w.writeln(&format!("\u{255A}{}\u{255D}", "\u{2550}".repeat(inner)));

    // Action bar
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [R] ");
    w.set_fg(Color::White);
    w.write_str("Reply  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[D] ");
    w.set_fg(Color::White);
    w.write_str("Delete  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[N] ");
    w.set_fg(Color::White);
    w.write_str("Next  ");
    w.set_fg(Color::LightCyan);
    w.write_str("[Q] ");
    w.set_fg(Color::White);
    w.writeln("Quit to inbox");
    w.reset_color();

    w.flush()
}

/// Render compose header showing recipient.
pub fn render_compose_header(recipient_handle: &str) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  COMPOSE MESSAGE");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.write_str("  To: ");
    w.set_fg(Color::Yellow);
    w.writeln(recipient_handle);
    w.reset_color();
    w.writeln("");

    w.flush()
}

/// Render new mail notification (shown when user has unread messages).
pub fn render_new_mail_notification(unread_count: i64) -> String {
    let mut w = AnsiWriter::new();

    let message = if unread_count == 1 {
        format!("*** You have {} new message. ***", unread_count)
    } else {
        format!("*** You have {} new messages. ***", unread_count)
    };

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  {}", message));
    w.reset_color();
    w.writeln("");

    w.flush()
}

/// Render mailbox full error.
pub fn render_mailbox_full_error() -> String {
    let mut w = AnsiWriter::new();

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.writeln("  Recipient's mailbox is full. Message not sent.");
    w.reset_color();
    w.writeln("");

    w.flush()
}

/// Render self-mail error.
pub fn render_self_mail_error() -> String {
    let mut w = AnsiWriter::new();

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.writeln("  You cannot send mail to yourself.");
    w.reset_color();
    w.writeln("");

    w.flush()
}

/// Render user not found error.
pub fn render_user_not_found_error(handle: &str) -> String {
    let mut w = AnsiWriter::new();

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.writeln(&format!("  User '{}' not found.", handle));
    w.reset_color();
    w.writeln("");

    w.flush()
}

/// Render compose help text.
pub fn render_compose_help() -> String {
    let mut w = AnsiWriter::new();

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  Message Commands:");
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln("    /s  - Send message");
    w.writeln("    /a  - Abort compose");
    w.writeln("    /h  - Show this help");
    w.writeln("    /l  - List entered lines");
    w.reset_color();
    w.writeln("");

    w.flush()
}

/// Format body lines with line numbers for /l command.
pub fn format_body_lines(lines: &[String]) -> String {
    let mut w = AnsiWriter::new();

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  Message Body:");
    w.reset_color();

    if lines.is_empty() {
        w.set_fg(Color::LightGray);
        w.writeln("    (empty)");
    } else {
        for (i, line) in lines.iter().enumerate() {
            w.set_fg(Color::LightGray);
            w.write_str(&format!("  {:3}: ", i + 1));
            w.set_fg(Color::White);
            w.writeln(line);
        }
    }

    w.reset_color();
    w.writeln("");

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_short() {
        assert_eq!(format_date_short("2026-01-26 14:15:00"), "Jan 26");
        assert_eq!(format_date_short("2026-12-01T09:30:00"), "Dec 01");
    }

    #[test]
    fn test_format_datetime() {
        assert_eq!(
            format_datetime("2026-01-26 16:15:00"),
            "January 26, 2026 at 4:15 PM"
        );
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
    }
}
