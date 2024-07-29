use crate::components::*;
use crate::game_state::GameState;
use crate::movements;
use crate::scores::Score;
use crate::scores::NB_ATTEMPTS;
use crate::tutorial;
use crate::tutorial::FirstPlayerAdded;
use crate::AllAssets;
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

pub fn spawn_first_player(
    mut commands: Commands,
    assets: Res<AllAssets>,
    characters: ResMut<Characters>,
    positions: Query<&Position>,
    journeys: Query<&Journey>,
) {
    let character = character_bundle(positions, journeys.iter().collect(), characters, assets)
        .expect("First player");

    let player = commands.spawn(character).insert(tutorial::FirstPlayer).id();

    commands.trigger_targets(FirstPlayerAdded, player)
}

fn character_bundle(
    character_positions: Query<&Position>,
    journeys: Vec<&Journey>,
    mut characters: ResMut<Characters>,
    assets: Res<AllAssets>,
) -> Option<impl Bundle> {
    let spawn_positions = journeys.iter().map(|journey| &journey.start_pos);
    let avoid_positions: Vec<&Position> =
        character_positions.iter().chain(spawn_positions).collect();
    let (start_pos, target_pos) = rand_journey_target(avoid_positions)?;

    let color = Color::Srgba(COLORS[characters.color_index % COLORS.len()]);
    characters.color_index += 1;

    Some((
        GameObject,
        Player,
        character_sprite(&assets, color, start_pos),
        start_pos,
        JustSpawned,
        Journey {
            path: Vec::new(),
            bot_index: -1,
            start_pos,
            target_pos,
            color,
            scale: rand::thread_rng().gen_range(0.6..0.9),
        },
    ))
}

pub fn character_sprite(assets: &AllAssets, color: Color, start_pos: Position) -> SpriteBundle {
    SpriteBundle {
        texture: assets.character_sprite.clone(),
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

fn rand_position_fallback() -> Position {
    let mut rng = rand::thread_rng();
    Position(IVec2::new(
        rng.gen_range(0..GRID_SIZE.x) as i32,
        rng.gen_range(0..GRID_SIZE.y) as i32,
    ))
}

fn rand_journey_target(avoid_positions: Vec<&Position>) -> Option<(Position, Position)> {
    let mut rng = rand::thread_rng();

    const FALLBACK_ATTEMPTS: i32 = 1_000;
    const MAX_ATTEMPTS: i32 = 1_000_000;

    let mut attempts = 0;

    loop {
        let sides: Vec<Side> = [Side::Top, Side::Down, Side::Left, Side::Right]
            .choose_multiple(&mut rng, 2)
            .cloned()
            .collect();

        let start_pos = if attempts >= FALLBACK_ATTEMPTS {
            rand_position_fallback()
        } else {
            sides[0].rand_position()
        };

        let target_pos = sides[1].rand_position();

        if avoid_positions.contains(&&start_pos) || start_pos == target_pos {
            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                return None;
            }
            continue;
        }
        return Some((start_pos, target_pos));
    }
}

pub fn add_new_character_on_finished_journey(
    trigger: Trigger<JourneyFinished>,
    mut commands: Commands,
    assets: Res<AllAssets>,
    mut sprites: Query<&mut Sprite>,
    characters: ResMut<Characters>,
    positions: Query<&Position>,
    mut journeys: Query<&mut Journey>,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<GameState>>,
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

    journeys
        .get_mut(trigger.entity())
        .expect("Bot journey")
        .bot_index = 0;

    let Some(character) =
        character_bundle(positions, journeys.iter().collect(), characters, assets)
    else {
        next_state.set(GameState::EndGame);
        return;
    };
    commands.spawn(character);
    score.remaining_attempts = NB_ATTEMPTS;
}
