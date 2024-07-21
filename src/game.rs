use bevy::{prelude::*, sprite::Anchor};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, (update_positions, position_to_transform, draw_grid));
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Player,
        SpriteBundle {
            texture: asset_server.load("ducky.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(CELL_SIZE)),
                anchor: Anchor::BottomLeft,
                ..default()
            },
            ..default()
        },
        Position(IVec2::new(5, 5)),
    ));
}

const DIRECTIONS: [IVec2; 4] = [IVec2::Y, IVec2::NEG_Y, IVec2::NEG_X, IVec2::X];
const ARROW_KEYS: [KeyCode; 4] = [
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
];
const WASD_KEYS: [KeyCode; 4] = [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD];
const GRID_SIZE: UVec2 = UVec2::new(16, 9);
const CELL_SIZE: f32 = 80.;

#[derive(Debug, Component, Clone, Copy)]
struct Position(IVec2);

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct Automated;

#[derive(Debug, Component)]
struct Travel {
    path: Vec<Position>,
    current_index: usize,
}

fn update_positions(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut positions: Query<&mut Position, With<Player>>,
) {
    for mut pos in positions.iter_mut() {
        let Some((_, direction)) = ARROW_KEYS
            .iter()
            .zip(DIRECTIONS.iter())
            .find(|(key, _)| keyboard.just_pressed(**key))
        else {
            continue;
        };

        pos.0 = (pos.0 + *direction).clamp(IVec2::ZERO, GRID_SIZE.as_ivec2() - 1);
    }
}

fn draw_grid(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Vec2::ZERO,
            0.0,
            GRID_SIZE,
            Vec2::new(CELL_SIZE, CELL_SIZE),
            LinearRgba::gray(0.05),
        )
        .outer_edges();
}

fn position_to_transform(mut positions: Query<(&mut Transform, &Position), Changed<Position>>) {
    for (mut transform, pos) in positions.iter_mut() {
        transform.translation = Vec3::from((
            pos.0.as_vec2() * CELL_SIZE - GRID_SIZE.as_vec2() * CELL_SIZE / 2.,
            transform.translation.z,
        ));
    }
}
