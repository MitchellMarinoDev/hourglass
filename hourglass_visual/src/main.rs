mod piece;
mod setup;

use bevy::prelude::*;
use piece::PieceExt;
use setup::{Board, BoardPiece, SetupPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_plugin(SetupPlugin)
        .add_system(update_pieces)
        .run();
}

fn update_pieces(board: Res<Board>, mut q_piece: Query<(&BoardPiece, &mut TextureAtlasSprite)>) {
    if !board.is_changed() {
        return;
    }

    for (piece, mut texture) in q_piece.iter_mut() {
        texture.index = board.squares[piece.idx].get_texture_idx();
    }
}
