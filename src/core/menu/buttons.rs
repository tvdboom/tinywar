use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy_renet::netcode::{NetcodeClientTransport, NetcodeServerTransport};
use bevy_renet::renet::{RenetClient, RenetServer};

use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::menu::utils::{add_text, cursor, recolor};
use crate::core::network::{new_renet_client, new_renet_server, Ip, ServerSendMsg};
use crate::core::persistence::{LoadGameMsg, SaveGameMsg};
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
use crate::utils::NameFromEnum;

#[derive(Component)]
pub struct MenuCmp;

#[derive(Component, Clone, Debug, PartialEq)]
pub enum MenuBtn {
    Singleplayer,
    Multiplayer,
    NewGame,
    LoadGame,
    HostGame,
    FindGame,
    Back,
    Continue,
    SaveGame,
    Settings,
    Quit,
}

#[derive(Component)]
pub struct DisabledButton;

#[derive(Component)]
pub struct LobbyTextCmp;

#[derive(Component)]
pub struct IpTextCmp;

pub fn on_click_menu_button(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    btn_q: Query<(Option<&DisabledButton>, &MenuBtn)>,
    server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
    mut settings: ResMut<Settings>,
    ip: Res<Ip>,
    mut load_game_msg: MessageWriter<LoadGameMsg>,
    mut save_game_msg: MessageWriter<SaveGameMsg>,
    mut server_send_msg: MessageWriter<ServerSendMsg>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let (disabled, btn) = btn_q.get(event.entity).unwrap();

    if disabled.is_some() {
        return;
    }

    match btn {
        MenuBtn::Singleplayer => {
            next_app_state.set(AppState::SinglePlayerMenu);
        },
        MenuBtn::Multiplayer => {
            next_app_state.set(AppState::MultiPlayerMenu);
        },
        MenuBtn::NewGame => {
            if *app_state.get() == AppState::SinglePlayerMenu {
            } else {
                let server = server.unwrap();

                let clients = server.clients_id();
                let n_players = clients.len() + 1;
            }

            next_app_state.set(AppState::Game);
        },
        MenuBtn::LoadGame => {
            load_game_msg.write(LoadGameMsg);
        },
        MenuBtn::HostGame => {
            // Remove client resources if they exist
            if client.is_some() {
                commands.remove_resource::<RenetClient>();
                commands.remove_resource::<NetcodeClientTransport>();
            }

            let (server, transport) = new_renet_server();
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_app_state.set(AppState::Lobby);
        },
        MenuBtn::FindGame => {
            let (server, transport) = new_renet_client(&ip.0);
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_app_state.set(AppState::Lobby);
        },
        MenuBtn::Back => match *app_state.get() {
            AppState::SinglePlayerMenu | AppState::MultiPlayerMenu | AppState::Settings => {
                next_app_state.set(AppState::MainMenu);
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

                next_app_state.set(AppState::MultiPlayerMenu);
            },
            AppState::Game => {
                next_game_state.set(GameState::GameMenu);
            },
            _ => unreachable!(),
        },
        MenuBtn::Continue => {
            next_game_state.set(GameState::Playing);
        },
        MenuBtn::SaveGame => {
            save_game_msg.write(SaveGameMsg(false));
        },
        MenuBtn::Settings => {
            if *game_state.get() == GameState::GameMenu {
                next_game_state.set(GameState::Settings);
            } else {
                next_app_state.set(AppState::Settings);
            }
        },
        MenuBtn::Quit => match *app_state.get() {
            AppState::Game => {
                if let Some(client) = client.as_mut() {
                    client.disconnect();
                    commands.remove_resource::<RenetClient>();
                } else if let Some(mut server) = server {
                    server.disconnect_all();
                    commands.remove_resource::<RenetServer>();
                    commands.remove_resource::<NetcodeServerTransport>();
                }

                next_game_state.set(GameState::default());
                next_app_state.set(AppState::MainMenu)
            },
            AppState::MainMenu => std::process::exit(0),
            _ => unreachable!(),
        },
    }
}

pub fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    btn: MenuBtn,
    assets: &WorldAssets,
    window: &Window,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(25.),
                height: Val::Percent(10.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Percent(1.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            btn.clone(),
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(on_click_menu_button)
        .with_children(|parent| {
            parent.spawn(add_text(btn.to_title(), "bold", BUTTON_TEXT_SIZE, assets, window));
        });
}
