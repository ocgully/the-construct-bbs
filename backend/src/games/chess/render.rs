//! Chess board ANSI rendering
//!
//! Visual theme: Classic chess with green and brown squares
//! Uses Unicode chess pieces for modern terminals

use crate::terminal::{AnsiWriter, Color};
use super::board::{Board, Square, Color as PieceColor};
use super::state::{GameState, GameStatus, PlayerColor};

/// Render the chess board
pub fn render_board(board: &Board, perspective: PlayerColor) -> String {
    render_board_with_highlights(board, perspective, None, &[])
}

/// Render the chess board with optional highlights
/// - last_move: highlights the from/to squares of the last move
/// - highlights: additional squares to highlight (e.g., legal moves)
pub fn render_board_with_highlights(
    board: &Board,
    perspective: PlayerColor,
    last_move: Option<(Square, Square)>,
    highlights: &[Square],
) -> String {
    let mut w = AnsiWriter::new();

    // File labels at top
    w.write_str("      ");
    let files: Vec<char> = if perspective == PlayerColor::White {
        ('a'..='h').collect()
    } else {
        ('a'..='h').rev().collect()
    };
    for f in &files {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!(" {} ", f));
    }
    w.writeln("");

    // Top border
    w.set_fg(Color::DarkGray);
    w.writeln("     +--------------------------+");

    // Ranks (rows)
    let ranks: Vec<u8> = if perspective == PlayerColor::White {
        (0..8).rev().collect()
    } else {
        (0..8).collect()
    };

    for rank in ranks {
        // Rank label
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("  {}  ", rank + 1));
        w.set_fg(Color::DarkGray);
        w.write_str("|");

        let file_range: Vec<u8> = if perspective == PlayerColor::White {
            (0..8).collect()
        } else {
            (0..8).rev().collect()
        };

        for file in file_range {
            let square = Square::new(file, rank);
            let is_light_square = (file + rank) % 2 == 1;

            // Check for highlights
            let is_last_move = last_move.map(|(from, to)| square == from || square == to).unwrap_or(false);
            let is_highlighted = highlights.contains(&square);

            // Set background color
            if is_last_move {
                w.set_bg(Color::Brown); // Highlight last move
            } else if is_highlighted {
                w.set_bg(Color::Blue); // Highlight legal moves
            } else if is_light_square {
                w.set_bg(Color::LightGray); // Light square
            } else {
                w.set_bg(Color::Green); // Dark square
            }

            // Get piece and render
            match board.get(square) {
                Some(piece) => {
                    // Set piece color
                    match piece.color {
                        PieceColor::White => w.set_fg(Color::White),
                        PieceColor::Black => w.set_fg(Color::Black),
                    }
                    w.write_str(&format!(" {} ", piece.unicode()));
                }
                None => {
                    w.write_str("   ");
                }
            }
        }

        w.reset_color();
        w.set_fg(Color::DarkGray);
        w.write_str("|");
        w.set_fg(Color::LightCyan);
        w.writeln(&format!("  {}", rank + 1));
    }

    // Bottom border
    w.set_fg(Color::DarkGray);
    w.writeln("     +--------------------------+");

    // File labels at bottom
    w.write_str("      ");
    for f in &files {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!(" {} ", f));
    }
    w.writeln("");
    w.reset_color();

    w.flush()
}

/// Render the game header with title art
pub fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::White);
    w.bold();
    w.writeln("");
    w.writeln("   ██████╗██╗  ██╗███████╗███████╗███████╗");
    w.writeln("  ██╔════╝██║  ██║██╔════╝██╔════╝██╔════╝");
    w.writeln("  ██║     ███████║█████╗  ███████╗███████╗");
    w.writeln("  ██║     ██╔══██║██╔══╝  ╚════██║╚════██║");
    w.writeln("  ╚██████╗██║  ██║███████╗███████║███████║");
    w.writeln("   ╚═════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.writeln("  ─────────────────────────────────────────");
    w.reset_color();
}

/// Render game status bar
pub fn render_status_bar(game: &GameState, user_id: i64) -> String {
    let mut w = AnsiWriter::new();

    // Player info
    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(50));

    // White player
    let white_to_move = game.to_move() == PlayerColor::White;
    if white_to_move && game.status == GameStatus::InProgress {
        w.set_fg(Color::Yellow);
        w.write_str("  >> ");
    } else {
        w.write_str("     ");
    }
    w.set_fg(Color::White);
    w.write_str(&format!("{} (White)", game.white_handle));
    w.set_fg(Color::DarkGray);
    w.writeln(&format!(" [ELO: {}]", game.white_elo));

    // Black player
    let black_to_move = game.to_move() == PlayerColor::Black;
    if black_to_move && game.status == GameStatus::InProgress {
        w.set_fg(Color::Yellow);
        w.write_str("  >> ");
    } else {
        w.write_str("     ");
    }
    if let Some(ref handle) = game.black_handle {
        w.set_fg(Color::LightGray);
        w.write_str(&format!("{} (Black)", handle));
        if let Some(elo) = game.black_elo {
            w.set_fg(Color::DarkGray);
            w.writeln(&format!(" [ELO: {}]", elo));
        }
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("  Waiting for opponent...");
    }

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(50));

    // Game status
    match &game.status {
        GameStatus::WaitingForOpponent => {
            w.set_fg(Color::Yellow);
            w.writeln("  Status: Waiting for opponent to join");
        }
        GameStatus::InProgress => {
            let player_color = game.player_color(user_id);
            let is_your_turn = game.is_player_turn(user_id);

            if is_your_turn {
                w.set_fg(Color::LightGreen);
                w.writeln("  YOUR TURN - Enter your move");
            } else if player_color.is_some() {
                w.set_fg(Color::LightCyan);
                w.writeln("  Waiting for opponent's move...");
            } else {
                w.set_fg(Color::LightGray);
                let to_move = if game.to_move() == PlayerColor::White { "White" } else { "Black" };
                w.writeln(&format!("  {} to move", to_move));
            }
        }
        GameStatus::Checkmate { winner } => {
            let winner_name = match winner {
                PlayerColor::White => &game.white_handle,
                PlayerColor::Black => game.black_handle.as_deref().unwrap_or("Black"),
            };
            w.set_fg(Color::LightRed);
            w.bold();
            w.writeln(&format!("  CHECKMATE! {} wins!", winner_name));
            w.reset_color();
        }
        GameStatus::Stalemate => {
            w.set_fg(Color::Yellow);
            w.writeln("  STALEMATE - Draw!");
        }
        GameStatus::Resigned { winner } => {
            let winner_name = match winner {
                PlayerColor::White => &game.white_handle,
                PlayerColor::Black => game.black_handle.as_deref().unwrap_or("Black"),
            };
            w.set_fg(Color::LightRed);
            w.writeln(&format!("  Resignation - {} wins!", winner_name));
        }
        GameStatus::Timeout { winner } => {
            let winner_name = match winner {
                PlayerColor::White => &game.white_handle,
                PlayerColor::Black => game.black_handle.as_deref().unwrap_or("Black"),
            };
            w.set_fg(Color::LightRed);
            w.writeln(&format!("  Timeout - {} wins!", winner_name));
        }
        GameStatus::DrawAgreed => {
            w.set_fg(Color::Yellow);
            w.writeln("  Draw by agreement");
        }
        GameStatus::Draw50Moves => {
            w.set_fg(Color::Yellow);
            w.writeln("  Draw by 50-move rule");
        }
        GameStatus::DrawRepetition => {
            w.set_fg(Color::Yellow);
            w.writeln("  Draw by threefold repetition");
        }
        GameStatus::DrawInsufficientMaterial => {
            w.set_fg(Color::Yellow);
            w.writeln("  Draw by insufficient material");
        }
    }

    // Move count
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  Move: {}", game.board.fullmove_number));
    w.reset_color();

    w.flush()
}

/// Render the move history
pub fn render_move_history(game: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Move History:");
    w.reset_color();

    if game.moves.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No moves yet");
    } else {
        w.set_fg(Color::LightGray);

        // Show last 10 moves (5 move pairs)
        let start = game.moves.len().saturating_sub(10);
        for i in (start..game.moves.len()).step_by(2) {
            let move_num = i / 2 + 1;
            let white_move = &game.moves[i].move_notation;
            let black_move = if i + 1 < game.moves.len() {
                &game.moves[i + 1].move_notation
            } else {
                ""
            };

            w.set_fg(Color::DarkGray);
            w.write_str(&format!("  {:>3}. ", move_num));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<7}", white_move));
            if !black_move.is_empty() {
                w.set_fg(Color::LightGray);
                w.write_str(black_move);
            }
            w.writeln("");
        }
    }

    w.reset_color();
    w.flush()
}

/// Render the lobby screen header
pub fn render_lobby_header() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CHESS LOBBY");
    w.reset_color();
    w.writeln("");

    w.flush()
}

/// Render open games list
pub fn render_open_games(games: &[(i64, String, i32, String)], user_elo: i32) -> String {
    let mut w = AnsiWriter::new();

    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("  Open Games:");
    w.reset_color();
    w.writeln("");

    if games.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No open games. Create one with [N]!");
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("     #   Player                ELO      Type");
        w.writeln("    ───────────────────────────────────────────");
        w.reset_color();

        for (i, (_game_id, handle, elo, matchmaking_type)) in games.iter().enumerate() {
            let elo_diff = (*elo - user_elo).abs();
            let elo_color = if elo_diff < 100 {
                Color::LightGreen
            } else if elo_diff < 300 {
                Color::Yellow
            } else {
                Color::LightRed
            };

            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    [{:>2}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<20}", handle));
            w.set_fg(elo_color);
            w.write_str(&format!("{:>5}", elo));
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("    {}", matchmaking_type));
        }
    }

    w.writeln("");
    w.reset_color();
    w.flush()
}

/// Render active games list
pub fn render_active_games(games: &[(i64, String, bool, String)]) -> String {
    let mut w = AnsiWriter::new();

    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln("  Your Active Games:");
    w.reset_color();
    w.writeln("");

    if games.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No active games.");
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("     #   Opponent              Turn        Started");
        w.writeln("    ───────────────────────────────────────────────");
        w.reset_color();

        for (i, (_game_id, opponent, is_your_turn, created)) in games.iter().enumerate() {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    [{:>2}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<20}", opponent));

            if *is_your_turn {
                w.set_fg(Color::LightGreen);
                w.write_str("YOUR TURN   ");
            } else {
                w.set_fg(Color::DarkGray);
                w.write_str("waiting     ");
            }

            w.set_fg(Color::DarkGray);
            w.writeln(created);
        }
    }

    w.writeln("");
    w.reset_color();
    w.flush()
}

/// Render challenges list
pub fn render_challenges(incoming: &[(i64, String, i32)], outgoing: &[(i64, String)]) -> String {
    let mut w = AnsiWriter::new();

    if !incoming.is_empty() {
        w.set_fg(Color::LightMagenta);
        w.bold();
        w.writeln("  Incoming Challenges:");
        w.reset_color();

        for (i, (_game_id, handle, elo)) in incoming.iter().enumerate() {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    [{}] ", (b'A' + i as u8) as char));
            w.set_fg(Color::White);
            w.write_str(&format!("{} ", handle));
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("[ELO: {}]", elo));
        }
        w.writeln("");
    }

    if !outgoing.is_empty() {
        w.set_fg(Color::Yellow);
        w.writeln("  Outgoing Challenges:");
        w.reset_color();

        for (_game_id, handle) in outgoing {
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("    Waiting for {} to respond...", handle));
        }
        w.writeln("");
    }

    w.reset_color();
    w.flush()
}

/// Render ELO leaderboard
pub fn render_leaderboard(entries: &[(i64, String, i32, i32, i32)]) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w);
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CHESS LEADERBOARD");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln("    Rank  Player                 ELO     W    L");
    w.writeln("    ──────────────────────────────────────────────");
    w.reset_color();

    for (i, (_user_id, handle, elo, wins, losses)) in entries.iter().enumerate() {
        let rank = i + 1;
        let rank_color = match rank {
            1 => Color::Yellow,
            2 => Color::White,
            3 => Color::Brown,
            _ => Color::LightGray,
        };

        w.set_fg(rank_color);
        w.write_str(&format!("    {:>4}  ", rank));
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("{:<20}", handle));
        w.set_fg(Color::LightGreen);
        w.write_str(&format!("{:>5}", elo));
        w.set_fg(Color::White);
        w.write_str(&format!("   {:>3}", wins));
        w.set_fg(Color::LightRed);
        w.writeln(&format!("  {:>3}", losses));
    }

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
    fn test_render_board_no_panic() {
        let board = Board::new();
        let output = render_board(&board, PlayerColor::White);
        assert!(!output.is_empty());
        assert!(output.contains("a"));
        assert!(output.contains("h"));
    }

    #[test]
    fn test_render_board_black_perspective() {
        let board = Board::new();
        let output = render_board(&board, PlayerColor::Black);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_render_status_bar() {
        let game = GameState::new(1, "TestPlayer", 1200, super::super::state::MatchmakingMode::Open);
        let output = render_status_bar(&game, 1);
        assert!(output.contains("TestPlayer"));
    }
}
