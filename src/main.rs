// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowResolution;

mod characters;
mod components;
mod destroyed;
mod draws;
mod inputs;
mod movements;

const WINDOW_SIZE: f32 = 600.;

const GRID_SIZE: UVec2 = UVec2::new(6, 6);
const CELL_SIZE: f32 = WINDOW_SIZE / 10.;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy Jam #5 - Cycles".to_string(),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        resolution: WindowResolution::new(WINDOW_SIZE, WINDOW_SIZE),
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        )
        .add_plugins(game_plugin)
        .run();
}

pub fn game_plugin(app: &mut App) {
    app.add_systems(Startup, (setup_camera, characters::spawn_player))
        .add_systems(
            Update,
            (
                // logic
                (
                    inputs::handle_input_movement,
                    movements::move_transit_entities,
                    movements::detect_collisions,
                    destroyed::destroyed_animation,
                ),
                // drawing
                movements::position_to_transform,
                (draws::draw_grid, draws::draw_paths),
                draws::draw_targets,
            )
                .chain(),
        )
        .init_resource::<characters::Characters>()
        .observe(characters::add_new_character_on_finished_journey);
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
