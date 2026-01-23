use std::env::current_dir;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

use crate::core::menu::systems::Host;
use crate::core::network::ServerSendMsg;
use crate::core::settings::Settings;
use crate::core::states::AppState;
use crate::TITLE;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub enum SaveState {
    #[default]
    WaitingForUpdate,
    SaveGame,
    WaitingForClients,
}

#[derive(Serialize, Deserialize)]
pub struct SaveAll {
    pub settings: Settings,
}

#[derive(Message)]
pub struct LoadGameMsg;

#[derive(Message)]
pub struct SaveGameMsg(pub bool);

fn save_to_bin(file_path: &str, data: &SaveAll) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    let buffer = encode_to_vec(data, standard()).expect("Failed to serialize data.");
    file.write_all(&buffer)?;

    Ok(())
}

fn load_from_bin(file_path: &str) -> io::Result<SaveAll> {
    let mut file = File::open(file_path)?;

    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    let (data, _) = decode_from_slice(&buffer, standard()).expect("Failed to deserialize data.");
    Ok(data)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game(
    mut commands: Commands,
    mut load_game_msg: MessageReader<LoadGameMsg>,
    server: Option<Res<RenetServer>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut server_send_msg: MessageWriter<ServerSendMsg>,
) {
    for _ in load_game_msg.read() {
        if let Some(file_path) = FileDialog::new().pick_file() {
            let file_path_str = file_path.to_string_lossy().to_string();
            let mut data = load_from_bin(&file_path_str).expect("Failed to load the game.");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(
    server: Option<Res<RenetServer>>,
    mut save_game_msg: MessageReader<SaveGameMsg>,
    mut server_send_msg: MessageWriter<ServerSendMsg>,
    settings: Res<Settings>,
    mut host: ResMut<Host>,
    mut state: Local<SaveState>,
    mut autosave: Local<bool>,
) {
    let save_game = |autosave: bool| {
        let file_path = if autosave {
            let mut path = current_dir().expect("Failed to get current directory.");
            path.push(TITLE);
            Some(path)
        } else {
            FileDialog::new().save_file()
        };

        if let Some(mut file_path) = file_path {
            if !file_path.extension().map(|e| e == "bin").unwrap_or(false) {
                file_path.set_extension("bin");
            }

            let file_path_str = file_path.to_string_lossy().to_string();
            let data = SaveAll {
                settings: *settings,
            };

            save_to_bin(&file_path_str, &data).expect("Failed to save the game.");
        }
    };

    if server.is_some() {
        match *state {
            SaveState::WaitingForUpdate => {
                for SaveGameMsg(save) in save_game_msg.read() {
                    *autosave = *save;
                    *state = SaveState::WaitingForClients;
                }
            },
            SaveState::WaitingForClients => {
                // Wait until all playing clients have sent an update
                // if host.received.len() == host.clients.values().filter(|c| !c.spectator).count() {
                //     *state = SaveState::SaveGame;
                // }
            },
            SaveState::SaveGame => {
                save_game(*autosave);
                *state = SaveState::WaitingForUpdate;
            },
        }
    } else {
        for SaveGameMsg(autosave) in save_game_msg.read() {
            save_game(*autosave);
        }
    }
}
