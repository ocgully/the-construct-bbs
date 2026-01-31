//! Comprehensive unit tests for the Chess module
//!
//! These tests cover:
//! - Board representation and FEN parsing
//! - Move generation and validation
//! - Special moves (castling, en passant, promotion)
//! - Game state management
//! - Check, checkmate, and stalemate detection

#[cfg(test)]
mod board_tests {
    use crate::games::chess::board::*;

    #[test]
    fn test_square_creation() {
        let sq = Square::new(4, 3);
        assert_eq!(sq.file(), 4);
        assert_eq!(sq.rank(), 3);
    }

    #[test]
    fn test_square_algebraic_parsing() {
        assert_eq!(Square::from_algebraic("a1"), Some(Square::new(0, 0)));
        assert_eq!(Square::from_algebraic("h8"), Some(Square::new(7, 7)));
        assert_eq!(Square::from_algebraic("e4"), Some(Square::new(4, 3)));
        assert_eq!(Square::from_algebraic("d5"), Some(Square::new(3, 4)));

        // Invalid inputs
        assert_eq!(Square::from_algebraic("i9"), None);
        assert_eq!(Square::from_algebraic("a"), None);
        assert_eq!(Square::from_algebraic("a0"), None);
        assert_eq!(Square::from_algebraic("a9"), None);
    }

    #[test]
    fn test_square_to_algebraic() {
        assert_eq!(Square::new(0, 0).to_algebraic(), "a1");
        assert_eq!(Square::new(7, 7).to_algebraic(), "h8");
        assert_eq!(Square::new(4, 3).to_algebraic(), "e4");
    }

    #[test]
    fn test_piece_symbols() {
        let white_king = Piece::new(PieceType::King, Color::White);
        let black_queen = Piece::new(PieceType::Queen, Color::Black);

        assert_eq!(white_king.symbol(), 'K');
        assert_eq!(black_queen.symbol(), 'q');
    }

    #[test]
    fn test_board_initial_position() {
        let board = Board::new();

        // White pieces on ranks 1-2
        assert_eq!(board.get(Square::new(0, 0)).unwrap().piece_type, PieceType::Rook);
        assert_eq!(board.get(Square::new(1, 0)).unwrap().piece_type, PieceType::Knight);
        assert_eq!(board.get(Square::new(2, 0)).unwrap().piece_type, PieceType::Bishop);
        assert_eq!(board.get(Square::new(3, 0)).unwrap().piece_type, PieceType::Queen);
        assert_eq!(board.get(Square::new(4, 0)).unwrap().piece_type, PieceType::King);

        for file in 0..8 {
            assert_eq!(board.get(Square::new(file, 1)).unwrap().piece_type, PieceType::Pawn);
            assert_eq!(board.get(Square::new(file, 1)).unwrap().color, Color::White);
        }

        // Empty squares in middle
        for rank in 2..6 {
            for file in 0..8 {
                assert!(board.get(Square::new(file, rank)).is_none());
            }
        }

        // Black pieces on ranks 7-8
        for file in 0..8 {
            assert_eq!(board.get(Square::new(file, 6)).unwrap().piece_type, PieceType::Pawn);
            assert_eq!(board.get(Square::new(file, 6)).unwrap().color, Color::Black);
        }
    }

    #[test]
    fn test_fen_parsing() {
        // Starting position
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(board.to_fen(), fen);

        // After 1.e4
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(board.side_to_move, Color::Black);
        assert_eq!(board.en_passant, Some(Square::new(4, 2)));
    }

    #[test]
    fn test_fen_roundtrip() {
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
            "8/8/8/8/8/8/8/4K2k w - - 0 1",
        ];

        for fen in positions {
            let board = Board::from_fen(fen).unwrap();
            assert_eq!(board.to_fen(), fen, "FEN roundtrip failed for: {}", fen);
        }
    }

    #[test]
    fn test_castling_rights() {
        let mut rights = CastlingRights::new();
        assert!(rights.can_castle(Color::White, true));
        assert!(rights.can_castle(Color::White, false));
        assert!(rights.can_castle(Color::Black, true));
        assert!(rights.can_castle(Color::Black, false));

        rights.remove_kingside(Color::White);
        assert!(!rights.can_castle(Color::White, true));
        assert!(rights.can_castle(Color::White, false));

        rights.remove_for_color(Color::Black);
        assert!(!rights.can_castle(Color::Black, true));
        assert!(!rights.can_castle(Color::Black, false));
    }
}

#[cfg(test)]
mod move_tests {
    use crate::games::chess::board::*;
    use crate::games::chess::moves::*;

    #[test]
    fn test_move_parsing() {
        let mv = Move::from_algebraic("e2e4").unwrap();
        assert_eq!(mv.from, Square::new(4, 1));
        assert_eq!(mv.to, Square::new(4, 3));
        assert!(mv.promotion.is_none());

        let mv = Move::from_algebraic("e7e8q").unwrap();
        assert_eq!(mv.promotion, Some(PieceType::Queen));

        // Invalid moves
        assert!(Move::from_algebraic("e2").is_none());
        assert!(Move::from_algebraic("xyz").is_none());
    }

    #[test]
    fn test_legal_moves_starting_position() {
        let board = Board::new();
        let moves = get_legal_moves(&board);

        // 16 pawn moves (each pawn can move 1 or 2 squares)
        // 4 knight moves (2 per knight)
        // = 20 total moves
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_pawn_moves() {
        // Test pawn single and double push
        let board = Board::new();
        let moves = get_legal_moves(&board);

        // e2-e3 should be valid
        assert!(moves.iter().any(|m| m.from == Square::new(4, 1) && m.to == Square::new(4, 2)));
        // e2-e4 should be valid
        assert!(moves.iter().any(|m| m.from == Square::new(4, 1) && m.to == Square::new(4, 3)));
    }

    #[test]
    fn test_pawn_capture() {
        // White pawn on e4, black pawn on d5
        let board = Board::from_fen("8/8/8/3p4/4P3/8/8/4K2k w - - 0 1").unwrap();
        let moves = get_legal_moves(&board);

        // e4xd5 should be valid
        assert!(moves.iter().any(|m|
            m.from == Square::new(4, 3) && m.to == Square::new(3, 4)
        ));
    }

    #[test]
    fn test_en_passant() {
        // Position after 1.e4 d5 2.e5 f5 - white can play exf6 e.p.
        let board = Board::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3").unwrap();
        let moves = get_legal_moves(&board);

        // e5xf6 e.p. should be valid
        assert!(moves.iter().any(|m|
            m.from == Square::new(4, 4) && m.to == Square::new(5, 5)
        ));
    }

    #[test]
    fn test_knight_moves() {
        let board = Board::from_fen("8/8/8/8/3N4/8/8/4K2k w - - 0 1").unwrap();
        let moves = get_legal_moves(&board);

        // Knight on d4 should have 8 possible moves
        let knight_moves: Vec<_> = moves.iter()
            .filter(|m| m.from == Square::new(3, 3))
            .collect();
        assert_eq!(knight_moves.len(), 8);
    }

    #[test]
    fn test_check_detection() {
        // White king in check from black rook
        let board = Board::from_fen("4k3/8/8/8/8/8/8/r3K3 w - - 0 1").unwrap();
        assert!(is_in_check(&board, Color::White));

        // Not in check
        let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(!is_in_check(&board, Color::White));
    }

    #[test]
    fn test_checkmate_detection() {
        // Back rank mate
        let _board = Board::from_fen("6k1/5ppp/8/8/8/8/8/r3K3 w - - 0 1").unwrap();
        // Actually this isn't checkmate because white can capture the rook
        // Let's use a proper checkmate
        let board = Board::from_fen("r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4").unwrap();
        assert!(is_checkmate(&board));
    }

    #[test]
    fn test_stalemate_detection() {
        // King trapped with no legal moves but not in check
        // White king on b6, white queen on c7, black king on a8
        // Black king cannot move: a7 attacked by Kb6 and Qc7, b8 attacked by Qc7, b7 attacked by Kb6 and Qc7
        let board = Board::from_fen("k7/2Q5/1K6/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(!is_in_check(&board, Color::Black));
        assert!(is_stalemate(&board));
    }

    #[test]
    fn test_castling() {
        // Both sides can castle
        let board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let moves = get_legal_moves(&board);

        // White kingside castle (e1-g1)
        assert!(moves.iter().any(|m|
            m.from == Square::new(4, 0) && m.to == Square::new(6, 0)
        ));
        // White queenside castle (e1-c1)
        assert!(moves.iter().any(|m|
            m.from == Square::new(4, 0) && m.to == Square::new(2, 0)
        ));
    }

    #[test]
    fn test_cannot_castle_out_of_check() {
        // King in check from white queen on e4 (e7 pawn removed so queen can see e8)
        let board = Board::from_fen("r3k2r/pppp1ppp/8/8/4Q3/8/PPPP1PPP/R3K2R b KQkq - 0 1").unwrap();

        // Verify setup: black king should be in check
        assert!(is_in_check(&board, Color::Black), "Black king should be in check from Qe4");

        let moves = get_legal_moves(&board);

        // Black cannot castle while in check
        assert!(!moves.iter().any(|m|
            m.from == Square::new(4, 7) && (m.to == Square::new(6, 7) || m.to == Square::new(2, 7))
        ));
    }

    #[test]
    fn test_promotion() {
        // White pawn about to promote
        let board = Board::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();
        let moves = get_legal_moves(&board);

        // Should have 4 promotion options
        let promo_moves: Vec<_> = moves.iter()
            .filter(|m| m.from == Square::new(0, 6) && m.promotion.is_some())
            .collect();
        assert_eq!(promo_moves.len(), 4);
    }

    #[test]
    fn test_make_move_updates_board() {
        let mut board = Board::new();

        let mv = Move::from_algebraic("e2e4").unwrap();
        let result = make_move(&mut board, mv).unwrap();

        assert!(!result.is_capture);
        assert!(!result.is_check);
        assert!(board.get(Square::new(4, 1)).is_none()); // e2 empty
        assert!(board.get(Square::new(4, 3)).is_some()); // e4 has pawn
        assert_eq!(board.side_to_move, Color::Black);
    }

    #[test]
    fn test_make_move_castling() {
        let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();

        let mv = Move::from_algebraic("e1g1").unwrap();
        let result = make_move(&mut board, mv).unwrap();

        assert!(result.is_castling);
        // King should be on g1
        assert_eq!(board.get(Square::new(6, 0)).unwrap().piece_type, PieceType::King);
        // Rook should be on f1
        assert_eq!(board.get(Square::new(5, 0)).unwrap().piece_type, PieceType::Rook);
    }

    #[test]
    fn test_make_move_en_passant() {
        let mut board = Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3").unwrap();

        let mv = Move::from_algebraic("e5d6").unwrap();
        let result = make_move(&mut board, mv).unwrap();

        assert!(result.is_en_passant);
        assert!(result.is_capture);
        // Captured pawn should be gone
        assert!(board.get(Square::new(3, 4)).is_none());
    }

    #[test]
    fn test_make_move_promotion() {
        let mut board = Board::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();

        let mv = Move::from_algebraic("a7a8q").unwrap();
        let result = make_move(&mut board, mv).unwrap();

        assert!(result.is_promotion);
        assert_eq!(board.get(Square::new(0, 7)).unwrap().piece_type, PieceType::Queen);
    }
}

#[cfg(test)]
mod state_tests {
    use crate::games::chess::state::*;
    use crate::games::chess::moves::Move;

    #[test]
    fn test_game_creation() {
        let game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);

        assert_eq!(game.white_user_id, 1);
        assert_eq!(game.white_handle, "Player1");
        assert_eq!(game.white_elo, 1200);
        assert!(game.black_user_id.is_none());
        assert_eq!(game.status, GameStatus::WaitingForOpponent);
    }

    #[test]
    fn test_game_join() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        assert_eq!(game.black_user_id, Some(2));
        assert_eq!(game.black_handle.as_deref(), Some("Player2"));
        assert_eq!(game.black_elo, Some(1300));
        assert_eq!(game.status, GameStatus::InProgress);
    }

    #[test]
    fn test_player_color() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        assert_eq!(game.player_color(1), Some(PlayerColor::White));
        assert_eq!(game.player_color(2), Some(PlayerColor::Black));
        assert_eq!(game.player_color(3), None);
    }

    #[test]
    fn test_is_player_turn() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        // White's turn initially
        assert!(game.is_player_turn(1));
        assert!(!game.is_player_turn(2));

        // Make a move
        let mv = Move::from_algebraic("e2e4").unwrap();
        game.make_move(mv).unwrap();

        // Now black's turn
        assert!(!game.is_player_turn(1));
        assert!(game.is_player_turn(2));
    }

    #[test]
    fn test_resign() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        game.resign(1).unwrap();

        assert_eq!(game.status, GameStatus::Resigned { winner: PlayerColor::Black });
    }

    #[test]
    fn test_draw_offer_and_accept() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        // White offers draw
        let accepted = game.offer_draw(1).unwrap();
        assert!(!accepted);
        assert!(game.white_draw_offer);
        assert!(!game.black_draw_offer);

        // Black accepts
        game.accept_draw(2).unwrap();
        assert_eq!(game.status, GameStatus::DrawAgreed);
    }

    #[test]
    fn test_move_list() {
        let mut game = GameState::new(1, "Player1", 1200, MatchmakingMode::Open);
        game.join(2, "Player2", 1300);

        game.make_move(Move::from_algebraic("e2e4").unwrap()).unwrap();
        game.make_move(Move::from_algebraic("e7e5").unwrap()).unwrap();
        game.make_move(Move::from_algebraic("g1f3").unwrap()).unwrap();

        let move_list = game.get_move_list();
        assert!(move_list.contains("e2e4"));
        assert!(move_list.contains("e7e5"));
        assert!(move_list.contains("g1f3"));
    }

    #[test]
    fn test_matchmaking_modes() {
        // Open game
        let game = GameState::new(1, "P1", 1200, MatchmakingMode::Open);
        assert!(matches!(game.matchmaking, MatchmakingMode::Open));

        // ELO matched
        let game = GameState::new(1, "P1", 1200, MatchmakingMode::EloMatched { min_elo: 1000, max_elo: 1400 });
        if let MatchmakingMode::EloMatched { min_elo, max_elo } = game.matchmaking {
            assert_eq!(min_elo, 1000);
            assert_eq!(max_elo, 1400);
        } else {
            panic!("Expected EloMatched");
        }

        // Challenge
        let game = GameState::new(1, "P1", 1200, MatchmakingMode::Challenge {
            target_user_id: 42,
            target_handle: "Target".to_string(),
        });
        if let MatchmakingMode::Challenge { target_user_id, target_handle } = &game.matchmaking {
            assert_eq!(*target_user_id, 42);
            assert_eq!(target_handle, "Target");
        } else {
            panic!("Expected Challenge");
        }
    }

    #[test]
    fn test_game_status_is_game_over() {
        assert!(!GameStatus::WaitingForOpponent.is_game_over());
        assert!(!GameStatus::InProgress.is_game_over());
        assert!(GameStatus::Checkmate { winner: PlayerColor::White }.is_game_over());
        assert!(GameStatus::Stalemate.is_game_over());
        assert!(GameStatus::Resigned { winner: PlayerColor::Black }.is_game_over());
        assert!(GameStatus::DrawAgreed.is_game_over());
    }

    #[test]
    fn test_game_status_winner() {
        assert_eq!(GameStatus::InProgress.winner(), None);
        assert_eq!(GameStatus::Checkmate { winner: PlayerColor::White }.winner(), Some(PlayerColor::White));
        assert_eq!(GameStatus::Resigned { winner: PlayerColor::Black }.winner(), Some(PlayerColor::Black));
        assert_eq!(GameStatus::Stalemate.winner(), None);
        assert_eq!(GameStatus::DrawAgreed.winner(), None);
    }
}

#[cfg(test)]
mod screen_tests {
    use crate::games::chess::screen::*;
    use crate::games::chess::state::MatchmakingMode;

    #[test]
    fn test_flow_creation() {
        let flow = ChessFlow::new(1, "TestPlayer", 1200);

        assert_eq!(flow.user_id, 1);
        assert_eq!(flow.handle, "TestPlayer");
        assert_eq!(flow.elo, 1200);
        assert!(matches!(flow.current_screen(), GameScreen::Lobby));
    }

    #[test]
    fn test_lobby_to_create_game() {
        let mut flow = ChessFlow::new(1, "Test", 1200);

        let action = flow.handle_char('N');
        assert!(matches!(action, ChessAction::RefreshLobby));
        assert!(matches!(flow.current_screen(), GameScreen::CreateGame));
    }

    #[test]
    fn test_create_open_game() {
        let mut flow = ChessFlow::new(1, "Test", 1200);
        flow.screen = GameScreen::CreateGame;

        let action = flow.handle_char('1');
        assert!(matches!(action, ChessAction::CreateGame { matchmaking: MatchmakingMode::Open }));
    }

    #[test]
    fn test_back_navigation() {
        let mut flow = ChessFlow::new(1, "Test", 1200);
        flow.screen = GameScreen::CreateGame;

        let action = flow.handle_char('Q');
        assert!(matches!(action, ChessAction::RefreshLobby));
        assert!(matches!(flow.current_screen(), GameScreen::Lobby));
    }

    #[test]
    fn test_join_open_game() {
        let mut flow = ChessFlow::new(1, "Test", 1200);
        flow.open_games = vec![(42, "Opponent".to_string(), 1300, "Open".to_string())];

        let action = flow.handle_char('1');
        assert!(matches!(action, ChessAction::JoinGame { game_id: 42 }));
    }

    #[test]
    fn test_continue_active_game() {
        let mut flow = ChessFlow::new(1, "Test", 1200);
        flow.active_games = vec![(99, "Opponent".to_string(), true, "2024-01-01".to_string())];

        // Press 'G' then '1' for G1
        flow.handle_char('G');
        let action = flow.handle_char('1');
        // The action should load the game
        assert!(matches!(action, ChessAction::Continue) || matches!(action, ChessAction::LoadGame { .. }));
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = ChessFlow::new(1, "Test", 1200);

        let _action = flow.handle_char('Q');
        assert!(matches!(flow.current_screen(), GameScreen::ConfirmQuit));

        let action = flow.handle_char('Y');
        assert!(matches!(action, ChessAction::Quit));
    }

    #[test]
    fn test_leaderboard_navigation() {
        let mut flow = ChessFlow::new(1, "Test", 1200);

        let action = flow.handle_char('L');
        assert!(matches!(action, ChessAction::LoadLeaderboard));
        assert!(matches!(flow.current_screen(), GameScreen::Leaderboard));
    }
}

#[cfg(test)]
mod render_tests {
    use crate::games::chess::board::Board;
    use crate::games::chess::state::{GameState, PlayerColor, MatchmakingMode};
    use crate::games::chess::render::*;

    #[test]
    fn test_render_board_contains_files() {
        let board = Board::new();
        let output = render_board(&board, PlayerColor::White);

        // Should contain file labels
        assert!(output.contains('a'));
        assert!(output.contains('h'));
    }

    #[test]
    fn test_render_board_contains_ranks() {
        let board = Board::new();
        let output = render_board(&board, PlayerColor::White);

        // Should contain rank labels
        assert!(output.contains('1'));
        assert!(output.contains('8'));
    }

    #[test]
    fn test_render_board_different_perspectives() {
        let board = Board::new();
        let white_view = render_board(&board, PlayerColor::White);
        let black_view = render_board(&board, PlayerColor::Black);

        // Both should render without panic
        assert!(!white_view.is_empty());
        assert!(!black_view.is_empty());
    }

    #[test]
    fn test_render_status_bar() {
        let game = GameState::new(1, "TestPlayer", 1200, MatchmakingMode::Open);
        let output = render_status_bar(&game, 1);

        assert!(output.contains("TestPlayer"));
        assert!(output.contains("1200"));
    }

    #[test]
    fn test_render_lobby_header() {
        let output = render_lobby_header();
        assert!(output.contains("CHESS"));
    }

    #[test]
    fn test_render_open_games() {
        let games = vec![
            (1, "Player1".to_string(), 1200, "Open".to_string()),
            (2, "Player2".to_string(), 1400, "ELO".to_string()),
        ];
        let output = render_open_games(&games, 1300);

        assert!(output.contains("Player1"));
        assert!(output.contains("Player2"));
        assert!(output.contains("1200"));
        assert!(output.contains("1400"));
    }

    #[test]
    fn test_render_active_games_empty() {
        let output = render_active_games(&[]);
        assert!(output.contains("No active games"));
    }

    #[test]
    fn test_render_leaderboard() {
        let entries = vec![
            (1, "Champion".to_string(), 2000, 50, 10),
            (2, "Strong".to_string(), 1800, 40, 20),
        ];
        let output = render_leaderboard(&entries);

        assert!(output.contains("LEADERBOARD"));
        assert!(output.contains("Champion"));
        assert!(output.contains("2000"));
    }
}
