use crate::core::audio::PlayAudioMsg;
use crate::core::map::map::Map;
use crate::core::mechanics::spawn::{DespawnMsg, SpawnBuildingMsg, SpawnUnitMsg};
use crate::core::menu::systems::Host;
use crate::core::network::{ClientMessage, ClientSendMsg, ServerMessage, ServerSendMsg};
use crate::core::player::{Players, Side};
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::GameState;
use crate::core::units::buildings::{Building, BuildingName};
use crate::core::units::units::{Unit, UnitName};
use crate::utils::scale_duration;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use bevy_renet::renet::RenetServer;
use rand::prelude::IteratorRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
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
    ArmorGain,
    Arrows,
    BlockRange,
    Boss,
    BuildingsBlock,
    BuildingsDefense,
    Castle,
    Clone,
    DoubleQueue,
    InstantHealing,
    InstantArmy,
    Lancer,
    Longbow,
    MagicSwap,
    Meditation,
    NoCollision,
    Penetration,
    Repair,
    Respawn,
    Run,
    Siege,
    SpawnTime,
    Tower,
    Warrior,
}

impl Boost {
    pub fn description(&self) -> &'static str {
        match self {
            Boost::ArmorGain => "Decrease damage to all your units by 30%.",
            Boost::Arrows => "Your archers deal 30% more damage.",
            Boost::BlockRange => "Block all damage on units from enemy ranged units.",
            Boost::Boss => "Spawn a mighty warrior with increased health and damage.",
            Boost::BuildingsBlock => "Block all damage dealt to your buildings.",
            Boost::BuildingsDefense => "Increase the damage of all units on buildings by 100%.",
            Boost::Castle => "Upgrade your base to a castle.",
            Boost::Clone => "Clones 8 random units of yours (in position).",
            Boost::DoubleQueue => "Two units are queued at the same time.",
            Boost::InstantHealing => "Instantly heal all your units to their maximum health.",
            Boost::InstantArmy => "Immediately spawn 6 random units in the base.",
            Boost::Lancer => "Increase your lancer's damage by 60%.",
            Boost::Longbow => "Increase the range of your archers by 50%.",
            Boost::MagicSwap => "All your unit's attack damage become magic damage.",
            Boost::Meditation => "Your priest's healing is 70% stronger.",
            Boost::NoCollision => "Your units don't collide with each other.",
            Boost::Penetration => "Increase the armor penetration of all your units with 5 points.",
            Boost::Repair => "Instantly repair all your buildings to their maximum health.",
            Boost::Respawn => "Respawn all dead units on buildings.",
            Boost::Run => "Increase the speed of all your units by 100%.",
            Boost::Siege => "Increase the damage to buildings by 50%.",
            Boost::SpawnTime => "Reduce all spawning times by 20%.",
            Boost::Tower => "Spawn a defense tower near the base.",
            Boost::Warrior => "Increase your warrior's damage by 50%.",
        }
    }

    pub fn condition<'a>(&self, mut buildings: impl Iterator<Item = &'a Building>) -> bool {
        match self {
            Boost::Castle => !buildings.any(|b| b.name == BuildingName::Castle),
            Boost::Tower => buildings.filter(|b| b.name == BuildingName::Tower).count() < 2,
            _ => true,
        }
    }

    pub fn duration(&self) -> u64 {
        match self {
            Boost::ArmorGain => 20,
            Boost::Arrows => 40,
            Boost::BlockRange => 15,
            Boost::BuildingsBlock => 10,
            Boost::BuildingsDefense => 25,
            Boost::DoubleQueue => 20,
            Boost::Lancer => 40,
            Boost::Longbow => 40,
            Boost::MagicSwap => 40,
            Boost::Meditation => 40,
            Boost::NoCollision => 20,
            Boost::Penetration => 30,
            Boost::Run => 15,
            Boost::Siege => 10,
            Boost::SpawnTime => 90,
            Boost::Warrior => 40,
            _ => 0,
        }
    }
}

pub fn check_boost_timer(
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_settings: ResMut<Settings>,
    time: Res<Time>,
) {
    let time = scale_duration(time.delta(), game_settings.speed);
    game_settings.boost_timer.tick(time);

    if game_settings.boost_timer.is_finished() {
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
    mut unit_q: Query<(&Transform, &mut Unit)>,
    mut building_q: Query<(Entity, &Transform, &mut Building)>,
    host: Option<Res<Host>>,
    players: Res<Players>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    mut spawn_building_msg: MessageWriter<SpawnBuildingMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    mut activate_boost_msg: MessageReader<ActivateBoostMsg>,
    mut client_send_msg: MessageWriter<ClientSendMsg>,
    mut server_send_msg: MessageWriter<ServerSendMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    for msg in activate_boost_msg.read() {
        let player = players.get_by_color(msg.color);

        if host.is_none() {
            play_audio_msg.write(PlayAudioMsg::new("horn"));
            client_send_msg.write(ClientSendMsg::new(ClientMessage::ActivateBoost(msg.boost)));
        } else {
            if players.me.color == msg.color {
                // Activates own boost
                play_audio_msg.write(PlayAudioMsg::new("horn"));
            } else {
                // Activates enemy boost
                play_audio_msg.write(PlayAudioMsg::new("warning"));
                server_send_msg.write(ServerSendMsg::new(ServerMessage::PlayWarning, None));
            }

            let mut rng = rng();
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
                Boost::Clone => {
                    for (unit_t, unit) in unit_q
                        .iter()
                        .filter(|(_, u)| u.color == player.color && u.on_building.is_none())
                        .choose_multiple(&mut rng, 8)
                    {
                        spawn_unit_msg.write(SpawnUnitMsg {
                            color: player.color,
                            unit: unit.name,
                            position: Some(unit_t.translation.truncate()),
                            on_building: None,
                            path: Some(unit.path),
                            entity: None,
                        });
                    }
                },
                Boost::InstantHealing => unit_q
                    .iter_mut()
                    .filter(|(_, u)| u.color == player.color)
                    .for_each(|(_, mut u)| u.health = u.name.health()),
                Boost::InstantArmy => {
                    for unit in UnitName::iter().choose_multiple(&mut rng, 6) {
                        spawn_unit_msg.write(SpawnUnitMsg::new(player.color, unit));
                    }
                },
                Boost::Repair => building_q
                    .iter_mut()
                    .filter(|(_, _, b)| b.color == player.color)
                    .for_each(|(_, _, mut b)| b.health = b.name.health()),
                Boost::Respawn => {
                    let current_positions: Vec<Vec2> = unit_q
                        .iter()
                        .filter_map(|(t, u)| {
                            (u.color == player.color && u.on_building.is_some())
                                .then_some(t.translation.truncate())
                        })
                        .collect();

                    for (e, t, b) in building_q.iter().filter(|(_, _, b)| b.color == player.color) {
                        for pos in b.name.units() {
                            if !current_positions.contains(&pos) {
                                spawn_unit_msg.write(SpawnUnitMsg {
                                    color: b.color,
                                    unit: UnitName::Archer,
                                    position: Some(t.translation.truncate() + pos),
                                    on_building: Some(e),
                                    path: None,
                                    entity: None,
                                });
                            }
                        }
                    }
                },
                Boost::Tower => {
                    let current_positions: Vec<Vec2> = building_q
                        .iter()
                        .filter_map(|(_, t, b)| {
                            (b.color == player.color && b.name == BuildingName::Tower)
                                .then_some(t.translation.truncate())
                        })
                        .collect();

                    let possible_positions = match player.side {
                        Side::Left => [
                            Map::tile_to_world(TilePos::new(7, 0)),
                            Map::tile_to_world(TilePos::new(2, 3)),
                        ],
                        Side::Right => [
                            Map::tile_to_world(TilePos::new(23, 0)),
                            Map::tile_to_world(TilePos::new(26, 4)),
                        ],
                    };

                    // Choose one of the two locations randomly that are not present in current positions
                    let position = possible_positions
                        .into_iter()
                        .filter(|p| !current_positions.contains(p))
                        .choose(&mut rng)
                        .expect("No free tower position.");

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
