mod assets;
mod audio;
mod camera;
mod constants;
pub mod map;
mod menu;
mod network;
mod persistence;
mod settings;
mod states;
mod systems;
mod units;
mod utils;

use crate::core::audio::*;
use crate::core::camera::{move_camera, move_camera_keyboard, setup_camera};
use crate::core::constants::WATER_COLOR;
use crate::core::map::systems::{draw_map, update_map};
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::{setup_game_menu, setup_game_settings, setup_menu, update_ip};
use crate::core::network::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::persistence::{load_game, save_game};
use crate::core::persistence::{LoadGameMsg, SaveGameMsg};
use crate::core::settings::Settings;
use crate::core::states::{AppState, AudioState, GameState};
use crate::core::systems::{check_keys_game, check_keys_menu, on_resize_system};
use crate::core::utils::despawn;
use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};
use strum::IntoEnumIterator;

pub struct GamePlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InPlayingGameSet;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .init_state::<AppState>()
            .init_state::<GameState>()
            .init_state::<AudioState>()
            // Messages
            .add_message::<PlayAudioMsg>()
            .add_message::<PauseAudioMsg>()
            .add_message::<StopAudioMsg>()
            .add_message::<MuteAudioMsg>()
            .add_message::<ChangeAudioMsg>()
            .add_message::<ServerSendMsg>()
            .add_message::<ClientSendMsg>()
            .add_message::<SaveGameMsg>()
            .add_message::<LoadGameMsg>()
            // Resources
            .insert_resource(ClearColor(WATER_COLOR))
            .init_resource::<Ip>()
            .init_resource::<PlayingAudio>()
            .init_resource::<Settings>()
            // Sets
            .configure_sets(First, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(PreUpdate, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(Update, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(PostUpdate, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(Last, InGameSet.run_if(in_state(AppState::Game)))
            .configure_sets(
                First,
                InPlayingGameSet.run_if(in_state(GameState::Playing)).in_set(InGameSet),
            )
            .configure_sets(
                PreUpdate,
                InPlayingGameSet.run_if(in_state(GameState::Playing)).in_set(InGameSet),
            )
            .configure_sets(
                Update,
                InPlayingGameSet.run_if(in_state(GameState::Playing)).in_set(InGameSet),
            )
            .configure_sets(
                PostUpdate,
                InPlayingGameSet.run_if(in_state(GameState::Playing)).in_set(InGameSet),
            )
            .configure_sets(
                Last,
                InPlayingGameSet.run_if(in_state(GameState::Playing)).in_set(InGameSet),
            )
            // Camera
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (move_camera, move_camera_keyboard).in_set(InGameSet))
            // Audio
            .add_systems(Startup, setup_audio)
            .add_systems(OnEnter(GameState::Playing), play_music)
            .add_systems(
                Update,
                (toggle_audio, update_audio, play_audio, pause_audio, stop_audio, mute_audio),
            )
            //Networking
            .add_systems(
                First,
                (
                    server_receive_message.run_if(resource_exists::<RenetServer>),
                    client_receive_message.run_if(resource_exists::<RenetClient>),
                ),
            )
            .add_systems(Update, server_update.run_if(resource_exists::<RenetServer>))
            .add_systems(
                Last,
                (
                    server_send_message.run_if(resource_exists::<RenetServer>),
                    client_send_message.run_if(resource_exists::<RenetClient>),
                ),
            );

        // Menu
        for state in AppState::iter().filter(|s| *s != AppState::Game) {
            app.add_systems(OnEnter(state), setup_menu)
                .add_systems(OnExit(state), despawn::<MenuCmp>);
        }
        app.add_systems(Update, update_ip.run_if(in_state(AppState::MultiPlayerMenu)));

        app
            // Utilities
            .add_systems(Update, (check_keys_menu, check_keys_game.in_set(InGameSet)))
            .add_systems(PostUpdate, on_resize_system)
            // In-game states
            .add_systems(OnEnter(AppState::Game), draw_map)
            .add_systems(Update, update_map.in_set(InGameSet))
            .add_systems(OnEnter(GameState::GameMenu), setup_game_menu)
            .add_systems(OnExit(GameState::GameMenu), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::Settings), setup_game_settings)
            .add_systems(OnExit(GameState::Settings), despawn::<MenuCmp>);

        // Persistence
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            (load_game, save_game.run_if(resource_exists::<Host>).in_set(InGameSet)),
        );
    }
}
