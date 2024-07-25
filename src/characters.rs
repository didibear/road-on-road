use crate::components::*;
use crate::movements;
use crate::CELL_SIZE;
use crate::GRID_SIZE;
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::seq::SliceRandom;
use rand::Rng;

const COLORS: [Srgba; 5] = [YELLOW, AQUA, RED, FUCHSIA, LIME];

#[derive(Debug, Resource, Default)]
pub struct Characters {
    pub color_index: usize,
}

pub fn spawn_player(
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

pub fn character_sprite(
    asset_server: &AssetServer,
    color: Color,
    start_pos: Position,
) -> SpriteBundle {
    SpriteBundle {
        texture: asset_server.load("ducky.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::splat(CELL_SIZE)),
            anchor: Anchor::Center,
            color,
            ..default()
        },
        transform: Transform::from_translation(Vec3::from((
            movements::position_translation(&start_pos),
            0.,
        ))),
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

pub fn add_new_character_on_finished_journey(
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
