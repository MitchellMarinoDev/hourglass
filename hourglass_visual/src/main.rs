mod piece;
mod setup;

use bevy::prelude::*;
use bevy_editor_pls::EditorPlugin;
use bevy_mod_picking::low_latency_window_plugin;
use hourglass_engine::Move;
use piece::PieceExt;
use setup::{
    Board, BoardPiece, BoardSquare, MoveHint, MoveHintAssets, PickedPiece, PromotionMenu,
    SetupPlugin,
};

#[derive(Debug, Copy, Clone, Resource, Default)]
struct PromotingPiece {
    o_move: Option<Move>,
}

fn main() {
    let mut app = App::new();

    app.insert_resource(PromotingPiece::default())
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugin(EditorPlugin::default())
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_plugin(SetupPlugin)
        .add_system(update_pieces)
        .add_system(show_moves)
        .add_system(clear_moves)
        .add_system(show_promotion_options)
        // .add_system(show_attacked_squares)
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

        for (mut image, hint) in q_move_hits.iter_mut() {
            if moves.iter().any(|m| m.to() == hint.idx) {
                let new_image = if board.piece_at_idx(hint.idx).is_color(!board.active_color()) {
                    move_hint_assets.take.clone()
                } else {
                    move_hint_assets.open.clone()
                };
                *image = new_image;
            }
        }
    }
}

fn show_attacked_squares(
    board: Res<Board>,
    mut q_board_squares: Query<(&BoardSquare, &mut Sprite)>,
) {
    if !board.is_changed() {
        return;
    }

    let attacked_squares = board.generate_attacks(!board.active_color());

    for (square, mut sprite) in q_board_squares.iter_mut() {
        let idx = square.idx;
        let attacked = attacked_squares[idx];
        let light_square = ((idx / 8) + idx) % 2 == 0;

        sprite.color = match (attacked, light_square) {
            (true, true) => Color::hex("#FFF2FF"),
            (true, false) => Color::hex("#9399E5"),
            (false, true) => Color::hex("#E5F2FF"),
            (false, false) => Color::hex("#3399E5"),
        }
        .unwrap();
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
        texture.index = board.piece_at_idx(piece.idx).get_texture_idx();
    }
}

fn show_promotion_options(
    promoting_piece: ResMut<PromotingPiece>,
    mut q_promotion_menu: Query<&mut Visibility, With<PromotionMenu>>,
) {
    if !promoting_piece.is_changed() {
        return;
    }

    match promoting_piece.o_move {
        Some(_) => {
            for mut vis in q_promotion_menu.iter_mut() {
                *vis = Visibility::Visible;
            }
        }
        None => {
            for mut vis in q_promotion_menu.iter_mut() {
                *vis = Visibility::Hidden;
            }
        }
    };
}
