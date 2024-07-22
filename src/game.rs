use bevy::{color::palettes::css::*, prelude::*, sprite::Anchor};
use rand::seq::SliceRandom;
use rand::Rng;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, (setup_camera, spawn_player))
        .add_systems(
            Update,
            (
                update_positions,
                position_to_transform,
                draw_grid,
                draw_targets,
                draw_paths,
            ),
        )
        .observe(add_new_character);
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start_pos = rand_position();
    let target_pos = rand_position();

    commands.spawn((
        Player,
        SpriteBundle {
            texture: asset_server.load("ducky.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(CELL_SIZE)),
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_translation(Vec3::from((
                position_translation(&start_pos),
                0.,
            ))),
            ..default()
        },
        start_pos,
        Journey {
            path: Vec::from([start_pos]),
            index: 0,
            start_pos,
            target_pos,
            color: rand_color(),
        },
    ));
}

fn rand_position() -> Position {
    Position(IVec2::new(
        rand::thread_rng().gen_range(0..GRID_SIZE.x) as i32,
        rand::thread_rng().gen_range(0..GRID_SIZE.y) as i32,
    ))
}

fn rand_color() -> Color {
    const COLORS: [Srgba; 15] = [
        LIGHT_BLUE,
        LIGHT_CORAL,
        LIGHT_CYAN,
        LIGHT_GOLDENROD_YELLOW,
        LIGHT_GRAY,
        LIGHT_GREEN,
        LIGHT_GREY,
        LIGHT_PINK,
        LIGHT_SALMON,
        LIGHT_SEA_GREEN,
        LIGHT_SKY_BLUE,
        LIGHT_SLATE_GRAY,
        LIGHT_SLATE_GREY,
        LIGHT_STEEL_BLUE,
        LIGHT_YELLOW,
    ];

    let mut rng = rand::thread_rng();
    Color::Srgba(*COLORS.choose(&mut rng).unwrap())
}

const GRID_SIZE: UVec2 = UVec2::new(16, 9);
const CELL_SIZE: f32 = 80.;

#[derive(Debug, Component, Clone, Copy)]
struct Position(IVec2);

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct Automated;

#[derive(Debug, Component)]
struct Journey {
    start_pos: Position,
    target_pos: Position,
    path: Vec<Position>,
    index: usize,
    color: Color,
}

#[derive(Event)]
struct JourneyFinished;

fn update_positions(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut positions: Query<(Entity, &mut Position, &mut Journey), With<Player>>,
) {
    let Some(direction) = keyboard_direction(&keyboard) else {
        return;
    };

    for (entity, mut current_pos, mut journey) in positions.iter_mut() {
        let next_pos = (current_pos.0 + direction).clamp(IVec2::ZERO, GRID_SIZE.as_ivec2() - 1);

        if next_pos != current_pos.0 {
            current_pos.0 = next_pos;

            journey.path.push(*current_pos);
            journey.index += 1;

            if journey_finished(journey) {
                commands.trigger_targets(JourneyFinished, entity)
            }
        }
    }
}

fn journey_finished(journey: Mut<Journey>) -> bool {
    let Some(target_index) = journey
        .path
        .iter()
        .rposition(|pos| pos.0 == journey.target_pos.0)
    else {
        return false;
    };
    let Some(back_to_start_index) = journey
        .path
        .iter()
        .rposition(|pos| pos.0 == journey.start_pos.0)
    else {
        return false;
    };
    target_index < back_to_start_index
}

fn add_new_character(
    trigger: Trigger<JourneyFinished>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Current character becomes a bot
    commands
        .entity(trigger.entity())
        .remove::<Player>()
        .insert(Automated);

    spawn_player(commands, asset_server)
}

fn keyboard_direction(keyboard: &Res<ButtonInput<KeyCode>>) -> Option<IVec2> {
    const DIRECTIONS: [IVec2; 4] = [IVec2::Y, IVec2::NEG_Y, IVec2::NEG_X, IVec2::X];
    const ARROW_KEYS: [KeyCode; 4] = [
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ];
    const WASD_KEYS: [KeyCode; 4] = [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD];

    for keys in [ARROW_KEYS, WASD_KEYS] {
        if let Some((_, direction)) = keys
            .iter()
            .zip(DIRECTIONS.iter())
            .find(|(key, _)| keyboard.just_pressed(**key))
        {
            return Some(*direction);
        };
    }
    None
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
fn draw_targets(mut gizmos: Gizmos, journeys: Query<&Journey>) {
    for journey in journeys.iter() {
        gizmos.circle_2d(
            position_translation(&journey.target_pos) + CELL_SIZE / 2.,
            CELL_SIZE / 2. * 0.95,
            journey.color,
        );
    }
}
fn draw_paths(mut gizmos: Gizmos, journeys: Query<&Journey>) {
    for journey in journeys.iter() {
        for pos in journey.path.iter() {
            gizmos.rect_2d(
                position_translation(pos) + CELL_SIZE / 2.,
                0.,
                Vec2::splat(CELL_SIZE * 0.95),
                journey.color,
            );
        }
    }
}

fn position_to_transform(mut positions: Query<(&mut Transform, &Position), Changed<Position>>) {
    for (mut transform, pos) in positions.iter_mut() {
        transform.translation = Vec3::from((position_translation(pos), 0.));
    }
}

fn position_translation(pos: &Position) -> Vec2 {
    pos.0.as_vec2() * CELL_SIZE - GRID_SIZE.as_vec2() * CELL_SIZE / 2.
}
