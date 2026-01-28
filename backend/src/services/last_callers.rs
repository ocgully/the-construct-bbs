use crate::db::session_history::SessionHistoryEntry;
use crate::terminal::{AnsiWriter, Color};

/// Render the Last Callers display as ANSI art.
///
/// Shows a table with columns: # | Handle | Date/Time | Duration
/// Entries are sorted by most recent first.
pub fn render_last_callers(entries: &[SessionHistoryEntry]) -> String {
    let mut w = AnsiWriter::new();
    let border_color = Color::LightCyan;
    let header_color = Color::Yellow;
    let label_color = Color::LightGray;
    let value_color = Color::White;

    // Clear screen
    w.clear_screen();

    // Title
    w.set_fg(header_color);
    w.bold();
    w.writeln("  LAST CALLERS");
    w.reset_color();
    w.writeln("");

    // Table header
    // Total width calculation: 4 + 20 + 34 + 17 = 75 data columns + 5 borders = 80 ✓
    w.set_fg(border_color);
    w.writeln(&format!("\u{2554}{}\u{2566}{}\u{2566}{}\u{2566}{}\u{2557}",
        "\u{2550}".repeat(4),   // #
        "\u{2550}".repeat(20),  // Handle
        "\u{2550}".repeat(34),  // Date/Time
        "\u{2550}".repeat(17),  // Duration
    ));

    // Header labels
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:^4}", "#"));
    w.reset_color();
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:<20}", " Handle"));
    w.reset_color();
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:<34}", " Date/Time"));
    w.reset_color();
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:<17}", " Duration"));
    w.reset_color();
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Separator
    w.writeln(&format!("\u{2560}{}\u{256C}{}\u{256C}{}\u{256C}{}\u{2563}",
        "\u{2550}".repeat(4),
        "\u{2550}".repeat(20),
        "\u{2550}".repeat(34),
        "\u{2550}".repeat(17),
    ));

    // Data rows
    if entries.is_empty() {
        w.set_fg(border_color);
        w.write_str("\u{2551}");
        w.set_fg(label_color);
        w.write_str(&format!("{:^75}", "No callers recorded yet"));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    } else {
        for (i, entry) in entries.iter().enumerate() {
            let duration_str = format_duration(entry.duration_minutes);

            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(value_color);
            w.bold();
            w.write_str(&format!("{:>3} ", i + 1));
            w.reset_color();
            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(value_color);
            w.write_str(&format!(" {:<19}", truncate(&entry.handle, 19)));
            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(label_color);
            w.write_str(&format!(" {:<33}", truncate(&entry.login_time, 33)));
            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(label_color);
            w.write_str(&format!(" {:<16}", duration_str));
            w.set_fg(border_color);
            w.writeln("\u{2551}");
        }
    }

    // Bottom border
    w.set_fg(border_color);
    w.writeln(&format!("\u{255A}{}\u{2569}{}\u{2569}{}\u{2569}{}\u{255D}",
        "\u{2550}".repeat(4),
        "\u{2550}".repeat(20),
        "\u{2550}".repeat(34),
        "\u{2550}".repeat(17),
    ));

    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

fn format_duration(minutes: i32) -> String {
    if minutes < 60 {
        format!("{}m", minutes)
    } else {
        format!("{}h {}m", minutes / 60, minutes % 60)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}...", &s[..max.saturating_sub(3)]) }
}
