mod assets;
mod camera;
mod constants;
mod map;
mod states;

use bevy::prelude::*;

use crate::core::camera::{move_camera, move_camera_keyboard, reset_camera, setup_camera};
use crate::core::constants::WATER_COLOR;
use crate::core::map::systems::draw_map;
use crate::core::states::{AppState, AudioState, GameState};

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
            // Resources
            .insert_resource(ClearColor(WATER_COLOR))
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
            .add_systems(Startup, (setup_camera, draw_map).chain())
            .add_systems(Update, (move_camera, move_camera_keyboard).in_set(InGameSet));
        // // Audio
        // .add_systems(Startup, setup_audio)
        // .add_systems(OnEnter(GameState::Playing), play_music)
        // .add_systems(
        //     Update,
        //     (toggle_audio, update_audio, play_audio, pause_audio, stop_audio, mute_audio),
        // )
        // //Networking
        // .add_systems(
        //     First,
        //     (
        //         server_receive_message.run_if(resource_exists::<RenetServer>),
        //         client_receive_message.run_if(resource_exists::<RenetClient>),
        //     ),
        // )
        // .add_systems(Update, server_update.run_if(resource_exists::<RenetServer>))
        // .add_systems(
        //     Last,
        //     (
        //         server_send_message.run_if(resource_exists::<RenetServer>),
        //         client_send_message.run_if(resource_exists::<RenetClient>),
        //     ),
        // );

        // Menu
        // for state in AppState::iter().filter(|s| *s != AppState::Game) {
        //     app.add_systems(OnEnter(state), setup_menu)
        //         .add_systems(OnExit(state), despawn::<MenuCmp>);
        // }
        // app.add_systems(Update, update_ip.run_if(in_state(AppState::MultiPlayerMenu)));

        // app
        // Persistence
        // .add_systems(
        //     Update,
        //     (load_game, save_game.run_if(resource_exists::<Host>).in_set(InGameSet)),
        // )
        // Utilities
        // .add_systems(
        //     Update,
        //     (
        //         check_keys_menu,
        //         check_keys.in_set(InPlayingGameSet),
        //         check_keys_combat
        //             .run_if(in_state(GameState::CombatMenu).or(in_state(GameState::Combat)))
        //             .in_set(InGameSet),
        //     ),
        // )
        // .add_systems(PostUpdate, on_resize_system)
        // In-game states
        // .add_systems(OnEnter(AppState::Game), draw_map);
    }
}
