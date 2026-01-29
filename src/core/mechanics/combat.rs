use crate::core::constants::RADIUS;
use crate::core::mechanics::explosion::ExplosionMsg;
use crate::core::mechanics::spawn::{DespawnMsg, SpawnArrowMsg};
use crate::core::player::Players;
use crate::core::settings::PlayerColor;
use crate::core::units::buildings::Building;
use crate::core::units::units::{Action, Unit, UnitName};
use bevy::prelude::*;
use bevy_tweening::CycleCompletedEvent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, Deref, DerefMut)]
pub struct BuildingDestroyCmp(pub Timer);

impl Default for BuildingDestroyCmp {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1500), TimerMode::Once))
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Arrow {
    pub color: PlayerColor,
    pub damage: f32,
    pub start: Vec2,
    pub destination: Vec2,
    pub total_distance: f32,
    pub traveled: f32,
    pub despawn_timer: Timer,
}

impl Arrow {
    pub const SPEED: f32 = 160.;
    pub const ON_GROUND_SECS: u64 = 2;

    pub fn new(color: PlayerColor, damage: f32, start: Vec2, destination: Vec2) -> Self {
        Arrow {
            color,
            damage,
            start,
            destination,
            total_distance: start.distance(destination),
            traveled: 0.,
            despawn_timer: Timer::new(Duration::from_secs(Self::ON_GROUND_SECS), TimerMode::Once),
        }
    }
}

#[derive(Message)]
pub struct ApplyDamageMsg {
    pub entity: Entity,
    pub damage: f32,
}

impl ApplyDamageMsg {
    pub fn new(entity: Entity, damage: f32) -> Self {
        ApplyDamageMsg {
            entity,
            damage,
        }
    }
}

pub fn resolve_attack(
    entity_q: Query<(Entity, &Transform)>,
    unit_q: Query<(&Transform, &Unit)>,
    players: Res<Players>,
    mut cycle_completed_msg: MessageReader<CycleCompletedEvent>,
    mut spawn_arrow_msg: MessageWriter<SpawnArrowMsg>,
    mut apply_damage_msg: MessageWriter<ApplyDamageMsg>,
) {
    // Apply damage after the attacking animation finished
    for msg in cycle_completed_msg.read() {
        if let Ok((unit_t, unit)) = unit_q.get(msg.anim_entity) {
            let player = players.get_by_color(unit.color);

            match unit.action {
                Action::Attack(e) | Action::Heal(e) => {
                    if let Ok((target_e, target_t)) = entity_q.get(e) {
                        if unit.name == UnitName::Archer {
                            // Archers don't apply damage but spawn arrows at the end of the animation
                            spawn_arrow_msg.write(SpawnArrowMsg {
                                color: unit.color,
                                damage: unit.damage(player),
                                start: Vec2::new(
                                    unit_t.translation.x
                                        + 0.25
                                            * RADIUS
                                            * if target_t.translation.x < unit_t.translation.x {
                                                -1.
                                            } else {
                                                1.
                                            },
                                    unit_t.translation.y + 0.25 * RADIUS,
                                ),
                                destination: target_t.translation.truncate(),
                                entity: None,
                            });
                        } else {
                            apply_damage_msg
                                .write(ApplyDamageMsg::new(target_e, unit.damage(player)));
                        }
                    }
                },
                _ => (),
            }
        }
    }
}

pub fn apply_damage_message(
    mut commands: Commands,
    mut unit_q: Query<(Entity, &mut Unit)>,
    mut building_q: Query<(Entity, &mut Building)>,
    mut apply_damage_msg: MessageReader<ApplyDamageMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    mut explosion_msg: MessageWriter<ExplosionMsg>,
) {
    for msg in apply_damage_msg.read() {
        if let Ok((unit_e, mut unit)) = unit_q.get_mut(msg.entity) {
            unit.health = (unit.health - msg.damage).clamp(0., unit.name.health());
            if unit.health == 0. {
                despawn_msg.write(DespawnMsg(unit_e));
            }
        }

        if let Ok((building_e, mut building)) = building_q.get_mut(msg.entity) {
            // First skip buildings that already started the explosion animations
            if building.health > 0. {
                building.health = (building.health - msg.damage).clamp(0., building.name.health());
                if building.health == 0. {
                    commands.entity(building_e).insert(BuildingDestroyCmp::default());
                    explosion_msg.write(ExplosionMsg(building_e));
                }
            }
        }
    }
}
