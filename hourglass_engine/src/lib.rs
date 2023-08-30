mod ai;
mod fen;
mod gen_attacks;
mod gen_moves;
mod pieces;

use lazy_static::lazy_static;

pub use pieces::*;

fn get_rook_castle_pos(player: Player, is_east: bool) -> (usize, usize) {
    match (player, is_east) {
        (Player::White, false) => (0, 3),
        (Player::White, true) => (7, 5),
        (Player::Black, false) => (56, 59),
        (Player::Black, true) => (63, 61),
    }
}

lazy_static! {
    static ref NUM_SQUARES_TO_EDGE: [[usize; 8]; 64] = {
        let mut squares_to_edge = [[0; 8]; 64];

        for file in 0..8 {
            for rank in 0..8 {
                let n_north = 7 - rank;
                let n_south = rank;
                let n_west = file;
                let n_east = 7 - file;

                let idx = rank * 8 + file;

                squares_to_edge[idx] = [
                    n_north,
                    n_south,
                    n_west,
                    n_east,
                    usize::min(n_north, n_west),
                    usize::min(n_south, n_east),
                    usize::min(n_north, n_east),
                    usize::min(n_south, n_west),
                ];
            }
        }

        squares_to_edge
    };
}

pub(crate) fn squares_to_edge(start: usize, dir: Direction) -> usize {
    NUM_SQUARES_TO_EDGE[start][dir as usize]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InvalidMoveErr {
    ParseErr,
    /// The 'from' position is not your piece or not a piece at all.
    NotYourPiece,
    /// That piece cannot move there.
    IllegalMove,
    /// You need to add a promotion to the piece.
    NoPromotion,
}

/// A checked move.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Move {
    from: usize,
    to: usize,
    promote: Option<Piece>,
}

impl Move {
    pub fn to(&self) -> usize {
        self.to
    }

    pub fn from(&self) -> usize {
        self.from
    }

    /// From unwrapped indicies.
    pub fn from_uidxs(from: usize, to: usize) -> Self {
        Move {
            from,
            to,
            promote: None,
        }
    }

    // Creates a new Move with the same `to` and `from`, adding a promote piece.
    #[must_use]
    pub fn with_promote(&self, promote: Option<Piece>) -> Self {
        Move { promote, ..*self }
    }

    /// From board indicies.
    pub fn from_idxs(from: usize, to: usize) -> Self {
        Move {
            from,
            to,
            promote: None,
        }
    }

    /// From a string move.
    pub fn from_str(str: &str) -> Option<Self> {
        if str.len() != 4 {
            return None;
        }

        let (from, to) = str.split_at(2);
        let from = idx_from_pos(from)?;
        let to = idx_from_pos(to)?;

        Some(Move {
            from,
            to,
            promote: None,
        })
    }

    pub fn new(from: usize, to: usize, promote: Option<Piece>) -> Self {
        Move { from, to, promote }
    }
}

// impl PartialEq for Move {
//     fn eq(&self, other: &Self) -> bool {
//         self.from == other.from
//             && self.to == other.to
//             && match (self.promote, other.promote) {
//                 (None, None) => true,
//                 (Some(_), None) => false,
//                 (None, Some(_)) => false,
//                 (Some(self_piece), Some(other_piece)) => self_piece.bits() == other_piece.bits(),
//             }
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Direction {
    North = 0,
    South = 1,
    West = 2,
    East = 3,
    NorthWest = 4,
    SouthEast = 5,
    NorthEast = 6,
    SouthWest = 7,
}

impl Direction {
    const ALL: [Direction; 8] = [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::NorthEast,
        Direction::SouthWest,
    ];

    const ROOK: [Direction; 4] = [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    const BISHOP: [Direction; 4] = [
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::NorthEast,
        Direction::SouthWest,
    ];

    pub(crate) fn offset(&self) -> isize {
        match *self {
            Direction::North => 8,
            Direction::South => -8,
            Direction::West => -1,
            Direction::East => 1,
            Direction::NorthWest => 7,
            Direction::SouthEast => -7,
            Direction::NorthEast => 9,
            Direction::SouthWest => -9,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Board {
    squares: [Piece; 64],
    castle_rights: CastleRights,
    active_color: Player,
    en_passant: Option<usize>,
    halfmove: u32,
    fullmove: u32,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Board::empty();
        board
            .load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("failed to parse starting board fen");
        board
    }

    pub fn empty() -> Self {
        Board {
            squares: [Piece::empty(); 64],
            castle_rights: CastleRights::empty(),
            active_color: Player::White,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
        }
    }

    pub fn try_move(&mut self, umove: Move) -> Result<(), InvalidMoveErr> {
        // check if the player owns the piece they are trying to move
        if self.squares[umove.from] & self.active_color.to_piece_color() == Piece::empty() {
            return Err(InvalidMoveErr::NotYourPiece);
        }

        let mut moves = vec![];
        self.get_moves_for(&mut moves, umove.from);
        if !moves.iter().any(|m| *m == umove) {
            // invalid move
            return Err(InvalidMoveErr::IllegalMove);
        }

        // handle en_passant
        if self.en_passant == Some(umove.to) {
            // if the move to value matches the curren en passant-able square,
            //     take the pawn that double pushed.
            let target = (umove.to as isize - self.active_color.forward_value() * 8) as usize;
            self.squares[target] = Piece::empty();
        }

        // record en passant
        if self.piece_at(umove.from) & Piece::PieceType == Piece::Pawn
            && (((umove.from as isize) - (umove.to as isize)).abs() == 16)
        {
            // pawn moved 2 spaces; record en passant
            let target = (umove.to as isize - self.active_color.forward_value() * 8) as usize;
            self.en_passant = Some(target);
        } else {
            self.en_passant = None;
        }

        let is_king = self.squares[umove.from] & Piece::PieceType == Piece::King;
        let move_dist = umove.to as isize - umove.from as isize;
        if is_king && move_dist.abs() == 2 {
            // this move is a castle; move the rook
            let (rook_from, rook_to) = get_rook_castle_pos(self.active_color, move_dist > 0);
            self.squares[rook_to] = self.squares[rook_from];
            self.squares[rook_from] = Piece::empty();
        }
        if is_king {
            // moving the king revokes it's castle rights
            self.castle_rights.revoke_all(self.active_color);
        }
        if self.squares[umove.from] & Piece::PieceType == Piece::Rook {
            // if a rook moves from it's starting square, it revokes the castling rights in that direction
            if self.active_color == Player::White {
                if umove.from == 0 {
                    self.castle_rights.revoke(CastleRights::WhiteQueenSide);
                }
                if umove.from == 7 {
                    self.castle_rights.revoke(CastleRights::WhiteKingSide);
                }
            } else {
                if umove.from == 56 {
                    self.castle_rights.revoke(CastleRights::BlackQueenSide);
                }
                if umove.from == 63 {
                    self.castle_rights.revoke(CastleRights::BlackKingSide);
                }
            }
        }

        // move the piece
        self.unchecked_make_move(umove)
    }

    fn unchecked_make_move(&mut self, umove: Move) -> Result<(), InvalidMoveErr> {
        let mut resulting_piece = self.squares[umove.from];

        let to_rank = umove.to / 8;
        if self.squares[umove.from] & Piece::PieceType == Piece::Pawn
            && (to_rank == 0 || to_rank == 7)
        {
            // Pawn will promote
            match umove.promote {
                None => return Err(InvalidMoveErr::NoPromotion),
                Some(promoting_piece) => {
                    resulting_piece = promoting_piece | self.active_color.to_piece_color()
                }
            }
        }

        self.squares[umove.to] = resulting_piece;
        self.squares[umove.from] = Piece::empty();

        self.active_color = !self.active_color;

        Ok(())
    }

    pub fn active_color(&self) -> Player {
        self.active_color
    }

    pub fn find_king(&self, player: Player) -> usize {
        for (idx, piece) in self.squares.iter().enumerate() {
            if *piece == Piece::King | player.to_piece_color() {
                return idx;
            }
        }
        panic!("God save the king.");
    }

    pub fn piece_at(&self, idx: usize) -> Piece {
        self.squares[idx]
    }
}

fn idx_from_pos(pos: &str) -> Option<usize> {
    let mut pos_chars = pos.chars();
    let mut idx = 0;

    match pos_chars.next() {
        Some('a') => idx += 0 * 8,
        Some('b') => idx += 1 * 8,
        Some('c') => idx += 2 * 8,
        Some('d') => idx += 3 * 8,
        Some('e') => idx += 4 * 8,
        Some('f') => idx += 5 * 8,
        Some('g') => idx += 6 * 8,
        Some('h') => idx += 7 * 8,
        _ => return None,
    }

    match pos_chars.next().map(|c| c.to_digit(10)) {
        Some(Some(v)) if (0..8).contains(&v) => idx += v as usize,
        _ => return None,
    }

    Some(idx)
}

#[cfg(test)]
mod tests {
    use crate::{Board, Move};

    #[test]
    fn test_try_move() {
        let mut board = Board::new();
        // e2 to e4 should be a valid starting move.
        board.try_move(Move::from_str("e2e4").unwrap()).unwrap();
    }
}
