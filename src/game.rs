use bevy::{color::palettes::css::*, prelude::*, sprite::Anchor};
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
        .init_resource::<Characters>()
        .observe(add_new_character_on_finished_journey);
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Debug, Resource, Default)]
struct Characters {
    color_index: usize,
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut characters: ResMut<Characters>,
) {
    let start_pos = rand_position();
    let target_pos = rand_position();

    let color = Color::Srgba(COLORS[characters.color_index % COLORS.len()]);
    characters.color_index += 1;

    commands.spawn((
        Player,
        SpriteBundle {
            texture: asset_server.load("ducky.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(CELL_SIZE)),
                anchor: Anchor::BottomLeft,
                color,
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
            bot_index: 0,
            start_pos,
            target_pos,
            color,
            scale: rand::thread_rng().gen_range(0.6..0.9),
        },
    ));
}

fn rand_position() -> Position {
    Position(IVec2::new(
        rand::thread_rng().gen_range(0..GRID_SIZE.x) as i32,
        rand::thread_rng().gen_range(0..GRID_SIZE.y) as i32,
    ))
}

const COLORS: [Srgba; 11] = [
    AQUA, RED, BLUE, FUCHSIA, GREEN, LIME, NAVY, OLIVE, PURPLE, TEAL, YELLOW,
];

const GRID_SIZE: UVec2 = UVec2::new(8, 8);
const CELL_SIZE: f32 = 80.;

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
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
    bot_index: usize,
    // display
    color: Color,
    scale: f32,
}

#[derive(Event)]
struct JourneyFinished;

fn update_positions(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut positions: Query<(Entity, &mut Position, &mut Journey), With<Player>>,
    mut bot_positions: Query<(&mut Position, &mut Journey), (With<Automated>, Without<Player>)>,
) {
    let Some(direction) = keyboard_direction(&keyboard) else {
        return;
    };

    for (entity, mut current_pos, mut journey) in positions.iter_mut() {
        let next_pos = (current_pos.0 + direction).clamp(IVec2::ZERO, GRID_SIZE.as_ivec2() - 1);

        if next_pos != current_pos.0 {
            current_pos.0 = next_pos;

            if journey_finished(&journey, &current_pos) {
                commands.trigger_targets(JourneyFinished, entity)
            } else {
                journey.path.push(*current_pos);
            }

            // move all bots
            for (mut bot_pos, mut bot_journey) in bot_positions.iter_mut() {
                bot_journey.bot_index = (bot_journey.bot_index + 1) % bot_journey.path.len();
                *bot_pos = bot_journey.path[bot_journey.bot_index]
            }
        }
    }
}

fn journey_finished(journey: &Journey, current_pos: &Position) -> bool {
    let has_reached_target = journey.path.contains(&journey.target_pos);
    let back_to_start = *current_pos == journey.start_pos;
    has_reached_target && back_to_start
}

fn add_new_character_on_finished_journey(
    trigger: Trigger<JourneyFinished>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprites: Query<&mut Sprite>,
    characters: ResMut<Characters>,
) {
    // Current character becomes a bot
    commands
        .entity(trigger.entity())
        .remove::<Player>()
        .insert(Automated);

    sprites
        .get_mut(trigger.entity())
        .expect("Bot sprite")
        .color
        .set_alpha(0.3);

    spawn_player(commands, asset_server, characters);
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
            position_translation(&journey.start_pos) + CELL_SIZE / 2.,
            CELL_SIZE / 2. * journey.scale,
            journey.color,
        );
        gizmos.circle_2d(
            position_translation(&journey.target_pos) + CELL_SIZE / 2.,
            CELL_SIZE / 2. * journey.scale,
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
                Vec2::splat(CELL_SIZE * journey.scale),
                journey.color.with_alpha(0.05),
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
