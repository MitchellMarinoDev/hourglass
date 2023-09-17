use rand::Rng;

use crate::{Board, Move, Piece, Player};

impl Board {
    pub fn get_best_move<'b, 'v>(&'b self, depth: u32, scoring: fn(&Board) -> f32) -> Option<Move> {
        let moves = self.generate_moves();
        let (idx, _score) = self.search(depth, &moves, scoring);
        return moves.get(idx).copied();
    }

    pub fn search<'b, 'v>(
        &'b self,
        depth: u32,
        moves: &'v Vec<Move>,
        scoring: fn(&Board) -> f32,
    ) -> (usize, f32) {
        if depth == 0 {
            return (0, scoring(self));
        }

        if moves.is_empty() {
            if self.is_in_check(self.active_color()) {
                return (0, f32::NEG_INFINITY);
            } else {
                return (0, 0.0);
            }
        }

        let mut i = 0;
        let mut best_score = f32::NEG_INFINITY;

        for (idx, umove) in moves.iter().enumerate() {
            let mut board = self.clone();
            let _ = board.make_simple_move(*umove);
            let score = -board.search(depth - 1, &board.generate_moves(), scoring).1;
            if score > best_score {
                best_score = score;
                i = idx;
            }
        }

        (i, best_score)
    }

    pub fn bogo_score(&self) -> f32 {
        let mut rng = rand::thread_rng();
        rng.gen()
    }

    pub fn score_material(&self) -> i32 {
        let current_color_mult = if self.active_color() == Player::Black {
            -1
        } else {
            1
        };
        self.squares.iter().map(Piece::score_value).sum::<i32>() * current_color_mult
    }
}
