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
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Board {
    pub squares: [Piece; 64],
}
