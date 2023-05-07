use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Piece: u8 {
        const None = 0;
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

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CastleRights: u8 {
        const None = 0;
        const WhiteKingSide = 1<<0;
        const WhiteQueenSide = 1<<1;
        const BlackKingSide = 1<<2;
        const BlackQueenSide = 1<<3;
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

/// An Unchecked Move.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct UMove {
    from: usize,
    to: usize,
}

impl UMove {
    pub fn from_idxs(from: usize, to: usize) -> Self {
        UMove { from, to }
    }

    pub fn new(str: &str) -> Option<Self> {
        if str.len() != 4 {
            return None;
        }

        let (from, to) = str.split_at(2);
        let from = idx_from_move(from)?;
        let to = idx_from_move(to)?;

        Some(UMove { from, to })
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
            squares: [Piece::None; 64],
            castle_rights: CastleRights::None,
            active_color: Player::White,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
        }
    }

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

    pub fn try_move(&mut self, umove: UMove) -> Result<(), InvalidMoveErr> {
        // check if the player owns the piece they are trying to move
        if !matches!(self.squares.get(umove.from), Some(piece) if *piece | self.active_color.to_piece_color() != Piece::None)
        {
            return Err(InvalidMoveErr::NotYourPiece);
        }

        // TODO: check if it is a vlid move

        self.squares[umove.to] = self.squares[umove.from];
        self.squares[umove.from] = Piece::None;

        Ok(())
    }

    pub fn piece_at(&self, idx: usize) -> Option<Piece> {
        self.squares.get(idx).copied()
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
                let piece = Self::piece_from_fen(c).ok_or(FenParseErr::invalid(
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
                'K' => self.castle_rights |= crate::CastleRights::WhiteKingSide,
                'Q' => self.castle_rights |= crate::CastleRights::WhiteQueenSide,
                'k' => self.castle_rights |= crate::CastleRights::BlackKingSide,
                'q' => self.castle_rights |= crate::CastleRights::BlackQueenSide,
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

        let idx = idx_from_move(en_passant).ok_or(FenParseErr::invalid(
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

fn idx_from_move(move_str: &str) -> Option<usize> {
    let mut move_chars = move_str.chars();
    let mut idx = 0;

    match move_chars.next() {
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

    match move_chars.next().map(|c| c.to_digit(10)) {
        Some(Some(v)) if (0..8).contains(&v) => idx += v as usize,
        _ => return None,
    }

    Some(idx)
}
