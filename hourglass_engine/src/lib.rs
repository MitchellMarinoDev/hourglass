mod fen;
mod pieces;

use derive_more::*;
use lazy_static::lazy_static;

pub use pieces::*;

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

pub(crate) fn squares_to_edge(start: BoardIdx, dir: Direction) -> usize {
    NUM_SQUARES_TO_EDGE[*start][dir as usize]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deref)]
pub struct BoardIdx(usize);

impl BoardIdx {
    /// Creates a new [`BoardIdx`] if it is in range (0..64).
    pub fn new(idx: usize) -> Option<Self> {
        if !(0..64).contains(&idx) {
            None
        } else {
            Some(BoardIdx(idx))
        }
    }

    /// Unwrapped new.
    ///
    /// Creates a new [`BoardIdx`], panicing if it is out of range (0..64).
    pub fn unew(idx: usize) -> Self {
        if !(0..64).contains(&idx) {
            panic!("unew called with a value ({idx}) outside of the range 0..64");
        }
        BoardIdx(idx)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InvalidMoveErr {
    ParseErr,
    /// The 'from' position is not your piece or not a piece at all.
    NotYourPiece,
    /// That piece cannot move there.
    IllegalMove,
}

/// An unchecked move.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct UMove {
    from: BoardIdx,
    to: BoardIdx,
}

impl UMove {
    /// From unwrapped indicies.
    pub fn from_uidxs(from: usize, to: usize) -> Self {
        UMove {
            from: BoardIdx::unew(from),
            to: BoardIdx::unew(to),
        }
    }

    /// From board indicies.
    pub fn from_idxs(from: BoardIdx, to: BoardIdx) -> Self {
        UMove { from, to }
    }

    /// From a string move.
    pub fn new(str: &str) -> Option<Self> {
        if str.len() != 4 {
            return None;
        }

        let (from, to) = str.split_at(2);
        let from = idx_from_pos(from)?;
        let to = idx_from_pos(to)?;

        Some(UMove { from, to })
    }
}

/// A checked move.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Move<'b> {
    _board: &'b Board,
    from: BoardIdx,
    to: BoardIdx,
}

impl<'b> Move<'b> {
    pub fn to(&self) -> BoardIdx {
        self.to
    }

    pub fn from(&self) -> BoardIdx {
        self.from
    }
}

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
    en_passant: Option<BoardIdx>,
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

    pub fn try_move(&mut self, umove: UMove) -> Result<(), InvalidMoveErr> {
        // check if the player owns the piece they are trying to move
        if self.squares[*umove.from] & self.active_color.to_piece_color() == Piece::empty() {
            return Err(InvalidMoveErr::NotYourPiece);
        }

        // TODO: check if it is a valid move

        self.squares[*umove.to] = self.squares[*umove.from];
        self.squares[*umove.from] = Piece::empty();

        Ok(())
    }

    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        for (idx, piece) in self.squares.iter().enumerate() {
            if piece.is_color(self.active_color) {
                self.get_moves_for(&mut moves, BoardIdx::unew(idx))
            }
        }

        moves
    }

    pub fn get_moves_for<'b, 'v>(&'b self, moves: &'v mut Vec<Move<'b>>, idx: BoardIdx) {
        let piece = self.piece_at(idx);

        if piece.is_sliding() {
            self.generate_sliding_moves(moves, idx, piece);
        }
    }

    pub fn generate_sliding_moves<'b, 'v>(
        &'b self,
        moves: &'v mut Vec<Move<'b>>,
        start: BoardIdx,
        piece: Piece,
    ) {
        let directions = match piece & Piece::PieceType {
            Piece::Bishop => &Direction::BISHOP[..],
            Piece::Rook => &Direction::ROOK[..],
            Piece::Queen => &Direction::ALL[..],
            _ => panic!("generate_sliding_moves called on a non-sliding piece"),
        };

        for dir in directions {
            for n in 0..squares_to_edge(start, *dir) as isize {
                let target = (*start as isize + dir.offset() * (n + 1)) as usize;
                let target_piece = self.squares[target];

                // Block by friendly
                if target_piece.is_color(self.active_color) {
                    break;
                }

                moves.push(Move {
                    _board: self,
                    from: start,
                    to: BoardIdx::unew(target),
                });

                if target_piece.is_color(!self.active_color) {
                    break;
                }
            }
        }
    }

    pub fn active_color(&self) -> Player {
        self.active_color
    }

    pub fn piece_at(&self, idx: BoardIdx) -> Piece {
        self.squares[*idx]
    }
}

fn idx_from_pos(pos: &str) -> Option<BoardIdx> {
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

    Some(BoardIdx::unew(idx))
}

#[cfg(test)]
mod tests {
    use crate::{Board, UMove};

    #[test]
    fn test_try_move() {
        let mut board = Board::new();
        // e2 to e4 should be a valid starting move.
        board.try_move(UMove::new("e2e4").unwrap()).unwrap();
    }
}
