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

/// Render the main menu (used when returning from a service)
pub fn render_main_menu(services: &[(String, String)]) -> String {
    let mut w = AnsiWriter::new();
    
    w.begin_sync();
    w.clear_screen();
    
    // Simple title
    w.set_fg(Color::LightCyan);
    w.bold();
    w.write_cp437(&[0xC9]); // ╔
    for _ in 0..78 {
        w.write_cp437(&[0xCD]); // ═
    }
    w.write_cp437(&[0xBB]); // ╗
    w.writeln("");
    
    w.write_cp437(&[0xBA]); // ║
    w.set_fg(Color::White);
    w.write_str("                          THE CONSTRUCT BBS                            ");
    w.set_fg(Color::LightCyan);
    w.write_cp437(&[0xBA]); // ║
    w.writeln("");
    
    w.write_cp437(&[0xC8]); // ╚
    for _ in 0..78 {
        w.write_cp437(&[0xCD]); // ═
    }
    w.write_cp437(&[0xBC]); // ╝
    w.writeln("");
    w.reset_color();
    
    w.writeln("");
    
    // Service list
    if !services.is_empty() {
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
    
    w.end_sync();
    
    w.flush()
}
