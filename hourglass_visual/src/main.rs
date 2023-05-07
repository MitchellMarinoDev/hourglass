mod piece;
mod setup;

use bevy::prelude::*;
use bevy_editor_pls::EditorPlugin;
use bevy_mod_picking::low_latency_window_plugin;
use hourglass_engine::BoardIdx;
use piece::PieceExt;
use setup::{Board, BoardPiece, BoardSquare, SetupPlugin};

fn main() {
    let mut app = App::new();

    app.register_type::<BoardSquare>();

    app.add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugin(EditorPlugin::default())
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
        texture.index = board.piece_at(BoardIdx::unew(piece.idx)).get_texture_idx();
    }
}
