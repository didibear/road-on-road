use crate::components::*;
use crate::inputs::journey_finished;
use crate::movements::sprite_position_translation;
use bevy::prelude::*;

#[derive(Component)]
pub struct FirstPlayer;

#[derive(Component)]
pub struct Tutorial;

#[derive(Event)]
pub struct FirstPlayerAdded;

pub fn spawn_first_tutorial(
    trigger: Trigger<FirstPlayerAdded>,
    mut commands: Commands,
    journeys: Query<&Journey, With<FirstPlayer>>,
) {
    let journey = journeys
        .get(trigger.entity())
        .expect("First player journey");

    let text_position = sprite_position_translation(journey.target_pos.0.as_vec2());

    commands.spawn((
        GameObject,
        Tutorial,
        Text2dBundle {
            text: Text::from_section("Go here", text_style()).with_justify(JustifyText::Center),
            transform: Transform::from_translation(Vec3::from((text_position, 500.))),
            ..default()
        },
    ));
}

fn text_style() -> TextStyle {
    TextStyle {
        font_size: 14.0,
        ..default()
    }
}

pub fn validate_first_tutorial(
    mut commands: Commands,
    journeys: Query<(&Journey, &Position), With<FirstPlayer>>,
    mut tutorials: Query<(Entity, &mut Transform, &mut Text), With<Tutorial>>,
) {
    for (tutorial_entity, mut transform, mut text) in tutorials.iter_mut() {
        for (journey, pos) in journeys.iter() {
            if journey_finished(journey, pos) {
                commands.entity(tutorial_entity).despawn();
                return;
            }

            let has_reached_target = *pos == journey.target_pos;
            if has_reached_target {
                transform.translation = Vec3::from((
                    sprite_position_translation(journey.start_pos.0.as_vec2()),
                    0.,
                ));
                text.sections[0].value = String::from("Go back");
            }
        }
    }
}
