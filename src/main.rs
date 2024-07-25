// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::asset::AssetMetaCheck;
use bevy::audio::{AudioPlugin, Volume};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_asset_loader::prelude::*;

mod characters;
mod components;
mod destroyed;
mod draws;
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
    app.init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Game)
                .load_collection::<AllAssets>(),
        )
        .add_systems(
            OnEnter(GameState::Game),
            (
                setup_camera,
                characters::spawn_first_player,
                scores::spawn_score_display,
            ),
        )
        .add_systems(
            Update,
            (
                // logic
                (
                    inputs::handle_input_movement,
                    movements::move_transit_entities,
                    movements::detect_collisions,
                    destroyed::destroyed_animation,
                    tutorial::validate_first_tutorial,
                    scores::update_max_nb_characters,
                    scores::update_score_display,
                ),
                // drawing
                movements::position_to_transform,
                (draws::draw_grid, draws::draw_paths),
                draws::draw_targets,
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        )
        .init_resource::<characters::Characters>()
        .init_resource::<scores::Score>()
        .observe(characters::add_new_character_on_finished_journey)
        .observe(tutorial::spawn_first_tutorial)
        .observe(scores::score_nb_journeys);
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(AssetCollection, Resource)]
pub struct AllAssets {
    #[asset(path = "images/ducky.png")]
    character_sprite: Handle<Image>,

    #[asset(path = "audio/move", collection(typed))]
    move_sound: Vec<Handle<AudioSource>>,
    #[asset(path = "audio/hurt", collection(typed))]
    hurt_sound: Vec<Handle<AudioSource>>,
    #[asset(path = "audio/coin.ogg")]
    coin_sound: Handle<AudioSource>,
    #[asset(path = "audio/goal.ogg")]
    goal_sound: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    AssetLoading,
    Game,
}
