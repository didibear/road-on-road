use bevy::prelude::*;

use crate::{characters, components::GameObject, scores, CELL_SIZE, WINDOW_SIZE};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    InGame,
    EndGame,
}

pub fn clear_up_game_entities(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameObject>>,
    mut characters: ResMut<characters::Characters>,
    mut scores: ResMut<scores::Score>,
) {
    for entity in game_entities.iter() {
        commands.entity(entity).despawn();
    }
    *characters = characters::Characters::default();
    *scores = scores::Score::default();
}

pub fn spawn_restart_text(mut commands: Commands) {
    commands.spawn((StateScoped(GameState::EndGame), lose_text()));
    commands.spawn((StateScoped(GameState::EndGame), restart_text()));
}

fn lose_text() -> Text2dBundle {
    let text_position = Vec3::new(0., WINDOW_SIZE / 2. - CELL_SIZE - 32., 0.);

    Text2dBundle {
        text: Text::from_section(
            "Game ended",
            TextStyle {
                font_size: 48.0,
                ..default()
            },
        )
        .with_justify(JustifyText::Center),
        transform: Transform::from_translation(text_position),
        ..default()
    }
}

fn restart_text() -> Text2dBundle {
    let text_position = Vec3::new(0., -(WINDOW_SIZE / 2. - CELL_SIZE), 0.);

    Text2dBundle {
        text: Text::from_section(
            "Press space to restart",
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_justify(JustifyText::Center),
        transform: Transform::from_translation(text_position),
        ..default()
    }
}

pub fn handle_restart_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || touches.any_just_released() {
        next_state.set(GameState::InGame);
    }
}
