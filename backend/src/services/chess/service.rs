//! Chess game service - session entry points and game coordination

#![allow(dead_code)]

use crate::games::chess::{
    Board, GameState, GameStatus, MatchmakingMode, PlayerColor,
    ChessFlow, ChessAction, GameScreen, Move,
    render::{
        render_header, render_board, render_board_with_highlights, render_status_bar,
        render_move_history, render_lobby_header, render_open_games, render_active_games,
        render_challenges, render_leaderboard,
    },
    moves::is_valid_move,
};
use crate::services::chess::db::ChessDb;
use crate::terminal::{AnsiWriter, Color};

/// Sentinel for session routing
pub const SENTINEL: &str = "__chess__";

/// Initialize chess session for a user
pub async fn start_game(db: &ChessDb, user_id: i64, handle: &str) -> Result<(ChessFlow, String), String> {
    // Get or create player rating
    let player = db.get_or_create_player(user_id, handle).await
        .map_err(|e| format!("Database error: {}", e))?;

    let mut flow = ChessFlow::new(user_id, handle, player.elo);

    // Load lobby data
    refresh_lobby_data(db, &mut flow).await?;

    let screen = render_screen(&flow);
    Ok((flow, screen))
}

/// Get or create player rating
pub async fn get_or_create_player_rating(db: &ChessDb, user_id: i64, handle: &str) -> Result<i32, String> {
    let player = db.get_or_create_player(user_id, handle).await
        .map_err(|e| format!("Database error: {}", e))?;
    Ok(player.elo)
}

/// Update ratings after a game ends
pub async fn update_ratings_after_game(
    db: &ChessDb,
    white_id: i64,
    black_id: i64,
    white_elo: i32,
    black_elo: i32,
    winner: Option<PlayerColor>,
) -> Result<(i32, i32), String> {
    let (white_score, black_score) = match winner {
        Some(PlayerColor::White) => (1.0, 0.0),
        Some(PlayerColor::Black) => (0.0, 1.0),
        None => (0.5, 0.5), // Draw
    };

    // Calculate new ELO ratings
    let k = 32; // K-factor
    let white_expected = 1.0 / (1.0 + 10f64.powf((black_elo as f64 - white_elo as f64) / 400.0));
    let black_expected = 1.0 - white_expected;

    let new_white_elo = white_elo + (k as f64 * (white_score - white_expected)) as i32;
    let new_black_elo = black_elo + (k as f64 * (black_score - black_expected)) as i32;

    // Ensure ELO doesn't go below 100
    let new_white_elo = new_white_elo.max(100);
    let new_black_elo = new_black_elo.max(100);

    // Update database
    let (white_win, white_loss, white_draw) = match winner {
        Some(PlayerColor::White) => (true, false, false),
        Some(PlayerColor::Black) => (false, true, false),
        None => (false, false, true),
    };
    let (black_win, black_loss, black_draw) = match winner {
        Some(PlayerColor::White) => (false, true, false),
        Some(PlayerColor::Black) => (true, false, false),
        None => (false, false, true),
    };

    db.update_player_rating(white_id, new_white_elo, white_win, white_loss, white_draw).await
        .map_err(|e| format!("Failed to update white rating: {}", e))?;
    db.update_player_rating(black_id, new_black_elo, black_win, black_loss, black_draw).await
        .map_err(|e| format!("Failed to update black rating: {}", e))?;

    Ok((new_white_elo, new_black_elo))
}

/// Handle a chess action and return the result
pub async fn handle_action(
    db: &ChessDb,
    flow: &mut ChessFlow,
    action: ChessAction,
) -> Result<Option<String>, String> {
    match action {
        ChessAction::Continue => Ok(None),

        ChessAction::Echo(s) => Ok(Some(s)),

        ChessAction::Render(s) => Ok(Some(s)),

        ChessAction::RefreshLobby => {
            refresh_lobby_data(db, flow).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::CreateGame { matchmaking } => {
            // Check concurrent game limit
            let active_count = db.count_active_games(flow.user_id).await
                .map_err(|e| format!("Database error: {}", e))?;
            let max_concurrent = db.get_max_concurrent().await
                .map_err(|e| format!("Database error: {}", e))?;

            if active_count >= max_concurrent {
                flow.last_message = Some(format!("You have reached the maximum of {} concurrent games.", max_concurrent));
                flow.screen = GameScreen::Lobby;
                return Ok(Some(render_screen(flow)));
            }

            let game = GameState::new(flow.user_id, &flow.handle, flow.elo, matchmaking);
            let _game_id = db.create_game(&game).await
                .map_err(|e| format!("Failed to create game: {}", e))?;

            flow.last_message = Some("Game created! Waiting for opponent...".to_string());
            flow.screen = GameScreen::Lobby;
            refresh_lobby_data(db, flow).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::JoinGame { game_id } => {
            // Check concurrent game limit
            let active_count = db.count_active_games(flow.user_id).await
                .map_err(|e| format!("Database error: {}", e))?;
            let max_concurrent = db.get_max_concurrent().await
                .map_err(|e| format!("Database error: {}", e))?;

            if active_count >= max_concurrent {
                flow.last_message = Some(format!("You have reached the maximum of {} concurrent games.", max_concurrent));
                return Ok(Some(render_screen(flow)));
            }

            db.join_game(game_id, flow.user_id, &flow.handle, flow.elo).await
                .map_err(|e| format!("Failed to join game: {}", e))?;

            // Load the game
            load_game_for_flow(db, flow, game_id).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::LoadGame { game_id } => {
            load_game_for_flow(db, flow, game_id).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::MakeMove { game_id, mv } => {
            // Load current game state
            let record = db.get_game(game_id).await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or("Game not found")?;

            // Parse the board
            let mut board = Board::from_fen(&record.fen)
                .ok_or("Failed to parse game state")?;

            // Validate and apply the move
            if !is_valid_move(&board, mv) {
                flow.last_message = Some("Invalid move!".to_string());
                return Ok(Some(render_screen(flow)));
            }

            let result = crate::games::chess::moves::make_move(&mut board, mv)
                .ok_or("Failed to apply move")?;

            // Get move number
            let moves = db.get_moves(game_id).await
                .map_err(|e| format!("Database error: {}", e))?;
            let move_number = moves.len() as i32 + 1;

            // Record the move
            db.record_move(game_id, move_number, &mv.to_algebraic(), &board.to_fen()).await
                .map_err(|e| format!("Failed to record move: {}", e))?;

            // Check for game end conditions
            if result.is_checkmate {
                let winner_id = if board.side_to_move == crate::games::chess::board::Color::White {
                    record.black_user_id
                } else {
                    Some(record.white_user_id)
                };
                db.update_game_status(game_id, "checkmate", winner_id).await
                    .map_err(|e| format!("Failed to update status: {}", e))?;

                // Update ratings
                if let Some(black_id) = record.black_user_id {
                    let winner_color = if winner_id == Some(record.white_user_id) {
                        Some(PlayerColor::White)
                    } else {
                        Some(PlayerColor::Black)
                    };
                    let _ = update_ratings_after_game(
                        db,
                        record.white_user_id,
                        black_id,
                        record.white_elo,
                        record.black_elo.unwrap_or(1200),
                        winner_color,
                    ).await;
                }
            } else if result.is_stalemate {
                db.update_game_status(game_id, "stalemate", None).await
                    .map_err(|e| format!("Failed to update status: {}", e))?;

                // Update ratings for draw
                if let Some(black_id) = record.black_user_id {
                    let _ = update_ratings_after_game(
                        db,
                        record.white_user_id,
                        black_id,
                        record.white_elo,
                        record.black_elo.unwrap_or(1200),
                        None,
                    ).await;
                }
            } else if board.halfmove_clock >= 100 {
                db.update_game_status(game_id, "draw_50_moves", None).await
                    .map_err(|e| format!("Failed to update status: {}", e))?;
            }

            // Reload the game
            load_game_for_flow(db, flow, game_id).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::Resign { game_id } => {
            let record = db.get_game(game_id).await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or("Game not found")?;

            let winner_id = if flow.user_id == record.white_user_id {
                record.black_user_id
            } else {
                Some(record.white_user_id)
            };

            db.update_game_status(game_id, "resigned", winner_id).await
                .map_err(|e| format!("Failed to update status: {}", e))?;

            // Update ratings
            if let Some(black_id) = record.black_user_id {
                let winner_color = if winner_id == Some(record.white_user_id) {
                    Some(PlayerColor::White)
                } else {
                    Some(PlayerColor::Black)
                };
                let _ = update_ratings_after_game(
                    db,
                    record.white_user_id,
                    black_id,
                    record.white_elo,
                    record.black_elo.unwrap_or(1200),
                    winner_color,
                ).await;
            }

            flow.last_message = Some("You resigned the game.".to_string());
            flow.screen = GameScreen::Lobby;
            refresh_lobby_data(db, flow).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::OfferDraw { game_id } => {
            let record = db.get_game(game_id).await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or("Game not found")?;

            let by_white = flow.user_id == record.white_user_id;
            db.set_draw_offer(game_id, by_white).await
                .map_err(|e| format!("Failed to offer draw: {}", e))?;

            flow.last_message = Some("Draw offered to opponent.".to_string());
            load_game_for_flow(db, flow, game_id).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::AcceptDraw { game_id } => {
            let record = db.get_game(game_id).await
                .map_err(|e| format!("Database error: {}", e))?
                .ok_or("Game not found")?;

            db.update_game_status(game_id, "draw_agreed", None).await
                .map_err(|e| format!("Failed to update status: {}", e))?;

            // Update ratings for draw
            if let Some(black_id) = record.black_user_id {
                let _ = update_ratings_after_game(
                    db,
                    record.white_user_id,
                    black_id,
                    record.white_elo,
                    record.black_elo.unwrap_or(1200),
                    None,
                ).await;
            }

            flow.last_message = Some("Draw accepted.".to_string());
            flow.screen = GameScreen::Lobby;
            refresh_lobby_data(db, flow).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::LoadLeaderboard => {
            let entries = db.get_leaderboard(20).await
                .map_err(|e| format!("Database error: {}", e))?;
            flow.set_leaderboard(entries);
            Ok(Some(render_screen(flow)))
        }

        ChessAction::ChallengePlayer { handle } => {
            // TODO: Look up user by handle in main BBS database
            // For now, just show an error
            flow.last_message = Some(format!("Challenge sent to {}!", handle));
            flow.screen = GameScreen::Lobby;
            refresh_lobby_data(db, flow).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::SaveGame => {
            // Chess games are auto-saved to database on each move
            Ok(None)
        }

        ChessAction::GameOver { game_id: _ } => {
            flow.current_game = None;
            flow.screen = GameScreen::Lobby;
            refresh_lobby_data(db, flow).await?;
            Ok(Some(render_screen(flow)))
        }

        ChessAction::Quit => {
            // Return to main menu
            Ok(None)
        }
    }
}

/// Refresh lobby data from database
async fn refresh_lobby_data(db: &ChessDb, flow: &mut ChessFlow) -> Result<(), String> {
    let open_games = db.get_open_games(flow.user_id).await
        .map_err(|e| format!("Database error: {}", e))?;
    let active_games = db.get_active_games(flow.user_id).await
        .map_err(|e| format!("Database error: {}", e))?;
    let incoming = db.get_incoming_challenges(flow.user_id).await
        .map_err(|e| format!("Database error: {}", e))?;
    let outgoing = db.get_outgoing_challenges(flow.user_id).await
        .map_err(|e| format!("Database error: {}", e))?;

    flow.update_lobby(open_games, active_games, incoming, outgoing);
    Ok(())
}

/// Load a game into the flow
async fn load_game_for_flow(db: &ChessDb, flow: &mut ChessFlow, game_id: i64) -> Result<(), String> {
    let record = db.get_game(game_id).await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Game not found")?;

    let moves = db.get_moves(game_id).await
        .map_err(|e| format!("Database error: {}", e))?;

    let board = Board::from_fen(&record.fen)
        .ok_or("Failed to parse game state")?;

    // Determine matchmaking mode from stored type
    let matchmaking = if record.matchmaking_type.starts_with("elo:") {
        let parts: Vec<&str> = record.matchmaking_type[4..].split('-').collect();
        if parts.len() == 2 {
            let min = parts[0].parse().unwrap_or(0);
            let max = parts[1].parse().unwrap_or(3000);
            MatchmakingMode::EloMatched { min_elo: min, max_elo: max }
        } else {
            MatchmakingMode::Open
        }
    } else if record.matchmaking_type.starts_with("challenge:") {
        let parts: Vec<&str> = record.matchmaking_type[10..].split(':').collect();
        if parts.len() >= 2 {
            let target_id = parts[0].parse().unwrap_or(0);
            let target_handle = parts[1].to_string();
            MatchmakingMode::Challenge { target_user_id: target_id, target_handle }
        } else {
            MatchmakingMode::Open
        }
    } else {
        MatchmakingMode::Open
    };

    // Determine game status
    let status = match record.status.as_str() {
        "waiting" => GameStatus::WaitingForOpponent,
        "in_progress" => GameStatus::InProgress,
        "checkmate" => {
            // Determine winner from whose turn it is (current player lost)
            let winner = if board.side_to_move == crate::games::chess::board::Color::White {
                PlayerColor::Black
            } else {
                PlayerColor::White
            };
            GameStatus::Checkmate { winner }
        }
        "stalemate" => GameStatus::Stalemate,
        "resigned" => {
            // Winner was stored in database
            let winner = if Some(record.white_user_id) == record.black_user_id {
                PlayerColor::Black
            } else {
                PlayerColor::White
            };
            GameStatus::Resigned { winner }
        }
        "timeout" => {
            let winner = PlayerColor::White; // Would need to check winner_user_id
            GameStatus::Timeout { winner }
        }
        "draw_agreed" => GameStatus::DrawAgreed,
        "draw_50_moves" => GameStatus::Draw50Moves,
        _ => GameStatus::InProgress,
    };

    let game = GameState {
        game_id: Some(game_id),
        board,
        white_user_id: record.white_user_id,
        white_handle: record.white_handle,
        white_elo: record.white_elo,
        black_user_id: record.black_user_id,
        black_handle: record.black_handle,
        black_elo: record.black_elo,
        status,
        moves: moves.iter().map(|(_, notation, fen)| {
            crate::games::chess::state::MoveRecord {
                move_notation: notation.clone(),
                fen_after: fen.clone(),
                timestamp: String::new(),
            }
        }).collect(),
        matchmaking,
        last_move_time: record.last_move_time,
        created_at: record.created_at,
        white_draw_offer: false,
        black_draw_offer: false,
    };

    flow.set_current_game(game);
    Ok(())
}

/// Render the current screen
pub fn render_screen(flow: &ChessFlow) -> String {
    let mut w = AnsiWriter::new();

    match flow.current_screen() {
        GameScreen::Lobby => {
            w.write_str(&render_lobby_header());

            // Show last message if any
            if let Some(ref msg) = flow.last_message {
                w.set_fg(Color::Yellow);
                w.writeln(&format!("  >> {} <<", msg));
                w.reset_color();
                w.writeln("");
            }

            // Show challenges
            w.write_str(&render_challenges(&flow.incoming_challenges, &flow.outgoing_challenges));

            // Show active games
            w.write_str(&render_active_games(&flow.active_games));

            // Show open games
            w.write_str(&render_open_games(&flow.open_games, flow.elo));

            // Menu options
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [N] ");
            w.set_fg(Color::White);
            w.writeln("New Game");

            w.set_fg(Color::LightCyan);
            w.write_str("    [L] ");
            w.set_fg(Color::White);
            w.writeln("Leaderboard");

            w.set_fg(Color::LightCyan);
            w.write_str("    [R] ");
            w.set_fg(Color::White);
            w.writeln("Refresh");

            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Quit to Main Menu");

            w.writeln("");
            w.set_fg(Color::DarkGray);
            w.writeln("  Enter game # to join, G# to continue active game, or letter to accept challenge");
            w.reset_color();
            w.writeln("");
            w.write_str("  > ");
        }

        GameScreen::CreateGame => {
            render_header(&mut w);

            w.set_fg(Color::Yellow);
            w.bold();
            w.writeln("  CREATE NEW GAME");
            w.reset_color();
            w.writeln("");

            w.set_fg(Color::LightCyan);
            w.write_str("    [1] ");
            w.set_fg(Color::White);
            w.writeln("Open Game - Anyone can join");

            w.set_fg(Color::LightCyan);
            w.write_str("    [2] ");
            w.set_fg(Color::White);
            w.writeln("ELO Matched - Find similar skill opponent");

            w.set_fg(Color::LightCyan);
            w.write_str("    [3] ");
            w.set_fg(Color::White);
            w.writeln("Challenge Player - Direct challenge");

            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Back to Lobby");

            w.reset_color();
            w.writeln("");
            w.write_str("  > ");
        }

        GameScreen::SelectEloRange { min, max, editing_min } => {
            render_header(&mut w);

            w.set_fg(Color::Yellow);
            w.bold();
            w.writeln("  ELO RANGE SELECTION");
            w.reset_color();
            w.writeln("");

            w.set_fg(Color::White);
            w.writeln(&format!("  Your ELO: {}", flow.elo));
            w.writeln("");

            if *editing_min {
                w.set_fg(Color::LightGreen);
                w.writeln(&format!("  Enter minimum ELO [{}]: ", min));
            } else {
                w.set_fg(Color::LightGreen);
                w.writeln(&format!("  Enter maximum ELO [{}]: ", max));
            }

            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Back");

            w.reset_color();
            w.writeln("");
            w.write_str("  > ");
        }

        GameScreen::ChallengePlayer { input } => {
            render_header(&mut w);

            w.set_fg(Color::Yellow);
            w.bold();
            w.writeln("  CHALLENGE PLAYER");
            w.reset_color();
            w.writeln("");

            w.set_fg(Color::White);
            w.writeln("  Enter the handle of the player to challenge:");
            w.writeln("");
            w.write_str(&format!("  > {}", input));
        }

        GameScreen::InGame { game_id: _ } => {
            if let Some(ref game) = flow.current_game {
                render_header(&mut w);

                // Determine perspective
                let perspective = game.player_color(flow.user_id).unwrap_or(PlayerColor::White);

                // Get last move for highlighting
                let last_move = if !game.moves.is_empty() {
                    let last = &game.moves[game.moves.len() - 1];
                    Move::from_algebraic(&last.move_notation).map(|m| (m.from, m.to))
                } else {
                    None
                };

                // Render board
                w.write_str(&render_board_with_highlights(&game.board, perspective, last_move, &[]));

                // Render status
                w.write_str(&render_status_bar(game, flow.user_id));

                // Render move history
                w.write_str(&render_move_history(game));

                // Show last message
                if let Some(ref msg) = flow.last_message {
                    w.writeln("");
                    w.set_fg(Color::Yellow);
                    w.writeln(&format!("  >> {} <<", msg));
                    w.reset_color();
                }

                // Menu options (only show relevant ones)
                w.writeln("");

                if game.is_player_turn(flow.user_id) {
                    w.set_fg(Color::LightGreen);
                    w.writeln("  Enter move (e.g., e2e4) or:");
                    w.reset_color();
                }

                if game.status == GameStatus::InProgress {
                    w.set_fg(Color::LightCyan);
                    w.write_str("    [D] ");
                    w.set_fg(Color::White);

                    // Check if opponent offered draw
                    let opponent_offered = match game.player_color(flow.user_id) {
                        Some(PlayerColor::White) => game.black_draw_offer,
                        Some(PlayerColor::Black) => game.white_draw_offer,
                        None => false,
                    };
                    if opponent_offered {
                        w.writeln("Accept Draw");
                    } else {
                        w.writeln("Offer Draw");
                    }

                    w.set_fg(Color::LightCyan);
                    w.write_str("    [R] ");
                    w.set_fg(Color::White);
                    w.writeln("Resign");
                }

                w.set_fg(Color::LightCyan);
                w.write_str("    [Q] ");
                w.set_fg(Color::White);
                w.writeln("Back to Lobby");

                w.reset_color();
                w.writeln("");
                w.write_str("  > ");
            }
        }

        GameScreen::EnterMove { game_id: _, input } => {
            if let Some(ref game) = flow.current_game {
                render_header(&mut w);

                let perspective = game.player_color(flow.user_id).unwrap_or(PlayerColor::White);
                w.write_str(&render_board(&game.board, perspective));

                w.writeln("");
                w.set_fg(Color::LightGreen);
                w.writeln("  Enter your move (e.g., e2e4, e7e8q for promotion):");
                w.writeln("");
                w.set_fg(Color::White);
                w.write_str(&format!("  > {}", input));
            }
        }

        GameScreen::ConfirmResign { game_id: _ } => {
            render_header(&mut w);

            w.set_fg(Color::LightRed);
            w.bold();
            w.writeln("");
            w.writeln("  RESIGN GAME?");
            w.reset_color();
            w.writeln("");
            w.set_fg(Color::White);
            w.writeln("  Are you sure you want to resign?");
            w.writeln("  Your opponent will win the game.");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.writeln("  [Y] Yes, Resign   [N] No, Cancel");
            w.reset_color();
            w.writeln("");
            w.write_str("  > ");
        }

        GameScreen::Leaderboard => {
            w.write_str(&render_leaderboard(&flow.leaderboard));
        }

        GameScreen::History => {
            render_header(&mut w);
            w.set_fg(Color::Yellow);
            w.writeln("  GAME HISTORY");
            w.reset_color();
            w.writeln("");
            w.set_fg(Color::DarkGray);
            w.writeln("  Press any key to return...");
        }

        GameScreen::ConfirmQuit => {
            render_header(&mut w);

            w.set_fg(Color::Yellow);
            w.bold();
            w.writeln("");
            w.writeln("  QUIT CHESS?");
            w.reset_color();
            w.writeln("");
            w.set_fg(Color::White);
            w.writeln("  Your active games will be saved.");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.writeln("  [Y] Yes, Quit   [N] No, Stay");
            w.reset_color();
            w.writeln("");
            w.write_str("  > ");
        }
    }

    w.flush()
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_elo_calculation() {
        // Equal players, white wins
        let white_elo = 1200;
        let black_elo = 1200;

        let k = 32;
        let white_expected = 1.0 / (1.0 + 10f64.powf((black_elo as f64 - white_elo as f64) / 400.0));

        // White wins (score = 1)
        let new_white_elo = white_elo + (k as f64 * (1.0 - white_expected)) as i32;
        let new_black_elo = black_elo + (k as f64 * (0.0 - (1.0 - white_expected))) as i32;

        // White should gain ~16, black should lose ~16
        assert!(new_white_elo > white_elo);
        assert!(new_black_elo < black_elo);
        assert!((new_white_elo - white_elo).abs() < 20);
    }
}
