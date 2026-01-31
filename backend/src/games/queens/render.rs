//! ANSI rendering for Queens game
//!
//! Visual identity: Elegant royal theme with deep purples, golds, and whites

use crate::terminal::{AnsiWriter, Color};
use super::data::QUEEN_SYMBOL;
use super::puzzle::DailyPuzzle;
use super::state::{GameState, PlayerStats};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format time as MM:SS
pub fn format_time(seconds: u32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{}:{:02}", mins, secs)
}

/// Render the game header with title art
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("");
    w.writeln("   ███████╗ ██╗   ██╗ ███████╗ ███████╗ ███╗   ██╗ ███████╗");
    w.writeln("   ██╔═══██╗██║   ██║ ██╔════╝ ██╔════╝ ████╗  ██║ ██╔════╝");
    w.writeln("   ██║   ██║██║   ██║ █████╗   █████╗   ██╔██╗ ██║ ███████╗");
    w.writeln("   ██║▄▄ ██║██║   ██║ ██╔══╝   ██╔══╝   ██║╚██╗██║ ╚════██║");
    w.writeln("   ╚██████╔╝╚██████╔╝ ███████╗ ███████╗ ██║ ╚████║ ███████║");
    w.writeln("    ╚══▀▀═╝  ╚═════╝  ╚══════╝ ╚══════╝ ╚═╝  ╚═══╝ ╚══════╝");
    w.reset_color();
    w.set_fg(Color::Yellow);
    w.writeln("                     Daily Puzzle Challenge");
    w.reset_color();
}

/// Render status bar
fn render_status_bar(w: &mut AnsiWriter, state: &GameState, stats: &PlayerStats, puzzle: &DailyPuzzle) {
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));

    w.set_fg(Color::LightMagenta);
    w.write_str(&format!(" {} ", puzzle.date));
    w.set_fg(Color::DarkGray);
    w.write_str("\u{2502} ");

    w.set_fg(Color::Yellow);
    w.write_str(&format!("Streak: {}", stats.current_streak));
    w.set_fg(Color::DarkGray);
    w.write_str(" \u{2502} ");

    w.set_fg(Color::LightCyan);
    w.write_str(&format!("Time: {}", format_time(state.elapsed_seconds())));
    w.set_fg(Color::DarkGray);
    w.write_str(" \u{2502} ");

    w.set_fg(Color::White);
    w.write_str(&format!("Queens: {}/{}", state.placements.len(), puzzle.size));
    w.set_fg(Color::DarkGray);
    w.write_str(" \u{2502} ");

    if state.hints_used > 0 {
        w.set_fg(Color::LightGray);
        w.write_str(&format!("Hints: {}", state.hints_used));
    } else {
        w.set_fg(Color::LightGreen);
        w.write_str("No hints");
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));
    w.reset_color();
}

// ============================================================================
// PUBLIC RENDER FUNCTIONS
// ============================================================================

/// Render intro screen
pub fn render_intro(puzzle: &DailyPuzzle, stats: &PlayerStats) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Welcome to Queens, the daily puzzle challenge!");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  Today's puzzle: {}x{} board with {} colored regions",
        puzzle.size, puzzle.size, puzzle.size));
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  Rules:");
    w.set_fg(Color::LightGray);
    w.writeln("  - Place exactly one queen in each colored region");
    w.writeln("  - No two queens can share the same row, column, or diagonal");
    w.writeln("  - One attempt per day - make it count!");
    w.writeln("");

    // Show streak info
    if stats.current_streak > 0 {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  Current streak: {} days", stats.current_streak));
        if stats.longest_streak > stats.current_streak {
            w.set_fg(Color::LightGray);
            w.writeln(&format!("  Best streak: {} days", stats.longest_streak));
        }
    } else {
        w.set_fg(Color::LightCyan);
        w.writeln("  Start your streak today!");
    }

    if let Some(best) = stats.best_time_seconds {
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  Best time: {}", format_time(best)));
    }

    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.writeln("  Press any key to begin...");
    w.reset_color();

    w.flush()
}

/// Render the main puzzle board
pub fn render_playing(state: &GameState, stats: &PlayerStats, puzzle: &DailyPuzzle) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state, stats, puzzle);

    // Show error message if any
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::LightRed);
        w.writeln(&format!("  >> {} <<", msg));
        w.writeln("");
    }

    // Column headers
    w.write_str("      ");
    for col in 0..puzzle.size {
        w.set_fg(Color::Yellow);
        w.write_str(&format!(" {} ", col + 1));
        if col < puzzle.size - 1 {
            w.write_str(" ");
        }
    }
    w.writeln("");

    // Top border
    w.write_str("    ");
    w.set_fg(Color::White);
    w.write_str("\u{250C}");
    for col in 0..puzzle.size {
        w.write_str("\u{2500}\u{2500}\u{2500}");
        if col < puzzle.size - 1 {
            w.write_str("\u{252C}");
        }
    }
    w.writeln("\u{2510}");

    // Board rows
    for row in 0..puzzle.size {
        // Row label
        let row_label = (b'A' + row as u8) as char;
        w.set_fg(Color::Yellow);
        w.write_str(&format!("  {} ", row_label));

        // Row content
        w.set_fg(Color::White);
        w.write_str("\u{2502}");

        for col in 0..puzzle.size {
            let region_color = puzzle.get_region_color(row, col);
            let is_cursor = state.cursor == (row, col);
            let has_queen = state.has_queen(row, col);

            // Cell background based on region
            if is_cursor {
                w.set_bg(Color::DarkGray);
            }

            // Cell content
            w.set_fg(region_color.ansi_color());
            if has_queen {
                w.set_fg(Color::White);
                w.bold();
                w.write_str(&format!(" {} ", QUEEN_SYMBOL));
            } else {
                // Show region indicator
                w.write_str(&format!(" {} ", region_color.char()));
            }

            w.reset_color();
            w.set_fg(Color::White);
            w.write_str("\u{2502}");
        }

        // Row label again (right side)
        w.set_fg(Color::Yellow);
        w.writeln(&format!(" {}", row_label));

        // Row separator or bottom border
        if row < puzzle.size - 1 {
            w.write_str("    ");
            w.set_fg(Color::White);
            w.write_str("\u{251C}");
            for col in 0..puzzle.size {
                w.write_str("\u{2500}\u{2500}\u{2500}");
                if col < puzzle.size - 1 {
                    w.write_str("\u{253C}");
                }
            }
            w.writeln("\u{2524}");
        }
    }

    // Bottom border
    w.write_str("    ");
    w.set_fg(Color::White);
    w.write_str("\u{2514}");
    for col in 0..puzzle.size {
        w.write_str("\u{2500}\u{2500}\u{2500}");
        if col < puzzle.size - 1 {
            w.write_str("\u{2534}");
        }
    }
    w.writeln("\u{2518}");

    // Column headers again (bottom)
    w.write_str("      ");
    for col in 0..puzzle.size {
        w.set_fg(Color::Yellow);
        w.write_str(&format!(" {} ", col + 1));
        if col < puzzle.size - 1 {
            w.write_str(" ");
        }
    }
    w.writeln("");

    // Region legend
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.write_str("  Regions: ");
    for region in 0..puzzle.size {
        let color = super::data::RegionColor::from_index(region);
        w.set_fg(color.ansi_color());
        w.write_str(&format!("{}={} ", color.char(), color.name()));
    }
    w.writeln("");

    // Controls
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Move: ");
    w.set_fg(Color::LightCyan);
    w.write_str("WASD/Arrows ");
    w.set_fg(Color::DarkGray);
    w.write_str("| Place: ");
    w.set_fg(Color::LightCyan);
    w.write_str("Space ");
    w.set_fg(Color::DarkGray);
    w.write_str("| Check: ");
    w.set_fg(Color::LightCyan);
    w.write_str("Enter ");
    w.set_fg(Color::DarkGray);
    w.write_str("| Hint: ");
    w.set_fg(Color::LightCyan);
    w.write_str("? ");
    w.set_fg(Color::DarkGray);
    w.write_str("| Clear: ");
    w.set_fg(Color::LightCyan);
    w.write_str("C ");
    w.set_fg(Color::DarkGray);
    w.write_str("| Quit: ");
    w.set_fg(Color::LightCyan);
    w.writeln("Q");

    w.reset_color();
    w.writeln("");
    w.write_str("  Coord or command: ");

    w.flush()
}

/// Render victory screen
pub fn render_victory(state: &GameState, stats: &PlayerStats, puzzle: &DailyPuzzle) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln("");
    w.writeln("  ██╗   ██╗ ██╗  ██████╗ ████████╗  ██████╗  ██████╗  ██╗   ██╗ ██╗");
    w.writeln("  ██║   ██║ ██║ ██╔════╝ ╚══██╔══╝ ██╔═══██╗ ██╔══██╗ ╚██╗ ██╔╝ ██║");
    w.writeln("  ██║   ██║ ██║ ██║         ██║    ██║   ██║ ██████╔╝  ╚████╔╝  ██║");
    w.writeln("  ╚██╗ ██╔╝ ██║ ██║         ██║    ██║   ██║ ██╔══██╗   ╚██╔╝   ╚═╝");
    w.writeln("   ╚████╔╝  ██║ ╚██████╗    ██║    ╚██████╔╝ ██║  ██║    ██║    ██╗");
    w.writeln("    ╚═══╝   ╚═╝  ╚═════╝    ╚═╝     ╚═════╝  ╚═╝  ╚═╝    ╚═╝    ╚═╝");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Puzzle completed: {}", puzzle.date));
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  YOUR RESULTS");
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "\u{2500}".repeat(40)));

    w.set_fg(Color::LightCyan);
    w.write_str("  Time: ");
    w.set_fg(Color::White);
    let time = state.completion_time.unwrap_or(0);
    w.writeln(&format_time(time));

    w.set_fg(Color::LightCyan);
    w.write_str("  Hints used: ");
    w.set_fg(if state.hints_used == 0 { Color::LightGreen } else { Color::Yellow });
    w.writeln(&format!("{}", state.hints_used));

    w.set_fg(Color::LightCyan);
    w.write_str("  Current streak: ");
    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("{} days", stats.current_streak));

    if stats.longest_streak > stats.current_streak {
        w.set_fg(Color::LightCyan);
        w.write_str("  Best streak: ");
        w.set_fg(Color::White);
        w.writeln(&format!("{} days", stats.longest_streak));
    }

    if let Some(best) = stats.best_time_seconds {
        w.set_fg(Color::LightCyan);
        w.write_str("  Best time ever: ");
        w.set_fg(Color::White);
        w.writeln(&format_time(best));
    }

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Come back tomorrow for a new puzzle!");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  [L] Leaderboard  [S] Stats  ");
    w.set_fg(Color::LightCyan);
    w.writeln("[Any key] Return to BBS");
    w.reset_color();

    w.flush()
}

/// Render already played screen
pub fn render_already_played(stats: &PlayerStats, puzzle: &DailyPuzzle) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  You've already completed today's puzzle!");
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Today's puzzle: {}", puzzle.date));

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  YOUR STATS");
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "\u{2500}".repeat(30)));

    w.set_fg(Color::LightCyan);
    w.write_str("  Current streak: ");
    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("{} days", stats.current_streak));

    w.set_fg(Color::LightCyan);
    w.write_str("  Longest streak: ");
    w.set_fg(Color::White);
    w.writeln(&format!("{} days", stats.longest_streak));

    w.set_fg(Color::LightCyan);
    w.write_str("  Games completed: ");
    w.set_fg(Color::White);
    w.writeln(&format!("{}", stats.games_completed));

    if let Some(best) = stats.best_time_seconds {
        w.set_fg(Color::LightCyan);
        w.write_str("  Best time: ");
        w.set_fg(Color::LightGreen);
        w.writeln(&format_time(best));
    }

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Come back tomorrow for a new puzzle!");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  [L] Leaderboard  [S] Stats  ");
    w.set_fg(Color::LightCyan);
    w.writeln("[Any key] Return to BBS");
    w.reset_color();

    w.flush()
}

/// Render stats screen
pub fn render_stats(stats: &PlayerStats) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  YOUR STATISTICS");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "\u{2500}".repeat(40)));

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  STREAKS");

    w.set_fg(Color::LightCyan);
    w.write_str("    Current streak: ");
    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("{} days", stats.current_streak));

    w.set_fg(Color::LightCyan);
    w.write_str("    Longest streak: ");
    w.set_fg(Color::White);
    w.writeln(&format!("{} days", stats.longest_streak));

    w.set_fg(Color::LightCyan);
    w.write_str("    Pause days remaining: ");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("{}/3", stats.pause_days_remaining));

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  PERFORMANCE");

    w.set_fg(Color::LightCyan);
    w.write_str("    Games completed: ");
    w.set_fg(Color::White);
    w.writeln(&format!("{}", stats.games_completed));

    if let Some(best) = stats.best_time_seconds {
        w.set_fg(Color::LightCyan);
        w.write_str("    Best time: ");
        w.set_fg(Color::LightGreen);
        w.writeln(&format_time(best));
    }

    if let Some(avg) = stats.avg_time_seconds {
        w.set_fg(Color::LightCyan);
        w.write_str("    Average time: ");
        w.set_fg(Color::White);
        w.writeln(&format_time(avg));
    }

    w.set_fg(Color::LightCyan);
    w.write_str("    Total hints used: ");
    w.set_fg(if stats.total_hints_used == 0 { Color::LightGreen } else { Color::Yellow });
    w.writeln(&format!("{}", stats.total_hints_used));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render leaderboard screen
pub fn render_leaderboard(entries: &[(String, u32, u32, u32)]) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  LEADERBOARD");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("    {:<4} {:<16} {:>8} {:>10} {:>8}",
        "Rank", "Handle", "Streak", "Best Time", "Games"));
    w.writeln(&format!("    {}", "\u{2500}".repeat(50)));
    w.reset_color();

    if entries.is_empty() {
        w.set_fg(Color::LightGray);
        w.writeln("    No players yet. Be the first!");
    } else {
        for (i, (handle, streak, best_time, games)) in entries.iter().enumerate() {
            let rank = i + 1;
            let rank_color = match rank {
                1 => Color::Yellow,
                2 => Color::White,
                3 => Color::Brown,
                _ => Color::LightGray,
            };

            w.set_fg(rank_color);
            w.write_str(&format!("    {:<4} ", rank));
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("{:<16}", handle));
            w.set_fg(Color::LightMagenta);
            w.write_str(&format!("{:>8}", streak));
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("{:>10}", format_time(*best_time)));
            w.set_fg(Color::White);
            w.writeln(&format!("{:>8}", games));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render confirm quit screen
pub fn render_confirm_quit() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  QUIT GAME");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Your progress will be lost and you cannot");
    w.writeln("  attempt this puzzle again today.");
    w.writeln("");
    w.set_fg(Color::LightRed);
    w.writeln("  This will break your streak!");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  Are you sure? [Y/N] ");
    w.reset_color();

    w.flush()
}

/// Render help screen
pub fn render_help() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  HOW TO PLAY QUEENS");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  OBJECTIVE");
    w.set_fg(Color::LightGray);
    w.writeln("  Place N queens on the board so that:");
    w.writeln("    - Each colored region has exactly one queen");
    w.writeln("    - No two queens share the same row");
    w.writeln("    - No two queens share the same column");
    w.writeln("    - No two queens share the same diagonal");

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  CONTROLS");
    w.set_fg(Color::LightCyan);
    w.write_str("    W/K/Up    ");
    w.set_fg(Color::LightGray);
    w.writeln("Move cursor up");
    w.set_fg(Color::LightCyan);
    w.write_str("    S/J/Down  ");
    w.set_fg(Color::LightGray);
    w.writeln("Move cursor down");
    w.set_fg(Color::LightCyan);
    w.write_str("    A/H/Left  ");
    w.set_fg(Color::LightGray);
    w.writeln("Move cursor left");
    w.set_fg(Color::LightCyan);
    w.write_str("    D/L/Right ");
    w.set_fg(Color::LightGray);
    w.writeln("Move cursor right");
    w.set_fg(Color::LightCyan);
    w.write_str("    Space/E   ");
    w.set_fg(Color::LightGray);
    w.writeln("Place or remove queen");
    w.set_fg(Color::LightCyan);
    w.write_str("    Enter     ");
    w.set_fg(Color::LightGray);
    w.writeln("Check solution");
    w.set_fg(Color::LightCyan);
    w.write_str("    A1-H8     ");
    w.set_fg(Color::LightGray);
    w.writeln("Toggle queen at coordinate");
    w.set_fg(Color::LightCyan);
    w.write_str("    ?         ");
    w.set_fg(Color::LightGray);
    w.writeln("Get a hint (places correct queen)");
    w.set_fg(Color::LightCyan);
    w.write_str("    C         ");
    w.set_fg(Color::LightGray);
    w.writeln("Clear all queens");
    w.set_fg(Color::LightCyan);
    w.write_str("    I         ");
    w.set_fg(Color::LightGray);
    w.writeln("View your stats");
    w.set_fg(Color::LightCyan);
    w.write_str("    Q         ");
    w.set_fg(Color::LightGray);
    w.writeln("Quit game");

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  TIPS");
    w.set_fg(Color::LightGray);
    w.writeln("    - Start with regions that have fewer cells");
    w.writeln("    - Look for forced placements first");
    w.writeln("    - Using hints still counts as completion");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(0), "0:00");
        assert_eq!(format_time(59), "0:59");
        assert_eq!(format_time(60), "1:00");
        assert_eq!(format_time(125), "2:05");
        assert_eq!(format_time(3661), "61:01");
    }

    #[test]
    fn test_render_intro_produces_output() {
        let stats = PlayerStats::new();
        let puzzle = super::super::puzzle::generate_daily_puzzle("2026-01-30");
        let output = render_intro(&puzzle, &stats);
        assert!(!output.is_empty());
        // The intro text says "Welcome to Queens" (not "QUEENS")
        assert!(output.contains("Queens"));
    }

    #[test]
    fn test_render_playing_produces_output() {
        let stats = PlayerStats::new();
        let puzzle = super::super::puzzle::generate_daily_puzzle("2026-01-30");
        let state = GameState::new("2026-01-30");
        let output = render_playing(&state, &stats, &puzzle);
        assert!(!output.is_empty());
        // Should contain board elements
        assert!(output.contains("1")); // Column number
    }

    #[test]
    fn test_render_victory_produces_output() {
        let stats = PlayerStats::new();
        let puzzle = super::super::puzzle::generate_daily_puzzle("2026-01-30");
        let mut state = GameState::new("2026-01-30");
        state.mark_completed();
        let output = render_victory(&state, &stats, &puzzle);
        assert!(!output.is_empty());
        // The victory screen contains "YOUR RESULTS" text
        assert!(output.contains("YOUR RESULTS"));
    }

    #[test]
    fn test_render_stats_produces_output() {
        let stats = PlayerStats::new();
        let output = render_stats(&stats);
        assert!(!output.is_empty());
        assert!(output.contains("STATISTICS"));
    }

    #[test]
    fn test_render_help_produces_output() {
        let output = render_help();
        assert!(!output.is_empty());
        assert!(output.contains("HOW TO PLAY"));
    }
}
