use bevy::prelude::*;
use bitflags::bitflags;

const SQUARE_SIZE: f32 = 50.;

#[derive(Resource)]
struct PieceImages {
    atlas: TextureAtlas,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let piece_img = asset_server.load("chess-pieces.png");
    let atlas = TextureAtlas::from_grid(piece_img, Vec2::splat(333.), 6, 2, None, None);
    commands.insert_resource(PieceImages { atlas });

    let dark_color = Color::rgb(0.2, 0.6, 0.9);
    let light_color = Color::rgb(0.9, 0.95, 1.);

    for file in 0..8 {
        for rank in 0..8 {
            let light_square = (file + rank) % 2 == 0;
            let color = if light_square {
                light_color
            } else {
                dark_color
            };

            spawn_square(
                &mut commands,
                color,
                Vec2::new(
                    -4. * SQUARE_SIZE + file as f32 * SQUARE_SIZE,
                    -4. * SQUARE_SIZE + rank as f32 * SQUARE_SIZE,
                ),
            )
        }
    }
}

fn spawn_square(commands: &mut Commands, color: Color, pos: Vec2) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(50., 50.)),
            ..default()
        },
        transform: Transform::from_translation(pos.extend(0.)),
        ..default()
    });
}
