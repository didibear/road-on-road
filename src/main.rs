// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::asset::AssetMetaCheck;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;

mod characters;
mod components;
mod destroyed;
mod draws;
mod game_state;
mod inputs;
mod movements;
mod scores;
mod sounds;
mod tutorial;

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
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume::new(0.3),
                    ..default()
                }),
        )
        .add_plugins(game_plugin)
        .run();
}

pub fn game_plugin(app: &mut App) {
    app.init_state::<game_state::GameState>()
        .enable_state_scoped_entities::<game_state::GameState>()
        .add_systems(Startup, (setup_camera, scores::spawn_score_display))
        .add_systems(
            OnEnter(game_state::GameState::InGame),
            characters::spawn_first_player,
        )
        .add_systems(
            OnEnter(game_state::GameState::EndGame),
            game_state::spawn_restart_text,
        )
        .add_systems(
            Update,
            (
                // logic
                (
                    inputs::handle_input_movement.run_if(in_state(game_state::GameState::InGame)),
                    game_state::handle_restart_input
                        .run_if(in_state(game_state::GameState::EndGame)),
                    movements::move_transit_entities,
                    movements::detect_collisions,
                    destroyed::destroyed_animation,
                    tutorial::validate_first_tutorial,
                    scores::update_score_display,
                ),
                // drawing
                movements::position_to_transform,
                (draws::draw_grid, draws::draw_paths),
                draws::draw_targets,
            )
                .chain(),
        )
        .add_systems(
            OnExit(game_state::GameState::EndGame),
            game_state::clear_up_game_entities,
        )
        .init_resource::<characters::Characters>()
        .init_resource::<scores::Score>()
        .init_resource::<AllAssets>()
        .observe(characters::add_new_character_on_finished_journey)
        .observe(tutorial::spawn_first_tutorial)
        .observe(scores::score_nb_journeys);
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Debug, Resource)]
pub struct AllAssets {
    character_sprite: Handle<Image>,
    move_sound: Vec<Handle<AudioSource>>,
    hurt_sound: Vec<Handle<AudioSource>>,
    coin_sound: Handle<AudioSource>,
    goal_sound: Handle<AudioSource>,
}

impl FromWorld for AllAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            // itch only supports assets directly in the `assets/` directory
            character_sprite: asset_server.load("ducky.png"),
            move_sound: vec![
                asset_server.load("move1.ogg"),
                asset_server.load("move2.ogg"),
                asset_server.load("move3.ogg"),
                asset_server.load("move4.ogg"),
            ],
            hurt_sound: vec![
                asset_server.load("hurt1.ogg"),
                asset_server.load("hurt2.ogg"),
                asset_server.load("hurt3.ogg"),
            ],
            coin_sound: asset_server.load("coin.ogg"),
            goal_sound: asset_server.load("goal.ogg"),
        }
    }
}
