use std::f32::consts::PI;

use bevy::{color::palettes::css::*, prelude::*, sprite::Anchor};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::WINDOW_SIZE;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, (setup_camera, spawn_player))
        .add_systems(
            Update,
            (
                // logic
                (
                    handle_input_movement,
                    move_transit_entities,
                    detect_collisions,
                    destroyed_animation,
                ),
                // drawing
                position_to_transform,
                (draw_grid, draw_paths),
                draw_targets,
            )
                .chain(),
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
    positions: Query<&Position>,
) {
    let avoid_positions: Vec<&Position> = positions.iter().collect();
    let (start_pos, target_pos) = rand_journey_target(avoid_positions);

    let color = Color::Srgba(COLORS[characters.color_index % COLORS.len()]);
    characters.color_index += 1;

    commands.spawn((
        Player,
        character_sprite(&asset_server, color, start_pos),
        start_pos,
        Journey {
            path: Vec::new(),
            bot_index: 0,
            start_pos,
            target_pos,
            color,
            scale: rand::thread_rng().gen_range(0.6..0.9),
        },
    ));
}

fn character_sprite(asset_server: &AssetServer, color: Color, start_pos: Position) -> SpriteBundle {
    SpriteBundle {
        texture: asset_server.load("ducky.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::splat(CELL_SIZE)),
            anchor: Anchor::Center,
            color,
            ..default()
        },
        transform: Transform::from_translation(Vec3::from((position_translation(&start_pos), 0.))),
        ..default()
    }
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Top,
    Down,
    Left,
    Right,
}

impl Side {
    fn rand_position(self) -> Position {
        let mut rng = rand::thread_rng();
        let (x, y) = match self {
            Self::Top => (rng.gen_range(1..GRID_SIZE.x - 1), (GRID_SIZE.y - 1)),
            Self::Down => (rng.gen_range(1..GRID_SIZE.x - 1), 0),
            Self::Left => (0, rng.gen_range(1..GRID_SIZE.y - 1)),
            Self::Right => ((GRID_SIZE.x - 1), rng.gen_range(1..GRID_SIZE.y - 1)),
        };
        Position(IVec2::new(x as i32, y as i32))
    }
}

fn rand_journey_target(avoid_positions: Vec<&Position>) -> (Position, Position) {
    let mut rng = rand::thread_rng();

    loop {
        let sides: Vec<Side> = [Side::Top, Side::Down, Side::Left, Side::Right]
            .choose_multiple(&mut rng, 2)
            .cloned()
            .collect();

        let start_pos = sides[0].rand_position();
        let target_pos = sides[1].rand_position();

        if avoid_positions.contains(&&start_pos) {
            continue;
        }
        return (start_pos, target_pos);
    }
}

const COLORS: [Srgba; 5] = [YELLOW, AQUA, RED, FUCHSIA, LIME];

const GRID_SIZE: UVec2 = UVec2::new(6, 6);
const CELL_SIZE: f32 = WINDOW_SIZE / 10.;

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
struct Position(IVec2);

#[derive(Debug, Component, Clone, Copy, PartialEq)]
struct Transition {
    start: Position,
    end: Position,
    current: Vec2,
}
impl Transition {
    fn new(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
            current: start.0.as_vec2(),
        }
    }
}
#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct Automated;

#[derive(Debug, Component)]
struct Destroyed;

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

fn handle_input_movement(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut positions: Query<(Entity, &Position, &mut Journey), (With<Player>, Without<Transition>)>,
    mut bot_positions: Query<(Entity, &Position, &mut Journey), (With<Automated>, Without<Player>)>,
) {
    let Some(direction) = keyboard_direction(&keyboard) else {
        return;
    };

    for (entity, current_pos, mut journey) in positions.iter_mut() {
        let next_pos =
            Position((current_pos.0 + direction).clamp(IVec2::ZERO, GRID_SIZE.as_ivec2() - 1));

        if next_pos != *current_pos {
            commands
                .entity(entity)
                .insert(Transition::new(*current_pos, next_pos));

            journey.path.push(*current_pos);

            // TODO: Manage journey finished after transition
            if journey_finished(&journey, &next_pos) {
                commands.trigger_targets(JourneyFinished, entity)
            }

            for (entity, bot_pos, mut bot_journey) in bot_positions.iter_mut() {
                bot_journey.bot_index = (bot_journey.bot_index + 1) % bot_journey.path.len();

                commands.entity(entity).insert(Transition::new(
                    *bot_pos,
                    bot_journey.path[bot_journey.bot_index],
                ));
            }
        }
    }
}

const SPEED: f32 = CELL_SIZE / 6.;

fn move_transit_entities(
    mut commands: Commands,
    mut transitions: Query<(Entity, &mut Transition, &mut Position)>,
    time: Res<Time>,
) {
    // current_pos.0 = next_pos;
    for (entity, mut transition, mut pos) in transitions.iter_mut() {
        let direction =
            (transition.end.0.as_vec2() - transition.start.0.as_vec2()).normalize_or_zero();

        transition.current += SPEED * direction * time.delta_seconds();

        if transition.start.0.as_vec2().distance(transition.current) >= 1. {
            commands.entity(entity).remove::<Transition>();
            *pos = transition.end;
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
    positions: Query<&Position>,
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

    spawn_player(commands, asset_server, characters, positions);
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

fn detect_collisions(
    mut commands: Commands,
    locations: Query<(Entity, &Transform, &Journey)>,
    players: Query<Entity, With<Player>>,
    journeys: Query<&Journey>,
    asset_server: Res<AssetServer>,
    mut sprites: Query<&mut Sprite>,
) {
    for [(entity_a, transform_a, journey_a), (entity_b, transform_b, _)] in
        locations.iter_combinations()
    {
        if transform_a.translation.distance(transform_b.translation) <= CELL_SIZE / 2. {
            let destroyed_entity = {
                // player die first
                if players.get(entity_a).is_ok() {
                    // unless they just spawn
                    if journey_a.path.len() <= 1 {
                        continue;
                    }
                    entity_a
                } else {
                    entity_b
                }
            };

            commands
                .entity(destroyed_entity)
                .remove::<(Position, Player, Automated)>()
                .insert(Destroyed);

            let was_player = players.get(destroyed_entity).is_ok();
            let journey = journeys.get(destroyed_entity).expect("Journey on destroy");

            if was_player {
                sprites
                    .get_mut(destroyed_entity)
                    .expect("Bot sprite")
                    .color
                    .set_alpha(0.3);

                commands.spawn((
                    Player,
                    character_sprite(&asset_server, journey.color, journey.start_pos),
                    journey.start_pos,
                    Journey {
                        path: Vec::new(),
                        ..*journey
                    },
                ));
            }
        }
    }
}

const DESTROYED_SPEED: f32 = CELL_SIZE * 8.;
const DESTROYED_ROTATION: f32 = 10.;

const SCRAPYARD_LOCATION: Vec2 = Vec2::splat(WINDOW_SIZE * 0.4);

fn destroyed_animation(
    mut commands: Commands,
    mut transforms: Query<(Entity, &mut Transform), With<Destroyed>>,
    time: Res<Time>,
) {
    for (entity, mut transform) in transforms.iter_mut() {
        // let destination = Vec3::from((SCRAPYARD_LOCATION, 0.));
        let destination =
            (transform.translation - Vec3::ZERO).normalize_or(Vec3::X) * WINDOW_SIZE * 0.6;

        let noise = Vec3::from((random_noise(CELL_SIZE), 0.));

        let direction = ((destination + noise) - transform.translation).normalize_or_zero();
        transform.translation += direction * DESTROYED_SPEED * time.delta_seconds();
        transform.rotate_local_z(-PI * DESTROYED_ROTATION * time.delta_seconds());

        // outside of the grid
        let margin = CELL_SIZE * 1.5;
        let playground_size = GRID_SIZE.as_vec2() * 0.5 * CELL_SIZE;

        let playground = Rect {
            min: -(playground_size + margin + random_noise(CELL_SIZE)),
            max: playground_size + margin + random_noise(CELL_SIZE),
        };
        if !playground.contains(transform.translation.xy()) {
            commands.entity(entity).remove::<Destroyed>();
        }
    }
}

fn random_noise(length: f32) -> Vec2 {
    let mut rng = rand::thread_rng();

    Vec2::new(rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5).normalize_or(Vec2::X) * length
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
fn draw_targets(mut gizmos: Gizmos, journeys: Query<&Journey, With<Player>>) {
    for journey in journeys.iter() {
        gizmos.circle_2d(
            position_translation(&journey.start_pos) + CELL_SIZE / 2.,
            CELL_SIZE / 2. * 1.1,
            journey.color,
        );
        gizmos.circle_2d(
            position_translation(&journey.target_pos) + CELL_SIZE / 2.,
            CELL_SIZE / 2. * 1.1,
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

fn position_to_transform(
    mut changed_position: Query<
        (&mut Transform, &Position),
        (Changed<Position>, Without<Transition>),
    >,
    mut in_transition: Query<(&mut Transform, &Transition), Changed<Transition>>,
) {
    for (mut transform, pos) in changed_position.iter_mut() {
        transform.translation = Vec3::from((sprite_position_translation(pos.0.as_vec2()), 0.));
    }
    for (mut transform, transition) in in_transition.iter_mut() {
        transform.translation = Vec3::from((sprite_position_translation(transition.current), 0.));
    }
}

fn position_translation(pos: &Position) -> Vec2 {
    grid_pos_translation(pos.0.as_vec2())
}

fn sprite_position_translation(pos: Vec2) -> Vec2 {
    grid_pos_translation(pos) + CELL_SIZE / 2.
}

fn grid_pos_translation(pos: Vec2) -> Vec2 {
    pos * CELL_SIZE - GRID_SIZE.as_vec2() * CELL_SIZE / 2.
}
