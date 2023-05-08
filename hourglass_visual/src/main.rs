mod piece;
mod setup;

use bevy::prelude::*;
use bevy_editor_pls::EditorPlugin;
use bevy_mod_picking::low_latency_window_plugin;
use piece::PieceExt;
use setup::{Board, BoardPiece, MoveHint, MoveHintAssets, PickedPiece, SetupPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugin(EditorPlugin::default())
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_plugin(SetupPlugin)
        .add_system(update_pieces)
        .add_system(show_moves)
        .add_system(clear_moves)
        .run();
}

fn show_moves(
    board: Res<Board>,
    move_hint_assets: Res<MoveHintAssets>,
    mut q_move_hits: Query<(&mut Handle<Image>, &MoveHint)>,
    q_picked_piece: Query<&BoardPiece, (With<PickedPiece>, Added<PickedPiece>)>,
) {
    if let Ok(picked_piece) = q_picked_piece.get_single() {
        let mut moves = Vec::new();
        board.get_moves_for(&mut moves, picked_piece.idx);

        println!("MOVES: {:?}", moves);

        for (mut image, hint) in q_move_hits.iter_mut() {
            if moves.iter().any(|m| m.to() == hint.idx) {
                *image = move_hint_assets.open.clone();
            }
        }
    }
}

fn clear_moves(
    move_hint_assets: Res<MoveHintAssets>,
    mut q_move_hits: Query<&mut Handle<Image>, With<MoveHint>>,
    q_picked_piece: Query<&PickedPiece>,
) {
    if q_picked_piece.get_single().is_err() {
        for mut image in q_move_hits.iter_mut() {
            *image = move_hint_assets.none.clone();
        }
    }
}

fn update_pieces(board: Res<Board>, mut q_piece: Query<(&BoardPiece, &mut TextureAtlasSprite)>) {
    if !board.is_changed() {
        return;
    }

    for (piece, mut texture) in q_piece.iter_mut() {
        texture.index = board.piece_at(piece.idx).get_texture_idx();
    }
}
