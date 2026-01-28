//! Chat command parser and message rendering functions.
//!
//! This module provides pure functions for:
//! - Parsing chat commands (/me, /who, /quit, /msg, /r, /page, /help)
//! - Rendering chat messages with appropriate ANSI colors
//!
//! These functions are decoupled from session state for testability.

use crate::connection::ChatMessage;
use crate::terminal::{AnsiWriter, Color};
use chrono::Utc;

// ============================================================================
// COMMAND PARSER
// ============================================================================

/// Parsed chat command variants.
///
/// Returned by `parse_chat_command()` to indicate what action
/// the user wants to perform.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatCommand {
    /// Empty input (just pressed Enter)
    Empty,
    /// Regular chat message (no leading slash)
    Message(String),
    /// Exit chat (/quit or /q)
    Quit,
    /// Show help (/help or /?)
    Help,
    /// Show users in chat (/who)
    Who,
    /// Action message (/me <action>)
    Action(String),
    /// Page a user (/page <handle>)
    Page(String),
    /// Direct message (/msg <handle> <message>)
    DirectMessage { target: String, text: String },
    /// Reply to last DM sender (/r <message>)
    Reply(String),
    /// Unknown command (starts with / but not recognized)
    Unknown(String),
    /// Usage error (command recognized but missing arguments)
    Error(String),
}

/// Parse user input into a ChatCommand.
///
/// # Arguments
/// * `input` - Raw user input string
///
/// # Returns
/// A `ChatCommand` variant representing the parsed command.
///
/// # Examples
/// ```
/// use bbs::services::chat::{parse_chat_command, ChatCommand};
///
/// assert_eq!(parse_chat_command("hello"), ChatCommand::Message("hello".to_string()));
/// assert_eq!(parse_chat_command("/quit"), ChatCommand::Quit);
/// assert_eq!(parse_chat_command("/me waves"), ChatCommand::Action("waves".to_string()));
/// ```
pub fn parse_chat_command(input: &str) -> ChatCommand {
    let trimmed = input.trim();

    // Empty input
    if trimmed.is_empty() {
        return ChatCommand::Empty;
    }

    // Not a command - regular message
    if !trimmed.starts_with('/') {
        return ChatCommand::Message(trimmed.to_string());
    }

    // Split command and arguments using splitn(2, ' ')
    let mut parts = trimmed.splitn(2, ' ');
    let cmd = parts.next().unwrap(); // Safe: we know it starts with /
    let args = parts.next().unwrap_or("").trim();

    // Match command (case-insensitive)
    match cmd.to_lowercase().as_str() {
        "/quit" | "/q" => ChatCommand::Quit,
        "/help" | "/?" => ChatCommand::Help,
        "/who" => ChatCommand::Who,
        "/me" => {
            if args.is_empty() {
                ChatCommand::Error("Usage: /me <action>".to_string())
            } else {
                ChatCommand::Action(args.to_string())
            }
        }
        "/page" => {
            if args.is_empty() {
                ChatCommand::Error("Usage: /page <handle>".to_string())
            } else {
                // Take only first word as the handle
                let target = args.split_whitespace().next().unwrap_or("");
                ChatCommand::Page(target.to_string())
            }
        }
        "/msg" => {
            // Split args into target (first word) and text (rest)
            let mut msg_parts = args.splitn(2, ' ');
            let target = msg_parts.next().unwrap_or("");
            let text = msg_parts.next().unwrap_or("").trim();

            if target.is_empty() || text.is_empty() {
                ChatCommand::Error("Usage: /msg <handle> <message>".to_string())
            } else {
                ChatCommand::DirectMessage {
                    target: target.to_string(),
                    text: text.to_string(),
                }
            }
        }
        "/r" => {
            if args.is_empty() {
                ChatCommand::Error("Usage: /r <message>".to_string())
            } else {
                ChatCommand::Reply(args.to_string())
            }
        }
        _ => ChatCommand::Unknown(cmd.to_string()),
    }
}

// ============================================================================
// MESSAGE RENDERING
// ============================================================================

/// Render a chat message as an ANSI-formatted string.
///
/// # Arguments
/// * `msg` - The chat message to render
/// * `my_handle` - The current user's handle (for filtering direct messages)
///
/// # Returns
/// An ANSI-formatted string ready for terminal output.
/// Returns empty string for direct messages not involving the current user.
///
/// # Color scheme
/// - Public: Green handle, LightGreen message
/// - Action: LightGreen (asterisk prefix)
/// - System: Yellow (triple asterisks)
/// - Direct: LightCyan for arrow+handle, White for text
/// - Join/Leave: Yellow (triple asterisks)
pub fn render_chat_message(msg: &ChatMessage, my_handle: &str) -> String {
    let mut w = AnsiWriter::new();

    match msg {
        ChatMessage::Public { sender, text } => {
            let timestamp = Utc::now().format("%H:%M");
            w.write_str(&format!("[{}] ", timestamp));
            w.set_fg(Color::Green);
            w.write_str(sender);
            w.reset_color();
            w.write_str(": ");
            w.set_fg(Color::LightGreen);
            w.write_str(text);
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Action { sender, action } => {
            w.set_fg(Color::LightGreen);
            w.write_str("* ");
            w.write_str(sender);
            w.write_str(" ");
            w.write_str(action);
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::System { text } => {
            w.set_fg(Color::Yellow);
            w.write_str("*** ");
            w.write_str(text);
            w.write_str(" ***");
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Direct { from, to, text } => {
            // Only render if I'm sender or recipient
            if from != my_handle && to != my_handle {
                return String::new();
            }
            let timestamp = Utc::now().format("%H:%M");
            let (prefix, other) = if from == my_handle {
                ("-> ", to.as_str())
            } else {
                ("<- ", from.as_str())
            };
            w.write_str(&format!("[{}] ", timestamp));
            w.set_fg(Color::LightCyan);
            w.write_str(prefix);
            w.write_str(other);
            w.reset_color();
            w.write_str(": ");
            w.set_fg(Color::White);
            w.write_str(text);
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Join { handle } => {
            w.set_fg(Color::Yellow);
            w.write_str(&format!("*** {} has joined the chat ***", handle));
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Leave { handle } => {
            w.set_fg(Color::Yellow);
            w.write_str(&format!("*** {} has left the chat ***", handle));
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Page { from, to: _ } => {
            w.set_fg(Color::Yellow);
            w.write_str(&format!("*** Page from {} ***", from));
            w.reset_color();
            w.writeln("");
        }
    }

    w.flush()
}

/// Render chat help text.
///
/// Returns ANSI-formatted help text showing available chat commands.
pub fn render_chat_help() -> String {
    let mut w = AnsiWriter::new();
    w.set_fg(Color::LightCyan);
    w.writeln("Chat Commands:");
    w.reset_color();
    w.writeln("  /help, /?     - Show this help");
    w.writeln("  /who          - List users in chat");
    w.writeln("  /me <action>  - Perform action (e.g., /me waves)");
    w.writeln("  /msg <user> <message> - Send private message");
    w.writeln("  /r <message>  - Reply to last private message");
    w.writeln("  /page <user>  - Page a user (bell notification)");
    w.writeln("  /quit, /q     - Exit chat");
    w.writeln("");
    w.flush()
}

/// Render the list of users currently in chat.
///
/// # Arguments
/// * `participants` - Slice of handle strings for users in chat
///
/// # Returns
/// ANSI-formatted list of participants.
pub fn render_chat_who(participants: &[String]) -> String {
    let mut w = AnsiWriter::new();
    w.set_fg(Color::LightCyan);
    w.writeln(&format!("Users in chat ({}):", participants.len()));
    w.reset_color();
    for handle in participants {
        w.writeln(&format!("  {}", handle));
    }
    w.writeln("");
    w.flush()
}

/// Render the chat welcome message.
///
/// Shown when user first enters the chat room.
pub fn render_chat_welcome() -> String {
    let mut w = AnsiWriter::new();
    w.set_fg(Color::LightCyan);
    w.writeln("=== The Construct Chat ===");
    w.reset_color();
    w.writeln("Type /help for commands, /quit to exit.");
    w.writeln("");
    w.flush()
}

/// Render a chat error message.
///
/// # Arguments
/// * `msg` - The error message text
///
/// # Returns
/// ANSI-formatted error message in LightRed.
pub fn render_chat_error(msg: &str) -> String {
    let mut w = AnsiWriter::new();
    w.set_fg(Color::LightRed);
    w.write_str(msg);
    w.reset_color();
    w.writeln("");
    w.flush()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Command Parser Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_empty_input() {
        assert_eq!(parse_chat_command(""), ChatCommand::Empty);
        assert_eq!(parse_chat_command("   "), ChatCommand::Empty);
    }

    #[test]
    fn test_parse_regular_message() {
        assert_eq!(
            parse_chat_command("hello world"),
            ChatCommand::Message("hello world".to_string())
        );
        assert_eq!(
            parse_chat_command("  spaced message  "),
            ChatCommand::Message("spaced message".to_string())
        );
    }

    #[test]
    fn test_parse_quit_command() {
        assert_eq!(parse_chat_command("/quit"), ChatCommand::Quit);
        assert_eq!(parse_chat_command("/q"), ChatCommand::Quit);
        assert_eq!(parse_chat_command("/QUIT"), ChatCommand::Quit);
        assert_eq!(parse_chat_command("/Q"), ChatCommand::Quit);
    }

    #[test]
    fn test_parse_help_command() {
        assert_eq!(parse_chat_command("/help"), ChatCommand::Help);
        assert_eq!(parse_chat_command("/?"), ChatCommand::Help);
        assert_eq!(parse_chat_command("/HELP"), ChatCommand::Help);
    }

    #[test]
    fn test_parse_who_command() {
        assert_eq!(parse_chat_command("/who"), ChatCommand::Who);
        assert_eq!(parse_chat_command("/WHO"), ChatCommand::Who);
    }

    #[test]
    fn test_parse_me_command() {
        assert_eq!(
            parse_chat_command("/me waves"),
            ChatCommand::Action("waves".to_string())
        );
        assert_eq!(
            parse_chat_command("/ME does something"),
            ChatCommand::Action("does something".to_string())
        );
        assert_eq!(
            parse_chat_command("/me"),
            ChatCommand::Error("Usage: /me <action>".to_string())
        );
        assert_eq!(
            parse_chat_command("/me   "),
            ChatCommand::Error("Usage: /me <action>".to_string())
        );
    }

    #[test]
    fn test_parse_page_command() {
        assert_eq!(
            parse_chat_command("/page SysOp"),
            ChatCommand::Page("SysOp".to_string())
        );
        // Only takes first word as handle
        assert_eq!(
            parse_chat_command("/page SysOp extra stuff"),
            ChatCommand::Page("SysOp".to_string())
        );
        assert_eq!(
            parse_chat_command("/page"),
            ChatCommand::Error("Usage: /page <handle>".to_string())
        );
    }

    #[test]
    fn test_parse_msg_command() {
        assert_eq!(
            parse_chat_command("/msg Alice Hello there!"),
            ChatCommand::DirectMessage {
                target: "Alice".to_string(),
                text: "Hello there!".to_string()
            }
        );
        assert_eq!(
            parse_chat_command("/MSG Bob hi"),
            ChatCommand::DirectMessage {
                target: "Bob".to_string(),
                text: "hi".to_string()
            }
        );
        // Missing message
        assert_eq!(
            parse_chat_command("/msg Alice"),
            ChatCommand::Error("Usage: /msg <handle> <message>".to_string())
        );
        // Missing both
        assert_eq!(
            parse_chat_command("/msg"),
            ChatCommand::Error("Usage: /msg <handle> <message>".to_string())
        );
    }

    #[test]
    fn test_parse_reply_command() {
        assert_eq!(
            parse_chat_command("/r Thanks!"),
            ChatCommand::Reply("Thanks!".to_string())
        );
        assert_eq!(
            parse_chat_command("/R multiple words here"),
            ChatCommand::Reply("multiple words here".to_string())
        );
        assert_eq!(
            parse_chat_command("/r"),
            ChatCommand::Error("Usage: /r <message>".to_string())
        );
    }

    #[test]
    fn test_parse_unknown_command() {
        assert_eq!(
            parse_chat_command("/foo"),
            ChatCommand::Unknown("/foo".to_string())
        );
        assert_eq!(
            parse_chat_command("/unknown stuff"),
            ChatCommand::Unknown("/unknown".to_string())
        );
    }

    // -------------------------------------------------------------------------
    // Message Rendering Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_render_public_message() {
        let msg = ChatMessage::Public {
            sender: "Alice".to_string(),
            text: "Hello!".to_string(),
        };
        let output = render_chat_message(&msg, "Bob");
        // Should contain sender name and message
        assert!(output.contains("Alice"));
        assert!(output.contains("Hello!"));
        // Should contain timestamp format [HH:MM]
        assert!(output.contains("["));
        assert!(output.contains("]"));
    }

    #[test]
    fn test_render_action_message() {
        let msg = ChatMessage::Action {
            sender: "Alice".to_string(),
            action: "waves".to_string(),
        };
        let output = render_chat_message(&msg, "Bob");
        // Should contain asterisk prefix
        assert!(output.contains("* Alice waves"));
    }

    #[test]
    fn test_render_system_message() {
        let msg = ChatMessage::System {
            text: "Server restarting".to_string(),
        };
        let output = render_chat_message(&msg, "Bob");
        // Should contain triple asterisks
        assert!(output.contains("*** Server restarting ***"));
    }

    #[test]
    fn test_render_direct_message_as_sender() {
        let msg = ChatMessage::Direct {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            text: "Secret message".to_string(),
        };
        let output = render_chat_message(&msg, "Alice");
        // Sender sees -> prefix with recipient handle
        assert!(output.contains("->"));
        assert!(output.contains("Bob"));
        assert!(output.contains("Secret message"));
    }

    #[test]
    fn test_render_direct_message_as_recipient() {
        let msg = ChatMessage::Direct {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            text: "Secret message".to_string(),
        };
        let output = render_chat_message(&msg, "Bob");
        // Recipient sees <- prefix with sender handle
        assert!(output.contains("<-"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Secret message"));
    }

    #[test]
    fn test_render_direct_message_not_involved() {
        let msg = ChatMessage::Direct {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            text: "Secret message".to_string(),
        };
        let output = render_chat_message(&msg, "Charlie");
        // Should return empty string - Charlie isn't involved
        assert!(output.is_empty());
    }

    #[test]
    fn test_render_join_message() {
        let msg = ChatMessage::Join {
            handle: "NewUser".to_string(),
        };
        let output = render_chat_message(&msg, "Alice");
        assert!(output.contains("*** NewUser has joined the chat ***"));
    }

    #[test]
    fn test_render_leave_message() {
        let msg = ChatMessage::Leave {
            handle: "LeavingUser".to_string(),
        };
        let output = render_chat_message(&msg, "Alice");
        assert!(output.contains("*** LeavingUser has left the chat ***"));
    }

    #[test]
    fn test_render_page_message() {
        let msg = ChatMessage::Page {
            from: "SysOp".to_string(),
            to: "User".to_string(),
        };
        let output = render_chat_message(&msg, "User");
        assert!(output.contains("*** Page from SysOp ***"));
    }

    // -------------------------------------------------------------------------
    // Helper Rendering Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_render_help() {
        let output = render_chat_help();
        assert!(output.contains("Chat Commands:"));
        assert!(output.contains("/help"));
        assert!(output.contains("/quit"));
        assert!(output.contains("/me"));
        assert!(output.contains("/msg"));
        assert!(output.contains("/r"));
        assert!(output.contains("/page"));
        assert!(output.contains("/who"));
    }

    #[test]
    fn test_render_who() {
        let participants = vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()];
        let output = render_chat_who(&participants);
        assert!(output.contains("Users in chat (3):"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("Charlie"));
    }

    #[test]
    fn test_render_who_empty() {
        let participants: Vec<String> = vec![];
        let output = render_chat_who(&participants);
        assert!(output.contains("Users in chat (0):"));
    }

    #[test]
    fn test_render_welcome() {
        let output = render_chat_welcome();
        assert!(output.contains("The Construct Chat"));
        assert!(output.contains("/help"));
        assert!(output.contains("/quit"));
    }

    #[test]
    fn test_render_error() {
        let output = render_chat_error("Something went wrong");
        assert!(output.contains("Something went wrong"));
        // Should contain ANSI escape for LightRed (code 91)
        assert!(output.contains("\x1B[91m"));
    }
}
