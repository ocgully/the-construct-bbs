//! Chess move generation and validation

use super::board::{Board, Piece, PieceType, Color, Square};
use serde::{Serialize, Deserialize};

/// A chess move
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceType>,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Move { from, to, promotion: None }
    }

    pub fn with_promotion(from: Square, to: Square, promotion: PieceType) -> Self {
        Move { from, to, promotion: Some(promotion) }
    }

    /// Parse move from algebraic notation (e.g., "e2e4", "e7e8q")
    pub fn from_algebraic(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        if s.len() < 4 {
            return None;
        }

        let from = Square::from_algebraic(&s[0..2])?;
        let to = Square::from_algebraic(&s[2..4])?;

        let promotion = if s.len() > 4 {
            match s.chars().nth(4)? {
                'q' => Some(PieceType::Queen),
                'r' => Some(PieceType::Rook),
                'b' => Some(PieceType::Bishop),
                'n' => Some(PieceType::Knight),
                _ => None,
            }
        } else {
            None
        };

        Some(Move { from, to, promotion })
    }

    pub fn to_algebraic(&self) -> String {
        let mut s = format!("{}{}", self.from.to_algebraic(), self.to.to_algebraic());
        if let Some(promo) = self.promotion {
            s.push(match promo {
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                _ => '?',
            });
        }
        s
    }
}

/// Result of making a move
#[derive(Debug, Clone)]
pub struct MoveResult {
    pub is_capture: bool,
    pub is_check: bool,
    pub is_checkmate: bool,
    pub is_stalemate: bool,
    pub is_castling: bool,
    pub is_en_passant: bool,
    pub is_promotion: bool,
    pub captured_piece: Option<Piece>,
}

/// Check if a move is valid for the current position
pub fn is_valid_move(board: &Board, mv: Move) -> bool {
    let legal_moves = get_legal_moves(board);
    legal_moves.iter().any(|m| m.from == mv.from && m.to == mv.to && m.promotion == mv.promotion)
}

/// Get all legal moves for the current side to move
pub fn get_legal_moves(board: &Board) -> Vec<Move> {
    let pseudo_moves = get_pseudo_legal_moves(board, board.side_to_move);
    let mut legal = Vec::new();

    for mv in pseudo_moves {
        let mut test_board = board.clone();
        apply_move_unchecked(&mut test_board, mv);
        if !is_in_check(&test_board, board.side_to_move) {
            legal.push(mv);
        }
    }

    legal
}

/// Get all pseudo-legal moves (may leave king in check)
fn get_pseudo_legal_moves(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();

    for (square, piece) in board.pieces_of_color(color) {
        match piece.piece_type {
            PieceType::Pawn => get_pawn_moves(board, square, color, &mut moves),
            PieceType::Knight => get_knight_moves(board, square, color, &mut moves),
            PieceType::Bishop => get_bishop_moves(board, square, color, &mut moves),
            PieceType::Rook => get_rook_moves(board, square, color, &mut moves),
            PieceType::Queen => get_queen_moves(board, square, color, &mut moves),
            PieceType::King => get_king_moves(board, square, color, &mut moves),
        }
    }

    moves
}

fn get_pawn_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    let direction: i8 = if color == Color::White { 1 } else { -1 };
    let start_rank = if color == Color::White { 1 } else { 6 };
    let promotion_rank = if color == Color::White { 7 } else { 0 };

    // Single push
    if let Some(to) = from.offset(0, direction) {
        if board.get(to).is_none() {
            if to.rank() == promotion_rank {
                for promo in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                    moves.push(Move::with_promotion(from, to, promo));
                }
            } else {
                moves.push(Move::new(from, to));
            }

            // Double push from starting position
            if from.rank() == start_rank {
                if let Some(to2) = from.offset(0, direction * 2) {
                    if board.get(to2).is_none() {
                        moves.push(Move::new(from, to2));
                    }
                }
            }
        }
    }

    // Captures (including en passant)
    for file_delta in [-1i8, 1i8] {
        if let Some(to) = from.offset(file_delta, direction) {
            let is_en_passant = board.en_passant == Some(to);
            let has_enemy = board.get(to).map(|p| p.color != color).unwrap_or(false);

            if has_enemy || is_en_passant {
                if to.rank() == promotion_rank {
                    for promo in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                        moves.push(Move::with_promotion(from, to, promo));
                    }
                } else {
                    moves.push(Move::new(from, to));
                }
            }
        }
    }
}

fn get_knight_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    let offsets = [
        (-2, -1), (-2, 1), (-1, -2), (-1, 2),
        (1, -2), (1, 2), (2, -1), (2, 1),
    ];

    for (df, dr) in offsets {
        if let Some(to) = from.offset(df, dr) {
            if board.get(to).map(|p| p.color != color).unwrap_or(true) {
                moves.push(Move::new(from, to));
            }
        }
    }
}

fn get_sliding_moves(board: &Board, from: Square, color: Color, directions: &[(i8, i8)], moves: &mut Vec<Move>) {
    for &(df, dr) in directions {
        let mut current = from;
        while let Some(to) = current.offset(df, dr) {
            match board.get(to) {
                None => {
                    moves.push(Move::new(from, to));
                    current = to;
                }
                Some(piece) => {
                    if piece.color != color {
                        moves.push(Move::new(from, to));
                    }
                    break;
                }
            }
        }
    }
}

fn get_bishop_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    get_sliding_moves(board, from, color, &[(-1, -1), (-1, 1), (1, -1), (1, 1)], moves);
}

fn get_rook_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    get_sliding_moves(board, from, color, &[(0, -1), (0, 1), (-1, 0), (1, 0)], moves);
}

fn get_queen_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    get_bishop_moves(board, from, color, moves);
    get_rook_moves(board, from, color, moves);
}

fn get_king_moves(board: &Board, from: Square, color: Color, moves: &mut Vec<Move>) {
    let offsets = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1), (0, 1),
        (1, -1), (1, 0), (1, 1),
    ];

    for (df, dr) in offsets {
        if let Some(to) = from.offset(df, dr) {
            if board.get(to).map(|p| p.color != color).unwrap_or(true) {
                moves.push(Move::new(from, to));
            }
        }
    }

    // Castling
    if !is_in_check(board, color) {
        let rank = if color == Color::White { 0 } else { 7 };

        // Kingside
        if board.castling.can_castle(color, true) {
            let f_sq = Square::new(5, rank);
            let g_sq = Square::new(6, rank);
            if board.get(f_sq).is_none() && board.get(g_sq).is_none() {
                // Check that squares are not attacked
                let mut test_board = board.clone();
                test_board.set(from, None);
                test_board.set(f_sq, Some(Piece::new(PieceType::King, color)));
                if !is_in_check(&test_board, color) {
                    moves.push(Move::new(from, g_sq));
                }
            }
        }

        // Queenside
        if board.castling.can_castle(color, false) {
            let d_sq = Square::new(3, rank);
            let c_sq = Square::new(2, rank);
            let b_sq = Square::new(1, rank);
            if board.get(d_sq).is_none() && board.get(c_sq).is_none() && board.get(b_sq).is_none() {
                // Check that squares are not attacked
                let mut test_board = board.clone();
                test_board.set(from, None);
                test_board.set(d_sq, Some(Piece::new(PieceType::King, color)));
                if !is_in_check(&test_board, color) {
                    moves.push(Move::new(from, c_sq));
                }
            }
        }
    }
}

/// Check if a color's king is in check
pub fn is_in_check(board: &Board, color: Color) -> bool {
    let king_square = match board.find_king(color) {
        Some(sq) => sq,
        None => return false, // No king (shouldn't happen in valid game)
    };

    is_square_attacked(board, king_square, color.opposite())
}

/// Check if a square is attacked by any piece of the given color
pub fn is_square_attacked(board: &Board, square: Square, by_color: Color) -> bool {
    // Check pawn attacks
    let pawn_direction: i8 = if by_color == Color::White { -1 } else { 1 };
    for file_delta in [-1i8, 1i8] {
        if let Some(attacker_sq) = square.offset(file_delta, pawn_direction) {
            if let Some(piece) = board.get(attacker_sq) {
                if piece.piece_type == PieceType::Pawn && piece.color == by_color {
                    return true;
                }
            }
        }
    }

    // Check knight attacks
    let knight_offsets = [
        (-2, -1), (-2, 1), (-1, -2), (-1, 2),
        (1, -2), (1, 2), (2, -1), (2, 1),
    ];
    for (df, dr) in knight_offsets {
        if let Some(attacker_sq) = square.offset(df, dr) {
            if let Some(piece) = board.get(attacker_sq) {
                if piece.piece_type == PieceType::Knight && piece.color == by_color {
                    return true;
                }
            }
        }
    }

    // Check king attacks (for adjacent squares)
    let king_offsets = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1), (0, 1),
        (1, -1), (1, 0), (1, 1),
    ];
    for (df, dr) in king_offsets {
        if let Some(attacker_sq) = square.offset(df, dr) {
            if let Some(piece) = board.get(attacker_sq) {
                if piece.piece_type == PieceType::King && piece.color == by_color {
                    return true;
                }
            }
        }
    }

    // Check sliding piece attacks (bishop, rook, queen)
    let diagonal_directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    let orthogonal_directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    for &(df, dr) in &diagonal_directions {
        let mut current = square;
        while let Some(next) = current.offset(df, dr) {
            if let Some(piece) = board.get(next) {
                if piece.color == by_color {
                    if piece.piece_type == PieceType::Bishop || piece.piece_type == PieceType::Queen {
                        return true;
                    }
                }
                break;
            }
            current = next;
        }
    }

    for &(df, dr) in &orthogonal_directions {
        let mut current = square;
        while let Some(next) = current.offset(df, dr) {
            if let Some(piece) = board.get(next) {
                if piece.color == by_color {
                    if piece.piece_type == PieceType::Rook || piece.piece_type == PieceType::Queen {
                        return true;
                    }
                }
                break;
            }
            current = next;
        }
    }

    false
}

/// Check if the current side to move is in checkmate
pub fn is_checkmate(board: &Board) -> bool {
    is_in_check(board, board.side_to_move) && get_legal_moves(board).is_empty()
}

/// Check if the current side to move is in stalemate
pub fn is_stalemate(board: &Board) -> bool {
    !is_in_check(board, board.side_to_move) && get_legal_moves(board).is_empty()
}

/// Apply a move to the board without checking legality
pub fn apply_move_unchecked(board: &mut Board, mv: Move) {
    let piece = match board.get(mv.from) {
        Some(p) => p,
        None => return,
    };

    // Handle en passant capture
    if piece.piece_type == PieceType::Pawn && board.en_passant == Some(mv.to) {
        let captured_pawn_rank = if piece.color == Color::White { mv.to.rank() - 1 } else { mv.to.rank() + 1 };
        board.set(Square::new(mv.to.file(), captured_pawn_rank), None);
    }

    // Handle castling
    if piece.piece_type == PieceType::King {
        let file_diff = (mv.to.file() as i8) - (mv.from.file() as i8);
        if file_diff.abs() == 2 {
            // Castling move
            let rank = mv.from.rank();
            if file_diff > 0 {
                // Kingside: move rook from h to f
                let rook = board.get(Square::new(7, rank));
                board.set(Square::new(7, rank), None);
                board.set(Square::new(5, rank), rook);
            } else {
                // Queenside: move rook from a to d
                let rook = board.get(Square::new(0, rank));
                board.set(Square::new(0, rank), None);
                board.set(Square::new(3, rank), rook);
            }
        }
        // Remove all castling rights for this color
        board.castling.remove_for_color(piece.color);
    }

    // Update castling rights if rook moves or is captured
    if piece.piece_type == PieceType::Rook {
        let rank = if piece.color == Color::White { 0 } else { 7 };
        if mv.from == Square::new(0, rank) {
            board.castling.remove_queenside(piece.color);
        } else if mv.from == Square::new(7, rank) {
            board.castling.remove_kingside(piece.color);
        }
    }

    // Check if a rook is captured (remove opponent's castling rights)
    if let Some(captured) = board.get(mv.to) {
        if captured.piece_type == PieceType::Rook {
            let rank = if captured.color == Color::White { 0 } else { 7 };
            if mv.to == Square::new(0, rank) {
                board.castling.remove_queenside(captured.color);
            } else if mv.to == Square::new(7, rank) {
                board.castling.remove_kingside(captured.color);
            }
        }
    }

    // Set en passant square
    board.en_passant = None;
    if piece.piece_type == PieceType::Pawn {
        let rank_diff = (mv.to.rank() as i8) - (mv.from.rank() as i8);
        if rank_diff.abs() == 2 {
            // Double pawn push, set en passant square
            let ep_rank = if piece.color == Color::White { mv.from.rank() + 1 } else { mv.from.rank() - 1 };
            board.en_passant = Some(Square::new(mv.from.file(), ep_rank));
        }
    }

    // Move the piece
    board.set(mv.from, None);
    let final_piece = if let Some(promo) = mv.promotion {
        Piece::new(promo, piece.color)
    } else {
        piece
    };
    board.set(mv.to, Some(final_piece));

    // Update counters
    if piece.piece_type == PieceType::Pawn || board.get(mv.to).is_some() {
        board.halfmove_clock = 0;
    } else {
        board.halfmove_clock += 1;
    }

    if piece.color == Color::Black {
        board.fullmove_number += 1;
    }

    // Switch side to move
    board.side_to_move = piece.color.opposite();
}

/// Make a move and return the result
pub fn make_move(board: &mut Board, mv: Move) -> Option<MoveResult> {
    if !is_valid_move(board, mv) {
        return None;
    }

    let piece = board.get(mv.from)?;
    let captured = board.get(mv.to);
    let is_en_passant = piece.piece_type == PieceType::Pawn && board.en_passant == Some(mv.to);
    let is_castling = piece.piece_type == PieceType::King && (mv.from.file() as i8 - mv.to.file() as i8).abs() == 2;

    apply_move_unchecked(board, mv);

    let is_check = is_in_check(board, board.side_to_move);
    let is_checkmate = is_check && get_legal_moves(board).is_empty();
    let is_stalemate = !is_check && get_legal_moves(board).is_empty();

    Some(MoveResult {
        is_capture: captured.is_some() || is_en_passant,
        is_check,
        is_checkmate,
        is_stalemate,
        is_castling,
        is_en_passant,
        is_promotion: mv.promotion.is_some(),
        captured_piece: captured,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_algebraic() {
        let mv = Move::from_algebraic("e2e4").unwrap();
        assert_eq!(mv.from, Square::new(4, 1));
        assert_eq!(mv.to, Square::new(4, 3));
        assert_eq!(mv.promotion, None);

        let mv = Move::from_algebraic("e7e8q").unwrap();
        assert_eq!(mv.promotion, Some(PieceType::Queen));
    }

    #[test]
    fn test_starting_moves() {
        let board = Board::new();
        let moves = get_legal_moves(&board);
        // 16 pawn moves + 4 knight moves = 20
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_is_in_check() {
        // Scholar's mate position (after Qxf7#)
        let board = Board::from_fen("r1bqkb1r/pppp1Qpp/2n2n2/4p3/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4").unwrap();
        assert!(is_in_check(&board, Color::Black));
        assert!(is_checkmate(&board));
    }

    #[test]
    fn test_en_passant() {
        // Position where white pawn on e5 can capture black pawn that just moved d7-d5
        let mut board = Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3").unwrap();

        let mv = Move::from_algebraic("e5d6").unwrap();
        assert!(is_valid_move(&board, mv));

        let result = make_move(&mut board, mv).unwrap();
        assert!(result.is_en_passant);
        assert!(result.is_capture);

        // Captured pawn should be gone
        assert!(board.get(Square::from_algebraic("d5").unwrap()).is_none());
    }

    #[test]
    fn test_castling() {
        // Position where white can castle kingside
        let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();

        let mv = Move::from_algebraic("e1g1").unwrap(); // Kingside castle
        assert!(is_valid_move(&board, mv));

        let result = make_move(&mut board, mv).unwrap();
        assert!(result.is_castling);

        // King should be on g1, rook on f1
        assert_eq!(board.get(Square::from_algebraic("g1").unwrap()).unwrap().piece_type, PieceType::King);
        assert_eq!(board.get(Square::from_algebraic("f1").unwrap()).unwrap().piece_type, PieceType::Rook);
    }

    #[test]
    fn test_promotion() {
        let mut board = Board::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();

        let mv = Move::from_algebraic("a7a8q").unwrap();
        assert!(is_valid_move(&board, mv));

        let result = make_move(&mut board, mv).unwrap();
        assert!(result.is_promotion);

        assert_eq!(board.get(Square::from_algebraic("a8").unwrap()).unwrap().piece_type, PieceType::Queen);
    }

    #[test]
    fn test_stalemate() {
        // Classic stalemate position - black king trapped in corner with no legal moves
        // White king on b6, white queen on c7, black king on a8
        // Black king cannot move: a7 attacked by Kb6 and Qc7, b8 attacked by Qc7, b7 attacked by Kb6 and Qc7
        let board = Board::from_fen("k7/2Q5/1K6/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(!is_in_check(&board, Color::Black));
        assert!(is_stalemate(&board));
    }

    #[test]
    fn test_cannot_castle_through_check() {
        // Position where f1 is attacked by bishop on c4 - king cannot castle kingside through f1
        // e2 pawn removed so bishop on c4 can attack f1 through the diagonal
        let board = Board::from_fen("r3k2r/pppppppp/8/8/2b5/8/PPPP1PPP/R3K2R w KQkq - 0 1").unwrap();

        // Verify setup: f1 should be attacked by the bishop
        assert!(is_square_attacked(&board, Square::new(5, 0), Color::Black),
                "f1 should be attacked by bishop on c4");

        // e1g1 should not be valid because f1 is attacked by bishop on c4
        let mv = Move::from_algebraic("e1g1").unwrap();
        assert!(!is_valid_move(&board, mv));
    }
}
