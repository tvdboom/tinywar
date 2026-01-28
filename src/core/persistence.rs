use std::env::current_dir;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

use crate::core::audio::ChangeAudioMsg;
use crate::core::mechanics::combat::Arrow;
use crate::core::menu::systems::Host;
use crate::core::multiplayer::{Population, UpdatePopulationMsg};
use crate::core::network::{ServerMessage, ServerSendMsg};
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
use crate::core::units::buildings::Building;
use crate::core::units::units::Unit;
use crate::TITLE;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveAll {
    pub settings: Settings,
    pub players: Players,
    pub population: Population,
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
    mut server_send_msg: MessageWriter<ServerSendMsg>,
    mut update_population_msg: MessageWriter<UpdatePopulationMsg>,
    mut change_audio_msg: MessageWriter<ChangeAudioMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for _ in load_game_msg.read() {
        if let Some(file_path) = FileDialog::new().pick_file() {
            let file_path_str = file_path.to_string_lossy().to_string();
            let mut data = load_from_bin(&file_path_str).expect("Failed to load the game.");

            let ids = data
                .players
                .iter()
                .filter_map(|p| p.is_human().then_some(p.id))
                .collect::<Vec<_>>();

            let n_humans = ids.len();
            if n_humans > 1 {
                if let Some(server) = server.as_ref() {
                    let n_clients = server.clients_id().len();
                    if n_clients != n_humans - 1 {
                        panic!("The loaded game contains {n_humans} players but the server has {} players.", n_clients + 1);
                    } else {
                        for (new_id, old_id) in server.clients_id().iter().zip(ids.iter().skip(1)) {
                            let player = data.players.iter_mut().find(|p| p.id == *old_id).unwrap();

                            // Update everything to the new player id
                            player.id = *new_id;

                            server_send_msg.write(ServerSendMsg::new(
                                ServerMessage::StartGame {
                                    player: player.clone(),
                                    enemy_color: data.settings.color,
                                },
                                Some(player.id),
                            ));
                        }
                    }
                } else {
                    panic!("The loaded game contains {n_humans} players but there is no server initiated.");
                }
            }

            update_population_msg.write(UpdatePopulationMsg(data.population));

            change_audio_msg.write(ChangeAudioMsg(Some(data.settings.audio)));

            commands.insert_resource(Host);
            commands.insert_resource(data.settings);
            commands.insert_resource(data.players);

            next_game_state.set(GameState::default());
            next_app_state.set(AppState::Game);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(
    unit_q: Query<(Entity, &Transform, &Sprite, &Unit)>,
    building_q: Query<(Entity, &Transform, &Building)>,
    arrow_q: Query<(Entity, &Transform, &Sprite, &Arrow)>,
    mut save_game_msg: MessageReader<SaveGameMsg>,
    settings: Res<Settings>,
    players: Res<Players>,
) {
    for msg in save_game_msg.read() {
        let file_path = if msg.0 {
            let mut path = current_dir().expect("Failed to get current directory.");
            path.push(TITLE.to_lowercase());
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
                settings: settings.clone(),
                players: players.clone(),
                population: Population {
                    units: unit_q
                        .iter()
                        .map(|(e, t, s, u)| (e, (t.translation.truncate(), s.flip_x, *u)))
                        .collect(),
                    buildings: building_q
                        .iter()
                        .map(|(e, t, b)| (e, (t.translation.truncate(), *b)))
                        .collect(),
                    arrows: arrow_q
                        .iter()
                        .map(|(e, t, s, a)| (e, (t.translation, t.rotation, s.rect, a.clone())))
                        .collect(),
                },
            };

            save_to_bin(&file_path_str, &data).expect("Failed to save the game.");
        }
    }
}

pub fn run_autosave(settings: Res<Settings>, mut save_game_msg: MessageWriter<SaveGameMsg>) {
    if settings.autosave {
        save_game_msg.write(SaveGameMsg(true));
    }
}
