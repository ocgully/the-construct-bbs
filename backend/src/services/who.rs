use crate::connection::node_manager::NodeInfo;
use crate::terminal::{AnsiWriter, Color};

/// Render the Who's Online display as ANSI art.
///
/// Shows a table with columns: Node | Handle | Activity | Idle
/// Uses CP437 box-drawing characters for the BBS aesthetic.
pub fn render_whos_online(nodes: &[(usize, NodeInfo)]) -> String {
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
    w.writeln("  WHO'S ONLINE");
    w.reset_color();
    w.writeln("");

    // Table header with box drawing
    // Total width: 6 + 20 + 30 + 20 = 76 data columns + 5 borders = 81 (need 80)
    // Adjusted: 6 + 20 + 30 + 18 = 74 data + 5 borders = 79 (need 80)
    // Adjusted: 6 + 20 + 30 + 19 = 75 data + 5 borders = 80 ✓
    w.set_fg(border_color);
    w.writeln(&format!("\u{2554}{}\u{2566}{}\u{2566}{}\u{2566}{}\u{2557}",
        "\u{2550}".repeat(6),   // Node
        "\u{2550}".repeat(20),  // Handle
        "\u{2550}".repeat(30),  // Activity
        "\u{2550}".repeat(19),  // Idle
    ));

    // Header row
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:^6}", "Node"));
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
    w.write_str(&format!("{:<30}", " Activity"));
    w.reset_color();
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.set_fg(header_color);
    w.bold();
    w.write_str(&format!("{:<19}", " Idle"));
    w.reset_color();
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Header separator
    w.writeln(&format!("\u{2560}{}\u{256C}{}\u{256C}{}\u{256C}{}\u{2563}",
        "\u{2550}".repeat(6),
        "\u{2550}".repeat(20),
        "\u{2550}".repeat(30),
        "\u{2550}".repeat(19),
    ));

    // Data rows
    if nodes.is_empty() {
        w.set_fg(border_color);
        w.write_str("\u{2551}");
        w.set_fg(label_color);
        w.write_str(&format!("{:^75}", "No users online"));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    } else {
        for (node_id, info) in nodes {
            // Calculate idle time from last_input
            let idle = chrono::Utc::now() - info.last_input;
            let idle_str = format_idle_time(idle);

            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(value_color);
            w.bold();
            w.write_str(&format!("{:^6}", node_id));
            w.reset_color();
            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(value_color);
            w.write_str(&format!(" {:<19}", truncate(&info.handle, 19)));
            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(label_color);
            w.write_str(&format!(" {:<29}", truncate(&info.current_activity, 29)));
            w.set_fg(border_color);
            w.write_str("\u{2551}");
            w.set_fg(label_color);
            w.write_str(&format!(" {:<18}", idle_str));
            w.set_fg(border_color);
            w.writeln("\u{2551}");
        }
    }

    // Bottom border
    w.set_fg(border_color);
    w.writeln(&format!("\u{255A}{}\u{2569}{}\u{2569}{}\u{2569}{}\u{255D}",
        "\u{2550}".repeat(6),
        "\u{2550}".repeat(20),
        "\u{2550}".repeat(30),
        "\u{2550}".repeat(19),
    ));

    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

fn format_idle_time(duration: chrono::Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 60 { return format!("{}s", secs); }
    let mins = secs / 60;
    if mins < 60 { return format!("{}m", mins); }
    let hours = mins / 60;
    format!("{}h {}m", hours, mins % 60)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}...", &s[..max.saturating_sub(3)]) }
}
