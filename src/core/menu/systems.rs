use std::net::IpAddr;

use bevy::prelude::*;
use bevy_renet::netcode::NetcodeServerTransport;
use bevy_renet::renet::{RenetClient, RenetServer};

use crate::core::assets::WorldAssets;
use crate::core::constants::{
    BUTTON_TEXT_SIZE, DISABLED_BUTTON_COLOR, NORMAL_BUTTON_COLOR, TITLE_TEXT_SIZE,
};
use crate::core::map::map::Map;
use crate::core::mechanics::spawn::SpawnBuildingMsg;
use crate::core::menu::buttons::{
    spawn_menu_button, DisabledButton, IpTextCmp, LobbyTextCmp, MenuBtn, MenuCmp,
};
use crate::core::menu::settings::{spawn_label, SettingsBtn};
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::network::{Host, Ip, ServerSendMsg};
use crate::core::player::{Player, Players};
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::{AppState, GameState};
use crate::core::units::buildings::BuildingName;
use crate::utils::get_local_ip;

#[derive(Message)]
pub struct StartNewGameMsg;

pub fn setup_menu(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    server: Option<Res<RenetServer>>,
    settings: Res<Settings>,
    ip: Res<Ip>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands
        .spawn((
            add_root_node(false),
            ImageNode::new(assets.image("bg")),
            MenuCmp,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::ZERO.with_top(Val::Percent(10.)),
                    ..default()
                })
                .with_children(|parent| match app_state.get() {
                    AppState::MainMenu => {
                        spawn_menu_button(parent, MenuBtn::Singleplayer, &assets, &window);
                        #[cfg(not(target_arch = "wasm32"))]
                        spawn_menu_button(parent, MenuBtn::Multiplayer, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::Settings, &assets, &window);
                        #[cfg(not(target_arch = "wasm32"))]
                        spawn_menu_button(parent, MenuBtn::Quit, &assets, &window);
                    }
                    AppState::SinglePlayerMenu => {
                        spawn_menu_button(parent, MenuBtn::NewGame, &assets, &window);
                        #[cfg(not(target_arch = "wasm32"))]
                        spawn_menu_button(parent, MenuBtn::LoadGame, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                    }
                    AppState::MultiPlayerMenu => {
                        parent.spawn((
                            add_text(
                                format!("Ip: {}", ip.0),
                                "bold",
                                BUTTON_TEXT_SIZE,
                                &assets,
                                &window,
                            ),
                            IpTextCmp,
                        ));
                        spawn_menu_button(parent, MenuBtn::HostGame, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::FindGame, &assets, &window);
                        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                    }
                    AppState::Lobby | AppState::ConnectedLobby => {
                        if let Some(server) = server {
                            let n_players = server.clients_id().len() + 1;

                            parent.spawn((
                                add_text(
                                    if n_players == 1 {
                                        format!("Waiting for other players to join {}...", get_local_ip())
                                    } else {
                                        format!("There are {n_players} players in the lobby.\nWaiting for other players to join {}...", get_local_ip())
                                    },
                                    "bold",
                                    BUTTON_TEXT_SIZE,
                                    &assets,
                                    &window,
                                ),
                                LobbyTextCmp,
                            ));

                            if n_players > 1 {
                                spawn_menu_button(parent, MenuBtn::NewGame, &assets, &window);
                                spawn_menu_button(parent, MenuBtn::LoadGame, &assets, &window);
                            }
                        } else {
                            parent.spawn((
                                add_text(
                                    "Searching for a game...",
                                    "bold",
                                    BUTTON_TEXT_SIZE,
                                    &assets,
                                    &window,
                                ),
                                LobbyTextCmp,
                            ));
                        }

                        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                    }
                    AppState::Settings => {
                        parent
                            .spawn((Node {
                                width: Val::Percent(40.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::ZERO.with_top(Val::Percent(-7.)),
                                ..default()
                            },))
                            .with_children(|parent| {
                                spawn_label(
                                    parent,
                                    "Color",
                                    vec![
                                        SettingsBtn::Blue,
                                        SettingsBtn::Black,
                                        SettingsBtn::Purple,
                                        SettingsBtn::Red,
                                        SettingsBtn::Yellow,
                                    ],
                                    &settings,
                                    &assets,
                                    &window,
                                );
                                spawn_label(
                                    parent,
                                    "Map size",
                                    vec![
                                        SettingsBtn::Small,
                                        SettingsBtn::Medium,
                                        SettingsBtn::Large,
                                    ],
                                    &settings,
                                    &assets,
                                    &window,
                                );
                                spawn_label(
                                    parent,
                                    "Audio",
                                    vec![
                                        SettingsBtn::Mute,
                                        SettingsBtn::NoMusic,
                                        SettingsBtn::Sound,
                                    ],
                                    &settings,
                                    &assets,
                                    &window,
                                );
                                spawn_label(
                                    parent,
                                    "Autosave",
                                    vec![
                                        SettingsBtn::True,
                                        SettingsBtn::False,
                                    ],
                                    &settings,
                                    &assets,
                                    &window,
                                );
                            });

                        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
                    }
                    _ => (),
                });

            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(3.),
                    bottom: Val::Percent(6.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(add_text("Created by Mavs", "medium", TITLE_TEXT_SIZE, &assets, &window));
                });
        });
}

pub fn update_ip(
    mut commands: Commands,
    mut btn_q: Query<(Entity, &mut BackgroundColor, &MenuBtn)>,
    mut text_q: Query<&mut Text, With<IpTextCmp>>,
    mut ip: ResMut<Ip>,
    mut not_local_ip: Local<bool>,
    mut invalid_ip: Local<bool>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for key in keyboard.get_just_released() {
        match key {
            KeyCode::Digit0 => ip.0.push('0'),
            KeyCode::Digit1 => ip.0.push('1'),
            KeyCode::Digit2 => ip.0.push('2'),
            KeyCode::Digit3 => ip.0.push('3'),
            KeyCode::Digit4 => ip.0.push('4'),
            KeyCode::Digit5 => ip.0.push('5'),
            KeyCode::Digit6 => ip.0.push('6'),
            KeyCode::Digit7 => ip.0.push('7'),
            KeyCode::Digit8 => ip.0.push('8'),
            KeyCode::Digit9 => ip.0.push('9'),
            KeyCode::Period => ip.0.push('.'),
            KeyCode::Backspace => {
                ip.0.pop();
            },
            KeyCode::Escape => {
                *ip = Ip::default();
            },
            _ => (),
        };
    }

    for (button_e, mut bgcolor, btn) in &mut btn_q {
        match btn {
            MenuBtn::HostGame => {
                if ip.0 == get_local_ip().to_string() {
                    // Only enable once when the ip becomes the local one
                    if *not_local_ip {
                        bgcolor.0 = NORMAL_BUTTON_COLOR;
                        commands.entity(button_e).remove::<DisabledButton>();
                        *not_local_ip = false;
                    }
                } else {
                    commands.entity(button_e).insert(DisabledButton);
                    bgcolor.0 = DISABLED_BUTTON_COLOR;
                    *not_local_ip = true;
                }
            },
            MenuBtn::FindGame => {
                if ip.0.parse::<IpAddr>().is_ok() {
                    // Only enable once when the ip becomes valid
                    if *invalid_ip {
                        bgcolor.0 = NORMAL_BUTTON_COLOR;
                        commands.entity(button_e).remove::<DisabledButton>();
                        *invalid_ip = false;
                    }
                } else {
                    commands.entity(button_e).insert(DisabledButton);
                    bgcolor.0 = DISABLED_BUTTON_COLOR;
                    *invalid_ip = true;
                }
            },
            _ => (),
        }
    }

    if let Ok(mut text) = text_q.single_mut() {
        text.0 = format!("Ip: {}", ip.0);
    }
}

pub fn setup_game_menu(
    mut commands: Commands,
    host: Option<Res<Host>>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands.spawn((add_root_node(true), MenuCmp)).with_children(|parent| {
        spawn_menu_button(parent, MenuBtn::Continue, &assets, &window);
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Only the host can save a multiplayer game
            if host.is_some() {
                spawn_menu_button(parent, MenuBtn::SaveGame, &assets, &window);
            }
        }
        spawn_menu_button(parent, MenuBtn::Settings, &assets, &window);
        spawn_menu_button(parent, MenuBtn::Quit, &assets, &window);
    });
}

pub fn setup_game_settings(
    mut commands: Commands,
    host: Option<Res<Host>>,
    settings: Res<Settings>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    commands.spawn((add_root_node(true), MenuCmp)).with_children(|parent| {
        parent
            .spawn((Node {
                width: Val::Percent(40.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::ZERO.with_top(Val::Percent(-7.)),
                ..default()
            },))
            .with_children(|parent| {
                spawn_label(
                    parent,
                    "Audio",
                    vec![SettingsBtn::Mute, SettingsBtn::NoMusic, SettingsBtn::Sound],
                    &settings,
                    &assets,
                    &window,
                );
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if host.is_some() {
                        spawn_label(
                            parent,
                            "Autosave",
                            vec![SettingsBtn::True, SettingsBtn::False],
                            &settings,
                            &assets,
                            &window,
                        );
                    }
                }
            });

        spawn_menu_button(parent, MenuBtn::Back, &assets, &window);
    });
}

pub fn exit_multiplayer_lobby(
    mut commands: Commands,
    server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    if let Some(client) = client.as_mut() {
        client.disconnect();
        commands.remove_resource::<RenetClient>();
    } else if let Some(mut server) = server {
        server.disconnect_all();
        commands.remove_resource::<RenetServer>();
        commands.remove_resource::<NetcodeServerTransport>();
    }
}

pub fn start_new_game_message(
    mut commands: Commands,
    mut start_new_game_msg: MessageReader<StartNewGameMsg>,
    server: Option<ResMut<RenetServer>>,
    mut settings: ResMut<Settings>,
    mut server_send_msg: MessageWriter<ServerSendMsg>,
    mut spawn_building_msg: MessageWriter<SpawnBuildingMsg>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if !start_new_game_msg.is_empty() {
        let enemy_id = if *app_state.get() == AppState::SinglePlayerMenu {
            settings.enemy_color = match settings.color {
                PlayerColor::Red => PlayerColor::Blue,
                _ => PlayerColor::Red,
            };

            1
        } else {
            let server = server.unwrap();
            *server.clients_id().first().unwrap()
        };

        let map = Map::new(&settings.map_size);

        // Spawn starting buildings
        let positions = map.starting_positions();
        spawn_building_msg.write(SpawnBuildingMsg::new(
            0,
            BuildingName::default(),
            positions[0],
            true,
        ));
        spawn_building_msg.write(SpawnBuildingMsg::new(
            enemy_id,
            BuildingName::default(),
            positions[1],
            true,
        ));

        commands.insert_resource(Host);
        commands.insert_resource(map);
        commands.insert_resource(Players {
            me: Player::new(0, settings.color),
            enemy: Player::new(enemy_id, settings.enemy_color),
        });
        next_game_state.set(GameState::default());
        next_app_state.set(AppState::Game);

        start_new_game_msg.clear();
    }
}
