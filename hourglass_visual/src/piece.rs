use chess::{Color, Piece};

pub(crate) trait PieceExt {
    fn get_texture_idx(&self, color: Color) -> usize;
}

impl PieceExt for Piece {
    fn get_texture_idx(&self, color: Color) -> usize {
        let idx = match (*self, color) {
            (Piece::King, Color::White) => 0,
            (Piece::Queen, Color::White) => 1,
            (Piece::Bishop, Color::White) => 2,
            (Piece::Knight, Color::White) => 3,
            (Piece::Rook, Color::White) => 4,
            (Piece::Pawn, Color::White) => 5,

            (Piece::King, Color::Black) => 6,
            (Piece::Queen, Color::Black) => 7,
            (Piece::Bishop, Color::Black) => 8,
            (Piece::Knight, Color::Black) => 9,
            (Piece::Rook, Color::Black) => 10,
            (Piece::Pawn, Color::Black) => 11,
        };

        idx
    }
}

impl PieceExt for Option<Piece> {
    fn get_texture_idx(&self, color: Color) -> usize {
        let piece = match *self {
            Some(p) => p,
            None => return 12,
        };

        let idx = match (piece, color) {
            (Piece::King, Color::White) => 0,
            (Piece::Queen, Color::White) => 1,
            (Piece::Bishop, Color::White) => 2,
            (Piece::Knight, Color::White) => 3,
            (Piece::Rook, Color::White) => 4,
            (Piece::Pawn, Color::White) => 5,

            (Piece::King, Color::Black) => 6,
            (Piece::Queen, Color::Black) => 7,
            (Piece::Bishop, Color::Black) => 8,
            (Piece::Knight, Color::Black) => 9,
            (Piece::Rook, Color::Black) => 10,
            (Piece::Pawn, Color::Black) => 11,
        };

        idx
    }
}
