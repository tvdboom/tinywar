mod assets;
mod audio;
mod camera;
mod constants;
pub mod map;
mod mechanics;
mod menu;
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
use crate::core::camera::*;
use crate::core::constants::WATER_COLOR;
use crate::core::map::map::Map;
use crate::core::map::systems::{draw_map, setup_end_game, MapCmp};
use crate::core::map::ui::systems::{draw_ui, update_ui, UiCmp};
use crate::core::mechanics::combat::{apply_damage_message, resolve_attack, ApplyDamageMsg};
use crate::core::mechanics::movement::apply_movement;
use crate::core::mechanics::queue::*;
use crate::core::mechanics::spawn::*;
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::*;
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
use crate::core::systems::*;
use crate::core::units::systems::{update_buildings, update_units};
use crate::core::utils::despawn;
use bevy::prelude::*;
use strum::IntoEnumIterator;
#[cfg(not(target_arch = "wasm32"))]
use {
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
            .add_message::<DespawnMsg>()
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

            #[cfg(not(target_arch = "wasm32"))]
            app.add_systems(OnExit(state), exit_multiplayer_lobby);
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
            .add_systems(Update, (update_ui, update_animations).in_set(InGameSet))
            .add_systems(Update, queue_message.in_set(InPlayingOrPausedSet))
            .add_systems(
                Update,
                (
                    queue_resolve,
                    spawn_unit_message,
                    spawn_building_message,
                    update_units,
                    update_buildings,
                    (apply_movement, resolve_attack, apply_damage_message)
                        .chain()
                        .run_if(resource_exists::<Host>),
                )
                    .in_set(InPlayingSet),
            )
            .add_systems(Last, despawn_message.in_set(InPlayingSet))
            .add_systems(OnExit(AppState::Game), (despawn::<MapCmp>, reset_camera))
            .add_systems(OnEnter(GameState::GameMenu), setup_game_menu)
            .add_systems(OnExit(GameState::GameMenu), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::EndGame), (despawn::<UiCmp>, setup_end_game))
            .add_systems(OnEnter(GameState::Settings), setup_game_settings)
            .add_systems(OnExit(GameState::Settings), despawn::<MenuCmp>);

        #[cfg(not(target_arch = "wasm32"))]
        app
            //Networking
            .add_message::<ServerSendMsg>()
            .add_message::<ClientSendMsg>()
            .init_resource::<Ip>()
            .add_systems(
                First,
                (
                    server_receive_message.run_if(resource_exists::<RenetServer>),
                    client_receive_message.run_if(resource_exists::<RenetClient>),
                ),
            )
            .add_systems(Update, server_update.run_if(resource_exists::<RenetServer>))
            .add_systems(Update, update_ip.run_if(in_state(AppState::MultiPlayerMenu)))
            .add_systems(
                Last,
                (
                    server_send_message.run_if(resource_exists::<RenetServer>),
                    client_send_message.run_if(resource_exists::<RenetClient>),
                ),
            )
            .add_systems(OnExit(AppState::Game), exit_multiplayer_lobby)
            // Persistence
            .add_message::<SaveGameMsg>()
            .add_message::<LoadGameMsg>()
            .add_systems(
                Update,
                (load_game, save_game.run_if(resource_exists::<Host>).in_set(InGameSet)),
            );
    }
}
