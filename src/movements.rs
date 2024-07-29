use crate::characters;
use crate::components::*;
use crate::sounds::play_random_sound;
use crate::AllAssets;
use bevy::prelude::*;

use crate::CELL_SIZE;
use crate::GRID_SIZE;

pub fn position_to_transform(
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

pub fn position_translation(pos: &Position) -> Vec2 {
    grid_pos_translation(pos.0.as_vec2())
}

pub fn sprite_position_translation(pos: Vec2) -> Vec2 {
    grid_pos_translation(pos) + CELL_SIZE / 2.
}

pub fn grid_pos_translation(pos: Vec2) -> Vec2 {
    pos * CELL_SIZE - GRID_SIZE.as_vec2() * CELL_SIZE / 2.
}

const SPEED: f32 = CELL_SIZE / 6.;

pub fn move_transit_entities(
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

pub fn detect_collisions(
    mut commands: Commands,
    locations: Query<(Entity, &Transform, &Journey), (WithPlayerOrAutomated, With<Transition>)>,
    players: Query<Entity, With<Player>>,
    journeys: Query<&Journey>,
    assets: Res<AllAssets>,
    mut sprites: Query<&mut Sprite>,
) {
    for [(entity_a, transform_a, journey_a), (entity_b, transform_b, journey_b)] in
        locations.iter_combinations()
    {
        if transform_a.translation.distance(transform_b.translation) <= CELL_SIZE / 2. {
            let destroyed_entity = {
                // if players.get(entity_a).is_ok() {
                //     entity_a
                // } else {
                //     entity_b
                // }
                let is_a_on_spawn = journey_a.path.len() <= 1 || journey_a.bot_index == 0;
                let is_b_on_spawn = journey_b.path.len() <= 1 || journey_b.bot_index == 0;

                if is_a_on_spawn && is_b_on_spawn {
                    continue;
                } else if is_a_on_spawn {
                    entity_b
                } else if is_b_on_spawn {
                    entity_a
                } else if players.get(entity_a).is_ok() {
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
                commands.spawn(play_random_sound(&assets.hurt_sound));
                sprites
                    .get_mut(destroyed_entity)
                    .expect("Bot sprite")
                    .color
                    .set_alpha(0.3);

                commands.spawn((
                    Player,
                    characters::character_sprite(&assets, journey.color, journey.start_pos),
                    journey.start_pos,
                    Journey {
                        path: Vec::new(),
                        ..*journey
                    },
                ));
            } else {
                commands.spawn((
                    Automated,
                    characters::character_sprite(
                        &assets,
                        journey.color.with_alpha(0.3),
                        journey.start_pos,
                    ),
                    journey.start_pos,
                    Journey {
                        path: journey.path.clone(),
                        bot_index: 0,
                        ..*journey
                    },
                ));
            }
        }
    }
}
