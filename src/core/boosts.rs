use crate::core::audio::PlayAudioMsg;
use crate::core::map::map::Map;
use crate::core::mechanics::spawn::{DespawnMsg, SpawnBuildingMsg};
use crate::core::menu::systems::Host;
use crate::core::network::{ClientMessage, ClientSendMsg};
use crate::core::player::{Players, Side};
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::GameState;
use crate::core::units::buildings::{Building, BuildingName};
use crate::utils::scale_duration;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use bevy_renet::renet::RenetServer;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Component)]
pub struct CardCmp;

#[derive(Message)]
pub struct ActivateBoostMsg {
    pub color: PlayerColor,
    pub boost: Boost,
}

impl ActivateBoostMsg {
    pub fn new(color: PlayerColor, boost: Boost) -> Self {
        Self {
            color,
            boost,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AfterBoostCount(pub usize);

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Boost {
    Castle,
    Longbow,
    SpawnTime,
    Tower,
    Warrior,
}

impl Boost {
    pub fn description(&self) -> &'static str {
        match self {
            Boost::Castle => "Upgrade your base to a castle.",
            Boost::Longbow => "Increase the range of your archers by 50%.",
            Boost::SpawnTime => "Reduces all spawning times by 20%.",
            Boost::Tower => "Spawns a defense tower near the base.",
            Boost::Warrior => "Increase your warrior's damage by 30%.",
        }
    }

    /// Whether this boost can only be selected once
    pub fn is_draining(&self) -> bool {
        match self {
            Boost::Castle | Boost::Tower => true,
            _ => false,
        }
    }

    pub fn duration(&self) -> u64 {
        match self {
            Boost::Longbow => 60,
            Boost::SpawnTime => 90,
            Boost::Warrior => 40,
            _ => 0,
        }
    }
}

pub fn check_boost_timer(
    mut play_audio_ev: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_settings: ResMut<Settings>,
    time: Res<Time>,
) {
    let time = scale_duration(time.delta(), game_settings.speed);
    game_settings.boost_timer.tick(time);

    if game_settings.boost_timer.is_finished() {
        play_audio_ev.write(PlayAudioMsg::new("message"));
        next_game_state.set(GameState::BoostSelection);
    }
}

pub fn update_boosts(settings: Res<Settings>, mut players: ResMut<Players>, time: Res<Time>) {
    players.me.boosts.retain_mut(|boost| {
        if boost.active {
            boost.timer.tick(scale_duration(time.delta(), settings.speed));
            return !boost.timer.just_finished();
        }
        true
    });
}

pub fn activate_boost_message(
    building_q: Query<(Entity, &Transform, &Building)>,
    host: Option<Res<Host>>,
    players: Res<Players>,
    mut spawn_building_msg: MessageWriter<SpawnBuildingMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    mut activate_boost_msg: MessageReader<ActivateBoostMsg>,
    mut client_send_msg: MessageWriter<ClientSendMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    for msg in activate_boost_msg.read() {
        let player = players.get_by_color(msg.color);

        play_audio_msg.write(PlayAudioMsg::new("horn"));

        if host.is_none() {
            client_send_msg.write(ClientSendMsg::new(ClientMessage::ActivateBoost(msg.boost)));
        } else {
            match msg.boost {
                Boost::Castle => {
                    if let Some((base_e, base_t, base)) =
                        building_q.iter().find(|(_, _, b)| b.is_base && b.color == player.color)
                    {
                        despawn_msg.write(DespawnMsg(base_e));

                        spawn_building_msg.write(SpawnBuildingMsg {
                            color: player.color,
                            building: BuildingName::Castle,
                            position: base_t.translation.truncate(),
                            is_base: true,
                            health: BuildingName::Castle.health() * base.health
                                / base.name.health(),
                            with_units: true,
                            entity: None,
                        });
                    }
                },
                Boost::Tower => {
                    let position = match player.side {
                        Side::Left => Map::tile_to_world(TilePos::new(2, 3)),
                        Side::Right => Map::tile_to_world(TilePos::new(26, 4)),
                    };

                    spawn_building_msg.write(SpawnBuildingMsg {
                        color: player.color,
                        building: BuildingName::Tower,
                        position,
                        is_base: false,
                        health: BuildingName::Tower.health(),
                        with_units: true,
                        entity: None,
                    });
                },
                _ => (),
            }
        }
    }
}

pub fn after_boost_check(
    server: Res<RenetServer>,
    mut boost_count: ResMut<AfterBoostCount>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if *game_state.get() == GameState::AfterBoostSelection
        && **boost_count == server.clients_id().len()
    {
        **boost_count = 0;
        next_game_state.set(GameState::Playing);
    }
}
