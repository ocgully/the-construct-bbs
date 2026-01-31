//! Sudoku ANSI rendering functions
//!
//! Visual identity: Clean, minimalist look with deep blue theme
//! representing logic and precision.

use crate::terminal::{AnsiWriter, Color};
use super::state::{GameState, CellState};
use super::screen::{GameScreen, SudokuFlow};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format seconds as MM:SS
pub fn format_time(seconds: u32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}

/// Render the game header with title
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::LightBlue);
    w.bold();
    w.writeln("");
    w.writeln("   ███████╗██╗   ██╗██████╗  ██████╗ ██╗  ██╗██╗   ██╗");
    w.writeln("   ██╔════╝██║   ██║██╔══██╗██╔═══██╗██║ ██╔╝██║   ██║");
    w.writeln("   ███████╗██║   ██║██║  ██║██║   ██║█████╔╝ ██║   ██║");
    w.writeln("   ╚════██║██║   ██║██║  ██║██║   ██║██╔═██╗ ██║   ██║");
    w.writeln("   ███████║╚██████╔╝██████╔╝╚██████╔╝██║  ██╗╚██████╔╝");
    w.writeln("   ╚══════╝ ╚═════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ");
    w.reset_color();
}

/// Render a smaller header for in-game screens
fn render_small_header(w: &mut AnsiWriter, date: &str) {
    w.clear_screen();
    w.set_fg(Color::LightBlue);
    w.bold();
    w.write_str("  SUDOKU");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  Daily Puzzle: {}", date));
    w.reset_color();
}

// ============================================================================
// PUBLIC RENDER FUNCTIONS
// ============================================================================

/// Render intro screen
pub fn render_intro(flow: &SudokuFlow) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Fill the 9x9 grid so that each row, column, and 3x3 box");
    w.writeln("  contains all digits from 1 to 9 exactly once.");
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln("  Everyone gets the same puzzle each day.");
    w.writeln("  Complete daily puzzles to build your streak!");
    w.writeln("");

    w.set_fg(Color::Yellow);
    if flow.current_streak > 0 {
        w.writeln(&format!("  Current Streak: {} days", flow.current_streak));
    }
    if flow.longest_streak > 0 {
        w.writeln(&format!("  Longest Streak: {} days", flow.longest_streak));
    }
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to start...");
    w.reset_color();

    w.flush()
}

/// Render the main puzzle screen
pub fn render_playing(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_small_header(&mut w, &state.puzzle_date);

    // Status bar
    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(60));
    w.reset_color();

    w.set_fg(Color::LightCyan);
    w.write_str(&format!("  Time: {}  ", format_time(state.elapsed_seconds)));
    w.set_fg(Color::DarkGray);
    w.write_str("│  ");

    w.set_fg(Color::White);
    w.write_str(&format!("Filled: {}/81  ", state.filled_count()));
    w.set_fg(Color::DarkGray);
    w.write_str("│  ");

    if state.error_count > 0 {
        w.set_fg(Color::LightRed);
    } else {
        w.set_fg(Color::LightGreen);
    }
    w.write_str(&format!("Errors: {}  ", state.error_count));
    w.set_fg(Color::DarkGray);
    w.write_str("│  ");

    if state.pencil_mode {
        w.set_fg(Color::LightMagenta);
        w.writeln("PENCIL");
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("pencil");
    }

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(60));
    w.reset_color();

    w.writeln("");

    // Render the grid
    render_grid(&mut w, state);

    // Show last message if any
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(if msg.contains("Incorrect") || msg.contains("Cannot") {
            Color::LightRed
        } else if msg.contains("Congratulations") {
            Color::LightGreen
        } else {
            Color::Yellow
        });
        w.writeln(&format!("  {}", msg));
        w.reset_color();
    }

    // Controls
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  WASD/HJKL: Move   1-9: Enter   0/Space: Clear");
    w.writeln("  P: Pencil mode   ?: Help   T: Stats   Q: Quit");
    w.reset_color();

    w.flush()
}

/// Render the 9x9 Sudoku grid
fn render_grid(w: &mut AnsiWriter, state: &GameState) {
    let (cursor_row, cursor_col) = state.cursor;

    // Top border
    w.set_fg(Color::LightBlue);
    w.writeln("        1   2   3   4   5   6   7   8   9");
    w.writeln("      ╔═══════════╦═══════════╦═══════════╗");

    for row in 0..9 {
        // Row number
        w.set_fg(Color::LightBlue);
        w.write_str(&format!("    {} ║", (b'A' + row as u8) as char));

        for col in 0..9 {
            let is_cursor = row == cursor_row && col == cursor_col;
            let is_error = state.last_error == Some((row, col));
            let has_conflict = state.has_conflict(row, col);

            // Determine cell background/highlighting
            if is_cursor {
                w.set_bg(Color::Blue);
            } else if is_error {
                w.set_bg(Color::Red);
            }

            // Cell content
            match state.grid[row][col] {
                CellState::Given(v) => {
                    w.set_fg(Color::White);
                    w.bold();
                    w.write_str(&format!(" {} ", v));
                    w.reset_color();
                }
                CellState::Entered(v) => {
                    if has_conflict {
                        w.set_fg(Color::LightRed);
                    } else {
                        w.set_fg(Color::LightCyan);
                    }
                    w.write_str(&format!(" {} ", v));
                    w.reset_color();
                }
                CellState::Empty => {
                    // Show pencil marks or empty
                    let marks = state.get_pencil_marks(row, col);
                    if !marks.is_empty() && marks.len() <= 3 {
                        w.set_fg(Color::DarkGray);
                        let marks_str: String = marks.iter().map(|n| n.to_string()).collect();
                        w.write_str(&format!("{:^3}", marks_str));
                        w.reset_color();
                    } else if !marks.is_empty() {
                        w.set_fg(Color::DarkGray);
                        w.write_str(" * ");
                        w.reset_color();
                    } else {
                        w.write_str(" · ");
                    }
                }
            }

            if is_cursor || is_error {
                w.reset_color();
            }

            // Box separator
            w.set_fg(Color::LightBlue);
            if col == 2 || col == 5 {
                w.write_str("║");
            } else if col < 8 {
                w.set_fg(Color::Blue);
                w.write_str("│");
            }
        }

        w.set_fg(Color::LightBlue);
        w.writeln("║");

        // Row separator
        if row == 2 || row == 5 {
            w.writeln("      ╠═══════════╬═══════════╬═══════════╣");
        } else if row < 8 {
            w.set_fg(Color::Blue);
            w.writeln("      ╟───┼───┼───╫───┼───┼───╫───┼───┼───╢");
        }
    }

    // Bottom border
    w.set_fg(Color::LightBlue);
    w.writeln("      ╚═══════════╩═══════════╩═══════════╝");
    w.reset_color();
}

/// Render completion screen
pub fn render_completed(state: &GameState, current_streak: u32, longest_streak: u32) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln("   ██████╗ ██████╗ ███╗   ███╗██████╗ ██╗     ███████╗████████╗███████╗██╗");
    w.writeln("  ██╔════╝██╔═══██╗████╗ ████║██╔══██╗██║     ██╔════╝╚══██╔══╝██╔════╝██║");
    w.writeln("  ██║     ██║   ██║██╔████╔██║██████╔╝██║     █████╗     ██║   █████╗  ██║");
    w.writeln("  ██║     ██║   ██║██║╚██╔╝██║██╔═══╝ ██║     ██╔══╝     ██║   ██╔══╝  ╚═╝");
    w.writeln("  ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║     ███████╗███████╗   ██║   ███████╗██╗");
    w.writeln("   ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚══════╝╚══════╝   ╚═╝   ╚══════╝╚═╝");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  Puzzle: {}", state.puzzle_date));
    w.writeln(&format!("  Time: {}", format_time(state.elapsed_seconds)));
    w.writeln(&format!("  Errors: {}", state.error_count));

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  Current Streak: {} day{}", current_streak, if current_streak == 1 { "" } else { "s" }));
    if longest_streak > current_streak {
        w.writeln(&format!("  Longest Streak: {} days", longest_streak));
    }
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Render already played screen
pub fn render_already_played(completion_time: Option<u32>, current_streak: u32, longest_streak: u32) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  You've already completed today's puzzle!");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    if let Some(time) = completion_time {
        w.writeln(&format!("  Your time: {}", format_time(time)));
    }
    w.writeln(&format!("  Current Streak: {} day{}", current_streak, if current_streak == 1 { "" } else { "s" }));
    if longest_streak > 0 {
        w.writeln(&format!("  Longest Streak: {} days", longest_streak));
    }

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Come back tomorrow for a new puzzle!");
    w.writeln("");
    w.writeln("  You have 3 pause days per week if you need to miss a day.");
    w.writeln("  (Pause days preserve your streak)");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [T] ");
    w.set_fg(Color::White);
    w.writeln("View Stats & Leaderboard");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Return to BBS");
    w.reset_color();

    w.flush()
}

/// Render stats/leaderboard screen
pub fn render_stats(
    current_streak: u32,
    longest_streak: u32,
    games_completed: u32,
    best_time: Option<u32>,
    leaderboard: &[(String, u32, u32)], // (handle, streak, games)
) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  YOUR STATS");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  Current Streak: {} days", current_streak));
    w.writeln(&format!("  Longest Streak: {} days", longest_streak));
    w.writeln(&format!("  Puzzles Completed: {}", games_completed));
    if let Some(time) = best_time {
        w.writeln(&format!("  Best Time: {}", format_time(time)));
    }

    // Leaderboard
    if !leaderboard.is_empty() {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln("  LEADERBOARD - Longest Streaks");
        w.reset_color();

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln(&format!("    {:<4} {:<16} {:>8} {:>10}", "Rank", "Player", "Streak", "Completed"));
        w.writeln(&format!("    {}", "─".repeat(45)));
        w.reset_color();

        for (i, (handle, streak, games)) in leaderboard.iter().enumerate() {
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
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("{:>8}", streak));
            w.set_fg(Color::White);
            w.writeln(&format!("{:>10}", games));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
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
    w.writeln("  HOW TO PLAY");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Fill every row, column, and 3x3 box with the numbers 1-9.");
    w.writeln("  Each number can only appear once in each row, column, and box.");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CONTROLS");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    W/K   ");
    w.set_fg(Color::White);
    w.writeln("Move cursor up");

    w.set_fg(Color::LightCyan);
    w.write_str("    S/J   ");
    w.set_fg(Color::White);
    w.writeln("Move cursor down");

    w.set_fg(Color::LightCyan);
    w.write_str("    A/H   ");
    w.set_fg(Color::White);
    w.writeln("Move cursor left");

    w.set_fg(Color::LightCyan);
    w.write_str("    D/L   ");
    w.set_fg(Color::White);
    w.writeln("Move cursor right");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    1-9   ");
    w.set_fg(Color::White);
    w.writeln("Enter a number");

    w.set_fg(Color::LightCyan);
    w.write_str("    0/SPC ");
    w.set_fg(Color::White);
    w.writeln("Clear cell");

    w.set_fg(Color::LightCyan);
    w.write_str("    P     ");
    w.set_fg(Color::White);
    w.writeln("Toggle pencil mode (for notes)");

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  STREAKS");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Complete puzzles on consecutive days to build your streak.");
    w.writeln("  You get 3 pause days per week - skip a day without losing");
    w.writeln("  your streak. Pause days reset every Monday.");

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
    w.writeln("  QUIT PUZZLE");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Your progress will be saved.");
    w.writeln("  You can resume this puzzle until midnight Eastern.");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  Are you sure? [Y/N] ");
    w.reset_color();

    w.flush()
}

/// Main render function - dispatch based on current screen
pub fn render_screen(flow: &SudokuFlow) -> String {
    match flow.current_screen() {
        GameScreen::Intro => render_intro(flow),
        GameScreen::Playing => {
            if let Some(state) = flow.game_state() {
                render_playing(state)
            } else {
                render_intro(flow)
            }
        }
        GameScreen::Completed => {
            if let Some(state) = flow.game_state() {
                render_completed(state, flow.current_streak, flow.longest_streak)
            } else {
                render_intro(flow)
            }
        }
        GameScreen::AlreadyPlayed => {
            render_already_played(flow.completion_time, flow.current_streak, flow.longest_streak)
        }
        GameScreen::Stats => {
            // Stats are rendered with data from the database
            // This is a placeholder - actual data comes from service layer
            render_stats(
                flow.current_streak,
                flow.longest_streak,
                0, // games_completed
                None, // best_time
                &[], // leaderboard
            )
        }
        GameScreen::Help => render_help(),
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(0), "00:00");
        assert_eq!(format_time(65), "01:05");
        assert_eq!(format_time(3661), "61:01");
    }

    #[test]
    fn test_render_intro() {
        let flow = SudokuFlow::new();
        let output = render_intro(&flow);
        // Check for key phrases in the intro
        assert!(output.contains("Fill the 9x9 grid"));
        assert!(output.contains("Press any key"));
    }

    #[test]
    fn test_render_help() {
        let output = render_help();
        assert!(output.contains("HOW TO PLAY"));
        assert!(output.contains("CONTROLS"));
        assert!(output.contains("STREAKS"));
    }

    #[test]
    fn test_render_confirm_quit() {
        let output = render_confirm_quit();
        assert!(output.contains("QUIT"));
        assert!(output.contains("Y/N"));
    }

    #[test]
    fn test_render_playing() {
        let mut flow = SudokuFlow::new();
        flow.start_puzzle();
        if let Some(state) = flow.game_state() {
            let output = render_playing(state);
            assert!(output.contains("Time:"));
            assert!(output.contains("Filled:"));
        }
    }

    #[test]
    fn test_render_already_played() {
        let output = render_already_played(Some(300), 5, 10);
        assert!(output.contains("already completed"));
        assert!(output.contains("05:00")); // 300 seconds
        assert!(output.contains("5"));
    }
}
