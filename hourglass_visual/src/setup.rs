use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use hourglass_engine::Piece;

use crate::piece::PieceExt;

const SQUARE_SIZE: f32 = 100.;

pub(crate) struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub(crate) struct Board(hourglass_engine::Board);

impl Board {
    pub fn new() -> Self {
        Board(hourglass_engine::Board::new())
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BoardPiece {
    pub(crate) idx: usize,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    let piece_img = asset_server.load("chess-pieces.png");
    let atlas = TextureAtlas::from_grid(piece_img, Vec2::splat(333.), 6, 3, None, None);
    let atlas_handle = texture_atlases.add(atlas);

    let dark_color = Color::rgb(0.2, 0.6, 0.9);
    let light_color = Color::rgb(0.9, 0.95, 1.);

    commands.insert_resource(Board::new());

    spawn_board(&mut commands, light_color, dark_color);
    spawn_pieces(&mut commands, atlas_handle);
}

fn spawn_pieces(commands: &mut Commands, texture_atlas: Handle<TextureAtlas>) {
    for file in 0..8 {
        for rank in 0..8 {
            let pos = Vec2::new(
                -3.5 * SQUARE_SIZE + file as f32 * SQUARE_SIZE,
                -3.5 * SQUARE_SIZE + rank as f32 * SQUARE_SIZE,
            );

            let sprite_idx = Piece::Black.get_texture_idx();
            commands.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        index: sprite_idx,
                        ..default()
                    },
                    transform: Transform::from_translation(pos.extend(0.)),
                    texture_atlas: texture_atlas.clone(),
                    ..default()
                },
                BoardPiece {
                    idx: rank * 8 + file,
                },
                PickableBundle::default(),
            ));
        }
    }
}

fn spawn_board(commands: &mut Commands, light_color: Color, dark_color: Color) {
    for file in 0..8 {
        for rank in 0..8 {
            let light_square = (file + rank) % 2 == 1;
            let color = if light_square {
                light_color
            } else {
                dark_color
            };

            spawn_square(
                commands,
                color,
                Vec2::new(
                    -3.5 * SQUARE_SIZE + file as f32 * SQUARE_SIZE,
                    -3.5 * SQUARE_SIZE + rank as f32 * SQUARE_SIZE,
                ),
            )
        }
    }
}

fn spawn_square(commands: &mut Commands, color: Color, pos: Vec2) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
            ..default()
        },
        transform: Transform::from_translation(pos.extend(0.)),
        ..default()
    });
}
