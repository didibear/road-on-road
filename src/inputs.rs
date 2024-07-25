use crate::{
    components::*,
    sounds::{play_random_sound, play_sound},
    AllAssets, GRID_SIZE,
};
use bevy::prelude::*;

pub fn handle_input_movement(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut positions: Query<(Entity, &Position, &mut Journey), (With<Player>, Without<Transition>)>,
    mut bot_positions: Query<(Entity, &Position, &mut Journey), (With<Automated>, Without<Player>)>,
    assets: Res<AllAssets>,
) {
    let Some(direction) = keyboard_direction(&keyboard).or_else(|| touch_direction(touches)) else {
        return;
    };

    for (entity, current_pos, mut journey) in positions.iter_mut() {
        let next_pos =
            Position((current_pos.0 + direction).clamp(IVec2::ZERO, GRID_SIZE.as_ivec2() - 1));

        if next_pos != *current_pos {
            commands
                .entity(entity)
                .insert(Transition::new(*current_pos, next_pos));

            commands.spawn(play_random_sound(&assets.move_sound));

            journey.path.push(*current_pos);

            // TODO: Manage journey finished after transition
            if journey_finished(&journey, &next_pos) {
                commands.trigger_targets(JourneyFinished, entity);
                commands.spawn(play_sound(&assets.coin_sound));
            } else if just_reached_target(&journey, &next_pos) {
                commands.spawn(play_sound(&assets.goal_sound));
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

pub fn journey_finished(journey: &Journey, current_pos: &Position) -> bool {
    let has_reached_target = journey.path.contains(&journey.target_pos);
    let back_to_start = *current_pos == journey.start_pos;
    has_reached_target && back_to_start
}
pub fn just_reached_target(journey: &Journey, current_pos: &Position) -> bool {
    let at_target = *current_pos == journey.target_pos;
    let never_before = !journey.path.contains(&journey.target_pos);
    at_target && never_before
}

const DIRECTIONS: [IVec2; 4] = [IVec2::Y, IVec2::NEG_Y, IVec2::NEG_X, IVec2::X];

fn keyboard_direction(keyboard: &Res<ButtonInput<KeyCode>>) -> Option<IVec2> {
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

fn touch_direction(touches: Res<Touches>) -> Option<IVec2> {
    let touch = touches.iter_just_released().next()?;
    dbg!(&touch);

    let mut direction = touch.distance().normalize_or_zero();
    if direction == Vec2::ZERO {
        return None;
    }

    // The screen seems to swap these two
    direction.x = -direction.x;

    let closest_direction = DIRECTIONS
        .iter()
        .max_by(|a, b| {
            a.as_vec2()
                .distance_squared(direction)
                .partial_cmp(&b.as_vec2().distance_squared(direction))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();

    closest_direction
}
