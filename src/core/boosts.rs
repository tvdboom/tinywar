use crate::core::audio::PlayAudioMsg;
use crate::core::constants::MAX_BOOSTS;
use crate::core::map::map::{Lane, Map};
use crate::core::mechanics::effects::EffectMsg;
use crate::core::mechanics::spawn::{DespawnMsg, SpawnBuildingMsg, SpawnUnitMsg};
use crate::core::menu::systems::Host;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::network::{ClientMessage, ClientSendMsg, ServerMessage, ServerSendMsg};
use crate::core::player::{Player, Players, SelectedBoost, Side};
use crate::core::settings::{GameMode, PlayerColor, Settings};
use crate::core::states::GameState;
use crate::core::units::buildings::{Building, BuildingName};
use crate::core::units::units::{Action, Unit, UnitName};
use crate::utils::{scale_duration, NameFromEnum};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Component)]
pub struct CardCmp;

#[derive(Message)]
pub struct ActivateBoostMsg {
    pub boost: Boost,
    pub color: PlayerColor,
}

impl ActivateBoostMsg {
    pub fn new(boost: Boost, color: PlayerColor) -> Self {
        Self {
            boost,
            color,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AfterBoostCount(pub usize);

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Boost {
    ArmorGain,
    Arrows,
    BearDefender,
    BlockRange,
    BuildingsBlock,
    BuildingsDefense,
    Castle,
    Clone,
    Conversion,
    ConvertGoblins,
    ConvertHammerheads,
    ConvertSharks,
    DoubleQueue,
    Frozen,
    GnomesBasic,
    GnomesMagic,
    InstantHealing,
    InstantArmy,
    Lancer,
    Lightning,
    Longbow,
    MagicPower,
    MagicSwap,
    Meditation,
    MinotaurRage,
    NoCollision,
    Penetration,
    QueueBears,
    QueueGnolls,
    QueueGoblins,
    QueueHammerheads,
    QueueMinotaurs,
    QueueShamans,
    QueueSharks,
    QueueSkulls,
    QueueTurtles,
    Repair,
    Respawn,
    Run,
    SharkTower,
    Siege,
    Skulls,
    Snakes,
    SpawnTime,
    SpawnTrolls,
    SpawnTurtles,
    Spiders,
    Tower,
    Warrior,
}

impl Boost {
    pub fn description(&self) -> &'static str {
        match self {
            Boost::ArmorGain => "Decrease damage to all your units by 30%.",
            Boost::Arrows => "Your archers deal 30% more damage.",
            Boost::BearDefender => "Every priest summons a bear next to him.",
            Boost::BlockRange => "Block all damage on units from enemy ranged units.",
            Boost::BuildingsBlock => "Block all damage dealt to your buildings.",
            Boost::BuildingsDefense => "Increase the damage of all units on buildings by 100%.",
            Boost::Castle => "Upgrade your base to a castle.",
            Boost::Clone => "Clones 8 random non-attacking units of yours (in position).",
            Boost::Conversion => "Converts 5 random enemy units to your side.",
            Boost::ConvertGoblins => "Transforms all your lancers into goblins.",
            Boost::ConvertHammerheads => "Transforms all your lancers into hammerheads.",
            Boost::ConvertSharks => "Transforms all your ground archers into sharks.",
            Boost::DoubleQueue => "Two units are queued at the same time.",
            Boost::Frozen => "All enemy units who aren't attacking stop their movement.",
            Boost::GnomesBasic => "Convert all basic (ground) enemy units into gnomes.",
            Boost::GnomesMagic => "Convert all magic (ground) enemy units into gnomes.",
            Boost::InstantHealing => "Instantly heal all your units to their maximum health.",
            Boost::InstantArmy => "Immediately spawn 6 random units in the base.",
            Boost::Lancer => "Increase your lancer's damage by 60%.",
            Boost::Lightning => "Reduce all unit's health by half",
            Boost::Longbow => "Increase the range of your archers by 50%.",
            Boost::MagicPower => "Increase all your unit's magic damage by 100%.",
            Boost::MagicSwap => "All your unit's physical damage become magic damage.",
            Boost::Meditation => "Your priest's healing is 70% stronger.",
            Boost::MinotaurRage => "Spawn a minotaur for every 3 enemy magical units (min 1).",
            Boost::NoCollision => "Your units don't collide with each other.",
            Boost::Penetration => "Increase the armor penetration of all your units with 5 points.",
            Boost::QueueBears => "Allow to add bears to the queue.",
            Boost::QueueGnolls => "Allow to add gnolls to the queue.",
            Boost::QueueGoblins => "Allow to add goblins to the queue.",
            Boost::QueueHammerheads => "Allow to add hammerheads to the queue.",
            Boost::QueueMinotaurs => "Allow to add minotaurs to the queue.",
            Boost::QueueShamans => "Allow to add shamans to the queue.",
            Boost::QueueSharks => "Allow to add sharks to the queue.",
            Boost::QueueSkulls => "Allow to add skulls to the queue.",
            Boost::QueueTurtles => "Allow to add turtles to the queue.",
            Boost::Repair => "Instantly repair all your buildings to their maximum health.",
            Boost::Respawn => "Respawn all dead units on buildings.",
            Boost::Run => "Increase the speed of all your units by 100%.",
            Boost::SharkTower => "Convert all your units on buildings into sharks.",
            Boost::Siege => "Increase all damage to buildings by 50%.",
            Boost::Skulls => "Spawn 15 skulls randomly over the map.",
            Boost::Snakes => "Spawn 20 snakes randomly over the map.",
            Boost::SpawnTime => "Reduce all spawning times by 20%.",
            Boost::SpawnTrolls => "Spawn 3 trolls, each towards a path.",
            Boost::SpawnTurtles => "Spawn 3 turtles, each towards a path.",
            Boost::Spiders => "Spawn 10 spiders randomly over the map.",
            Boost::Tower => "Spawn a defense tower near the base.",
            Boost::Warrior => "Increase your warrior's damage by 50%.",
        }
    }

    pub fn condition<'a>(
        &self,
        mut buildings: impl Iterator<Item = &'a Building>,
        player: &Player,
    ) -> bool {
        match self {
            Boost::Castle => !buildings.any(|b| b.name == BuildingName::Castle),
            Boost::Tower => buildings.filter(|b| b.name == BuildingName::Tower).count() < 2,
            a if a.to_name().starts_with("Queue") => {
                player.boosts.iter().map(|b| b.name.to_name().starts_with("Queue")).count() == 0
            },
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
            Boost::Frozen => 5,
            Boost::Lancer => 40,
            Boost::Longbow => 40,
            Boost::MagicPower => 15,
            Boost::MagicSwap => 40,
            Boost::Meditation => 40,
            Boost::NoCollision => 20,
            Boost::Penetration => 30,
            Boost::Run => 15,
            Boost::Siege => 10,
            Boost::SpawnTime => 50,
            Boost::Warrior => 40,
            b if b.to_name().starts_with("Queue") => 40,
            _ => 0,
        }
    }
}

pub fn check_boost_timer(
    building_q: Query<&Building>,
    mut activate_boost_msg: MessageWriter<ActivateBoostMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<Settings>,
    mut players: ResMut<Players>,
    time: Res<Time>,
) {
    let time = scale_duration(time.delta(), settings.speed);
    settings.boost_timer.tick(time);

    if settings.boost_timer.is_finished() {
        let me_full = players.me.boosts.len() >= MAX_BOOSTS;
        let enemy_full = players.enemy.boosts.len() >= MAX_BOOSTS;

        match settings.game_mode {
            _ if me_full && enemy_full => (),
            GameMode::SinglePlayer if me_full => {
                let boost = Boost::iter()
                    .filter(|b| {
                        b.condition(
                            building_q.iter().filter(|b| b.color == players.enemy.color),
                            &players.enemy,
                        ) && !players.enemy.boosts.iter().map(|b| b.name).contains(b)
                    })
                    .choose(&mut rng())
                    .unwrap();

                players.enemy.boosts.push(SelectedBoost::new(boost).active());
                activate_boost_msg.write(ActivateBoostMsg::new(boost, players.enemy.color));
            },
            GameMode::Multiplayer if me_full => next_game_state.set(GameState::AfterBoostSelection),
            _ => next_game_state.set(GameState::BoostSelection),
        }
    }
}

pub fn update_boosts(settings: Res<Settings>, mut players: ResMut<Players>, time: Res<Time>) {
    let me = players.me.color;
    for player in players.iter_mut().filter(|p| p.color == me || !p.is_human()) {
        player.boosts.retain_mut(|boost| {
            if boost.active {
                boost.timer.tick(scale_duration(time.delta(), settings.speed));
                return !boost.timer.remaining().is_zero();
            }
            true
        });
    }
}

pub fn activate_boost_message(
    mut unit_q: Query<(Entity, &Transform, &mut Sprite, &mut Unit)>,
    mut building_q: Query<(Entity, &Transform, &mut Building)>,
    host: Option<Res<Host>>,
    players: Res<Players>,
    map: Res<Map>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    mut spawn_building_msg: MessageWriter<SpawnBuildingMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    mut activate_boost_msg: MessageReader<ActivateBoostMsg>,
    mut effect_msg: MessageWriter<EffectMsg>,
    #[cfg(not(target_arch = "wasm32"))] mut client_send_msg: MessageWriter<ClientSendMsg>,
    #[cfg(not(target_arch = "wasm32"))] mut server_send_msg: MessageWriter<ServerSendMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    for msg in activate_boost_msg.read() {
        let player = players.get_by_color(msg.color);

        if host.is_none() {
            play_audio_msg.write(PlayAudioMsg::new("horn"));
            #[cfg(not(target_arch = "wasm32"))]
            client_send_msg.write(ClientSendMsg::new(ClientMessage::ActivateBoost(msg.boost)));
        } else {
            if players.me.color == msg.color {
                // Activates own boost
                play_audio_msg.write(PlayAudioMsg::new("horn"));
            } else {
                // Activates enemy boost
                play_audio_msg.write(PlayAudioMsg::new("warning"));
                #[cfg(not(target_arch = "wasm32"))]
                server_send_msg.write(ServerSendMsg::new(ServerMessage::PlayWarning, None));
            }

            let mut rng = rng();
            match msg.boost {
                Boost::BearDefender => {
                    for (_, unit_t, _, unit) in unit_q.iter().filter(|(_, _, _, u)| {
                        u.color == player.color && u.name == UnitName::Priest
                    }) {
                        spawn_unit_msg.write(SpawnUnitMsg {
                            color: player.color,
                            unit: UnitName::Bear,
                            position: Some(unit_t.translation.truncate()),
                            on_building: None,
                            lane: Some(unit.lane),
                            dust_effect: true,
                            entity: None,
                        });
                    }
                },
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
                            dust_effect: true,
                            with_units: true,
                            entity: None,
                        });
                    }
                },
                Boost::Clone => {
                    for (_, unit_t, _, unit) in unit_q
                        .iter()
                        .filter(|(_, _, _, u)| {
                            u.color == player.color
                                && u.on_building.is_none()
                                && !matches!(u.action, Action::Attack(_))
                        })
                        .choose_multiple(&mut rng, 8)
                    {
                        spawn_unit_msg.write(SpawnUnitMsg {
                            color: player.color,
                            unit: unit.name,
                            position: Some(unit_t.translation.truncate()),
                            on_building: None,
                            lane: Some(unit.lane),
                            dust_effect: true,
                            entity: None,
                        });
                    }
                },
                Boost::Conversion => {
                    for (e, _, _, mut u) in unit_q
                        .iter_mut()
                        .filter(|(_, _, _, u)| u.color != player.color && u.on_building.is_none())
                        .choose_multiple(&mut rng, 5)
                    {
                        effect_msg.write(EffectMsg::dust(e));
                        u.color = player.color;
                        u.action = Action::Idle; // Reset action to stop attacking own units
                    }
                },
                b @ Boost::ConvertGoblins | b @ Boost::ConvertHammerheads => {
                    let unit = match b {
                        Boost::ConvertGoblins => UnitName::Goblin,
                        Boost::ConvertHammerheads => UnitName::Hammerhead,
                        _ => unreachable!(),
                    };
                    for (e, _, mut s, mut u) in unit_q.iter_mut().filter(|(_, _, _, u)| {
                        u.color == player.color && u.name == UnitName::Lancer
                    }) {
                        effect_msg.write(EffectMsg::dust(e));
                        s.custom_size = Some(Vec2::splat(unit.size()));
                        u.name = unit;
                        u.action = Action::default();
                    }
                },
                Boost::ConvertSharks => {
                    for (e, _, _, mut u) in unit_q.iter_mut().filter(|(_, _, _, u)| {
                        u.color == player.color
                            && u.name == UnitName::Archer
                            && u.on_building.is_none()
                    }) {
                        effect_msg.write(EffectMsg::dust(e));
                        u.name = UnitName::Shark;
                        u.action = Action::default();
                    }
                },
                b @ Boost::GnomesBasic | b @ Boost::GnomesMagic => {
                    for (e, _, mut s, mut u) in unit_q.iter_mut().filter(|(_, _, _, u)| {
                        u.color != player.color
                            && u.on_building.is_none()
                            && if b == Boost::GnomesBasic {
                                u.name.is_basic_unit()
                            } else {
                                !u.name.is_basic_unit()
                            }
                    }) {
                        effect_msg.write(EffectMsg::dust(e));
                        s.custom_size = Some(Vec2::splat(UnitName::Gnome.size()));
                        u.name = UnitName::Gnome;
                        u.health = u.health / u.name.health() * UnitName::Gnome.health();
                        u.action = Action::default();
                    }
                },
                Boost::InstantHealing => unit_q
                    .iter_mut()
                    .filter(|(_, _, _, u)| u.color == player.color)
                    .for_each(|(_, _, _, mut u)| u.health = u.name.health()),
                Boost::InstantArmy => {
                    for unit in UnitName::iter().choose_multiple(&mut rng, 6) {
                        spawn_unit_msg.write(SpawnUnitMsg {
                            color: player.color,
                            unit,
                            position: None,
                            on_building: None,
                            lane: None,
                            dust_effect: true,
                            entity: None,
                        });
                    }
                },
                Boost::Lightning => unit_q.iter_mut().for_each(|(_, _, _, mut u)| u.health *= 0.5),
                Boost::MinotaurRage => {
                    let enemies = unit_q
                        .iter()
                        .filter(|(_, _, _, u)| u.color != player.color && !u.name.is_basic_unit())
                        .count();
                    for _ in 0..(enemies / 3).max(1) {
                        spawn_unit_msg.write(SpawnUnitMsg::new(player.color, UnitName::Minotaur));
                    }
                },
                Boost::Repair => building_q
                    .iter_mut()
                    .filter(|(_, _, b)| b.color == player.color)
                    .for_each(|(_, _, mut b)| b.health = b.name.health()),
                Boost::Respawn => {
                    let current_positions: Vec<Vec2> = unit_q
                        .iter()
                        .filter_map(|(_, t, _, u)| {
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
                                    lane: None,
                                    dust_effect: true,
                                    entity: None,
                                });
                            }
                        }
                    }
                },
                b @ Boost::Skulls | b @ Boost::Snakes | b @ Boost::Spiders => {
                    let (amount, unit) = match b {
                        Boost::Skulls => (20, UnitName::Skull),
                        Boost::Snakes => (20, UnitName::Snake),
                        Boost::Spiders => (10, UnitName::Spider),
                        _ => unreachable!(),
                    };

                    for (lane, tile) in map
                        .lanes
                        .iter()
                        .flat_map(|(l, v)| v[3..v.len() - 3].iter().map(|t| (*l, *t)))
                        .choose_multiple(&mut rng, amount)
                    {
                        spawn_unit_msg.write(SpawnUnitMsg {
                            color: player.color,
                            unit,
                            position: Some(Map::tile_to_world(tile)),
                            on_building: None,
                            lane: Some(lane),
                            dust_effect: true,
                            entity: None,
                        });
                    }
                },
                Boost::SharkTower => {
                    unit_q
                        .iter_mut()
                        .filter(|(_, _, _, u)| u.color == player.color && u.on_building.is_some())
                        .for_each(|(e, _, _, mut u)| {
                            effect_msg.write(EffectMsg::dust(e));
                            u.name = UnitName::Shark;
                            u.action = Action::default();
                        });
                },
                b @ Boost::SpawnTrolls | b @ Boost::SpawnTurtles => {
                    let unit = match b {
                        Boost::SpawnTrolls => UnitName::Troll,
                        Boost::SpawnTurtles => UnitName::Turtle,
                        _ => unreachable!(),
                    };

                    for path in Lane::iter() {
                        spawn_unit_msg.write(SpawnUnitMsg {
                            color: player.color,
                            unit,
                            position: None,
                            on_building: None,
                            lane: Some(path),
                            dust_effect: false,
                            entity: None,
                        });
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
                            Map::tile_to_world(TilePos::new(28, 4)),
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
                        dust_effect: true,
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
    mut boost_count: ResMut<AfterBoostCount>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if **boost_count == 1 {
        **boost_count = 0;
        next_game_state.set(GameState::Playing);
    }
}
