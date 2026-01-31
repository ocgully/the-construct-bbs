//! Chess board representation and piece definitions

use serde::{Serialize, Deserialize};

/// A square on the chess board (0-63, a1=0, h8=63)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Square(pub u8);

impl Square {
    pub fn new(file: u8, rank: u8) -> Self {
        debug_assert!(file < 8 && rank < 8);
        Square(rank * 8 + file)
    }

    pub fn from_algebraic(s: &str) -> Option<Self> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 2 {
            return None;
        }
        let file = match chars[0] {
            'a'..='h' => (chars[0] as u8) - b'a',
            'A'..='H' => (chars[0] as u8) - b'A',
            _ => return None,
        };
        let rank = match chars[1] {
            '1'..='8' => (chars[1] as u8) - b'1',
            _ => return None,
        };
        Some(Square::new(file, rank))
    }

    pub fn file(&self) -> u8 {
        self.0 % 8
    }

    pub fn rank(&self) -> u8 {
        self.0 / 8
    }

    pub fn to_algebraic(&self) -> String {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        format!("{}{}", file, rank)
    }

    /// Check if square is valid (0-63)
    pub fn is_valid(&self) -> bool {
        self.0 < 64
    }

    /// Get square offset by (file_delta, rank_delta), returns None if off board
    pub fn offset(&self, file_delta: i8, rank_delta: i8) -> Option<Square> {
        let new_file = (self.file() as i8) + file_delta;
        let new_rank = (self.rank() as i8) + rank_delta;
        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            Some(Square::new(new_file as u8, new_rank as u8))
        } else {
            None
        }
    }
}

/// Piece color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// Piece type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn symbol(&self, color: Color) -> char {
        match (self, color) {
            (PieceType::Pawn, Color::White) => 'P',
            (PieceType::Knight, Color::White) => 'N',
            (PieceType::Bishop, Color::White) => 'B',
            (PieceType::Rook, Color::White) => 'R',
            (PieceType::Queen, Color::White) => 'Q',
            (PieceType::King, Color::White) => 'K',
            (PieceType::Pawn, Color::Black) => 'p',
            (PieceType::Knight, Color::Black) => 'n',
            (PieceType::Bishop, Color::Black) => 'b',
            (PieceType::Rook, Color::Black) => 'r',
            (PieceType::Queen, Color::Black) => 'q',
            (PieceType::King, Color::Black) => 'k',
        }
    }

    pub fn unicode(&self, color: Color) -> char {
        match (self, color) {
            (PieceType::King, Color::White) => '\u{2654}',
            (PieceType::Queen, Color::White) => '\u{2655}',
            (PieceType::Rook, Color::White) => '\u{2656}',
            (PieceType::Bishop, Color::White) => '\u{2657}',
            (PieceType::Knight, Color::White) => '\u{2658}',
            (PieceType::Pawn, Color::White) => '\u{2659}',
            (PieceType::King, Color::Black) => '\u{265A}',
            (PieceType::Queen, Color::Black) => '\u{265B}',
            (PieceType::Rook, Color::Black) => '\u{265C}',
            (PieceType::Bishop, Color::Black) => '\u{265D}',
            (PieceType::Knight, Color::Black) => '\u{265E}',
            (PieceType::Pawn, Color::Black) => '\u{265F}',
        }
    }
}

/// A chess piece
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Piece { piece_type, color }
    }

    pub fn symbol(&self) -> char {
        self.piece_type.symbol(self.color)
    }

    pub fn unicode(&self) -> char {
        self.piece_type.unicode(self.color)
    }
}

/// Castling rights
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn new() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }

    pub fn can_castle(&self, color: Color, kingside: bool) -> bool {
        match (color, kingside) {
            (Color::White, true) => self.white_kingside,
            (Color::White, false) => self.white_queenside,
            (Color::Black, true) => self.black_kingside,
            (Color::Black, false) => self.black_queenside,
        }
    }

    pub fn remove_for_color(&mut self, color: Color) {
        match color {
            Color::White => {
                self.white_kingside = false;
                self.white_queenside = false;
            }
            Color::Black => {
                self.black_kingside = false;
                self.black_queenside = false;
            }
        }
    }

    pub fn remove_kingside(&mut self, color: Color) {
        match color {
            Color::White => self.white_kingside = false,
            Color::Black => self.black_kingside = false,
        }
    }

    pub fn remove_queenside(&mut self, color: Color) {
        match color {
            Color::White => self.white_queenside = false,
            Color::Black => self.black_queenside = false,
        }
    }
}

/// The chess board
/// Note: Uses custom serde to serialize as FEN string for compatibility
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// 64 squares, indexed by Square.0
    squares: [Option<Piece>; 64],
    /// Side to move
    pub side_to_move: Color,
    /// Castling rights
    pub castling: CastlingRights,
    /// En passant target square (the square the pawn passed through)
    pub en_passant: Option<Square>,
    /// Halfmove clock (for 50-move rule)
    pub halfmove_clock: u16,
    /// Fullmove number
    pub fullmove_number: u16,
}

impl Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as FEN string
        serializer.serialize_str(&self.to_fen())
    }
}

impl<'de> Deserialize<'de> for Board {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize from FEN string
        let fen = String::deserialize(deserializer)?;
        Board::from_fen(&fen).ok_or_else(|| serde::de::Error::custom("Invalid FEN string"))
    }
}

impl Board {
    /// Create a new board with the standard starting position
    pub fn new() -> Self {
        let mut board = Board {
            squares: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::new(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        };

        // Place white pieces
        board.set(Square::new(0, 0), Some(Piece::new(PieceType::Rook, Color::White)));
        board.set(Square::new(1, 0), Some(Piece::new(PieceType::Knight, Color::White)));
        board.set(Square::new(2, 0), Some(Piece::new(PieceType::Bishop, Color::White)));
        board.set(Square::new(3, 0), Some(Piece::new(PieceType::Queen, Color::White)));
        board.set(Square::new(4, 0), Some(Piece::new(PieceType::King, Color::White)));
        board.set(Square::new(5, 0), Some(Piece::new(PieceType::Bishop, Color::White)));
        board.set(Square::new(6, 0), Some(Piece::new(PieceType::Knight, Color::White)));
        board.set(Square::new(7, 0), Some(Piece::new(PieceType::Rook, Color::White)));
        for file in 0..8 {
            board.set(Square::new(file, 1), Some(Piece::new(PieceType::Pawn, Color::White)));
        }

        // Place black pieces
        board.set(Square::new(0, 7), Some(Piece::new(PieceType::Rook, Color::Black)));
        board.set(Square::new(1, 7), Some(Piece::new(PieceType::Knight, Color::Black)));
        board.set(Square::new(2, 7), Some(Piece::new(PieceType::Bishop, Color::Black)));
        board.set(Square::new(3, 7), Some(Piece::new(PieceType::Queen, Color::Black)));
        board.set(Square::new(4, 7), Some(Piece::new(PieceType::King, Color::Black)));
        board.set(Square::new(5, 7), Some(Piece::new(PieceType::Bishop, Color::Black)));
        board.set(Square::new(6, 7), Some(Piece::new(PieceType::Knight, Color::Black)));
        board.set(Square::new(7, 7), Some(Piece::new(PieceType::Rook, Color::Black)));
        for file in 0..8 {
            board.set(Square::new(file, 6), Some(Piece::new(PieceType::Pawn, Color::Black)));
        }

        board
    }

    /// Create an empty board
    pub fn empty() -> Self {
        Board {
            squares: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::default(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Get piece at square
    pub fn get(&self, square: Square) -> Option<Piece> {
        self.squares[square.0 as usize]
    }

    /// Set piece at square
    pub fn set(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.0 as usize] = piece;
    }

    /// Find the king's position for a color
    pub fn find_king(&self, color: Color) -> Option<Square> {
        for i in 0..64 {
            if let Some(piece) = self.squares[i] {
                if piece.piece_type == PieceType::King && piece.color == color {
                    return Some(Square(i as u8));
                }
            }
        }
        None
    }

    /// Get all pieces of a color
    pub fn pieces_of_color(&self, color: Color) -> Vec<(Square, Piece)> {
        let mut result = Vec::new();
        for i in 0..64 {
            if let Some(piece) = self.squares[i] {
                if piece.color == color {
                    result.push((Square(i as u8), piece));
                }
            }
        }
        result
    }

    /// Convert to FEN string
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Board position
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let square = Square::new(file, rank);
                match self.get(square) {
                    Some(piece) => {
                        if empty_count > 0 {
                            fen.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }
                        fen.push(piece.symbol());
                    }
                    None => empty_count += 1,
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling rights
        fen.push(' ');
        let mut castling = String::new();
        if self.castling.white_kingside { castling.push('K'); }
        if self.castling.white_queenside { castling.push('Q'); }
        if self.castling.black_kingside { castling.push('k'); }
        if self.castling.black_queenside { castling.push('q'); }
        if castling.is_empty() { castling.push('-'); }
        fen.push_str(&castling);

        // En passant
        fen.push(' ');
        match self.en_passant {
            Some(sq) => fen.push_str(&sq.to_algebraic()),
            None => fen.push('-'),
        }

        // Halfmove clock and fullmove number
        fen.push_str(&format!(" {} {}", self.halfmove_clock, self.fullmove_number));

        fen
    }

    /// Parse from FEN string
    pub fn from_fen(fen: &str) -> Option<Self> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        let mut board = Board::empty();

        // Parse board position
        let rows: Vec<&str> = parts[0].split('/').collect();
        if rows.len() != 8 {
            return None;
        }

        for (rank_idx, row) in rows.iter().rev().enumerate() {
            let rank = rank_idx as u8;
            let mut file: u8 = 0;
            for ch in row.chars() {
                if ch.is_ascii_digit() {
                    file += ch.to_digit(10)? as u8;
                } else {
                    let color = if ch.is_uppercase() { Color::White } else { Color::Black };
                    let piece_type = match ch.to_ascii_lowercase() {
                        'p' => PieceType::Pawn,
                        'n' => PieceType::Knight,
                        'b' => PieceType::Bishop,
                        'r' => PieceType::Rook,
                        'q' => PieceType::Queen,
                        'k' => PieceType::King,
                        _ => return None,
                    };
                    board.set(Square::new(file, rank), Some(Piece::new(piece_type, color)));
                    file += 1;
                }
            }
        }

        // Side to move
        board.side_to_move = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None,
        };

        // Castling rights
        board.castling = CastlingRights::default();
        if parts[2] != "-" {
            for ch in parts[2].chars() {
                match ch {
                    'K' => board.castling.white_kingside = true,
                    'Q' => board.castling.white_queenside = true,
                    'k' => board.castling.black_kingside = true,
                    'q' => board.castling.black_queenside = true,
                    _ => {}
                }
            }
        }

        // En passant
        if parts[3] != "-" {
            board.en_passant = Square::from_algebraic(parts[3]);
        }

        // Halfmove clock and fullmove number (optional)
        if parts.len() > 4 {
            board.halfmove_clock = parts[4].parse().unwrap_or(0);
        }
        if parts.len() > 5 {
            board.fullmove_number = parts[5].parse().unwrap_or(1);
        }

        Some(board)
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_algebraic() {
        assert_eq!(Square::new(0, 0).to_algebraic(), "a1");
        assert_eq!(Square::new(7, 7).to_algebraic(), "h8");
        assert_eq!(Square::new(4, 3).to_algebraic(), "e4");

        assert_eq!(Square::from_algebraic("a1"), Some(Square::new(0, 0)));
        assert_eq!(Square::from_algebraic("h8"), Some(Square::new(7, 7)));
        assert_eq!(Square::from_algebraic("e4"), Some(Square::new(4, 3)));
        assert_eq!(Square::from_algebraic("i9"), None);
    }

    #[test]
    fn test_square_offset() {
        let e4 = Square::new(4, 3);
        assert_eq!(e4.offset(1, 1), Some(Square::new(5, 4)));
        assert_eq!(e4.offset(-4, -3), Some(Square::new(0, 0)));
        assert_eq!(e4.offset(-5, 0), None); // off board
    }

    #[test]
    fn test_board_starting_position() {
        let board = Board::new();

        // Check white pieces
        assert_eq!(board.get(Square::new(4, 0)), Some(Piece::new(PieceType::King, Color::White)));
        assert_eq!(board.get(Square::new(3, 0)), Some(Piece::new(PieceType::Queen, Color::White)));

        // Check black pieces
        assert_eq!(board.get(Square::new(4, 7)), Some(Piece::new(PieceType::King, Color::Black)));

        // Check pawns
        assert_eq!(board.get(Square::new(4, 1)), Some(Piece::new(PieceType::Pawn, Color::White)));
        assert_eq!(board.get(Square::new(4, 6)), Some(Piece::new(PieceType::Pawn, Color::Black)));

        // Check empty squares
        assert_eq!(board.get(Square::new(4, 4)), None);
    }

    #[test]
    fn test_fen_roundtrip() {
        let board = Board::new();
        let fen = board.to_fen();
        let parsed = Board::from_fen(&fen).unwrap();
        assert_eq!(board.to_fen(), parsed.to_fen());
    }

    #[test]
    fn test_starting_position_fen() {
        let board = Board::new();
        let fen = board.to_fen();
        assert_eq!(fen, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    #[test]
    fn test_find_king() {
        let board = Board::new();
        assert_eq!(board.find_king(Color::White), Some(Square::new(4, 0)));
        assert_eq!(board.find_king(Color::Black), Some(Square::new(4, 7)));
    }
}
