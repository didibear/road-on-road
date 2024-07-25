use crate::{components::*, CELL_SIZE, WINDOW_SIZE};
use bevy::prelude::*;

#[derive(Debug, Resource, Default)]
pub struct Score {
    nb_journeys: u32,
    max_nb_characters: u32,
}

impl Score {
    fn text(&self) -> String {
        format!(
            "Total Journeys: {}, Max Ducks: {}",
            self.nb_journeys, self.max_nb_characters
        )
    }
}

#[derive(Debug, Component)]
pub struct ScoreDisplay;

pub fn update_max_nb_characters(
    characters: Query<Entity, Or<(With<Automated>, With<Player>)>>,
    mut score: ResMut<Score>,
) {
    score.max_nb_characters = score
        .max_nb_characters
        .max(characters.iter().count() as u32);
}

pub fn score_nb_journeys(_trigger: Trigger<JourneyFinished>, mut score: ResMut<Score>) {
    score.nb_journeys += 1
}

pub fn spawn_score_display(mut commands: Commands, score: Res<Score>) {
    let text_position = Vec3::new(0., WINDOW_SIZE / 2. - CELL_SIZE, 0.);

    let text_style = TextStyle {
        font_size: 20.0,
        ..default()
    };

    commands.spawn((
        ScoreDisplay,
        Text2dBundle {
            text: Text::from_section(score.text(), text_style).with_justify(JustifyText::Center),
            transform: Transform::from_translation(text_position),
            ..default()
        },
    ));
}

pub fn update_score_display(
    mut score_displays: Query<&mut Text, With<ScoreDisplay>>,
    score: Res<Score>,
) {
    for mut text in score_displays.iter_mut() {
        text.sections[0].value = score.text();
    }
}
