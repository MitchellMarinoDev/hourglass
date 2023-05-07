use hourglass_engine::Piece;

pub(crate) trait PieceExt {
    fn get_texture_idx(&self) -> usize;
}

impl PieceExt for hourglass_engine::Piece {
    fn get_texture_idx(&self) -> usize {
        let mut idx = match *self & Piece::PieceType {
            Piece::King => 0,
            Piece::Queen => 1,
            Piece::Bishop => 2,
            Piece::Knight => 3,
            Piece::Rook => 4,
            Piece::Pawn => 5,
            _ => return 12,
        };

        idx += match *self & Piece::PlayerType {
            Piece::White => 0,
            Piece::Black => 6,
            _ => return 12,
        };

        idx
    }
}
