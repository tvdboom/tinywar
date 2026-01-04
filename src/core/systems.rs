use bevy::prelude::*;
use bevy::window::WindowResized;
use bevy_renet::netcode::NetcodeServerTransport;
use bevy_renet::renet::{RenetClient, RenetServer};

use crate::core::menu::utils::TextSize;
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};

pub fn on_resize_system(
    mut resize_reader: MessageReader<WindowResized>,
    mut text: Query<(&mut TextFont, &TextSize)>,
) {
    for window in resize_reader.read() {
        for (mut text, size) in text.iter_mut() {
            text.font_size = size.0 * window.height / 460.
        }
    }
}

pub fn check_keys_menu(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match app_state.get() {
            AppState::SinglePlayerMenu | AppState::MultiPlayerMenu | AppState::Settings => {
                next_app_state.set(AppState::MainMenu)
            },
            AppState::Lobby | AppState::ConnectedLobby => {
                if let Some(client) = client.as_mut() {
                    client.disconnect();
                    commands.remove_resource::<RenetClient>();
                } else if let Some(mut server) = server {
                    server.disconnect_all();
                    commands.remove_resource::<RenetServer>();
                    commands.remove_resource::<NetcodeServerTransport>();
                }

                next_app_state.set(AppState::MultiPlayerMenu)
            },
            AppState::Game => match game_state.get() {
                GameState::Playing => next_game_state.set(GameState::GameMenu),
                GameState::Paused | GameState::GameMenu => next_game_state.set(GameState::Playing),
                GameState::EndGame => next_app_state.set(AppState::MainMenu),
                GameState::Settings => next_game_state.set(GameState::GameMenu),
            },
            _ => (),
        }
    }
}

pub fn check_keys_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<Settings>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match game_state.get() {
            GameState::Playing => next_game_state.set(GameState::Paused),
            GameState::Paused => next_game_state.set(GameState::Playing),
            _ => (),
        }
    } else if keyboard.just_released(KeyCode::ArrowRight) {
        settings.speed = (settings.speed * 2.).min(64.0);
    } else if keyboard.just_released(KeyCode::ArrowLeft) {
        settings.speed = (settings.speed * 0.5).max(0.25);
    }
}
