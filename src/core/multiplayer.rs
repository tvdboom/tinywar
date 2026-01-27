use crate::core::constants::{BUILDINGS_Z, UNITS_Z};
use crate::core::mechanics::combat::Arrow;
use crate::core::mechanics::spawn::{DespawnMsg, SpawnArrowMsg, SpawnBuildingMsg, SpawnUnitMsg};
use crate::core::network::{ClientMessage, ClientSendMsg, ServerMessage, ServerSendMsg};
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::core::units::buildings::Building;
use crate::core::units::units::{Action, Unit};
use bevy::prelude::*;
use bimap::BiMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct EntityMap(pub BiMap<Entity, Entity>);

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Population {
    pub units: HashMap<Entity, (Vec2, bool, Unit)>,
    pub buildings: HashMap<Entity, (Vec2, Building)>,
    pub arrows: HashMap<Entity, (Vec2, Quat, Option<Rect>, Arrow)>,
}

#[derive(Message)]
pub struct UpdatePopulationMsg(pub Population);

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
    unit_q: Query<(Entity, &Transform, &Sprite, &Unit)>,
    building_q: Query<(Entity, &Transform, &Building)>,
    arrow_q: Query<(Entity, &Transform, &Sprite, &Arrow)>,
    settings: Res<Settings>,
    entity_map: Res<EntityMap>,
    mut server_send_message: MessageWriter<ServerSendMsg>,
) {
    server_send_message.write(ServerSendMsg {
        message: ServerMessage::Status {
            speed: settings.speed,
            population: Population {
                units: unit_q
                    .iter()
                    .map(|(e, t, s, u)| {
                        let mut u = u.clone();

                        // Map actions to the entity on the server
                        if let Action::Attack(e) = &mut u.action {
                            *e = *entity_map.get_by_right(e).unwrap_or(e);
                        }

                        (e, (t.translation.truncate(), s.flip_x, u))
                    })
                    .collect(),
                buildings: building_q
                    .iter()
                    .map(|(e, t, b)| (e, (t.translation.truncate(), *b)))
                    .collect(),
                arrows: arrow_q
                    .iter()
                    .map(|(e, t, s, a)| (e, (t.translation.truncate(), t.rotation, s.rect, a.clone())))
                    .collect(),
            },
        },
        client: None,
    });
}

pub fn update_population_message(
    mut update_population_ev: MessageReader<UpdatePopulationMsg>,
    mut unit_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Unit)>,
    mut building_q: Query<(Entity, &mut Transform, &mut Building), (Without<Unit>, Without<Arrow>)>,
    mut arrow_q: Query<
        (Entity, &mut Transform, &mut Sprite, &mut Arrow),
        (Without<Unit>, Without<Building>),
    >,
    entity_map: Res<EntityMap>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    mut spawn_building_msg: MessageWriter<SpawnBuildingMsg>,
    mut spawn_arrow_msg: MessageWriter<SpawnArrowMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
) {
    if let Some(UpdatePopulationMsg(population)) = update_population_ev.read().last() {
        // Despawn all that are not in the new population
        for entity in unit_q
            .iter()
            .map(|(e, _, _, _)| e)
            .chain(building_q.iter().map(|(e, _, _)| e))
            .chain(arrow_q.iter().map(|(e, _, _, _)| e))
        {
            if !population
                .units
                .contains_key(entity_map.get_by_right(&entity).unwrap_or(&Entity::PLACEHOLDER))
            {
                despawn_msg.write(DespawnMsg(entity));
            }
        }

        // Update the current population
        for (unit_e, (t, s, u)) in &population.units {
            if let Some(e) = entity_map.get_by_left(unit_e) {
                if let Ok((_, mut unit_t, mut unit_s, mut unit)) = unit_q.get_mut(*e) {
                    unit_t.translation = t.extend(UNITS_Z);
                    unit_s.flip_x = *s;
                    *unit = *u;
                }
            } else {
                spawn_unit_msg.write(SpawnUnitMsg {
                    color: u.color,
                    unit: u.name,
                    position: Some(*t),
                    on_building: u.on_building,
                    entity: Some(*unit_e),
                });
            }
        }

        for (building_e, (t, b)) in &population.buildings {
            if let Some(e) = entity_map.get_by_left(building_e) {
                if let Ok((_, mut building_t, mut building)) = building_q.get_mut(*e) {
                    building_t.translation = t.extend(BUILDINGS_Z);
                    *building = *b;
                }
            } else {
                spawn_building_msg.write(SpawnBuildingMsg {
                    color: b.color,
                    building: b.name,
                    position: *t,
                    is_base: b.is_base,
                    with_units: false,
                    entity: Some(*building_e),
                });
            }
        }

        for (arrow_e, (t, r, s, a)) in &population.arrows {
            if let Some(e) = entity_map.get_by_left(arrow_e) {
                if let Ok((_, mut arrow_t, mut arrow_s, mut arrow)) = arrow_q.get_mut(*e) {
                    arrow_t.translation = t.extend(BUILDINGS_Z);
                    arrow_t.rotation = *r;
                    arrow_s.rect = *s;
                    *arrow = a.clone();
                }
            } else {
                spawn_arrow_msg.write(SpawnArrowMsg {
                    color: a.color,
                    damage: a.damage,
                    start: a.start,
                    destination: a.destination,
                    entity: Some(*arrow_e),
                });
            }
        }
    }

    update_population_ev.clear();
}
