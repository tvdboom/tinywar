use std::net::{IpAddr, UdpSocket};
use std::time::SystemTime;

use crate::core::audio::PlayAudioMsg;
use crate::core::boosts::{ActivateBoostMsg, AfterBoostCount, Boost};
use crate::core::constants::MAX_BOOSTS;
use crate::core::mechanics::effects::{Effect, EffectMsg};
use crate::core::mechanics::spawn::SpawnUnitMsg;
use crate::core::menu::buttons::LobbyTextCmp;
use crate::core::menu::systems::Host;
use crate::core::multiplayer::{EntityMap, Population, UpdatePopulationMsg};
use crate::core::player::{Player, Players, SelectedBoost, Side, Strategy};
use crate::core::settings::{GameMode, PlayerColor, Settings};
use crate::core::states::{AppState, GameState};
use crate::core::units::units::UnitName;
use crate::core::utils::ClientId;
use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::{ConnectionConfig, DefaultChannel, ServerEvent};
use bevy_renet::*;
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
    NPlayers(usize),
    StartGame {
        player: Player,
        enemy_color: PlayerColor,
    },
    State(GameState),
    Status {
        speed: f32,
        boosts: Vec<SelectedBoost>,
        strategy: Strategy,
        population: Population,
    },
    Effect {
        effect: Effect,
        entity: Entity,
    },
    PlayWarning,
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
    Status(Player),
    SpawnUnit(UnitName),
    ActivateBoost(Boost),
}

impl ClientMessage {
    pub fn channel(&self) -> DefaultChannel {
        DefaultChannel::ReliableOrdered
    }
}

pub fn local_ip() -> IpAddr {
    if cfg!(target_arch = "wasm32") {
        // WebAssembly in browsers cannot access local network interfaces
        "127.0.0.1".parse().unwrap()
    } else {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Socket not found.");

        if socket.connect("8.8.8.8:80").is_ok() {
            socket.local_addr().ok().map(|addr| addr.ip()).unwrap()
        } else {
            // Fails if not connected to internet
            "127.0.0.1".parse().unwrap()
        }
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
    event: On<RenetServerEvent>,
    mut n_players_q: Query<&mut Text, With<LobbyTextCmp>>,
    mut server: ResMut<RenetServer>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    match **event {
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
    mut players: Option<ResMut<Players>>,
    mut boost_count: ResMut<AfterBoostCount>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    mut activate_boost_msg: MessageWriter<ActivateBoostMsg>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for id in server.clients_id() {
        while let Some(message) = server.receive_message(id, DefaultChannel::ReliableOrdered) {
            let (d, _) = decode_from_slice(&message, standard()).unwrap();
            match d {
                ClientMessage::ShareColor(enemy_color) => settings.enemy_color = enemy_color,
                ClientMessage::State(state) => match state {
                    GameState::GameMenu | GameState::Paused | GameState::UnitInfo
                        if *game_state.get() == GameState::Playing =>
                    {
                        next_game_state.set(GameState::Paused);
                    },
                    GameState::Playing => next_game_state.set(state),
                    GameState::AfterBoostSelection => **boost_count += 1,
                    _ => (),
                },
                ClientMessage::Status(player) => {
                    if let Some(players) = &mut players {
                        players.enemy = player;
                    }
                },
                ClientMessage::SpawnUnit(unit) => {
                    spawn_unit_msg.write(SpawnUnitMsg::new(settings.enemy_color, unit));
                },
                ClientMessage::ActivateBoost(boost) => {
                    activate_boost_msg.write(ActivateBoostMsg::new(boost, settings.enemy_color));
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
    mut players: Option<ResMut<Players>>,
    mut boost_count: ResMut<AfterBoostCount>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut client_send_msg: MessageWriter<ClientSendMsg>,
    mut update_population_msg: MessageWriter<UpdatePopulationMsg>,
    mut effect_msg: MessageWriter<EffectMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
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
                player,
                enemy_color,
            } => {
                settings.reset();
                settings.game_mode = GameMode::Multiplayer;
                settings.color = player.color;
                settings.enemy_color = enemy_color;

                commands.remove_resource::<Host>();
                commands.insert_resource(EntityMap::default());
                commands.insert_resource(AfterBoostCount::default());
                commands.insert_resource(Players {
                    me: player,
                    enemy: Player::new(0, enemy_color, Side::Left),
                });
                next_game_state.set(GameState::default());
                next_app_state.set(AppState::Game);
            },
            ServerMessage::State(state) => match state {
                GameState::GameMenu | GameState::Paused | GameState::UnitInfo
                    if *game_state.get() == GameState::Playing =>
                {
                    next_game_state.set(GameState::Paused)
                },
                GameState::Playing | GameState::EndGame => {
                    **boost_count = 0;
                    next_game_state.set(state)
                },
                GameState::BoostSelection => {
                    if let Some(players) = &players {
                        if players.me.boosts.len() != MAX_BOOSTS {
                            next_game_state.set(state)
                        } else {
                            next_game_state.set(GameState::AfterBoostSelection)
                        }
                    }
                },
                GameState::AfterBoostSelection => {
                    **boost_count += 1;
                    if !matches!(
                        game_state.get(),
                        GameState::BoostSelection | GameState::AfterBoostSelection
                    ) {
                        next_game_state.set(GameState::BoostSelection);
                    }
                },
                _ => (),
            },
            ServerMessage::Effect {
                effect,
                entity,
            } => {
                effect_msg.write(EffectMsg {
                    effect,
                    entity,
                });
            },
            ServerMessage::PlayWarning => {
                play_audio_msg.write(PlayAudioMsg::new("warning"));
            },
            _ => unreachable!(),
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let (d, _) = decode_from_slice(&message, standard()).unwrap();
        match d {
            ServerMessage::Status {
                speed,
                strategy,
                boosts,
                population,
            } => {
                settings.speed = speed;

                if let Some(players) = &mut players {
                    players.enemy.strategy = strategy;
                    players.enemy.boosts = boosts;
                }

                update_population_msg.write(UpdatePopulationMsg(population));
            },
            _ => unreachable!(),
        }
    }
}
