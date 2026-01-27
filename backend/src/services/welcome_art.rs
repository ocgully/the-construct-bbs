use crate::terminal::{AnsiWriter, Color};

/// Render the ANSI art welcome screen with CP437 box-drawing and CGA colors
///
/// This welcome screen is the acid test for Phase 1: if box-drawing renders
/// correctly with all 16 CGA colors, the terminal foundation works.
pub fn render_welcome(services: &[(String, String)]) -> String {
    let mut w = AnsiWriter::new();
    
    w.begin_sync();
    w.clear_screen();
    w.hide_cursor();
    
    // Top border using CP437 double-line box-drawing
    w.set_fg(Color::LightCyan);
    w.bold();
    w.write_cp437(&[0xC9]); // ╔ top-left double
    for _ in 0..78 {
        w.write_cp437(&[0xCD]); // ═ horizontal double
    }
    w.write_cp437(&[0xBB]); // ╗ top-right double
    w.writeln("");
    
    // Title: THE CONSTRUCT (centered)
    w.write_cp437(&[0xBA]); // ║ vertical double
    w.set_fg(Color::White);
    w.write_str("                          THE CONSTRUCT BBS                            ");
    w.set_fg(Color::LightCyan);
    w.write_cp437(&[0xBA]); // ║
    w.writeln("");
    
    // Bottom border of title box
    w.write_cp437(&[0xC8]); // ╚ bottom-left double
    for _ in 0..78 {
        w.write_cp437(&[0xCD]); // ═ horizontal double
    }
    w.write_cp437(&[0xBC]); // ╝ bottom-right double
    w.writeln("");
    w.reset_color();
    
    w.writeln("");
    
    // Subtitle
    w.set_fg(Color::LightCyan);
    w.write_str("                       Terminal Foundation v0.1");
    w.writeln("");
    w.reset_color();
    
    // System info
    w.set_fg(Color::LightGray);
    w.write_str("                  80x24 Terminal | CP437 Encoding | CGA Palette");
    w.writeln("");
    w.writeln("");
    
    // Decorative divider using CP437 single-line box-drawing
    w.set_fg(Color::Brown);
    w.write_cp437(&[0xC3]); // ├ left tee
    for _ in 0..78 {
        w.write_cp437(&[0xC4]); // ─ horizontal single
    }
    w.write_cp437(&[0xB4]); // ┤ right tee
    w.writeln("");
    w.reset_color();
    
    w.writeln("");
    
    // Color test: display all 16 CGA colors
    w.set_fg(Color::Yellow);
    w.write_str("CGA Color Palette Test:");
    w.writeln("");
    w.reset_color();
    
    // First row: colors 0-7
    let colors = [
        (Color::Black, "Blk"),
        (Color::Red, "Red"),
        (Color::Green, "Grn"),
        (Color::Brown, "Brn"),
        (Color::Blue, "Blu"),
        (Color::Magenta, "Mag"),
        (Color::Cyan, "Cyn"),
        (Color::LightGray, "LGr"),
    ];
    
    w.write_str("  ");
    for (color, label) in &colors {
        w.set_bg(*color);
        w.set_fg(Color::White);
        w.write_str(label);
        w.reset_color();
        w.write_str(" ");
    }
    w.writeln("");
    
    // Second row: colors 8-15
    let bright_colors = [
        (Color::DarkGray, "DGr"),
        (Color::LightRed, "LRd"),
        (Color::LightGreen, "LGn"),
        (Color::Yellow, "Yel"),
        (Color::LightBlue, "LBl"),
        (Color::LightMagenta, "LMg"),
        (Color::LightCyan, "LCy"),
        (Color::White, "Wht"),
    ];
    
    w.write_str("  ");
    for (color, label) in &bright_colors {
        w.set_bg(*color);
        w.set_fg(Color::Black);
        w.write_str(label);
        w.reset_color();
        w.write_str(" ");
    }
    w.writeln("");
    
    w.writeln("");
    
    // Box-drawing test: small box using single-line CP437 characters
    w.set_fg(Color::LightGreen);
    w.write_str("Box-Drawing Test: ");
    w.reset_color();
    
    w.write_cp437(&[0xDA]); // ┌ top-left single
    for _ in 0..10 {
        w.write_cp437(&[0xC4]); // ─ horizontal single
    }
    w.write_cp437(&[0xBF]); // ┐ top-right single
    w.writeln("");
    
    for _ in 0..2 {
        w.write_str("                  ");
        w.write_cp437(&[0xB3]); // │ vertical single
        w.write_str("          ");
        w.write_cp437(&[0xB3]); // │
        w.writeln("");
    }
    
    w.write_str("                  ");
    w.write_cp437(&[0xC0]); // └ bottom-left single
    for _ in 0..10 {
        w.write_cp437(&[0xC4]); // ─ horizontal single
    }
    w.write_cp437(&[0xD9]); // ┘ bottom-right single
    w.writeln("");
    
    w.writeln("");
    
    // Decorative divider
    w.set_fg(Color::Brown);
    w.write_cp437(&[0xC3]); // ├
    for _ in 0..78 {
        w.write_cp437(&[0xC4]); // ─
    }
    w.write_cp437(&[0xB4]); // ┤
    w.writeln("");
    w.reset_color();
    
    w.writeln("");
    
    // Service menu
    if services.is_empty() {
        w.set_fg(Color::Yellow);
        w.writeln("No services available.");
    } else {
        w.set_fg(Color::Yellow);
        w.writeln("Available Services:");
        w.writeln("");
        
        for (idx, (name, description)) in services.iter().enumerate() {
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("  [{}] ", idx + 1));
            w.set_fg(Color::White);
            w.write_str(name);
            w.set_fg(Color::LightGray);
            w.writeln(&format!(" - {}", description));
        }
        
        w.writeln("");
    }
    
    // Prompt
    w.set_fg(Color::LightCyan);
    w.write_str("Enter service number or (q)uit to disconnect: ");
    w.reset_color();
    
    w.show_cursor();
    w.end_sync();
    
    w.flush()
}

/// Render the main menu with user info, services, profile, and quit options.
///
/// Shows the user's handle, level, and node info in the header, followed by
/// numbered services, a Profile option [P], and a Quit option [Q].
pub fn render_main_menu_with_user(
    services: &[(String, String)],
    handle: &str,
    user_level: &str,
    node_id: Option<usize>,
    max_nodes: usize,
) -> String {
    let mut w = AnsiWriter::new();

    w.begin_sync();
    w.clear_screen();

    // Build right-side node info
    let node_info = match node_id {
        Some(id) => format!("Node {} of {}", id, max_nodes),
        None => format!("{} nodes", max_nodes),
    };

    // Build left-side user info
    let user_info = format!("Logged in as: {} [{}]", handle, user_level);

    // Top border
    w.set_fg(Color::LightCyan);
    w.bold();
    w.write_cp437(&[0xC9]); // ╔
    for _ in 0..78 {
        w.write_cp437(&[0xCD]); // ═
    }
    w.write_cp437(&[0xBB]); // ╗
    w.writeln("");

    // Title row: "  THE CONSTRUCT BBS" on left, node info on right
    w.write_cp437(&[0xBA]); // ║
    w.set_fg(Color::White);
    w.bold();
    let title = "  THE CONSTRUCT BBS";
    w.write_str(title);
    // Pad between title and node info: 78 - title.len() - node_info.len() - 2 (trailing spaces)
    let title_len = title.len();
    let node_len = node_info.len();
    let spacing = if 78 > title_len + node_len + 2 {
        78 - title_len - node_len - 2
    } else {
        1
    };
    w.reset_color();
    w.write_str(&" ".repeat(spacing));
    w.set_fg(Color::Yellow);
    w.write_str(&node_info);
    w.write_str("  ");
    w.set_fg(Color::LightCyan);
    w.bold();
    w.write_cp437(&[0xBA]); // ║
    w.writeln("");

    // User info row: "  Logged in as: Handle [Level]"
    w.write_cp437(&[0xBA]); // ║
    w.set_fg(Color::LightGreen);
    w.write_str("  ");
    w.write_str(&user_info);
    let user_pad = if 78 > user_info.len() + 2 {
        78 - user_info.len() - 2
    } else {
        0
    };
    w.reset_color();
    w.write_str(&" ".repeat(user_pad));
    w.set_fg(Color::LightCyan);
    w.bold();
    w.write_cp437(&[0xBA]); // ║
    w.writeln("");

    // Separator
    w.write_cp437(&[0xCC]); // ╠
    for _ in 0..78 {
        w.write_cp437(&[0xCD]); // ═
    }
    w.write_cp437(&[0xB9]); // ╣
    w.writeln("");

    // Empty row inside box
    w.write_cp437(&[0xBA]); // ║
    w.write_str(&" ".repeat(78));
    w.write_cp437(&[0xBA]); // ║
    w.writeln("");

    // Service list inside the box
    if !services.is_empty() {
        for (idx, (name, description)) in services.iter().enumerate() {
            w.write_cp437(&[0xBA]); // ║
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("  [{}] ", idx + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:20}", name));
            w.set_fg(Color::LightGray);
            w.write_str(description);
            w.reset_color();
            let printed = 6 + 20 + description.len();
            let lpad = if printed < 78 { 78 - printed } else { 0 };
            w.write_str(&" ".repeat(lpad));
            w.set_fg(Color::LightCyan);
            w.bold();
            w.write_cp437(&[0xBA]); // ║
            w.writeln("");
        }
    }

    // Profile option
    {
        w.write_cp437(&[0xBA]); // ║
        w.set_fg(Color::LightGreen);
        w.write_str("  [P] ");
        w.set_fg(Color::White);
        w.write_str("Your Profile       ");
        w.set_fg(Color::LightGray);
        w.write_str("View your user profile");
        w.reset_color();
        let printed = 6 + 19 + 22;
        let rpad = if printed < 78 { 78 - printed } else { 0 };
        w.write_str(&" ".repeat(rpad));
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.writeln("");
    }

    // Quit option
    {
        w.write_cp437(&[0xBA]); // ║
        w.set_fg(Color::LightGreen);
        w.write_str("  [Q] ");
        w.set_fg(Color::White);
        w.write_str("Quit               ");
        w.set_fg(Color::LightGray);
        w.write_str("Log off the BBS");
        w.reset_color();
        let printed = 6 + 19 + 15;
        let rpad = if printed < 78 { 78 - printed } else { 0 };
        w.write_str(&" ".repeat(rpad));
        w.set_fg(Color::LightCyan);
        w.bold();
        w.write_cp437(&[0xBA]); // ║
        w.writeln("");
    }

    // Empty row
    w.write_cp437(&[0xBA]); // ║
    w.write_str(&" ".repeat(78));
    w.write_cp437(&[0xBA]); // ║
    w.writeln("");

    // Bottom border
    w.write_cp437(&[0xC8]); // ╚
    for _ in 0..78 {
        w.write_cp437(&[0xCD]); // ═
    }
    w.write_cp437(&[0xBC]); // ╝
    w.writeln("");
    w.reset_color();

    w.writeln("");

    // Prompt
    w.set_fg(Color::LightCyan);
    w.write_str("Enter selection: ");
    w.reset_color();

    w.end_sync();

    w.flush()
}

/// Render the main menu (legacy version without user info, kept for compatibility)
pub fn render_main_menu(services: &[(String, String)]) -> String {
    render_main_menu_with_user(services, "Guest", "User", None, 16)
}
