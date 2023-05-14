mod fen;
mod pieces;

use derive_more::*;
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

/// A checked move.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Move {
    from: BoardIdx,
    to: BoardIdx,
}

impl Move {
    pub fn to(&self) -> BoardIdx {
        self.to
    }

    pub fn from(&self) -> BoardIdx {
        self.from
    }

    /// From unwrapped indicies.
    pub fn from_uidxs(from: usize, to: usize) -> Self {
        Move {
            from: BoardIdx::unew(from),
            to: BoardIdx::unew(to),
        }
    }

    /// From board indicies.
    pub fn from_idxs(from: BoardIdx, to: BoardIdx) -> Self {
        Move { from, to }
    }

    /// From a string move.
    pub fn new(str: &str) -> Option<Self> {
        if str.len() != 4 {
            return None;
        }

        let (from, to) = str.split_at(2);
        let from = idx_from_pos(from)?;
        let to = idx_from_pos(to)?;

        Some(Move { from, to })
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

    pub fn try_move(&mut self, umove: Move) -> Result<(), InvalidMoveErr> {
        // check if the player owns the piece they are trying to move
        if self.squares[*umove.from] & self.active_color.to_piece_color() == Piece::empty() {
            return Err(InvalidMoveErr::NotYourPiece);
        }

        let mut moves = vec![];
        self.get_moves_for(&mut moves, umove.from);
        if !moves.iter().any(|m| m.to == umove.to) {
            // invalid move
            return Err(InvalidMoveErr::IllegalMove);
        }

        // handle en_passant
        if self.en_passant == Some(umove.to) {
            // if the move to value matches the curren en passant-able square,
            //     take the pawn that double pushed.
            let target = (*umove.to as isize - self.active_color.forward_value() * 8) as usize;
            self.squares[target] = Piece::empty();
        }

        // record en passant
        if self.piece_at(umove.from) & Piece::PieceType == Piece::Pawn
            && (((*umove.from as isize) - (*umove.to as isize)).abs() == 16)
        {
            // pawn moved 2 spaces; record en passant
            let target = (*umove.to as isize - self.active_color.forward_value() * 8) as usize;
            self.en_passant = Some(BoardIdx::unew(target));
        } else {
            self.en_passant = None;
        }

        let is_king = self.squares[*umove.from] & Piece::PieceType == Piece::King;
        let move_dist = *umove.to as isize - *umove.from as isize;
        if is_king && move_dist.abs() == 2 {
            // this move is a castle; move the rook
            let (rook_from, rook_to) = get_rook_castle_pos(self.active_color, move_dist > 0);
            self.squares[rook_to] = self.squares[rook_from];
            self.squares[rook_from] = Piece::empty();
            self.castle_rights.revoke_all(self.active_color);
        }

        // move the piece
        self.squares[*umove.to] = self.squares[*umove.from];
        self.squares[*umove.from] = Piece::empty();

        self.active_color = !self.active_color;
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

    pub fn get_moves_for<'b, 'v>(&'b self, moves: &'v mut Vec<Move>, idx: BoardIdx) {
        let piece = self.piece_at(idx);

        if !piece.is_color(self.active_color) {
            return;
        }

        let piece_type = piece & Piece::PieceType;
        if piece.is_sliding() {
            self.generate_sliding_moves(moves, idx, piece);
        } else if piece_type == Piece::Knight {
            self.generate_knight_moves(moves, idx);
        } else if piece_type == Piece::Pawn {
            self.generate_pawn_moves(moves, idx);
        } else if piece_type == Piece::King {
            self.generate_king_moves(moves, idx);
        }
    }

    fn generate_sliding_moves(&self, moves: &mut Vec<Move>, start: BoardIdx, piece: Piece) {
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
                    from: start,
                    to: BoardIdx::unew(target),
                });

                if target_piece.is_color(!self.active_color) {
                    break;
                }
            }
        }
    }

    fn generate_knight_moves(&self, moves: &mut Vec<Move>, start: BoardIdx) {
        const KNIGHT_MOVES: [(isize, isize); 8] = [
            (-2, 1),
            (-1, 2),
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
        ];

        for (dx, dy) in KNIGHT_MOVES.iter() {
            let x_dir = if *dx > 0 {
                Direction::East
            } else {
                Direction::West
            };

            let y_dir = if *dy > 0 {
                Direction::North
            } else {
                Direction::South
            };

            if squares_to_edge(start, x_dir) >= dx.abs() as usize
                && squares_to_edge(start, y_dir) >= dy.abs() as usize
            {
                // target square is in bounds.
                let target = BoardIdx::unew((*start as isize + (dy * 8) + dx) as usize);
                if !self.piece_at(target).is_color(self.active_color) {
                    moves.push(Move {
                        from: start,
                        to: target,
                    });
                }
            }
        }
    }

    fn generate_pawn_moves(&self, moves: &mut Vec<Move>, start: BoardIdx) {
        if squares_to_edge(start, self.active_color.forward_dir()) < 1 {
            return;
        }
        let forward_target = (*start as isize + self.active_color.forward_value() * 8) as usize;

        // pawns can take diagonally
        if squares_to_edge(start, Direction::West) >= 1 {
            let target = BoardIdx::unew(forward_target - 1);
            if self.piece_at(target).is_color(!self.active_color) || self.en_passant == Some(target)
            {
                moves.push(Move {
                    from: start,
                    to: target,
                });
            }
        }
        if squares_to_edge(start, Direction::East) >= 1 {
            let target = BoardIdx::unew(forward_target + 1);
            if self.piece_at(target).is_color(!self.active_color) || self.en_passant == Some(target)
            {
                moves.push(Move {
                    from: start,
                    to: target,
                });
            }
        }

        if self.squares[forward_target] == Piece::empty() {
            moves.push(Move {
                from: start,
                to: BoardIdx::unew(forward_target),
            });
        } else {
            return;
        }

        // if it is on the starting rank, it can move forward 2.
        if (self.active_color == Player::White && *start / 8 == 1)
            || (self.active_color == Player::Black && *start / 8 == 6)
        {
            let target =
                BoardIdx::unew((*start as isize + self.active_color.forward_value() * 16) as usize);
            if self.piece_at(target) == Piece::empty() {
                moves.push(Move {
                    from: start,
                    to: target,
                })
            }
        }
    }
    pub fn generate_king_moves(&self, moves: &mut Vec<Move>, start: BoardIdx) {
        for dir in Direction::ALL {
            if squares_to_edge(start, dir) >= 1 {
                let target = BoardIdx::unew((*start as isize + dir.offset()) as usize);
                let target_piece = self.piece_at(target);

                // Block by friendly
                if target_piece.is_color(self.active_color) {
                    continue;
                }

                moves.push(Move {
                    from: start,
                    to: target,
                });
            }
        }

        // Castling
        self.generate_king_castle_directions(moves, start, Direction::West);
        self.generate_king_castle_directions(moves, start, Direction::East);
    }

    fn generate_king_castle_directions(
        &self,
        moves: &mut Vec<Move>,
        start: BoardIdx,
        dir: Direction,
    ) {
        if cfg!(debug_assertions) {
            assert!(dir == Direction::West || dir == Direction::East);
        }

        let (needed_castle_right, squares_in_between) = match (self.active_color, dir) {
            (Player::White, Direction::West) => (CastleRights::WhiteQueenSide, -3..=-1),
            (Player::White, Direction::East) => (CastleRights::WhiteKingSide, 1..=2),
            (Player::Black, Direction::West) => (CastleRights::BlackQueenSide, -3..=-1),
            (Player::Black, Direction::East) => (CastleRights::BlackKingSide, 1..=2),
            _ => panic!("generate_king_castle_directions called with a direction other than `West` or `East`")
        };

        if !self.castle_rights.has_right(needed_castle_right) {
            return;
        }

        if cfg!(debug_assertions) {
            // since the player still has their castle rights,
            //     we can do some extra checks in debug mode
            if self.active_color == Player::White {
                // ensure that the king is on its starting square
                assert_eq!(*start, 4);
                // ensure that the king side rook is still there
                assert_eq!(self.squares[7], Piece::White | Piece::Rook);
            } else {
                // ensure that the king is on its starting square
                assert_eq!(*start, 60);
                // ensure that the king side rook is still there
                assert_eq!(self.squares[63], Piece::Black | Piece::Rook);
            }
        }

        for i in squares_in_between {
            let idx = (*start as isize + i) as usize;
            if self.squares[idx] != Piece::empty() {
                // a piece is in the way
                return;
            }
        }

        let target = BoardIdx::unew((*start as isize + dir.offset() * 2) as usize);
        let target_piece = self.piece_at(target);

        if target_piece != Piece::empty() {
            // a piece is in the way
            return;
        }

        moves.push(Move {
            from: start,
            to: target,
        });
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
    use crate::{Board, Move};

    #[test]
    fn test_try_move() {
        let mut board = Board::new();
        // e2 to e4 should be a valid starting move.
        board.try_move(Move::new("e2e4").unwrap()).unwrap();
    }
}
