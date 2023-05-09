use bitflags::bitflags;

use crate::Direction;

bitflags! {
    /// Bitflags for represending a piece, ex. King, Rook, Pawn,
    ///     along with its color.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Piece: u8 {
        const King = 1;
        const Pawn = 2;
        const Knight = 3;
        const Bishop = 4;
        const Rook = 5;
        const Queen = 6;

        const White = 8;
        const Black = 16;

        const PieceType = 0b111;
        const PlayerType = 0b11000;
    }

    // Bitflags representing the castle rights for both players.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CastleRights: u8 {
        const WhiteKingSide = 1<<0;
        const WhiteQueenSide = 1<<1;
        const BlackKingSide = 1<<2;
        const BlackQueenSide = 1<<3;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub fn to_piece_color(&self) -> Piece {
        match *self {
            Player::White => Piece::White,
            Player::Black => Piece::Black,
        }
    }

    pub(crate) fn forward_dir(&self) -> Direction {
        match self {
            Player::White => Direction::North,
            Player::Black => Direction::South,
        }
    }

    pub(crate) fn forward_value(&self) -> isize {
        match self {
            Player::White => 1,
            Player::Black => -1,
        }
    }
}

impl std::ops::Not for Player {
    type Output = Player;

    fn not(self) -> Self::Output {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

impl Piece {
    pub fn is_color(&self, p: Player) -> bool {
        *self & p.to_piece_color() != Piece::empty()
    }

    pub fn is_sliding(&self) -> bool {
        matches!(
            *self & Piece::PieceType,
            Piece::Queen | Piece::Rook | Piece::Bishop
        )
    }
}
