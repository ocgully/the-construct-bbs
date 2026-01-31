//! ANSI rendering for Acromania
//!
//! Creates the visual interface with a unique neon/party aesthetic.
//! Uses bright magentas, cyans, and yellows for a party game feel.

use crate::terminal::{AnsiWriter, Color};
use super::data::format_acronym;
use super::game::{AcroGame, RoundPhase};
use super::lobby::{GameLobby, MatchType};
use super::scoring::RoundResult;

// ============================================================================
// THEME COLORS
// ============================================================================
// Acromania uses a neon party palette:
// - Primary: Magenta/LightMagenta for headers and emphasis
// - Secondary: Cyan/LightCyan for interactive elements
// - Accent: Yellow for scores and highlights
// - Text: White/LightGray for content

const HEADER_COLOR: Color = Color::LightMagenta;
const ACCENT_COLOR: Color = Color::Yellow;
const OPTION_COLOR: Color = Color::LightCyan;
const TEXT_COLOR: Color = Color::White;
const DIM_COLOR: Color = Color::DarkGray;
const SUCCESS_COLOR: Color = Color::LightGreen;
const ERROR_COLOR: Color = Color::LightRed;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Render the game header/title art
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln("");
    w.writeln("   █████╗  ██████╗██████╗  ██████╗ ███╗   ███╗ █████╗ ███╗   ██╗██╗ █████╗");
    w.writeln("  ██╔══██╗██╔════╝██╔══██╗██╔═══██╗████╗ ████║██╔══██╗████╗  ██║██║██╔══██╗");
    w.writeln("  ███████║██║     ██████╔╝██║   ██║██╔████╔██║███████║██╔██╗ ██║██║███████║");
    w.writeln("  ██╔══██║██║     ██╔══██╗██║   ██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║██╔══██║");
    w.writeln("  ██║  ██║╚██████╗██║  ██║╚██████╔╝██║ ╚═╝ ██║██║  ██║██║ ╚████║██║██║  ██║");
    w.writeln("  ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝");
    w.reset_color();
    w.set_fg(DIM_COLOR);
    w.writeln("                         The Acronym Party Game");
    w.reset_color();
}

/// Render a horizontal divider
fn render_divider(w: &mut AnsiWriter) {
    w.set_fg(DIM_COLOR);
    w.writeln(&"\u{2500}".repeat(80));
    w.reset_color();
}

/// Render timer bar
fn render_timer(w: &mut AnsiWriter, seconds_remaining: u64, total_seconds: u64, label: &str) {
    let filled = if total_seconds > 0 {
        ((seconds_remaining as f64 / total_seconds as f64) * 40.0) as usize
    } else {
        0
    };
    let empty = 40 - filled;

    w.set_fg(TEXT_COLOR);
    w.write_str(&format!("  {} ", label));

    // Color based on urgency
    let bar_color = if seconds_remaining <= 10 {
        ERROR_COLOR
    } else if seconds_remaining <= 20 {
        ACCENT_COLOR
    } else {
        SUCCESS_COLOR
    };

    w.set_fg(bar_color);
    w.write_str(&"█".repeat(filled));
    w.set_fg(DIM_COLOR);
    w.write_str(&"░".repeat(empty));

    w.set_fg(TEXT_COLOR);
    w.writeln(&format!(" {:02}s", seconds_remaining));
    w.reset_color();
}

// ============================================================================
// PUBLIC RENDER FUNCTIONS
// ============================================================================

/// Render the main menu for Acromania
pub fn render_acro_menu() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_divider(&mut w);

    w.writeln("");
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln("  Welcome to Acromania!");
    w.reset_color();
    w.set_fg(TEXT_COLOR);
    w.writeln("  Create acronyms, vote on the best ones, win points!");
    w.writeln("");

    w.set_fg(OPTION_COLOR);
    w.write_str("    [J] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Join Public Game");

    w.set_fg(OPTION_COLOR);
    w.write_str("    [C] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Create Public Game");

    w.set_fg(OPTION_COLOR);
    w.write_str("    [P] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Create Private Game (Friends Only)");

    w.set_fg(OPTION_COLOR);
    w.write_str("    [I] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Join by Invite Code");

    w.writeln("");
    w.set_fg(OPTION_COLOR);
    w.write_str("    [L] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Leaderboard");

    w.set_fg(OPTION_COLOR);
    w.write_str("    [H] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("How to Play");

    w.set_fg(OPTION_COLOR);
    w.write_str("    [Q] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Quit to Main Menu");

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render the how to play screen
pub fn render_how_to_play() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_divider(&mut w);

    w.writeln("");
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln("  HOW TO PLAY");
    w.reset_color();
    w.writeln("");

    w.set_fg(TEXT_COLOR);
    w.writeln("  1. Each round, you'll see a random acronym (e.g., W.T.F.L.)");
    w.writeln("");
    w.writeln("  2. Create a phrase where each word starts with the letters");
    w.writeln("     Example: \"Weasels Typically Fear Llamas\"");
    w.writeln("");
    w.writeln("  3. After everyone submits (or time runs out), vote for");
    w.writeln("     your favorite answer - but you can't vote for yourself!");
    w.writeln("");
    w.writeln("  4. Points are awarded based on votes received.");
    w.writeln("");

    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln("  SCORING:");
    w.reset_color();
    w.set_fg(TEXT_COLOR);
    w.writeln("    - 100 points per vote");
    w.writeln("    - Up to 50 bonus points for fast submissions");
    w.writeln("    - 200 bonus for unanimous vote");
    w.writeln("    - 10 points just for participating");
    w.writeln("");

    w.set_fg(DIM_COLOR);
    w.writeln("  10 rounds total, acronyms get longer as you progress!");
    w.writeln("");

    w.set_fg(OPTION_COLOR);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render the lobby screen
pub fn render_lobby(lobby: &GameLobby) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_divider(&mut w);

    w.writeln("");

    // Show invite code for private games
    if let MatchType::Private { invite_code } = &lobby.match_type {
        w.set_fg(ACCENT_COLOR);
        w.bold();
        w.write_str("  INVITE CODE: ");
        w.set_fg(TEXT_COLOR);
        w.writeln(invite_code);
        w.reset_color();
        w.set_fg(DIM_COLOR);
        w.writeln("  Share this code with friends to join!");
        w.writeln("");
    } else {
        w.set_fg(SUCCESS_COLOR);
        w.writeln("  PUBLIC GAME - Anyone can join");
        w.writeln("");
    }

    // Player list
    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln(&format!("  PLAYERS ({}/{})", lobby.player_count(), lobby.config.max_players));
    w.reset_color();
    w.writeln("");

    for (i, player) in lobby.players.iter().enumerate() {
        let is_host = player.user_id == lobby.host_user_id;
        let status = if player.is_ready { "READY" } else { "..." };

        w.set_fg(if player.is_ready { SUCCESS_COLOR } else { DIM_COLOR });
        w.write_str(&format!("    {}. ", i + 1));

        if is_host {
            w.set_fg(ACCENT_COLOR);
            w.write_str(&format!("{} ", player.handle));
            w.set_fg(HEADER_COLOR);
            w.write_str("[HOST] ");
        } else {
            w.set_fg(TEXT_COLOR);
            w.write_str(&format!("{} ", player.handle));
        }

        w.set_fg(if player.is_ready { SUCCESS_COLOR } else { DIM_COLOR });
        w.writeln(status);
    }

    w.writeln("");
    render_divider(&mut w);

    // Options
    w.writeln("");
    w.set_fg(OPTION_COLOR);
    w.write_str("    [R] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Toggle Ready");

    if lobby.can_start() {
        w.set_fg(SUCCESS_COLOR);
        w.write_str("    [S] ");
        w.set_fg(TEXT_COLOR);
        w.writeln("Start Game (Host)");
    } else {
        w.set_fg(DIM_COLOR);
        w.writeln(&format!("    Need {} players to start", lobby.config.min_players));
    }

    w.set_fg(OPTION_COLOR);
    w.write_str("    [Q] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Leave Lobby");

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render the acronym reveal screen
pub fn render_acronym_reveal(game: &AcroGame) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    let round = match &game.current_round {
        Some(r) => r,
        None => return w.flush(),
    };

    w.writeln("");
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln(&format!("                    ROUND {} OF {}", round.number, game.config.total_rounds));
    w.reset_color();

    if round.phase == RoundPhase::FaceOff {
        w.set_fg(ERROR_COLOR);
        w.bold();
        w.writeln("                      === FACE-OFF ===");
        w.reset_color();
    }

    w.writeln("");

    // Category (if any)
    if let Some(category) = round.category {
        w.set_fg(HEADER_COLOR);
        w.write_str("                    Category: ");
        w.set_fg(TEXT_COLOR);
        w.writeln(category.name);
        w.set_fg(DIM_COLOR);
        w.writeln(&format!("                    {}", category.description));
    } else {
        w.set_fg(DIM_COLOR);
        w.writeln("                    Category: Open (anything goes!)");
    }

    w.writeln("");
    w.writeln("");

    // Big acronym display
    let formatted = format_acronym(&round.acronym);
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln(&format!("                         {}", formatted));
    w.reset_color();

    w.writeln("");
    w.writeln("");

    // Countdown
    let remaining = game.time_remaining();
    w.set_fg(TEXT_COLOR);
    w.writeln(&format!("                    Starting in {} seconds...", remaining));

    w.flush()
}

/// Render the submission screen
pub fn render_submission(game: &AcroGame, player_input: &str) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    let round = match &game.current_round {
        Some(r) => r,
        None => return w.flush(),
    };

    // Timer
    let remaining = game.time_remaining();
    render_timer(&mut w, remaining, game.config.submission_time_secs, "Time:");

    w.writeln("");

    // Round info
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.write_str(&format!("  Round {}", round.number));
    if let Some(cat) = round.category {
        w.set_fg(DIM_COLOR);
        w.write_str(&format!(" - {}", cat.name));
    }
    w.writeln("");
    w.reset_color();

    w.writeln("");

    // Acronym
    let formatted = format_acronym(&round.acronym);
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln(&format!("                         {}", formatted));
    w.reset_color();

    w.writeln("");

    // Letter hints
    w.set_fg(DIM_COLOR);
    w.write_str("  ");
    for (i, c) in round.acronym.chars().enumerate() {
        if i > 0 {
            w.write_str(" ");
        }
        w.write_str(&format!("{}_____", c));
    }
    w.writeln("");

    w.writeln("");
    render_divider(&mut w);
    w.writeln("");

    // Input area
    w.set_fg(TEXT_COLOR);
    w.writeln("  Type your phrase and press ENTER to submit:");
    w.writeln("");
    w.set_fg(OPTION_COLOR);
    w.write_str("  > ");
    w.set_fg(TEXT_COLOR);
    w.writeln(player_input);

    // Submission status
    let submitted_count = round.submissions.len();
    let total_players = game.players.len();
    w.writeln("");
    w.set_fg(DIM_COLOR);
    w.writeln(&format!("  ({}/{} players have submitted)", submitted_count, total_players));

    w.flush()
}

/// Render the voting screen
pub fn render_voting(game: &AcroGame, options: &[(u64, String)], current_vote: Option<u64>) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    let round = match &game.current_round {
        Some(r) => r,
        None => return w.flush(),
    };

    // Timer
    let remaining = game.time_remaining();
    render_timer(&mut w, remaining, game.config.voting_time_secs, "Vote:");

    w.writeln("");

    // Acronym reminder
    let formatted = format_acronym(&round.acronym);
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln(&format!("  The acronym was: {}", formatted));
    w.reset_color();

    w.writeln("");
    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln("  VOTE FOR YOUR FAVORITE (you can't vote for yourself):");
    w.reset_color();
    w.writeln("");

    // Voting options
    for (i, (id, text)) in options.iter().enumerate() {
        let num = i + 1;
        let is_selected = current_vote == Some(*id);

        if is_selected {
            w.set_fg(SUCCESS_COLOR);
            w.write_str(&format!("  >> [{:2}] ", num));
            w.bold();
            w.write_str(text);
            w.reset_color();
            w.set_fg(SUCCESS_COLOR);
            w.writeln(" << YOUR VOTE");
        } else {
            w.set_fg(OPTION_COLOR);
            w.write_str(&format!("     [{:2}] ", num));
            w.set_fg(TEXT_COLOR);
            w.writeln(text);
        }
    }

    if options.is_empty() {
        w.set_fg(DIM_COLOR);
        w.writeln("  No submissions this round!");
    }

    w.writeln("");
    w.set_fg(DIM_COLOR);
    w.writeln("  Enter a number to vote (you can change your vote until time runs out)");

    w.flush()
}

/// Render the round results screen
pub fn render_results(game: &AcroGame, results: &[RoundResult]) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    let round = match &game.current_round {
        Some(r) => r,
        None => return w.flush(),
    };

    w.writeln("");
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln(&format!("  ROUND {} RESULTS", round.number));
    w.reset_color();

    let formatted = format_acronym(&round.acronym);
    w.set_fg(DIM_COLOR);
    w.writeln(&format!("  Acronym: {}", formatted));
    w.writeln("");

    for (i, result) in results.iter().enumerate() {
        let rank = i + 1;

        // Winner highlighting
        if result.is_winner {
            w.set_fg(ACCENT_COLOR);
            w.bold();
            w.write_str(&format!("  #{} ", rank));
        } else {
            w.set_fg(DIM_COLOR);
            w.write_str(&format!("  #{} ", rank));
        }

        w.set_fg(TEXT_COLOR);
        w.writeln(&format!("\"{}\"", result.submission_text));

        w.set_fg(HEADER_COLOR);
        w.write_str(&format!("     by {} ", result.handle));

        w.set_fg(ACCENT_COLOR);
        w.write_str(&format!("- {} votes ", result.votes_received));

        w.set_fg(SUCCESS_COLOR);
        w.write_str(&format!("({} pts", result.total_points));

        // Show bonuses
        if result.speed_bonus > 0 {
            w.set_fg(DIM_COLOR);
            w.write_str(&format!(", +{} speed", result.speed_bonus));
        }
        if result.unanimous_bonus > 0 {
            w.set_fg(ACCENT_COLOR);
            w.write_str(", UNANIMOUS!");
        }
        w.writeln(")");
        w.reset_color();

        w.writeln("");
    }

    if results.is_empty() {
        w.set_fg(DIM_COLOR);
        w.writeln("  No valid submissions this round.");
    }

    // Current standings
    w.writeln("");
    render_divider(&mut w);
    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln("  CURRENT STANDINGS:");
    w.reset_color();

    let standings = game.get_standings();
    for (i, (_, handle, score)) in standings.iter().take(5).enumerate() {
        let rank = i + 1;
        let color = match rank {
            1 => ACCENT_COLOR,
            2 => TEXT_COLOR,
            3 => Color::Brown,
            _ => DIM_COLOR,
        };

        w.set_fg(color);
        w.writeln(&format!("    {}. {} - {} pts", rank, handle, score));
    }

    w.flush()
}

/// Render the final results screen
pub fn render_final_results(game: &AcroGame) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln("");
    w.writeln("  ███████╗██╗███╗   ██╗ █████╗ ██╗         ███████╗ ██████╗ ██████╗ ██████╗ ███████╗");
    w.writeln("  ██╔════╝██║████╗  ██║██╔══██╗██║         ██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔════╝");
    w.writeln("  █████╗  ██║██╔██╗ ██║███████║██║         ███████╗██║     ██║   ██║██████╔╝█████╗  ");
    w.writeln("  ██╔══╝  ██║██║╚██╗██║██╔══██║██║         ╚════██║██║     ██║   ██║██╔══██╗██╔══╝  ");
    w.writeln("  ██║     ██║██║ ╚████║██║  ██║███████╗    ███████║╚██████╗╚██████╔╝██║  ██║███████╗");
    w.writeln("  ╚═╝     ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝    ╚══════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝");
    w.reset_color();

    w.writeln("");
    render_divider(&mut w);

    let standings = game.get_standings();

    if let Some((_, handle, score)) = standings.first() {
        w.writeln("");
        w.set_fg(ACCENT_COLOR);
        w.bold();
        w.writeln(&format!("                    WINNER: {}", handle));
        w.writeln(&format!("                    SCORE: {} points", score));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln("  FINAL STANDINGS:");
    w.reset_color();
    w.writeln("");

    for (i, (_, handle, score)) in standings.iter().enumerate() {
        let rank = i + 1;
        let (medal, color) = match rank {
            1 => ("", ACCENT_COLOR),
            2 => ("", TEXT_COLOR),
            3 => ("", Color::Brown),
            _ => (" ", DIM_COLOR),
        };

        w.set_fg(color);
        w.writeln(&format!("    {} {:2}. {:<20} {:>6} pts", medal, rank, handle, score));
    }

    w.writeln("");
    render_divider(&mut w);
    w.set_fg(DIM_COLOR);
    w.writeln("  Thanks for playing Acromania!");
    w.writeln("");
    w.set_fg(OPTION_COLOR);
    w.writeln("  Press any key to return to menu...");
    w.reset_color();

    w.flush()
}

/// Render an error message
pub fn render_error(message: &str) -> String {
    let mut w = AnsiWriter::new();
    w.set_fg(ERROR_COLOR);
    w.bold();
    w.write_str("  ERROR: ");
    w.reset_color();
    w.set_fg(TEXT_COLOR);
    w.writeln(message);
    w.flush()
}

/// Render game list for joining
pub fn render_game_list(lobbies: &[(u64, usize, usize)]) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_divider(&mut w);

    w.writeln("");
    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln("  AVAILABLE GAMES:");
    w.reset_color();
    w.writeln("");

    if lobbies.is_empty() {
        w.set_fg(DIM_COLOR);
        w.writeln("  No games available. Create one!");
    } else {
        for (i, (id, players, max)) in lobbies.iter().enumerate() {
            w.set_fg(OPTION_COLOR);
            w.write_str(&format!("    [{}] ", i + 1));
            w.set_fg(TEXT_COLOR);
            w.writeln(&format!("Game #{} - {}/{} players", id, players, max));
        }
    }

    w.writeln("");
    w.set_fg(OPTION_COLOR);
    w.write_str("    [B] ");
    w.set_fg(TEXT_COLOR);
    w.writeln("Back");

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render invite code entry prompt
pub fn render_invite_prompt(current_input: &str) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_divider(&mut w);

    w.writeln("");
    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln("  JOIN PRIVATE GAME");
    w.reset_color();
    w.writeln("");
    w.set_fg(TEXT_COLOR);
    w.writeln("  Enter the 6-character invite code:");
    w.writeln("");
    w.set_fg(OPTION_COLOR);
    w.write_str("  > ");
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.write_str(current_input);
    w.reset_color();
    w.writeln("");
    w.writeln("");
    w.set_fg(DIM_COLOR);
    w.writeln("  Press ENTER to join, or Q to cancel");

    w.flush()
}

/// Render waiting for game start message
#[allow(dead_code)]
pub fn render_waiting_start(seconds: u64) -> String {
    let mut w = AnsiWriter::new();
    w.set_fg(ACCENT_COLOR);
    w.bold();
    w.writeln("");
    w.writeln(&format!("                    Game starting in {} seconds...", seconds));
    w.reset_color();
    w.flush()
}

/// Render submission confirmed message
pub fn render_submission_confirmed(text: &str) -> String {
    let mut w = AnsiWriter::new();
    w.set_fg(SUCCESS_COLOR);
    w.bold();
    w.writeln("");
    w.writeln("  Submission locked in!");
    w.reset_color();
    w.set_fg(TEXT_COLOR);
    w.writeln(&format!("  \"{}\"", text));
    w.set_fg(DIM_COLOR);
    w.writeln("");
    w.writeln("  Waiting for other players...");
    w.flush()
}

/// Render leaderboard
pub fn render_leaderboard(entries: &[(String, i64, u32, u32)]) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_divider(&mut w);

    w.writeln("");
    w.set_fg(HEADER_COLOR);
    w.bold();
    w.writeln("  ACROMANIA HALL OF FAME");
    w.reset_color();
    w.writeln("");

    w.set_fg(DIM_COLOR);
    w.writeln(&format!("    {:<4} {:<20} {:>10} {:>8} {:>8}", "Rank", "Player", "Best Score", "Games", "Wins"));
    w.writeln(&format!("    {}", "\u{2500}".repeat(55)));
    w.reset_color();

    if entries.is_empty() {
        w.set_fg(DIM_COLOR);
        w.writeln("    No games played yet. Be the first!");
    } else {
        for (i, (handle, score, games, wins)) in entries.iter().enumerate() {
            let rank = i + 1;
            let color = match rank {
                1 => ACCENT_COLOR,
                2 => TEXT_COLOR,
                3 => Color::Brown,
                _ => DIM_COLOR,
            };

            w.set_fg(color);
            w.writeln(&format!("    {:<4} {:<20} {:>10} {:>8} {:>8}", rank, handle, score, games, wins));
        }
    }

    w.writeln("");
    w.set_fg(OPTION_COLOR);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_menu_not_empty() {
        let output = render_acro_menu();
        assert!(!output.is_empty());
        assert!(output.contains("Welcome to Acromania"));
    }

    #[test]
    fn test_render_how_to_play_not_empty() {
        let output = render_how_to_play();
        assert!(!output.is_empty());
        assert!(output.contains("HOW TO PLAY"));
    }

    #[test]
    fn test_render_error() {
        let output = render_error("Test error");
        assert!(output.contains("ERROR"));
        assert!(output.contains("Test error"));
    }

    #[test]
    fn test_render_game_list_empty() {
        let output = render_game_list(&[]);
        assert!(output.contains("No games available"));
    }

    #[test]
    fn test_render_game_list_with_games() {
        let lobbies = vec![(1, 3, 16), (2, 5, 16)];
        let output = render_game_list(&lobbies);
        assert!(output.contains("Game #1"));
        assert!(output.contains("3/16"));
    }
}
