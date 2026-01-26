use crate::core::constants::{BUILDINGS_Z, UNITS_Z};
use crate::core::mechanics::spawn::{DespawnMsg, SpawnBuildingMsg, SpawnUnitMsg};
use crate::core::network::{ClientMessage, ClientSendMsg, ServerMessage, ServerSendMsg};
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::core::units::buildings::Building;
use crate::core::units::units::{Action, Unit};
use crate::core::utils::ClientId;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use bimap::BiMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct EntityMap(pub BiMap<Entity, Entity>);

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Population {
    pub units: HashMap<Entity, (Vec2, bool, Unit)>,
    pub buildings: HashMap<Entity, (Vec2, Building)>,
}

#[derive(Message)]
pub struct UpdatePopulationMsg {
    pub id: ClientId,
    pub population: Population,
}

pub fn update_game_state(
    mut server_send_message: MessageWriter<ServerSendMsg>,
    mut client_send_message: MessageWriter<ClientSendMsg>,
    game_state: Res<State<GameState>>,
) {
    server_send_message.write(ServerSendMsg {
        message: ServerMessage::State(*game_state.get()),
        client: None,
    });

    client_send_message.write(ClientSendMsg {
        message: ClientMessage::State(*game_state.get()),
    });
}

pub fn server_send_status(
    server: Res<RenetServer>,
    unit_q: Query<(Entity, &Transform, &Sprite, &Unit)>,
    building_q: Query<(Entity, &Transform, &Building)>,
    settings: Res<Settings>,
    players: Res<Players>,
    entity_map: Res<EntityMap>,
    mut server_send_message: MessageWriter<ServerSendMsg>,
) {
    for id in server.clients_id().iter() {
        server_send_message.write(ServerSendMsg {
            message: ServerMessage::Status {
                speed: settings.speed,
                population: Population {
                    units: unit_q
                        .iter()
                        .filter(|(_, _, _, u)| u.color != players.me.color)
                        .map(|(e, t, s, u)| {
                            let mut u = u.clone();

                            // Map actions to the entity on the server
                            if let Action::Attack(e) = &mut u.action {
                                *e = *entity_map.0.get_by_right(e).unwrap_or(e);
                            }

                            (e, (t.translation.truncate(), s.flip_x, u))
                        })
                        .collect(),
                    buildings: building_q
                        .iter()
                        .filter_map(|(e, t, b)| {
                            (b.color != players.me.color)
                                .then_some((e, (t.translation.truncate(), *b)))
                        })
                        .collect(),
                },
            },
            client: Some(*id),
        });
    }
}

pub fn update_population_message(
    mut update_population_ev: MessageReader<UpdatePopulationMsg>,
    mut unit_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Unit)>,
    mut building_q: Query<(Entity, &mut Transform, &mut Building), Without<Unit>>,
    players: Res<Players>,
    entity_map: Res<EntityMap>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    mut spawn_building_msg: MessageWriter<SpawnBuildingMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
) {
    let mut seen = HashSet::new();
    for msg in update_population_ev.read() {
        if !seen.insert(msg.id) {
            continue;
        }

        // Despawn all that are not in the new population
        for (unit_e, _, _, _) in &unit_q {
            if !msg.population.units.contains_key(entity_map.0.get_by_right(&unit_e).unwrap()) {
                despawn_msg.write(DespawnMsg(unit_e));
            }
        }

        for (building_e, _, _) in &building_q {
            if !msg
                .population
                .buildings
                .contains_key(entity_map.0.get_by_right(&building_e).unwrap())
            {
                despawn_msg.write(DespawnMsg(building_e));
            }
        }

        // Update the current population
        for (unit_e, (t, s, u)) in msg.population.units.iter() {
            if let Some(e) = entity_map.0.get_by_left(unit_e) {
                if let Ok((_, mut unit_t, mut unit_s, mut unit)) = unit_q.get_mut(*e) {
                    unit_t.translation = t.extend(UNITS_Z);
                    unit_s.flip_x = *s;
                    *unit = *u;
                }
            } else {
                spawn_unit_msg.write(SpawnUnitMsg {
                    id: players.me.id,
                    unit: u.name,
                    position: Some(*t),
                    on_building: u.on_building,
                });
            }
        }

        for (building_e, (t, b)) in msg.population.buildings.iter() {
            if let Some(e) = entity_map.0.get_by_left(building_e) {
                if let Ok((_, mut building_t, mut building)) = building_q.get_mut(*e) {
                    building_t.translation = t.extend(BUILDINGS_Z);
                    *building = *b;
                }
            } else {
                spawn_building_msg.write(SpawnBuildingMsg {
                    id: players.me.id,
                    building: b.name,
                    position: *t,
                    is_base: b.is_base,
                    with_units: false,
                });
            }
        }
    }
}
