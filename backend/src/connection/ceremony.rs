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
    _config: &ConnectionConfig,
) -> Result<usize, String> {
    // Check node availability first
    let (active, max) = node_manager.get_status().await;
    if active >= max {
        // Signal frontend to play modem-fail sound
        let _ = tx
            .send(r#"{"type":"modem","status":"fail"}"#.to_string())
            .await;
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

    // Signal frontend to play modem-success sound
    let _ = tx
        .send(r#"{"type":"modem","status":"success"}"#.to_string())
        .await;

    // Build and send ceremony lines with typewriter pacing.
    // These fill the visual space while the modem sound plays (~8-10 seconds).

    // Helper: send a green-colored ceremony log line
    macro_rules! log_line {
        ($tx:expr, $color:expr, $text:expr) => {{
            let mut w = AnsiWriter::new();
            w.set_fg($color);
            w.write_str($text);
            w.reset_color();
            send_ceremony_line($tx, &w.flush()).await;
        }};
        ($tx:expr, $color:expr, bold, $text:expr) => {{
            let mut w = AnsiWriter::new();
            w.set_fg($color);
            w.bold();
            w.write_str($text);
            w.reset_color();
            send_ceremony_line($tx, &w.flush()).await;
        }};
    }

    send_ceremony_line(tx, "").await;

    // Modem dial
    log_line!(tx, Color::LightGreen, "ATDT 555-0199");
    sleep(Duration::from_millis(800)).await;

    // Ring
    log_line!(tx, Color::Green, "RING... RING...");
    sleep(Duration::from_millis(1000)).await;

    // Connect speed
    log_line!(tx, Color::Yellow, bold, "CONNECT 38400/ARQ/V.34/LAPM/V.42BIS");
    sleep(Duration::from_millis(600)).await;

    send_ceremony_line(tx, "").await;

    // Carrier detect
    log_line!(tx, Color::Green, "Checking carrier detect... OK");
    sleep(Duration::from_millis(500)).await;

    // Protocol negotiation
    log_line!(tx, Color::Green, "Negotiating protocols...");
    sleep(Duration::from_millis(600)).await;

    log_line!(tx, Color::Green, "ZModem protocol initialized.");
    sleep(Duration::from_millis(400)).await;

    // Terminal detection
    log_line!(tx, Color::Green, "Detecting terminal type...");
    sleep(Duration::from_millis(500)).await;

    log_line!(tx, Color::Green, "ANSI/CP437 terminal detected. 80x24 mode.");
    sleep(Duration::from_millis(400)).await;

    send_ceremony_line(tx, "").await;

    // System handshake with blinking dots animation (~15 seconds)
    {
        let green = "\x1B[32m";
        let reset = "\x1B[0m";
        let base = format!("{}Performing system handshake", green);

        // Send initial text without newline
        let _ = tx.send(base.clone()).await;

        // Animate dots: cycle through ., .., ... for ~15 seconds
        // 10 cycles × 3 states × 500ms = 15 seconds
        for _ in 0..10 {
            for num_dots in 1..=3u8 {
                sleep(Duration::from_millis(500)).await;
                let dots: String = ".".repeat(num_dots as usize);
                let pad = " ".repeat(3 - num_dots as usize);
                let _ = tx.send(format!("\r{}{}{}", base, dots, pad)).await;
            }
        }

        // Final: show OK and complete the line
        sleep(Duration::from_millis(500)).await;
        let _ = tx
            .send(format!(
                "\r{}Performing system handshake... OK{}\r\n",
                green, reset
            ))
            .await;
    }

    send_ceremony_line(tx, "").await;

    // Connection established (before loading)
    log_line!(tx, Color::LightCyan, bold, "Connection established.");
    sleep(Duration::from_millis(400)).await;

    // Verify before loading
    log_line!(tx, Color::Green, "Verifying connection integrity... OK");
    sleep(Duration::from_millis(400)).await;

    send_ceremony_line(tx, "").await;

    // Loading
    log_line!(tx, Color::LightCyan, bold, "Loading The Construct BBS v0.1.0...");
    sleep(Duration::from_millis(600)).await;

    log_line!(tx, Color::Green, "Allocating session resources...");
    sleep(Duration::from_millis(400)).await;

    log_line!(tx, Color::Green, "Synchronizing system clock... EST");
    sleep(Duration::from_millis(400)).await;

    send_ceremony_line(tx, "").await;

    // Node assignment (last entry)
    {
        let mut w = AnsiWriter::new();
        w.set_fg(Color::Yellow);
        w.write_str(&format!("Connected to Node {} of {}", node_id, max_nodes));
        w.reset_color();
        send_ceremony_line(tx, &w.flush()).await;
    }
    sleep(Duration::from_millis(500)).await;

    send_ceremony_line(tx, "").await;

    Ok(node_id)
}

/// Send the "ALL LINES BUSY" rejection message with CP437 box-drawing art.
/// Waits 3 seconds after displaying so the user can read the message.
pub async fn send_line_busy(tx: &mpsc::Sender<String>, _max_nodes: usize) {
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
    // Clear the screen so ceremony text doesn't bleed into splash/login
    let _ = tx.send("\x1B[2J\x1B[H".to_string()).await;

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
/// Uses a plain-text figlet-style logo (no box border) for a compact layout.
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

    // ASCII art logo -- centered "THE CONSTRUCT" figlet
    let art_lines: &[&str] = &[
        r"         _________                         __                        __   ",
        r"     THE \_   ___ \  ____   ____   _______/  |________ __ __   _____/  |_ ",
        r"         /    \  \/ /  _ \ /    \ /  ___/\   __\_  __ \  |  \_/ ___\   __\",
        r"         \     \___(  <_> )   |  \\___ \  |  |  |  | \/  |  /\  \___|  |  ",
        r"          \______  /\____/|___|  /____  > |__|  |__|  |____/  \___  >__|  ",
        r"                 \/            \/     \/                          \/       ",
    ];

    for art in art_lines {
        lines.push(line!(|w: &mut AnsiWriter| {
            w.set_fg(Color::LightGreen);
            w.bold();
            w.write_str(art);
        }));
    }

    // Blank line
    lines.push(String::new());

    // Tagline
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::Yellow);
        w.write_str("                      A haven for travelers, near and far");
    }));

    // Blank line
    lines.push(String::new());

    // System info
    lines.push(line!(|w: &mut AnsiWriter| {
        w.set_fg(Color::Green);
        w.write_str("         Running on Wildyahoos | v0.1.0 | ANSI/CP437 | 16-Color CGA");
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
        assert!(all_text.contains("Performing system handshake"));
        assert!(all_text.contains("Connection established"));
        assert!(all_text.contains("Loading The Construct BBS"));
        assert!(all_text.contains("Connected to Node 1 of 4"));

        // Verify order: Connection established before Loading, Node count last
        let estab_pos = all_text.find("Connection established").unwrap();
        let loading_pos = all_text.find("Loading The Construct BBS").unwrap();
        let node_pos = all_text.find("Connected to Node").unwrap();
        assert!(estab_pos < loading_pos, "Connection established should come before Loading");
        assert!(loading_pos < node_pos, "Node count should be last");
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
