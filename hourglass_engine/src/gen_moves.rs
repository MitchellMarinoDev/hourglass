use log::debug;

use crate::Board;
use crate::{squares_to_edge, CastleRights, Direction, Move, Piece, Player};

impl Board {
    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        for (idx, piece) in self.squares.iter().enumerate() {
            if piece.is_color(self.active_color) {
                self.get_moves_for(&mut moves, idx)
            }
        }

        moves
    }

    pub fn get_moves_for<'b, 'v>(&'b self, moves: &'v mut Vec<Move>, idx: usize) {
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

    fn generate_sliding_moves(&self, moves: &mut Vec<Move>, start: usize, piece: Piece) {
        let directions = match piece & Piece::PieceType {
            Piece::Bishop => &Direction::BISHOP[..],
            Piece::Rook => &Direction::ROOK[..],
            Piece::Queen => &Direction::ALL[..],
            _ => panic!("generate_sliding_moves called on a non-sliding piece"),
        };

        for dir in directions {
            for n in 0..squares_to_edge(start, *dir) as isize {
                let target = (start as isize + dir.offset() * (n + 1)) as usize;
                let target_piece = self.squares[target];

                // Block by friendly
                if target_piece.is_color(self.active_color) {
                    break;
                }

                self.add_move(moves, Move::from_idxs(start, target));

                if target_piece.is_color(!self.active_color) {
                    break;
                }
            }
        }
    }

    fn generate_knight_moves(&self, moves: &mut Vec<Move>, start: usize) {
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
                let target = (start as isize + (dy * 8) + dx) as usize;
                if !self.piece_at(target).is_color(self.active_color) {
                    self.add_move(moves, Move::from_idxs(start, target));
                }
            }
        }
    }

    fn generate_pawn_moves(&self, moves: &mut Vec<Move>, start: usize) {
        if squares_to_edge(start, self.active_color.forward_dir()) < 1 {
            return;
        }
        let forward_target = (start as isize + self.active_color.forward_value() * 8) as usize;

        // pawns can take diagonally
        if squares_to_edge(start, Direction::West) >= 1 {
            let target = forward_target - 1;
            if self.piece_at(target).is_color(!self.active_color) || self.en_passant == Some(target)
            {
                self.add_pawn_move(moves, Move::from_idxs(start, target))
            }
        }
        if squares_to_edge(start, Direction::East) >= 1 {
            let target = forward_target + 1;
            if self.piece_at(target).is_color(!self.active_color) || self.en_passant == Some(target)
            {
                self.add_pawn_move(moves, Move::from_idxs(start, target))
            }
        }

        if self.squares[forward_target] != Piece::empty() {
            return;
        }

        self.add_pawn_move(moves, Move::from_idxs(start, forward_target));

        // if it is on the starting rank, it can move forward 2.
        if (self.active_color == Player::White && start / 8 == 1)
            || (self.active_color == Player::Black && start / 8 == 6)
        {
            let target = (start as isize + self.active_color.forward_value() * 16) as usize;
            if self.piece_at(target) == Piece::empty() {
                self.add_pawn_move(moves, Move::from_idxs(start, target))
            }
        }
    }

    fn add_pawn_move(&self, moves: &mut Vec<Move>, umove: Move) {
        let target_rank = umove.to / 8;
        // if the pawn made it to the first or last rank, it needs to promote
        if target_rank == 0 || target_rank == 7 {
            for promote in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
                self.add_move(moves, umove.with_promote(Some(promote)));
            }
        } else {
            self.add_move(moves, umove);
        }
    }

    fn generate_king_moves(&self, moves: &mut Vec<Move>, start: usize) {
        for dir in Direction::ALL {
            if squares_to_edge(start, dir) >= 1 {
                let target = (start as isize + dir.offset()) as usize;
                let target_piece = self.piece_at(target);

                // Block by friendly
                if target_piece.is_color(self.active_color) {
                    continue;
                }
                self.add_move(moves, Move::from_idxs(start, target));
            }
        }

        // Castling
        self.generate_king_castle_directions(moves, start, Direction::West);
        self.generate_king_castle_directions(moves, start, Direction::East);
    }

    fn generate_king_castle_directions(&self, moves: &mut Vec<Move>, start: usize, dir: Direction) {
        if cfg!(debug_assertions) {
            assert!(dir == Direction::West || dir == Direction::East);
        }

        let (needed_castle_right, squares_in_between, king_move_range) = match (self.active_color, dir) {
            (Player::White, Direction::West) => (CastleRights::WhiteQueenSide, -3..=-1, -2..=-1),
            (Player::White, Direction::East) => (CastleRights::WhiteKingSide, 1..=2, 1..=2),
            (Player::Black, Direction::West) => (CastleRights::BlackQueenSide, -3..=-1, -2..=-1),
            (Player::Black, Direction::East) => (CastleRights::BlackKingSide, 1..=2, 1..=2),
            _ => panic!("generate_king_castle_directions called with a direction other than `West` or `East`")
        };

        if !self.castle_rights.has_right(needed_castle_right) {
            return;
        }

        if cfg!(debug_assertions) {
            // since the player still has their castle rights,
            //     we can do some extra checks in debug mode
            if self.active_color == Player::White {
                let rook_square = match needed_castle_right {
                    CastleRights::WhiteKingSide => 7,
                    CastleRights::WhiteQueenSide => 0,
                    _ => panic!(
                        "the needed castle right should be the same color as the active player"
                    ),
                };

                // ensure that the king is on its starting square
                assert_eq!(start, 4);
                // ensure that the rook is still there
                assert_eq!(self.squares[rook_square], Piece::White | Piece::Rook);
            } else {
                let rook_square = match needed_castle_right {
                    CastleRights::BlackKingSide => 56,
                    CastleRights::BlackQueenSide => 63,
                    _ => panic!(
                        "the needed castle right should be the same color as the active player"
                    ),
                };

                // ensure that the king is on its starting square
                assert_eq!(start, 60);
                // ensure that the rook is still there
                assert_eq!(self.squares[rook_square], Piece::Black | Piece::Rook);
            }
        }

        let attacked_squares = self.generate_attacks(!self.active_color());

        for i in king_move_range {
            let idx = (start as isize + i) as usize;
            if attacked_squares[idx] {
                // the king may not move across an attacked square
                //    when castling
                return;
            }
        }

        for i in squares_in_between {
            let idx = (start as isize + i) as usize;
            if self.squares[idx] != Piece::empty() {
                // a piece is in the way
                return;
            }
        }

        let target = (start as isize + dir.offset() * 2) as usize;
        let target_piece = self.piece_at(target);

        if target_piece != Piece::empty() {
            // a piece is in the way
            return;
        }

        self.add_move(moves, Move::from_idxs(start, target));
    }

    fn add_move<'b, 'v>(&'b self, moves: &'v mut Vec<Move>, umove: Move) {
        // you cannot move into check
        let mut new_board = self.clone();
        new_board.unchecked_make_move(umove).unwrap();
        if new_board.generate_attacks(new_board.active_color())
            [new_board.find_king(!new_board.active_color())]
        {
            debug!(
                "You may not make the move {:?}, as you would move into check",
                umove
            );
            return;
        }

        moves.push(umove);
    }
}
