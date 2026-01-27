use crate::menu::config::{MenuConfig, MenuItem};
use crate::menu::quotes::random_stoic_quote;
use crate::menu::state::MenuState;
use crate::terminal::ansi::{AnsiWriter, Color};

#[derive(Debug, Clone, Copy)]
pub enum BorderStyle {
    Double,  // Main menu -- CP437: C9/CD/BB/C8/BC/BA (double-line)
    Single,  // Submenus -- CP437: DA/C4/BF/C0/D9/B3 (single-line)
}

/// Render a horizontal border line with specified style
fn render_border_line(w: &mut AnsiWriter, style: BorderStyle, position: &str) {
    w.set_fg(Color::LightCyan);

    match (style, position) {
        (BorderStyle::Double, "top") => {
            w.write_cp437(&[0xC9]); // ╔
            for _ in 0..78 {
                w.write_cp437(&[0xCD]); // ═
            }
            w.write_cp437(&[0xBB]); // ╗
        }
        (BorderStyle::Double, "bottom") => {
            w.write_cp437(&[0xC8]); // ╚
            for _ in 0..78 {
                w.write_cp437(&[0xCD]); // ═
            }
            w.write_cp437(&[0xBC]); // ╝
        }
        (BorderStyle::Single, "top") => {
            w.write_cp437(&[0xDA]); // ┌
            for _ in 0..78 {
                w.write_cp437(&[0xC4]); // ─
            }
            w.write_cp437(&[0xBF]); // ┐
        }
        (BorderStyle::Single, "bottom") => {
            w.write_cp437(&[0xC0]); // └
            for _ in 0..78 {
                w.write_cp437(&[0xC4]); // ─
            }
            w.write_cp437(&[0xD9]); // ┘
        }
        _ => {}
    }
    w.writeln("");
}

/// Render a title line with border characters
fn render_title_line(w: &mut AnsiWriter, style: BorderStyle, title: &str) {
    let border_char = match style {
        BorderStyle::Double => 0xBA, // ║
        BorderStyle::Single => 0xB3, // │
    };

    w.set_fg(Color::LightCyan);
    w.write_cp437(&[border_char]);

    let padding = (78_usize.saturating_sub(title.len())) / 2;
    w.set_fg(Color::White);
    w.bold();
    w.write_str(&" ".repeat(padding));
    w.write_str(title);
    w.write_str(&" ".repeat(78 - padding - title.len()));
    w.reset_color();

    w.set_fg(Color::LightCyan);
    w.write_cp437(&[border_char]);
    w.writeln("");
}

pub fn render_main_menu(
    config: &MenuConfig,
    user_level: u8,
    handle: &str,
    user_level_name: &str,
    node_id: Option<usize>,
    max_nodes: usize,
) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.begin_sync();

    // Double-line box header
    render_border_line(&mut w, BorderStyle::Double, "top");
    render_title_line(&mut w, BorderStyle::Double, "THE CONSTRUCT BBS");
    render_border_line(&mut w, BorderStyle::Double, "bottom");

    w.writeln("");

    // MOTD area with random Stoic quote
    let quote = random_stoic_quote();
    let quote_padding = (80_usize.saturating_sub(quote.len())) / 2;
    w.set_fg(Color::DarkGray);
    w.write_str(&" ".repeat(quote_padding));
    w.set_fg(Color::LightGray);
    w.write_str(quote);
    w.writeln("");
    w.reset_color();

    w.writeln("");

    // Single-line divider
    w.set_fg(Color::Brown);
    w.write_cp437(&[0xC3]); // ├
    for _ in 0..78 {
        w.write_cp437(&[0xC4]); // ─
    }
    w.write_cp437(&[0xB4]); // ┤
    w.writeln("");
    w.reset_color();

    w.writeln("");

    // Menu items - adaptive layout
    let items = config.main_items(user_level);

    if items.len() <= 7 {
        // Single column layout
        for item in items {
            w.write_str("  ");
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("[{}]", item.hotkey()));
            w.set_fg(Color::White);
            w.write_str(&format!(" {}", item.name()));
            w.writeln("");
            w.reset_color();
        }
    } else {
        // Two column layout
        let mid = (items.len() + 1) / 2;
        let left_items = &items[..mid];
        let right_items = &items[mid..];

        for i in 0..mid {
            // Left column
            w.write_str("  ");
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("[{}]", left_items[i].hotkey()));
            w.set_fg(Color::White);
            let left_name = format!(" {}", left_items[i].name());
            w.write_str(&left_name);

            // Right column (if exists)
            if i < right_items.len() {
                let padding = 40_usize.saturating_sub(2 + 3 + left_name.len());
                w.write_str(&" ".repeat(padding));
                w.set_fg(Color::LightGreen);
                w.write_str(&format!("[{}]", right_items[i].hotkey()));
                w.set_fg(Color::White);
                w.write_str(&format!(" {}", right_items[i].name()));
            }

            w.writeln("");
            w.reset_color();
        }
    }

    w.writeln("");

    // User info line
    w.set_fg(Color::LightGreen);
    w.write_str(&format!("Logged in as: {} [{}]", handle, user_level_name));

    // Right-aligned node info
    let node_info = if let Some(node) = node_id {
        format!("Node {} of {}", node, max_nodes)
    } else {
        format!("Node ? of {}", max_nodes)
    };
    let left_text_len = format!("Logged in as: {} [{}]", handle, user_level_name).len();
    let padding = 80_usize.saturating_sub(left_text_len + node_info.len());
    w.write_str(&" ".repeat(padding));
    w.set_fg(Color::Yellow);
    w.write_str(&node_info);
    w.writeln("");
    w.reset_color();

    // Prompt
    w.set_fg(Color::LightCyan);
    w.write_str("Your choice? ");
    w.reset_color();

    w.show_cursor();
    w.end_sync();

    w.flush()
}

pub fn render_submenu(
    submenu_key: &str,
    submenu_name: &str,
    items: &[&MenuItem],
    _user_level: u8,
) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.begin_sync();

    // Single-line box header
    render_border_line(&mut w, BorderStyle::Single, "top");
    render_title_line(&mut w, BorderStyle::Single, submenu_name);
    render_border_line(&mut w, BorderStyle::Single, "bottom");

    w.writeln("");

    // Menu items (single column)
    for item in items {
        w.write_str("  ");
        w.set_fg(Color::LightGreen);
        w.write_str(&format!("[{}]", item.hotkey()));
        w.set_fg(Color::White);
        w.write_str(&format!(" {}", item.name()));
        w.writeln("");
        w.reset_color();
    }

    w.writeln("");

    // Back option
    w.write_str("  ");
    w.set_fg(Color::LightGreen);
    w.write_str("[Q]");
    w.set_fg(Color::White);
    w.write_str(" Back to Main Menu");
    w.writeln("");
    w.reset_color();

    w.writeln("");

    // Prompt
    w.set_fg(Color::LightCyan);
    w.write_str("Your choice? ");
    w.reset_color();

    w.show_cursor();
    w.end_sync();

    w.flush()
}

pub fn render_help(state: &MenuState, config: &MenuConfig, user_level: u8) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.begin_sync();

    // Single-line box header
    render_border_line(&mut w, BorderStyle::Single, "top");
    render_title_line(&mut w, BorderStyle::Single, "HELP");
    render_border_line(&mut w, BorderStyle::Single, "bottom");

    w.writeln("");

    match state {
        MenuState::MainMenu => {
            let items = config.main_items(user_level);

            w.set_fg(Color::LightCyan);
            w.writeln("Available Commands:");
            w.writeln("");
            w.reset_color();

            for item in items {
                w.write_str("  ");
                w.set_fg(Color::LightGreen);
                w.write_str(&format!("[{}]", item.hotkey()));
                w.set_fg(Color::White);
                w.write_str(&format!(" {} - ", item.name()));
                w.set_fg(Color::LightGray);
                match item {
                    MenuItem::Submenu { .. } => w.write_str("Enter submenu"),
                    MenuItem::Service { .. } => w.write_str("Launch service"),
                    MenuItem::Command { .. } => w.write_str("Execute command"),
                }
                w.writeln("");
                w.reset_color();
            }

            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.writeln("Command Stacking:");
            w.set_fg(Color::LightGray);
            w.writeln("  Type multiple keys quickly to navigate directly to a service.");
            w.writeln("  Example: G1 enters Games menu and launches first game.");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.writeln("Special Keys:");
            w.set_fg(Color::LightGray);
            w.writeln("  [?] - Show this help screen");
            w.writeln("  [Enter] - Redraw current menu");
            w.reset_color();
        }
        MenuState::Submenu { submenu_key } => {
            let items = config.submenu_items(submenu_key, user_level);
            let submenu_name = config.submenu_name(submenu_key);

            w.set_fg(Color::LightCyan);
            w.writeln(&format!("{} Menu Commands:", submenu_name));
            w.writeln("");
            w.reset_color();

            for item in items {
                w.write_str("  ");
                w.set_fg(Color::LightGreen);
                w.write_str(&format!("[{}]", item.hotkey()));
                w.set_fg(Color::White);
                w.write_str(&format!(" {} - ", item.name()));
                w.set_fg(Color::LightGray);
                match item {
                    MenuItem::Service { .. } => w.write_str("Launch service"),
                    MenuItem::Command { .. } => w.write_str("Execute command"),
                    MenuItem::Submenu { .. } => w.write_str("Enter submenu"),
                }
                w.writeln("");
                w.reset_color();
            }

            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.writeln("Special Keys:");
            w.set_fg(Color::LightGray);
            w.writeln("  [Q] - Return to main menu");
            w.writeln("  [?] - Show this help screen");
            w.writeln("  [Enter] - Redraw current menu");
            w.reset_color();
        }
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.write_str("Press any key to return to menu...");
    w.reset_color();

    w.show_cursor();
    w.end_sync();

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_main_menu_contains_title() {
        let config = MenuConfig::default();
        let output = render_main_menu(&config, 0, "TestUser", "User", Some(1), 16);
        assert!(output.contains("THE CONSTRUCT BBS"));
        assert!(output.contains("Your choice?"));
        assert!(output.contains("TestUser"));
        assert!(output.contains("Node 1 of 16"));
    }

    #[test]
    fn test_render_main_menu_contains_motd_quote() {
        let config = MenuConfig::default();
        let output = render_main_menu(&config, 0, "TestUser", "User", Some(1), 16);
        // Should contain at least part of a Stoic quote (e.g., "--")
        assert!(output.contains("--"));
    }

    #[test]
    fn test_render_submenu_contains_title() {
        let config = MenuConfig::default();
        let items_vec = config.submenu_items("games", 0);
        let items: Vec<&MenuItem> = items_vec.iter().map(|&item| item).collect();
        let output = render_submenu("games", "Games", &items, 0);
        assert!(output.contains("Games"));
        assert!(output.contains("Back to Main Menu"));
        assert!(output.contains("Your choice?"));
    }

    #[test]
    fn test_render_help_main_menu() {
        let config = MenuConfig::default();
        let state = MenuState::MainMenu;
        let output = render_help(&state, &config, 0);
        assert!(output.contains("HELP"));
        assert!(output.contains("Available Commands:"));
        assert!(output.contains("Command Stacking:"));
        assert!(output.contains("Press any key to return to menu..."));
    }

    #[test]
    fn test_render_help_submenu() {
        let config = MenuConfig::default();
        let state = MenuState::Submenu {
            submenu_key: "games".to_string(),
        };
        let output = render_help(&state, &config, 0);
        assert!(output.contains("HELP"));
        assert!(output.contains("Menu Commands:"));
        assert!(output.contains("[Q] - Return to main menu"));
        assert!(output.contains("Press any key to return to menu..."));
    }

    #[test]
    fn test_render_main_menu_with_items() {
        let mut config = MenuConfig::default();
        config.main = vec![
            MenuItem::Command {
                hotkey: "P".to_string(),
                name: "Profile".to_string(),
                command: "profile".to_string(),
                min_level: 0,
                order: 1,
            },
            MenuItem::Command {
                hotkey: "Q".to_string(),
                name: "Quit".to_string(),
                command: "quit".to_string(),
                min_level: 0,
                order: 2,
            },
        ];

        let output = render_main_menu(&config, 0, "TestUser", "User", Some(1), 16);
        assert!(output.contains("[P] Profile"));
        assert!(output.contains("[Q] Quit"));
    }
}
