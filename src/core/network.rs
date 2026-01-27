use std::net::{IpAddr, UdpSocket};
use std::time::SystemTime;

use crate::core::mechanics::spawn::SpawnUnitMsg;
use crate::core::menu::buttons::LobbyTextCmp;
use crate::core::multiplayer::{Population, UpdatePopulationMsg};
use crate::core::player::{Player, Players, Side};
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::{AppState, GameState};
use crate::core::units::units::UnitName;
use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::*;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};

const PROTOCOL_ID: u64 = 7;

#[derive(Resource, Deref, DerefMut)]
pub struct Ip(pub String);

impl Default for Ip {
    fn default() -> Self {
        Self(local_ip().to_string())
    }
}

#[derive(Message)]
pub struct ServerSendMsg {
    pub message: ServerMessage,
    pub client: Option<ClientId>,
}

impl ServerSendMsg {
    pub fn new(message: ServerMessage, client: Option<ClientId>) -> Self {
        Self {
            message,
            client,
        }
    }
}

#[derive(Message)]
pub struct ClientSendMsg {
    pub message: ClientMessage,
}

impl ClientSendMsg {
    pub fn new(message: ClientMessage) -> Self {
        Self {
            message,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    LoadGame {
        turn: usize,
        p_colonizable: usize,
    },
    NPlayers(usize),
    StartGame {
        id: ClientId,
        color: PlayerColor,
        enemy_id: ClientId,
        enemy_color: PlayerColor,
    },
    State(GameState),
    Status {
        speed: f32,
        population: Population,
    },
}

impl ServerMessage {
    pub fn channel(&self) -> DefaultChannel {
        match self {
            ServerMessage::Status {
                ..
            } => DefaultChannel::Unreliable,
            _ => DefaultChannel::ReliableOrdered,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    ShareColor(PlayerColor),
    State(GameState),
    SpawnUnit(UnitName),
}

impl ClientMessage {
    pub fn channel(&self) -> DefaultChannel {
        DefaultChannel::ReliableOrdered
    }
}

pub fn local_ip() -> IpAddr {
    if cfg!(target_arch = "wasm32") {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Socket not found.");

        socket.connect("8.8.8.8:80").expect("Failed to connect to socket.");
        socket.local_addr().ok().map(|addr| addr.ip()).unwrap()
    } else {
        // WebAssembly in browsers cannot access local network interfaces
        "127.0.0.1".parse().unwrap()
    }
}

pub fn new_renet_client(ip: &String) -> (RenetClient, NetcodeClientTransport) {
    let server_addr = format!("{ip}:5000").parse().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(ConnectionConfig::default());

    println!("Client created.");
    (client, transport)
}

pub fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let public_addr = "0.0.0.0:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).expect("Socket already in use.");
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 4,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let server = RenetServer::new(ConnectionConfig::default());

    println!("Server created.");
    (server, transport)
}

pub fn server_update(
    mut n_players_q: Query<&mut Text, With<LobbyTextCmp>>,
    mut server: ResMut<RenetServer>,
    mut server_msg: MessageReader<ServerEvent>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for msg in server_msg.read() {
        match msg {
            ServerEvent::ClientConnected {
                client_id,
            } => {
                println!("Client {client_id} connected");
            },
            ServerEvent::ClientDisconnected {
                client_id,
                reason,
            } => {
                println!("Client {client_id} disconnected. Reason: {reason}.");

                if *app_state == AppState::Game {
                    next_game_state.set(GameState::GameMenu);
                }
            },
        }

        if *app_state != AppState::Game {
            let n_players = server.clients_id().len() + 1;

            // Update the number of players in the lobby
            let message = encode_to_vec(ServerMessage::NPlayers(n_players), standard()).unwrap();
            server.broadcast_message(DefaultChannel::ReliableOrdered, message);

            if let Ok(mut text) = n_players_q.single_mut() {
                if n_players == 1 {
                    text.0 = format!("Waiting for other players to join {}...", local_ip());
                    next_app_state.set(AppState::Lobby);
                } else {
                    text.0 = format!("There are {n_players} players in the lobby.\nWaiting for other players to join {}...", local_ip());
                    next_app_state.set(AppState::ConnectedLobby);
                }
            }
        }
    }
}

pub fn server_send_message(
    mut server_send_msg: MessageReader<ServerSendMsg>,
    mut server: ResMut<RenetServer>,
) {
    for msg in server_send_msg.read() {
        let message = encode_to_vec(&msg.message, standard()).unwrap();
        if let Some(client_id) = msg.client {
            server.send_message(client_id, msg.message.channel(), message);
        } else {
            server.broadcast_message(msg.message.channel(), message);
        }
    }
}

pub fn server_receive_message(
    mut server: ResMut<RenetServer>,
    mut settings: ResMut<Settings>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for id in server.clients_id() {
        while let Some(message) = server.receive_message(id, DefaultChannel::ReliableOrdered) {
            let (d, _) = decode_from_slice(&message, standard()).unwrap();
            match d {
                ClientMessage::ShareColor(enemy_color) => settings.enemy_color = enemy_color,
                ClientMessage::State(state) => match state {
                    GameState::GameMenu | GameState::Paused
                        if *game_state.get() == GameState::Playing =>
                    {
                        next_game_state.set(GameState::Paused);
                    },
                    GameState::Playing => next_game_state.set(GameState::Playing),
                    GameState::EndGame => next_game_state.set(GameState::EndGame),
                    _ => (),
                },
                ClientMessage::SpawnUnit(unit) => {
                    spawn_unit_msg.write(SpawnUnitMsg::new(settings.enemy_color, unit));
                },
            }
        }
    }
}

pub fn client_send_message(
    mut client_send_msg: MessageReader<ClientSendMsg>,
    mut client: ResMut<RenetClient>,
) {
    for msg in client_send_msg.read() {
        let message = encode_to_vec(&msg.message, standard()).unwrap();
        client.send_message(msg.message.channel(), message);
    }
}

pub fn client_receive_message(
    mut commands: Commands,
    mut n_players_q: Query<&mut Text, With<LobbyTextCmp>>,
    mut client: ResMut<RenetClient>,
    mut settings: ResMut<Settings>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut client_send_msg: MessageWriter<ClientSendMsg>,
    mut update_population_msg: MessageWriter<UpdatePopulationMsg>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let (d, _) = decode_from_slice(&message, standard()).unwrap();
        match d {
            ServerMessage::NPlayers(i) => {
                if let Ok(mut text) = n_players_q.single_mut() {
                    text.0 = format!("There are {i} players in the lobby.\nWaiting for the host to start the game...");
                }

                client_send_msg
                    .write(ClientSendMsg::new(ClientMessage::ShareColor(settings.color)));
            },
            ServerMessage::StartGame {
                id,
                color,
                enemy_id,
                enemy_color,
            } => {
                settings.color = color;
                settings.enemy_color = enemy_color;

                commands.insert_resource(Players {
                    me: Player::new(id, color, Side::Right),
                    enemy: Player::new(enemy_id, enemy_color, Side::Left),
                });
                next_game_state.set(GameState::default());
                next_app_state.set(AppState::Game);
            },
            ServerMessage::LoadGame {
                turn,
                p_colonizable,
            } => {
                next_app_state.set(AppState::Game);
            },
            ServerMessage::State(state) => match state {
                GameState::GameMenu | GameState::Paused
                    if *game_state.get() == GameState::Playing =>
                {
                    next_game_state.set(GameState::Paused)
                },
                s @ GameState::Playing => next_game_state.set(s),
                _ => (),
            },
            _ => unreachable!(),
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let (d, _) = decode_from_slice(&message, standard()).unwrap();
        match d {
            ServerMessage::Status {
                speed,
                population,
            } => {
                settings.speed = speed;
                update_population_msg.write(UpdatePopulationMsg(population));
            },
            _ => unreachable!(),
        }
    }
}
