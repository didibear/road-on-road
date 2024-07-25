use crate::{components::*, GRID_SIZE};
use bevy::prelude::*;

pub(crate) fn handle_input_movement(
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

fn journey_finished(journey: &Journey, current_pos: &Position) -> bool {
    let has_reached_target = journey.path.contains(&journey.target_pos);
    let back_to_start = *current_pos == journey.start_pos;
    has_reached_target && back_to_start
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
