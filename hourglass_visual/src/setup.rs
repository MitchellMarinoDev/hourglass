use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use hourglass_engine::InvalidMoveErr;
use hourglass_engine::Move;
use hourglass_engine::Piece;
use hourglass_engine::Player;

use crate::piece::PieceExt;
use crate::PromotingPiece;

const SQUARE_SIZE: f32 = 100.;

pub(crate) enum InputSource {
    Bot {
        score: fn(&hourglass_engine::Board) -> f32,
        depth: u32,
    },
    Human,
}

#[derive(Resource)]
pub(crate) struct InputSourceWhite(InputSource);
#[derive(Resource)]
pub(crate) struct InputSourceBlack(InputSource);

pub(crate) struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .insert_resource(InputSourceWhite(InputSource::Human))
            .insert_resource(InputSourceBlack(InputSource::Bot {
                score: hourglass_engine::Board::bogo_score,
                depth: 4,
            }))
            .add_system(bot_move);
    }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub(crate) struct Board(hourglass_engine::Board);

impl Board {
    pub fn new() -> Self {
        Board(hourglass_engine::Board::new())
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct BoardPiece {
    pub(crate) idx: usize,
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MoveHintAssets {
    pub none: Handle<Image>,
    pub open: Handle<Image>,
    pub take: Handle<Image>,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct MoveHint {
    pub(crate) idx: usize,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct BoardSquare {
    pub(crate) idx: usize,
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PickedPiece;

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PromotionMenu(Piece);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((Camera2dBundle::default(), RaycastPickCamera::default()));

    let piece_img = asset_server.load("chess-pieces.png");
    let atlas = TextureAtlas::from_grid(piece_img, Vec2::splat(333.), 6, 3, None, None);
    let atlas_handle = texture_atlases.add(atlas);

    let move_hint_assets = MoveHintAssets {
        none: asset_server.load("empty.png"),
        open: asset_server.load("dot.png"),
        take: asset_server.load("circle.png"),
    };
    commands.insert_resource(move_hint_assets.clone());

    let dark_color = Color::hex("#3399E5").unwrap();
    let light_color = Color::hex("#E5F2FF").unwrap();

    commands.insert_resource(Board::new());

    spawn_promotion_menu(&mut commands, atlas_handle.clone(), &mut meshes);
    spawn_board(&mut commands, light_color, dark_color, &mut meshes);
    spawn_pieces(&mut commands, atlas_handle);
    spawn_move_hints(&mut commands, move_hint_assets);
}

fn bot_move(
    mut board: ResMut<Board>,
    input_white: Res<InputSourceWhite>,
    input_black: Res<InputSourceBlack>,
) {
    if board.active_color() == Player::White {
        match input_white.0 {
            InputSource::Human => {}
            InputSource::Bot { score, depth } => {
                let umove = board.get_best_move(depth, score).unwrap();
                match board.try_move(umove) {
                    Ok(()) => {}
                    Err(e) => error!("Error playing best move {:?}", e),
                }
            }
        }
    } else {
        match input_black.0 {
            InputSource::Human => {}
            InputSource::Bot { score, depth } => {
                let umove = board.get_best_move(depth, score).unwrap();
                match board.try_move(umove) {
                    Ok(()) => {}
                    Err(e) => error!("Error playing best move {:?}", e),
                }
            }
        }
    }
}

fn spawn_promotion_menu(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::splat(100.))));

    let pieces: [Piece; 4] = [
        Piece::White | Piece::Bishop,
        Piece::White | Piece::Rook,
        Piece::White | Piece::Knight,
        Piece::White | Piece::Queen,
    ];

    for (idx, piece) in pieces.into_iter().enumerate() {
        commands.spawn((
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                    index: piece.get_texture_idx(),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    -475.0,
                    (idx as f32 - 1.5) * 100.0,
                    0.0,
                )),
                texture_atlas: texture_atlas.clone(),
                ..default()
            },
            mesh.clone(),
            PromotionMenu(piece),
            Name::new("PromotionMenu"),
            RaycastPickTarget::default(),
            PickableBundle::default(),
            OnPointer::<Click>::run_callback(select_promotion),
        ));
    }
}

fn select_promotion(
    In(event): In<ListenedEvent<Click>>,
    q_promotion_menu: Query<&PromotionMenu>,
    mut promoting_piece: ResMut<PromotingPiece>,
    mut board: ResMut<Board>,
) -> Bubble {
    let piece = q_promotion_menu.get(event.target).unwrap().0;

    let move_result = board.try_move(
        promoting_piece
            .o_move
            .unwrap()
            .with_promote(Some(piece & Piece::PieceType)),
    );

    match move_result {
        Ok(()) => {}
        Err(e) => {
            warn!("Error moving piece: {e:?}");
        }
    }

    promoting_piece.o_move = None;

    Bubble::Burst
}

fn spawn_move_hints(commands: &mut Commands, move_hint_assets: MoveHintAssets) {
    for file in 0..8 {
        for rank in 0..8 {
            let pos = Vec2::new(
                -3.5 * SQUARE_SIZE + file as f32 * SQUARE_SIZE,
                -3.5 * SQUARE_SIZE + rank as f32 * SQUARE_SIZE,
            );

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(pos.extend(0.)),
                    texture: move_hint_assets.none.clone(),
                    ..default()
                },
                MoveHint {
                    idx: rank * 8 + file,
                },
            ));
        }
    }
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
            ));
        }
    }
}

fn spawn_board(
    commands: &mut Commands,
    light_color: Color,
    dark_color: Color,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::splat(100.))));

    for file in 0..8 {
        for rank in 0..8 {
            let light_square = (file + rank) % 2 == 1;
            let color = if light_square {
                light_color
            } else {
                dark_color
            };

            spawn_square(commands, color, file, rank, mesh.clone())
        }
    }
}

fn spawn_square(
    commands: &mut Commands,
    color: Color,
    file: usize,
    rank: usize,
    mesh: Handle<Mesh>,
) {
    let pos = Vec2::new(
        -3.5 * SQUARE_SIZE + file as f32 * SQUARE_SIZE,
        -3.5 * SQUARE_SIZE + rank as f32 * SQUARE_SIZE,
    );

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.)),
            ..default()
        },
        BoardSquare {
            idx: rank * 8 + file,
        },
        mesh,
        RaycastPickTarget::default(),
        PickableBundle::default(),
        OnPointer::<DragStart>::run_callback(pickup_piece),
        OnPointer::<DragEnd>::run_callback(drag_end),
        OnPointer::<Drag>::run_callback(move_piece),
        OnPointer::<Drop>::run_callback(drop_piece_on),
    ));
}

fn pickup_piece(
    In(event): In<ListenedEvent<DragStart>>,
    mut commands: Commands,
    q_piece: Query<(Entity, &BoardPiece)>,
    q_board: Query<&BoardSquare>,
) -> Bubble {
    // make sure the entity doesn't get picked twice
    // rememeber to add it back
    commands.entity(event.target).remove::<Pickable>();

    let board_square = match q_board.get(event.target) {
        Ok(bs) => bs,
        Err(_) => {
            warn!("Target of drag start is not a board square");
            return Bubble::Up;
        }
    };

    for (entity, piece) in q_piece.iter() {
        if piece.idx == board_square.idx {
            commands.entity(entity).insert(PickedPiece);
        }
    }
    Bubble::Burst
}

fn drag_end(
    In(event): In<ListenedEvent<DragEnd>>,
    mut commands: Commands,
    mut q_picked_piece: Query<(Entity, &BoardPiece, &mut Transform), With<PickedPiece>>,
) -> Bubble {
    // we removed this when we picked up the piece.
    // we add it back now.
    commands.entity(event.target).insert(Pickable);

    if let Ok((entity, piece, mut transform)) = q_picked_piece.get_single_mut() {
        commands.entity(entity).remove::<PickedPiece>();
        // figure out a place to put the piece and change the board to reflect
        //     that
        let rank = piece.idx / 8;
        let file = piece.idx % 8;
        let pos = Vec3::new(
            -3.5 * SQUARE_SIZE + file as f32 * SQUARE_SIZE,
            -3.5 * SQUARE_SIZE + rank as f32 * SQUARE_SIZE,
            0.,
        );
        transform.translation = pos;
    }

    Bubble::Burst
}

fn move_piece(
    In(event): In<ListenedEvent<Drag>>,
    mut q_picked_piece: Query<&mut Transform, With<PickedPiece>>,
) -> Bubble {
    if let Ok(mut piece_transform) = q_picked_piece.get_single_mut() {
        piece_transform.translation += event.pointer_event.delta.extend(0.);
    }
    Bubble::Burst
}

fn drop_piece_on(
    In(event): In<ListenedEvent<Drop>>,
    mut promoting_piece: ResMut<PromotingPiece>,
    mut board: ResMut<Board>,
    q_board_square: Query<&BoardSquare>,
) -> Bubble {
    let from_square = match q_board_square.get(event.dropped) {
        Ok(dp) => dp,
        Err(_) => {
            info!("the object dropped on this was not a board square");
            return Bubble::Up;
        }
    };
    let this = q_board_square
        .get(event.target)
        .expect("this should be called on a board square");

    let umove = Move::from_idxs(from_square.idx, this.idx);
    match board.try_move(umove) {
        Ok(()) => {}
        Err(InvalidMoveErr::NoPromotion) => {
            promoting_piece.o_move = Some(umove);
        }
        Err(_) => {}
    }

    Bubble::Burst
}
