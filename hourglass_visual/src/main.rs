mod piece;
mod setup;

use bevy::prelude::*;
use bevy_mod_picking::low_latency_window_plugin;
use piece::PieceExt;
use setup::{Board, BoardPiece, PickedPiece, SetupPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
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
