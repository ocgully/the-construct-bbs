use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::config::ConnectionConfig;
use crate::connection::NodeManager;
use crate::terminal::{AnsiWriter, Color};

/// Send a single ceremony line via the tx channel, appending \r\n.
async fn send_ceremony_line(tx: &mpsc::Sender<String>, text: &str) {
    let line = format!("{}\r\n", text);
    let _ = tx.send(line).await;
}

/// Run the full connection ceremony: modem simulation, protocol negotiation,
/// and node assignment. Returns the assigned node_id on success, or Err if
/// all lines are busy (triggers line-busy flow).
///
/// The ceremony writes directly to the tx channel with timed delays for
/// authentic typewriter pacing, bypassing the output_buffer.
pub async fn run_connection_ceremony(
    tx: &mpsc::Sender<String>,
    node_manager: &NodeManager,
    config: &ConnectionConfig,
) -> Result<usize, String> {
    // Check node availability first
    let (active, max) = node_manager.get_status().await;
    if active >= max {
        send_line_busy(tx, max).await;
        return Err("All lines busy".to_string());
    }

    // Assign node eagerly with placeholder info (updated after login)
    let node_id = node_manager
        .assign_node(0, "connecting".to_string())
        .await
        .map_err(|e| {
            // Race condition: node filled between check and assign
            e
        })?;

    let (_, max_nodes) = node_manager.get_status().await;

    // Build and send ceremony lines with typewriter pacing
    // Blank line
    send_ceremony_line(tx, "").await;

    // Modem dial command
    let mut w = AnsiWriter::new();
    w.set_fg(Color::LightGreen);
    w.write_str("ATDT 555-0199");
    w.reset_color();
    send_ceremony_line(tx, &w.flush()).await;
    sleep(Duration::from_millis(500)).await;

    // Connect speed
    let mut w = AnsiWriter::new();
    w.set_fg(Color::Yellow);
    w.bold();
    w.write_str("CONNECT 38400");
    w.reset_color();
    send_ceremony_line(tx, &w.flush()).await;
    sleep(Duration::from_millis(300)).await;

    // Blank line
    send_ceremony_line(tx, "").await;

    // Protocol negotiation
    let mut w = AnsiWriter::new();
    w.set_fg(Color::Green);
    w.write_str("Negotiating protocols...");
    w.reset_color();
    send_ceremony_line(tx, &w.flush()).await;
    sleep(Duration::from_millis(400)).await;

    // Terminal detection
    let mut w = AnsiWriter::new();
    w.set_fg(Color::Green);
    w.write_str("ANSI/CP437 terminal detected.");
    w.reset_color();
    send_ceremony_line(tx, &w.flush()).await;
    sleep(Duration::from_millis(300)).await;

    // Blank line
    send_ceremony_line(tx, "").await;

    // Connecting message
    let mut w = AnsiWriter::new();
    w.set_fg(Color::LightCyan);
    w.bold();
    w.write_str("Connecting to The Construct BBS...");
    w.reset_color();
    send_ceremony_line(tx, &w.flush()).await;
    sleep(Duration::from_millis(600)).await;

    // Node assignment info
    let mut w = AnsiWriter::new();
    w.set_fg(Color::Yellow);
    w.write_str(&format!("Connected to Node {} of {}", node_id, max_nodes));
    w.reset_color();
    send_ceremony_line(tx, &w.flush()).await;

    // Blank line
    send_ceremony_line(tx, "").await;

    Ok(node_id)
}

/// Send the "ALL LINES BUSY" rejection message with CP437 box-drawing art.
/// Waits 3 seconds after displaying so the user can read the message.
pub async fn send_line_busy(tx: &mpsc::Sender<String>, max_nodes: usize) {
    let mut w = AnsiWriter::new();

    w.set_fg(Color::LightRed);
    w.writeln("");
    w.writeln("");

    // CP437 box-drawing: top border
    // ┌──────────────────────────────────────────┐
    w.write_cp437(&[0xDA]); // ┌
    for _ in 0..42 {
        w.write_cp437(&[0xC4]); // ─
    }
    w.write_cp437(&[0xBF]); // ┐
    w.writeln("");

    // │   ALL LINES ARE BUSY - PLEASE TRY AGAIN  │
    w.write_cp437(&[0xB3]); // │
    w.write_str("   ALL LINES ARE BUSY - PLEASE TRY AGAIN  ");
    w.write_cp437(&[0xB3]); // │
    w.writeln("");

    // │                                          │
    w.write_cp437(&[0xB3]); // │
    w.write_str("                                          ");
    w.write_cp437(&[0xB3]); // │
    w.writeln("");

    // │   The Construct BBS - 0 nodes available  │
    w.write_cp437(&[0xB3]); // │
    w.write_str(&format!(
        "   The Construct BBS - {} nodes available  ",
        0
    ));
    w.write_cp437(&[0xB3]); // │
    w.writeln("");

    // └──────────────────────────────────────────┘
    w.write_cp437(&[0xC0]); // └
    for _ in 0..42 {
        w.write_cp437(&[0xC4]); // ─
    }
    w.write_cp437(&[0xD9]); // ┘
    w.writeln("");

    w.reset_color();
    w.writeln("");

    let _ = tx.send(w.flush()).await;

    // Give user time to read the message before disconnect
    sleep(Duration::from_secs(3)).await;
}

/// Send the ANSI art splash screen line-by-line with baud-rate simulation.
/// Each line is sent with synchronized rendering and a delay proportional
/// to line length divided by baud_cps.
pub async fn send_splash_screen(tx: &mpsc::Sender<String>, baud_cps: u32) {
    let splash_lines = build_splash_art();

    for line in &splash_lines {
        // Wrap each line in sync markers to prevent tearing
        let mut output = String::new();
        output.push_str("\x1B[?2026h"); // begin sync
        output.push_str(line);
        output.push_str("\r\n");
        output.push_str("\x1B[?2026l"); // end sync

        let _ = tx.send(output).await;

        // Baud rate delay: visible_length * 1000 / baud_cps, minimum 50ms
        let visible_len = strip_ansi_len(line);
        let delay_ms = if baud_cps > 0 {
            ((visible_len as u64) * 1000 / baud_cps as u64).max(50)
        } else {
            50
        };
        sleep(Duration::from_millis(delay_ms)).await;
    }
}

/// Estimate visible character count by stripping ANSI escape sequences.
fn strip_ansi_len(s: &str) -> usize {
    let mut count = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() || ch == 'm' || ch == 'h' || ch == 'l' {
                in_escape = false;
            }
            continue;
        }
        if ch == '\x1B' {
            in_escape = true;
            continue;
        }
        count += 1;
    }
    count
}

/// Build the ANSI art splash screen as a vector of pre-formatted lines.
/// Uses CP437 box-drawing and CGA colors for authentic BBS atmosphere.
fn build_splash_art() -> Vec<String> {
    let mut lines = Vec::new();

    // Helper to build a single ANSI line
    macro_rules! line {
        ($body:expr) => {{
            let mut w = AnsiWriter::new();
            $body(&mut w);
            w.reset_color();
            w.flush()
        }};
    }

    // Blank line at top
    lines.push(String::new());

    // Double-line top border
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xC9]); // ╔
        for _ in 0..78 {
            w.write_cp437(&[0xCD]); // ═
        }
        w.write_cp437(&[0xBB]); // ╗
    }));

    // Empty row inside box
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.write_str(&" ".repeat(78));
        w.write_cp437(&[0xBA]); // ║
    }));

    // Title: THE CONSTRUCT
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.set_fg(Color::White);
        w.bold();
        w.write_str("              "); // 14 spaces
        w.write_cp437(&[0xDB, 0xDB]); // ██
        w.write_str(" THE CONSTRUCT BBS ");
        w.write_cp437(&[0xDB, 0xDB]); // ██
        // Pad to 78 chars: 14 + 2 + 19 + 2 = 37, need 41 more
        w.write_str(&" ".repeat(41));
        w.set_fg(Color::LightCyan);
        w.write_cp437(&[0xBA]); // ║
    }));

    // Empty row
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.write_str(&" ".repeat(78));
        w.write_cp437(&[0xBA]); // ║
    }));

    // Tagline
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.set_fg(Color::Yellow);
        // Center: "Where the underground connects" = 30 chars
        w.write_str("                        Where the underground connects                        ");
        w.set_fg(Color::LightCyan);
        w.write_cp437(&[0xBA]); // ║
    }));

    // Empty row
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.write_str(&" ".repeat(78));
        w.write_cp437(&[0xBA]); // ║
    }));

    // Decorative divider inside box (single-line)
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xCC]); // ╠
        w.set_fg(Color::Brown);
        for _ in 0..78 {
            w.write_cp437(&[0xC4]); // ─
        }
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xB9]); // ╣
    }));

    // Empty row
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.write_str(&" ".repeat(78));
        w.write_cp437(&[0xBA]); // ║
    }));

    // System info line 1
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.set_fg(Color::Green);
        w.write_str("      Running on Rust ");
        w.write_cp437(&[0xB3]); // │
        w.write_str(" ANSI/CP437 Terminal ");
        w.write_cp437(&[0xB3]); // │
        w.write_str(" 16-Color CGA Palette           ");
        w.set_fg(Color::LightCyan);
        w.write_cp437(&[0xBA]); // ║
    }));

    // Empty row
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.write_str(&" ".repeat(78));
        w.write_cp437(&[0xBA]); // ║
    }));

    // Decorative block art row
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.set_fg(Color::DarkGray);
        w.write_str("    ");
        // Decorative blocks
        for _ in 0..70 {
            w.write_cp437(&[0xB0]); // ░
        }
        w.write_str("    ");
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
    }));

    // Empty row
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.write_str(&" ".repeat(78));
        w.write_cp437(&[0xBA]); // ║
    }));

    // Double-line bottom border
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xC8]); // ╚
        for _ in 0..78 {
            w.write_cp437(&[0xCD]); // ═
        }
        w.write_cp437(&[0xBC]); // ╝
    }));

    // Blank line after splash
    lines.push(String::new());

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_len() {
        assert_eq!(strip_ansi_len("hello"), 5);
        assert_eq!(strip_ansi_len("\x1B[31mhello\x1B[0m"), 5);
        assert_eq!(strip_ansi_len("\x1B[1m\x1B[96mtest\x1B[0m"), 4);
        assert_eq!(strip_ansi_len(""), 0);
    }

    #[test]
    fn test_build_splash_art_has_lines() {
        let lines = build_splash_art();
        assert!(lines.len() >= 10, "Splash should have at least 10 lines");
        assert!(lines.len() <= 25, "Splash should be compact (<=25 lines)");
    }

    #[tokio::test]
    async fn test_send_ceremony_line() {
        let (tx, mut rx) = mpsc::channel::<String>(8);
        send_ceremony_line(&tx, "hello").await;
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, "hello\r\n");
    }

    #[tokio::test]
    async fn test_line_busy_sends_message() {
        let (tx, mut rx) = mpsc::channel::<String>(8);

        // Run line_busy in background (has 3s sleep)
        let handle = tokio::spawn(async move {
            send_line_busy(&tx, 8).await;
        });

        // Should receive the busy message
        let msg = rx.recv().await.unwrap();
        assert!(msg.contains("ALL LINES ARE BUSY"));
        assert!(msg.contains("PLEASE TRY AGAIN"));

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_ceremony_assigns_node() {
        let (tx, mut rx) = mpsc::channel::<String>(64);
        let node_manager = NodeManager::new(4);
        let config = ConnectionConfig::default();

        // Run ceremony in background (has sleeps)
        let handle = tokio::spawn(async move {
            run_connection_ceremony(&tx, &node_manager, &config).await
        });

        // Drain messages
        let mut messages = Vec::new();
        while let Some(msg) = rx.recv().await {
            messages.push(msg);
        }

        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // First node assigned

        // Verify ceremony text was sent
        let all_text: String = messages.join("");
        assert!(all_text.contains("ATDT 555-0199"));
        assert!(all_text.contains("CONNECT 38400"));
        assert!(all_text.contains("Negotiating protocols"));
        assert!(all_text.contains("ANSI/CP437 terminal detected"));
        assert!(all_text.contains("Connecting to The Construct BBS"));
        assert!(all_text.contains("Connected to Node 1 of 4"));
    }

    #[tokio::test]
    async fn test_ceremony_line_busy_when_full() {
        let (tx, mut rx) = mpsc::channel::<String>(64);
        let node_manager = NodeManager::new(1);

        // Fill all nodes
        node_manager.assign_node(1, "Alice".into()).await.unwrap();

        let config = ConnectionConfig::default();

        let handle = tokio::spawn(async move {
            run_connection_ceremony(&tx, &node_manager, &config).await
        });

        // Drain messages
        let mut messages = Vec::new();
        while let Some(msg) = rx.recv().await {
            messages.push(msg);
        }

        let result = handle.await.unwrap();
        assert!(result.is_err());

        let all_text: String = messages.join("");
        assert!(all_text.contains("ALL LINES ARE BUSY"));
    }
}
