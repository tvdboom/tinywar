use std::net::IpAddr;

use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use {
    crate::core::network::local_ip,
    crate::core::network::{Ip, ServerMessage, ServerSendMsg},
    bevy_renet::netcode::NetcodeServerTransport,
    bevy_renet::renet::{RenetClient, RenetServer},
};

use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::map::map::Map;
use crate::core::mechanics::spawn::SpawnBuildingMsg;
use crate::core::menu::buttons::*;
use crate::core::menu::settings::{spawn_label, SettingsBtn};
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::multiplayer::EntityMap;
use crate::core::player::{Player, Players, Side};
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::{AppState, GameState};
use crate::core::units::buildings::BuildingName;

#[derive(Resource)]
pub struct Host;

#[derive(Message)]
pub struct StartNewGameMsg;

pub fn setup_menu(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    #[cfg(not(target_arch = "wasm32"))] server: Option<Res<RenetServer>>,
    settings: Res<Settings>,
    #[cfg(not(target_arch = "wasm32"))] ip: Res<Ip>,
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
                    #[cfg(not(target_arch = "wasm32"))]
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
                    #[cfg(not(target_arch = "wasm32"))]
                    AppState::Lobby | AppState::ConnectedLobby => {
                        if let Some(server) = server {
                            let n_players = server.clients_id().len() + 1;

                            parent.spawn((
                                add_text(
                                    if n_players == 1 {
                                        format!("Waiting for other players to join {}...", local_ip())
                                    } else {
                                        format!("There are {n_players} players in the lobby.\nWaiting for other players to join {}...", local_ip())
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
                                    "Audio",
                                    vec![
                                        SettingsBtn::Mute,
                                        SettingsBtn::Sound,
                                        SettingsBtn::Music,
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

#[cfg(not(target_arch = "wasm32"))]
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
            KeyCode::Digit0 => ip.push('0'),
            KeyCode::Digit1 => ip.push('1'),
            KeyCode::Digit2 => ip.push('2'),
            KeyCode::Digit3 => ip.push('3'),
            KeyCode::Digit4 => ip.push('4'),
            KeyCode::Digit5 => ip.push('5'),
            KeyCode::Digit6 => ip.push('6'),
            KeyCode::Digit7 => ip.push('7'),
            KeyCode::Digit8 => ip.push('8'),
            KeyCode::Digit9 => ip.push('9'),
            KeyCode::Period => ip.push('.'),
            KeyCode::Backspace => {
                ip.pop();
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
                if ip.0 == local_ip().to_string() {
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
                if ip.parse::<IpAddr>().is_ok() {
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
                    vec![SettingsBtn::Mute, SettingsBtn::Sound, SettingsBtn::Music],
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

#[cfg(not(target_arch = "wasm32"))]
pub fn exit_multiplayer_lobby(
    mut commands: Commands,
    server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
) {
    if let Some(client) = client.as_mut() {
        client.disconnect();
        commands.remove_resource::<RenetClient>();
        println!("Client removed.");
    } else if let Some(mut server) = server {
        server.disconnect_all();
        commands.remove_resource::<RenetServer>();
        commands.remove_resource::<NetcodeServerTransport>();
        println!("Server removed.");
    }
}

pub fn start_new_game_message(
    mut commands: Commands,
    mut start_new_game_msg: MessageReader<StartNewGameMsg>,
    #[cfg(not(target_arch = "wasm32"))] server: Option<ResMut<RenetServer>>,
    mut settings: ResMut<Settings>,
    #[cfg(not(target_arch = "wasm32"))] mut server_send_msg: MessageWriter<ServerSendMsg>,
    mut spawn_building_msg: MessageWriter<SpawnBuildingMsg>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if !start_new_game_msg.is_empty() {
        let (enemy_id, enemy_color) = if *app_state.get() == AppState::SinglePlayerMenu {
            let enemy_color = match settings.color {
                PlayerColor::Red => PlayerColor::Blue,
                _ => PlayerColor::Red,
            };

            (1, enemy_color)
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let enemy_id = *server.unwrap().clients_id().first().unwrap();
                let enemy_color = if settings.color == settings.enemy_color {
                    match settings.color {
                        PlayerColor::Red => PlayerColor::Blue,
                        _ => PlayerColor::Red,
                    }
                } else {
                    settings.enemy_color
                };

                server_send_msg.write(ServerSendMsg::new(
                    ServerMessage::StartGame {
                        id: enemy_id,
                        color: enemy_color,
                        enemy_id: 0,
                        enemy_color: settings.color,
                    },
                    Some(enemy_id),
                ));

                (enemy_id, enemy_color)
            }

            #[cfg(target_arch = "wasm32")]
            unreachable!()
        };

        settings.enemy_color = enemy_color;

        // Spawn starting buildings
        for (color, position) in
            [settings.color, enemy_color].into_iter().zip(Map::starting_positions())
        {
            spawn_building_msg.write(SpawnBuildingMsg {
                color,
                building: BuildingName::default(),
                position,
                is_base: true,
                with_units: true,
                entity: None,
            });
        }

        commands.insert_resource(Host);
        commands.insert_resource(EntityMap::default());
        commands.insert_resource(Players {
            me: Player::new(0, settings.color, Side::Left),
            enemy: Player::new(enemy_id, enemy_color, Side::Right),
        });
        next_game_state.set(GameState::default());
        next_app_state.set(AppState::Game);

        start_new_game_msg.clear();
    }
}
