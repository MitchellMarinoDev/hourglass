use crate::{squares_to_edge, Board, Direction, Piece, Player};

impl Board {
    pub fn is_in_check(&self, player: Player) -> bool {
        let king_pos = self.find_king(player);
        self.generate_attacks(player)[king_pos]
    }

    /// Gets the squares attacked by the given player
    pub fn generate_attacks(&self, player: Player) -> [bool; 64] {
        let mut map = [false; 64];

        for (idx, piece) in self.squares.iter().enumerate() {
            if piece.is_color(player) {
                self.get_attacked_squares_for(&mut map, player, idx)
            }
        }

        map
    }

    pub fn get_attacked_squares_for<'b, 'v>(
        &'b self,
        map: &'v mut [bool; 64],
        player: Player,
        idx: usize,
    ) {
        let piece = self.piece_at_idx(idx);

        let piece_type = piece & Piece::PieceType;
        if piece.is_sliding() {
            self.get_attacked_sliding(map, player, idx, piece);
        } else if piece_type == Piece::Knight {
            self.get_attacked_knight(map, player, idx);
        } else if piece_type == Piece::Pawn {
            self.get_attacked_pawn(map, player, idx);
        } else if piece_type == Piece::King {
            self.get_attacked_king(map, player, idx);
        }
    }

    fn get_attacked_sliding(
        &self,
        map: &mut [bool; 64],
        player: Player,
        start: usize,
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
                let target = (start as isize + dir.offset() * (n + 1)) as usize;
                let target_piece = self.squares[target];

                // Block by friendly
                if target_piece.is_color(player) {
                    break;
                }

                map[target] = true;

                if target_piece.is_color(!player) {
                    break;
                }
            }
        }
    }

    fn get_attacked_knight(&self, map: &mut [bool; 64], player: Player, start: usize) {
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
                if !self.piece_at_idx(target).is_color(player) {
                    map[target] = true;
                }
            }
        }
    }

    fn get_attacked_pawn(&self, map: &mut [bool; 64], player: Player, start: usize) {
        if squares_to_edge(start, player.forward_dir()) < 1 {
            return;
        }
        let forward_target = (start as isize + player.forward_value() * 8) as usize;

        if squares_to_edge(start, Direction::West) >= 1 {
            let target = forward_target - 1;
            if !self.piece_at_idx(target).is_color(player) {
                map[target] = true;
            }
        }
        if squares_to_edge(start, Direction::East) >= 1 {
            let target = forward_target + 1;
            if !self.piece_at_idx(target).is_color(player) {
                map[target] = true;
            }
        }
    }

    fn get_attacked_king(&self, map: &mut [bool; 64], player: Player, start: usize) {
        for dir in Direction::ALL {
            if squares_to_edge(start, dir) >= 1 {
                let target = (start as isize + dir.offset()) as usize;
                let target_piece = self.piece_at_idx(target);

                // Block by friendly
                if target_piece.is_color(player) {
                    continue;
                }
                map[target] = true;
            }
        }
    }
}
