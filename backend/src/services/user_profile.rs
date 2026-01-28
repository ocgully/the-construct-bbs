use crate::terminal::{AnsiWriter, Color};

/// Render the user lookup prompt screen.
pub fn render_lookup_prompt() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  USER LOOKUP");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  Enter handle to look up (or Q to cancel): ");
    w.reset_color();
    w.flush()
}

/// Render a "user not found" error message.
pub fn render_user_not_found(handle: &str) -> String {
    let mut w = AnsiWriter::new();
    w.writeln("");
    w.set_fg(Color::LightRed);
    w.writeln(&format!("  User '{}' not found.", handle));
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to try again...");
    w.reset_color();
    w.flush()
}

/// Render the "press any key" footer after viewing a profile.
pub fn render_profile_footer() -> String {
    let mut w = AnsiWriter::new();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
    w.flush()
}
