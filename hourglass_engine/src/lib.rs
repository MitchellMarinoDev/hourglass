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
pub struct Move<'b> {
    _board: &'b Board,
    from: BoardIdx,
    to: BoardIdx,
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
        println!("From: {:?}", self.squares[*umove.from]);
        println!("ActiveColor: {:?}", self.active_color);
        println!(
            "AND: {:?}",
            self.squares[*umove.from] & self.active_color.to_piece_color()
        );
        if self.squares[*umove.from] & self.active_color.to_piece_color() == Piece::empty() {
            return Err(InvalidMoveErr::NotYourPiece);
        }

        // TODO: check if it is a valid move

        self.squares[*umove.to] = self.squares[*umove.from];
        self.squares[*umove.from] = Piece::empty();

        Ok(())
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
