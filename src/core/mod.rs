mod assets;
mod audio;
mod boosts;
mod camera;
mod constants;
pub mod map;
mod mechanics;
mod menu;
#[cfg(not(target_arch = "wasm32"))]
mod multiplayer;
#[cfg(not(target_arch = "wasm32"))]
mod network;
#[cfg(not(target_arch = "wasm32"))]
mod persistence;
mod player;
mod settings;
mod states;
mod systems;
mod units;
mod utils;

use crate::core::audio::*;
use crate::core::boosts::{after_boost_check, check_boost_timer, initiate_boost_message, update_boosts, CardCmp, InitiateBoostMsg};
use crate::core::camera::*;
use crate::core::constants::{UPDATE_TIMER, WATER_COLOR};
use crate::core::map::map::Map;
use crate::core::map::systems::{draw_map, setup_end_game, MapCmp};
use crate::core::map::ui::boosts::{setup_after_boost, setup_boost_selection};
use crate::core::map::ui::systems::{draw_ui, update_ui, update_ui2, UiCmp};
use crate::core::mechanics::combat::{apply_damage_message, resolve_attack, ApplyDamageMsg};
use crate::core::mechanics::movement::apply_movement;
use crate::core::mechanics::queue::*;
use crate::core::mechanics::spawn::*;
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::*;
use crate::core::persistence::run_autosave;
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
use crate::core::systems::*;
use crate::core::units::systems::{update_buildings, update_units};
use crate::core::utils::despawn;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;
use strum::IntoEnumIterator;
#[cfg(not(target_arch = "wasm32"))]
use {
    crate::core::multiplayer::*,
    crate::core::network::*,
    crate::core::persistence::{load_game, save_game, LoadGameMsg, SaveGameMsg},
    bevy_renet::renet::{RenetClient, RenetServer},
};

pub struct GamePlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InPlayingSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InPlayingOrPausedSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InPlayingOrPausedOrEndSet;

macro_rules! configure_stages {
    ($app:expr, $set:ident, $run_if:expr) => {
        $app.configure_sets(First, $set.run_if($run_if))
            .configure_sets(PreUpdate, $set.run_if($run_if))
            .configure_sets(Update, $set.run_if($run_if))
            .configure_sets(PostUpdate, $set.run_if($run_if))
            .configure_sets(Last, $set.run_if($run_if));
    };
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .init_state::<AppState>()
            .init_state::<GameState>()
            // Messages
            .add_message::<PlayAudioMsg>()
            .add_message::<PauseAudioMsg>()
            .add_message::<StopAudioMsg>()
            .add_message::<MuteAudioMsg>()
            .add_message::<ChangeAudioMsg>()
            .add_message::<StartNewGameMsg>()
            .add_message::<QueueUnitMsg>()
            .add_message::<SpawnBuildingMsg>()
            .add_message::<SpawnUnitMsg>()
            .add_message::<SpawnArrowMsg>()
            .add_message::<DespawnMsg>()
            .add_message::<InitiateBoostMsg>()
            .add_message::<ApplyDamageMsg>()
            // Resources
            .insert_resource(ClearColor(WATER_COLOR))
            .init_resource::<PlayingAudio>()
            .init_resource::<Settings>()
            .init_resource::<Map>();

        // Sets
        configure_stages!(app, InGameSet, in_state(AppState::Game));
        configure_stages!(
            app,
            InPlayingSet,
            in_state(GameState::Playing).and(in_state(AppState::Game))
        );
        configure_stages!(
            app,
            InPlayingOrPausedSet,
            in_state(GameState::Playing)
                .or(in_state(GameState::Paused))
                .and(in_state(AppState::Game))
        );
        configure_stages!(
            app,
            InPlayingOrPausedOrEndSet,
            in_state(GameState::Playing)
                .or(in_state(GameState::Paused))
                .or(in_state(GameState::EndGame))
                .and(in_state(AppState::Game))
        );

        app
            // Camera
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (move_camera, move_camera_keys).in_set(InPlayingOrPausedOrEndSet))
            // Audio
            .add_systems(Startup, setup_audio)
            .add_systems(OnEnter(GameState::Playing), play_music)
            .add_systems(
                Update,
                (toggle_audio, update_audio, play_audio, pause_audio, stop_audio, mute_audio),
            );

        // Menu
        for state in AppState::iter().filter(|s| *s != AppState::Game) {
            app.add_systems(OnEnter(state), setup_menu)
                .add_systems(OnExit(state), despawn::<MenuCmp>);
        }
        app.add_systems(Update, start_new_game_message.run_if(not(in_state(AppState::Game))));

        app
            // Utilities
            .add_systems(
                Update,
                (
                    check_keys_menu,
                    check_keys_game.in_set(InGameSet),
                    check_keys_playing_game.in_set(InPlayingOrPausedSet),
                ),
            )
            .add_systems(PostUpdate, on_resize_message)
            // In-game states
            .add_systems(OnEnter(AppState::Game), (draw_map, draw_ui))
            .add_systems(Update, (update_ui, update_ui2, update_animations).in_set(InGameSet))
            .add_systems(Update, queue_message.in_set(InPlayingOrPausedSet))
            .add_systems(
                Update,
                (
                    update_boosts,
                    initiate_boost_message,
                    queue_resolve,
                    spawn_unit_message,
                    spawn_building_message,
                    spawn_arrow_message,
                    update_units,
                    update_buildings,
                    (check_boost_timer, apply_movement, resolve_attack, apply_damage_message)
                        .chain()
                        .run_if(resource_exists::<Host>),
                )
                    .in_set(InPlayingSet),
            )
            .add_systems(Last, despawn_message.in_set(InPlayingSet))
            .add_systems(OnExit(AppState::Game), (despawn::<MapCmp>, reset_camera))
            .add_systems(OnEnter(GameState::BoostSelection), setup_boost_selection)
            .add_systems(OnExit(GameState::BoostSelection), despawn::<CardCmp>)
            .add_systems(OnEnter(GameState::AfterBoostSelection), setup_after_boost)
            .add_systems(OnExit(GameState::AfterBoostSelection), despawn::<CardCmp>)
            .add_systems(OnEnter(GameState::GameMenu), setup_game_menu)
            .add_systems(OnExit(GameState::GameMenu), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::EndGame), (despawn::<UiCmp>, setup_end_game))
            .add_systems(OnEnter(GameState::Settings), setup_game_settings)
            .add_systems(OnExit(GameState::Settings), despawn::<MenuCmp>);

        #[cfg(not(target_arch = "wasm32"))]
        app
            // Networking && multiplayer
            .add_message::<ServerSendMsg>()
            .add_message::<ClientSendMsg>()
            .add_message::<UpdatePopulationMsg>()
            .init_resource::<Ip>()
            .init_resource::<EntityMap>()
            .add_systems(
                First,
                (
                    server_receive_message.run_if(resource_exists::<RenetServer>),
                    client_receive_message.run_if(resource_exists::<RenetClient>),
                ),
            )
            .add_systems(PreUpdate, update_population_message.in_set(InGameSet))
            .add_systems(Update, update_game_state.run_if(state_changed::<GameState>))
            .add_systems(Update, update_player.in_set(InGameSet))
            .add_systems(Update, server_update.run_if(resource_exists::<RenetServer>))
            .add_systems(
                Update,
                after_boost_check
                    .run_if(resource_exists::<RenetServer>)
                    .run_if(in_state(GameState::AfterBoostSelection)),
            )
            .add_systems(Update, update_ip.run_if(in_state(AppState::MultiPlayerMenu)))
            .add_systems(
                Last,
                (
                    (
                        server_send_message,
                        server_send_status
                            .run_if(on_timer(Duration::from_millis(UPDATE_TIMER)))
                            .in_set(InPlayingSet),
                    )
                        .run_if(resource_exists::<RenetServer>),
                    (client_send_message,).run_if(resource_exists::<RenetClient>),
                ),
            )
            .add_systems(OnEnter(AppState::MultiPlayerMenu), exit_multiplayer_lobby)
            // Persistence
            .add_message::<SaveGameMsg>()
            .add_message::<LoadGameMsg>()
            .add_systems(
                Update,
                (
                    load_game,
                    (
                        save_game,
                        run_autosave.run_if(on_timer(Duration::from_secs(10))).in_set(InPlayingSet),
                    )
                        .run_if(resource_exists::<Host>)
                        .in_set(InGameSet),
                ),
            );
    }
}
