use std::fmt::Display;

use crate::{
    idx_from_pos,
    pieces::{CastleRights, Piece, Player},
    Board,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FenPart {
    Board,
    ActiveColor,
    CastleRights,
    EnPassant,
    HalfMove,
    FullMove,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FenParseErr {
    MissingComponent {
        part: FenPart,
    },
    InvalidData {
        part: FenPart,
        char_idx: usize,
        err_msg: &'static str,
    },
    TooManyComponents,
}

impl FenParseErr {
    pub fn missing(part: FenPart) -> Self {
        FenParseErr::MissingComponent { part }
    }

    pub fn invalid(part: FenPart, char_idx: usize, err_msg: &'static str) -> Self {
        FenParseErr::InvalidData {
            part,
            char_idx,
            err_msg,
        }
    }
}

impl Display for FenParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenParseErr::MissingComponent { part } => write!(f, "missing component {:?}", part),
            FenParseErr::InvalidData {
                part,
                char_idx,
                err_msg,
            } => write!(
                f,
                "invalid data for part {:?}, at char {}: {}",
                part, char_idx, err_msg
            ),
            FenParseErr::TooManyComponents => write!(f, "too many components in the fen"),
        }
    }
}

impl std::error::Error for FenParseErr {}

fn piece_from_fen(c: char) -> Option<Piece> {
    let piece_type = match c.to_lowercase().next().unwrap() {
        'k' => Piece::King,
        'p' => Piece::Pawn,
        'n' => Piece::Knight,
        'b' => Piece::Bishop,
        'r' => Piece::Rook,
        'q' => Piece::Queen,
        _ => return None,
    };

    let player_type = if c.is_uppercase() {
        Piece::White
    } else {
        Piece::Black
    };

    Some(player_type | piece_type)
}

impl Board {
    pub fn load_fen(&mut self, fen: &str) -> Result<(), FenParseErr> {
        use FenPart::*;

        let mut split = fen.split(' ');
        let board = split.next().ok_or(FenParseErr::missing(Board))?;
        let active_color = split.next().ok_or(FenParseErr::missing(ActiveColor))?;
        let castling = split.next().ok_or(FenParseErr::missing(CastleRights))?;
        let en_passant = split.next().ok_or(FenParseErr::missing(EnPassant))?;
        let halfmove = split.next().ok_or(FenParseErr::missing(HalfMove))?;
        let fullmove = split.next().ok_or(FenParseErr::missing(FullMove))?;
        if split.next().is_some() {
            return Err(FenParseErr::TooManyComponents);
        }

        self.parse_board(board)?;
        self.parse_active_color(active_color)?;
        self.fun_name(castling)?;
        self.parse_en_passant(en_passant)?;
        self.parse_halfmove(halfmove)?;
        self.parse_fullmove(fullmove)?;

        Ok(())
    }

    fn parse_active_color(&mut self, active_color: &str) -> Result<(), FenParseErr> {
        self.active_color = match active_color {
            "w" => Player::White,
            "b" => Player::Black,
            _ => {
                return Err(FenParseErr::invalid(
                    FenPart::ActiveColor,
                    0,
                    "the active color must be 'w' or 'b'",
                ));
            }
        };
        Ok(())
    }

    fn parse_board(&mut self, board: &str) -> Result<(), FenParseErr> {
        let mut file: usize = 0;
        let mut rank: usize = 7;
        Ok(for (char_idx, c) in board.chars().enumerate() {
            if c == '/' {
                file = 0;
                rank -= 1;
                continue;
            }
            if let Some(num) = c.to_digit(10) {
                file += num as usize;
            } else {
                let piece = piece_from_fen(c).ok_or(FenParseErr::invalid(
                    FenPart::Board,
                    char_idx,
                    "invalid char",
                ))?;
                let square = self
                    .squares
                    .get_mut(rank * 8 + file)
                    .ok_or(FenParseErr::invalid(
                        FenPart::Board,
                        char_idx,
                        "overran board",
                    ))?;
                *square = piece;
                file += 1;
            }
        })
    }

    fn fun_name(&mut self, castling: &str) -> Result<(), FenParseErr> {
        for (c_idx, c) in castling.chars().enumerate() {
            match c {
                'K' => self.castle_rights |= CastleRights::WhiteKingSide,
                'Q' => self.castle_rights |= CastleRights::WhiteQueenSide,
                'k' => self.castle_rights |= CastleRights::BlackKingSide,
                'q' => self.castle_rights |= CastleRights::BlackQueenSide,
                _ => {
                    return Err(FenParseErr::invalid(
                        FenPart::CastleRights,
                        c_idx,
                        "character must be either 'K', 'Q', 'k', or 'q'",
                    ));
                }
            }
        }
        Ok(())
    }

    fn parse_en_passant(&mut self, en_passant: &str) -> Result<(), FenParseErr> {
        if en_passant == "-" {
            self.en_passant = None;
            return Ok(());
        }

        let idx = idx_from_pos(en_passant).ok_or(FenParseErr::invalid(
            FenPart::EnPassant,
            0,
            "the en passant section must be a board position or a '-'",
        ))?;

        self.en_passant = Some(idx);
        Ok(())
    }

    fn parse_halfmove(&mut self, halfmove: &str) -> Result<(), FenParseErr> {
        self.halfmove = halfmove.parse().map_err(|_| {
            FenParseErr::invalid(
                FenPart::HalfMove,
                0,
                "halfmove part of the fen should be an unsigned int",
            )
        })?;
        Ok(())
    }

    fn parse_fullmove(&mut self, fullmove: &str) -> Result<(), FenParseErr> {
        self.fullmove = fullmove.parse().map_err(|_| {
            FenParseErr::invalid(
                FenPart::FullMove,
                0,
                "fullmove part of the fen should be an unsigned int",
            )
        })?;
        Ok(())
    }
}
