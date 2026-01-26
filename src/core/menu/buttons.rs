use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::menu::systems::StartNewGameMsg;
use crate::core::menu::utils::{add_text, recolor};
use crate::core::states::{AppState, GameState};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
#[cfg(not(target_arch = "wasm32"))]
use {
    crate::core::network::{new_renet_client, new_renet_server, Ip},
    crate::core::persistence::{LoadGameMsg, SaveGameMsg},
};

#[derive(Component)]
pub struct MenuCmp;

#[derive(Component, Clone, Debug, PartialEq)]
pub enum MenuBtn {
    Singleplayer,
    #[cfg(not(target_arch = "wasm32"))]
    Multiplayer,
    NewGame,
    #[cfg(not(target_arch = "wasm32"))]
    LoadGame,
    #[cfg(not(target_arch = "wasm32"))]
    HostGame,
    #[cfg(not(target_arch = "wasm32"))]
    FindGame,
    Back,
    Continue,
    #[cfg(not(target_arch = "wasm32"))]
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
    #[cfg(not(target_arch = "wasm32"))] ip: Res<Ip>,
    mut start_new_game_msg: MessageWriter<StartNewGameMsg>,
    #[cfg(not(target_arch = "wasm32"))] mut load_game_msg: MessageWriter<LoadGameMsg>,
    #[cfg(not(target_arch = "wasm32"))] mut save_game_msg: MessageWriter<SaveGameMsg>,
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
        #[cfg(not(target_arch = "wasm32"))]
        MenuBtn::Multiplayer => {
            next_app_state.set(AppState::MultiPlayerMenu);
        },
        MenuBtn::NewGame => {
            start_new_game_msg.write(StartNewGameMsg);
        },
        #[cfg(not(target_arch = "wasm32"))]
        MenuBtn::LoadGame => {
            load_game_msg.write(LoadGameMsg);
        },
        #[cfg(not(target_arch = "wasm32"))]
        MenuBtn::HostGame => {
            let (server, transport) = new_renet_server();
            commands.insert_resource(server);
            commands.insert_resource(transport);

            next_app_state.set(AppState::Lobby);
        },
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
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
        .observe(cursor::<Release>(SystemCursorIcon::Default))
        .observe(on_click_menu_button)
        .with_children(|parent| {
            parent.spawn(add_text(btn.to_title(), "bold", BUTTON_TEXT_SIZE, assets, window));
        });
}
