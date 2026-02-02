use crate::menu::config::{MenuConfig, MenuItem};
use crate::menu::state::MenuState;
use crate::terminal::ansi::{AnsiWriter, Color};

/// Calculate the visible width of a string, excluding ANSI escape codes.
/// ANSI escape sequences follow the pattern ESC [ ... m (for SGR codes).
fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if in_escape {
            // End of escape sequence when we hit 'm' (SGR terminator)
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1B' {
            // Start of escape sequence
            in_escape = true;
        } else {
            width += 1;
        }
    }

    width
}

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

/// Groups items by category and returns them in order
fn group_by_category<'a>(items: &[&'a MenuItem]) -> Vec<(String, Vec<&'a MenuItem>)> {
    use std::collections::BTreeMap;

    // Define category order (categories not in this list go last)
    let category_order: &[&str] = &[
        "Casual/Daily",
        "Strategy/Trade",
        "RPG/Adventure",
        "Action",
        "Sandbox/Epic",
    ];

    let mut grouped: BTreeMap<String, Vec<&'a MenuItem>> = BTreeMap::new();

    for item in items {
        let cat = item.category().unwrap_or("Other").to_string();
        grouped.entry(cat).or_default().push(*item);
    }

    // Sort categories by defined order
    let mut result: Vec<(String, Vec<&MenuItem>)> = grouped.into_iter().collect();
    result.sort_by_key(|(cat, _)| {
        category_order
            .iter()
            .position(|&c| c == cat)
            .unwrap_or(usize::MAX)
    });

    result
}

/// Render a category header (padding handled by caller in multi-column mode)
fn render_category_header(w: &mut AnsiWriter, category: &str, _col_width: usize) {
    w.set_fg(Color::Yellow);
    w.bold();
    w.write_str(category);
    w.reset_color();
}

/// Render a menu item with hotkey (padding handled by caller in multi-column mode)
fn render_menu_item(w: &mut AnsiWriter, item: &MenuItem, _col_width: usize) {
    w.write_str("  ");
    w.set_fg(Color::LightGreen);
    w.write_str(&format!("[{}]", item.hotkey()));
    w.set_fg(Color::White);
    w.write_str(&format!(" {}", item.name()));
    w.reset_color();
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

    // Check if this is the games menu with categories
    let has_categories = submenu_key == "games" && items.iter().any(|i| i.category().is_some());

    if has_categories && items.len() > 10 {
        // Multi-column categorized layout for games
        let categories = group_by_category(items);
        let num_categories = categories.len();

        // Use 3 columns for 3+ categories, otherwise single column
        let num_cols = if num_categories >= 3 { 3 } else { 1 };
        // 3 columns: 24 + 3 gap + 24 + 3 gap + 24 = 78 chars
        let col_width = if num_cols == 3 { 24 } else { 76 };
        let col_gap = "   "; // 3 space gap between columns

        if num_cols == 3 {
            // Split categories into three columns
            let cats_per_col = (num_categories + 2) / 3;
            let col1_end = cats_per_col.min(num_categories);
            let col2_end = (cats_per_col * 2).min(num_categories);

            let col1_cats = &categories[..col1_end];
            let col2_cats = &categories[col1_end..col2_end];
            let col3_cats = &categories[col2_end..];

            // Build column content helper
            fn build_column_lines(
                cats: &[(String, Vec<&MenuItem>)],
                col_width: usize,
            ) -> Vec<String> {
                let mut lines = Vec::new();
                for (cat, cat_items) in cats {
                    let mut header = AnsiWriter::new();
                    render_category_header(&mut header, cat, col_width);
                    lines.push(header.flush());
                    for item in cat_items {
                        let mut item_w = AnsiWriter::new();
                        render_menu_item(&mut item_w, item, col_width);
                        lines.push(item_w.flush());
                    }
                }
                lines
            }

            let col1_lines = build_column_lines(col1_cats, col_width);
            let col2_lines = build_column_lines(col2_cats, col_width);
            let col3_lines = build_column_lines(col3_cats, col_width);

            let max_rows = col1_lines.len().max(col2_lines.len()).max(col3_lines.len());

            // Render all three columns side by side with gaps
            for i in 0..max_rows {
                let c1 = col1_lines.get(i).map(|s| s.as_str()).unwrap_or("");
                let c2 = col2_lines.get(i).map(|s| s.as_str()).unwrap_or("");
                let c3 = col3_lines.get(i).map(|s| s.as_str()).unwrap_or("");

                // Write column 1 and pad to col_width based on visible characters
                w.write_str(c1);
                let c1_visible = visible_width(c1);
                w.write_str(&" ".repeat(col_width.saturating_sub(c1_visible)));

                w.write_str(col_gap);

                // Write column 2 and pad to col_width based on visible characters
                w.write_str(c2);
                let c2_visible = visible_width(c2);
                w.write_str(&" ".repeat(col_width.saturating_sub(c2_visible)));

                w.write_str(col_gap);

                // Write column 3 (no padding needed for last column)
                w.write_str(c3);
                w.writeln("");
            }
        } else {
            // Single column for fewer categories
            for (cat, cat_items) in &categories {
                render_category_header(&mut w, cat, col_width);
                w.writeln("");
                for item in cat_items {
                    render_menu_item(&mut w, item, col_width);
                    w.writeln("");
                }
                w.writeln("");
            }
        }
    } else {
        // Standard single column layout for small menus
        for item in items {
            w.write_str("  ");
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("[{}]", item.hotkey()));
            w.set_fg(Color::White);
            w.write_str(&format!(" {}", item.name()));
            w.writeln("");
            w.reset_color();
        }
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
    fn test_visible_width_plain_text() {
        assert_eq!(visible_width("hello"), 5);
        assert_eq!(visible_width(""), 0);
        assert_eq!(visible_width("  [A] Game Name"), 15);
    }

    #[test]
    fn test_visible_width_with_ansi_codes() {
        // ANSI color code followed by text followed by reset
        assert_eq!(visible_width("\x1B[33mHello\x1B[0m"), 5);
        // Multiple ANSI codes
        assert_eq!(visible_width("\x1B[1m\x1B[33mBold Yellow\x1B[0m"), 11);
        // Just ANSI codes, no text
        assert_eq!(visible_width("\x1B[33m\x1B[0m"), 0);
    }

    #[test]
    fn test_visible_width_with_ansi_writer() {
        let mut w = AnsiWriter::new();
        w.set_fg(Color::Yellow);
        w.bold();
        w.write_str("Action");
        w.reset_color();
        let output = w.flush();
        // "Action" is 6 visible chars, rest are ANSI codes
        assert_eq!(visible_width(&output), 6);
    }

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
                category: None,
            },
            MenuItem::Command {
                hotkey: "Q".to_string(),
                name: "Quit".to_string(),
                command: "quit".to_string(),
                min_level: 0,
                order: 2,
                category: None,
            },
        ];

        let output = render_main_menu(&config, 0, "TestUser", "User", Some(1), 16);
        assert!(output.contains("[P]"));
        assert!(output.contains("Profile"));
        assert!(output.contains("[Q]"));
        assert!(output.contains("Quit"));
    }

    #[test]
    fn test_render_submenu_with_categories() {
        let mut config = MenuConfig::default();
        config.games = vec![
            MenuItem::Command {
                hotkey: "1".to_string(),
                name: "Sudoku".to_string(),
                command: "sudoku".to_string(),
                min_level: 0,
                order: 1,
                category: Some("Casual/Daily".to_string()),
            },
            MenuItem::Command {
                hotkey: "2".to_string(),
                name: "Chess".to_string(),
                command: "chess".to_string(),
                min_level: 0,
                order: 2,
                category: Some("Casual/Daily".to_string()),
            },
            MenuItem::Command {
                hotkey: "3".to_string(),
                name: "Star Trader".to_string(),
                command: "star_trader".to_string(),
                min_level: 0,
                order: 3,
                category: Some("Strategy/Trade".to_string()),
            },
            MenuItem::Command {
                hotkey: "4".to_string(),
                name: "Dystopia".to_string(),
                command: "dystopia".to_string(),
                min_level: 0,
                order: 4,
                category: Some("Strategy/Trade".to_string()),
            },
            MenuItem::Command {
                hotkey: "A".to_string(),
                name: "Dragon Slayer".to_string(),
                command: "dragon_slayer".to_string(),
                min_level: 0,
                order: 10,
                category: Some("RPG/Adventure".to_string()),
            },
            MenuItem::Command {
                hotkey: "B".to_string(),
                name: "Usurper".to_string(),
                command: "usurper".to_string(),
                min_level: 0,
                order: 11,
                category: Some("RPG/Adventure".to_string()),
            },
            MenuItem::Command {
                hotkey: "C".to_string(),
                name: "Last Dream".to_string(),
                command: "last_dream".to_string(),
                min_level: 0,
                order: 12,
                category: Some("RPG/Adventure".to_string()),
            },
            MenuItem::Command {
                hotkey: "G".to_string(),
                name: "Tanks".to_string(),
                command: "tanks".to_string(),
                min_level: 0,
                order: 20,
                category: Some("Action".to_string()),
            },
            MenuItem::Command {
                hotkey: "H".to_string(),
                name: "Summit".to_string(),
                command: "summit".to_string(),
                min_level: 0,
                order: 21,
                category: Some("Action".to_string()),
            },
            MenuItem::Command {
                hotkey: "J".to_string(),
                name: "Mineteria".to_string(),
                command: "mineteria".to_string(),
                min_level: 0,
                order: 30,
                category: Some("Sandbox/Epic".to_string()),
            },
            MenuItem::Command {
                hotkey: "K".to_string(),
                name: "Fortress".to_string(),
                command: "fortress".to_string(),
                min_level: 0,
                order: 31,
                category: Some("Sandbox/Epic".to_string()),
            },
        ];

        let items_vec = config.submenu_items("games", 0);
        let items: Vec<&MenuItem> = items_vec.iter().copied().collect();
        let output = render_submenu("games", "Games", &items, 0);

        // Should contain category headers
        assert!(output.contains("Casual/Daily"));
        assert!(output.contains("Strategy/Trade"));
        assert!(output.contains("RPG/Adventure"));
        assert!(output.contains("Action"));
        assert!(output.contains("Sandbox/Epic"));

        // Should contain all game names and hotkeys
        assert!(output.contains("[1]"));
        assert!(output.contains("Sudoku"));
        assert!(output.contains("[A]"));
        assert!(output.contains("Dragon Slayer"));
        assert!(output.contains("[J]"));
        assert!(output.contains("Mineteria"));

        // Should contain back option
        assert!(output.contains("[Q]"));
        assert!(output.contains("Back to Main Menu"));
    }

    #[test]
    fn test_render_submenu_without_categories_uses_single_column() {
        let mut config = MenuConfig::default();
        config.games = vec![
            MenuItem::Command {
                hotkey: "1".to_string(),
                name: "Game One".to_string(),
                command: "game_one".to_string(),
                min_level: 0,
                order: 1,
                category: None,
            },
            MenuItem::Command {
                hotkey: "2".to_string(),
                name: "Game Two".to_string(),
                command: "game_two".to_string(),
                min_level: 0,
                order: 2,
                category: None,
            },
        ];

        let items_vec = config.submenu_items("games", 0);
        let items: Vec<&MenuItem> = items_vec.iter().copied().collect();
        let output = render_submenu("games", "Games", &items, 0);

        // Should contain game names (single column layout)
        assert!(output.contains("[1]"));
        assert!(output.contains("Game One"));
        assert!(output.contains("[2]"));
        assert!(output.contains("Game Two"));
        assert!(output.contains("[Q]"));
        assert!(output.contains("Back to Main Menu"));
    }

    #[test]
    fn test_group_by_category() {
        let items = vec![
            MenuItem::Command {
                hotkey: "1".to_string(),
                name: "Game A".to_string(),
                command: "a".to_string(),
                min_level: 0,
                order: 1,
                category: Some("Casual/Daily".to_string()),
            },
            MenuItem::Command {
                hotkey: "2".to_string(),
                name: "Game B".to_string(),
                command: "b".to_string(),
                min_level: 0,
                order: 2,
                category: Some("Strategy/Trade".to_string()),
            },
            MenuItem::Command {
                hotkey: "3".to_string(),
                name: "Game C".to_string(),
                command: "c".to_string(),
                min_level: 0,
                order: 3,
                category: Some("Casual/Daily".to_string()),
            },
        ];

        let refs: Vec<&MenuItem> = items.iter().collect();
        let grouped = group_by_category(&refs);

        // Categories should be ordered: Casual/Daily first, then Strategy/Trade
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].0, "Casual/Daily");
        assert_eq!(grouped[0].1.len(), 2);
        assert_eq!(grouped[1].0, "Strategy/Trade");
        assert_eq!(grouped[1].1.len(), 1);
    }
}
