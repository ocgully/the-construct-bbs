use crate::services::profile::format_duration_minutes;
use crate::terminal::{AnsiWriter, Color};

/// Render the goodbye screen with session stats and farewell art.
///
/// Displays a box-drawn ANSI card showing:
/// - Personalized farewell with the user's handle
/// - Session time (this visit)
/// - Total calls (lifetime)
/// - Total time online (lifetime, including this session)
/// - "NO CARRIER" modem disconnect indicator
pub fn render_goodbye(
    handle: &str,
    session_minutes: i64,
    total_calls: i64,
    total_time_minutes: i64,
) -> String {
    let mut w = AnsiWriter::new();

    let border_color = Color::LightCyan;
    let inner = 78; // 80 - 2 border chars

    // Helper: pad a string to a given width (left-aligned)
    let pad = |s: &str, width: usize| -> String {
        if s.len() >= width {
            s[..width].to_string()
        } else {
            format!("{}{}", s, " ".repeat(width - s.len()))
        }
    };

    // Blank line before card
    w.writeln("");

    // Top border
    w.set_fg(border_color);
    w.writeln(&format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(inner)));

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Thanks for calling line
    {
        let msg = format!("Thanks for calling, {}!", handle);
        // Center the message: pad left to center it within inner width
        let left_pad = if msg.len() < inner - 4 {
            (inner - msg.len()) / 2
        } else {
            2
        };
        let right_pad = inner - left_pad - msg.len();

        w.set_fg(border_color);
        w.write_str("\u{2551}");
        w.write_str(&" ".repeat(left_pad));
        w.set_fg(Color::Yellow);
        w.bold();
        w.write_str(&msg);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(inner)));

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Session stats
    let session_time_str = if session_minutes == 1 {
        "1 minute".to_string()
    } else {
        format!("{} minutes", session_minutes)
    };
    let total_calls_str = format!("{}", total_calls);
    let total_time_str = format_duration_minutes(total_time_minutes);

    let stats: Vec<(&str, &str)> = vec![
        ("Session Time:", &session_time_str),
        ("Total Calls:", &total_calls_str),
        ("Total Time:", &total_time_str),
    ];

    for (label, value) in &stats {
        let label_padded = pad(label, 16);
        let content_len = 3 + 16 + value.len();
        let right_pad = if content_len < inner {
            inner - content_len
        } else {
            0
        };

        w.set_fg(border_color);
        w.write_str("\u{2551}   ");
        w.set_fg(Color::LightGray);
        w.write_str(&label_padded);
        w.set_fg(Color::White);
        w.bold();
        w.write_str(value);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(inner)));

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // "Call again soon!" message (centered)
    {
        let msg = "Call again soon! The Construct awaits...";
        let left_pad = if msg.len() < inner - 4 {
            (inner - msg.len()) / 2
        } else {
            2
        };
        let right_pad = inner - left_pad - msg.len();

        w.set_fg(border_color);
        w.write_str("\u{2551}");
        w.write_str(&" ".repeat(left_pad));
        w.set_fg(Color::LightGreen);
        w.write_str(msg);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // "NO CARRIER" (centered)
    {
        let msg = "\u{2500}\u{2500} NO CARRIER \u{2500}\u{2500}";
        let left_pad = if msg.len() < inner - 4 {
            (inner - msg.len()) / 2
        } else {
            2
        };
        let right_pad = inner - left_pad - msg.len();

        w.set_fg(border_color);
        w.write_str("\u{2551}");
        w.write_str(&" ".repeat(left_pad));
        w.set_fg(Color::LightRed);
        w.bold();
        w.write_str(msg);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Bottom border
    w.set_fg(border_color);
    w.writeln(&format!("\u{255A}{}\u{255D}", "\u{2550}".repeat(inner)));
    w.reset_color();

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goodbye_contains_handle() {
        let output = render_goodbye("DarkAngel", 12, 48, 344);
        assert!(output.contains("DarkAngel"));
    }

    #[test]
    fn test_goodbye_contains_session_time() {
        let output = render_goodbye("TestUser", 12, 48, 344);
        assert!(output.contains("12 minutes"));
    }

    #[test]
    fn test_goodbye_contains_total_calls() {
        let output = render_goodbye("TestUser", 12, 48, 344);
        assert!(output.contains("48"));
    }

    #[test]
    fn test_goodbye_contains_total_time() {
        let output = render_goodbye("TestUser", 12, 48, 344);
        assert!(output.contains("5h 44m"));
    }

    #[test]
    fn test_goodbye_contains_no_carrier() {
        let output = render_goodbye("TestUser", 0, 1, 0);
        assert!(output.contains("NO CARRIER"));
    }

    #[test]
    fn test_goodbye_contains_farewell() {
        let output = render_goodbye("TestUser", 0, 1, 0);
        assert!(output.contains("Call again soon!"));
    }

    #[test]
    fn test_goodbye_box_drawing() {
        let output = render_goodbye("TestUser", 5, 10, 60);
        assert!(output.contains("\u{2554}")); // top-left
        assert!(output.contains("\u{2557}")); // top-right
        assert!(output.contains("\u{255A}")); // bottom-left
        assert!(output.contains("\u{255D}")); // bottom-right
    }

    #[test]
    fn test_goodbye_singular_minute() {
        let output = render_goodbye("TestUser", 1, 1, 1);
        assert!(output.contains("1 minute"));
    }
}
