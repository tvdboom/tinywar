use crate::core::assets::WorldAssets;
use crate::core::constants::{ARROW_ON_GROUND_SECS, RADIUS, UNITS_Z};
use crate::core::map::systems::MapCmp;
use crate::core::mechanics::spawn::DespawnMsg;
use crate::core::settings::PlayerColor;
use crate::core::units::units::{Action, Unit, UnitName};
use bevy::prelude::*;
use bevy_tweening::CycleCompletedEvent;
use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

#[derive(Component)]
pub struct Arrow {
    pub color: PlayerColor,
    pub damage: f32,
    pub start: Vec3,
    pub destination: Vec3,
    pub total_distance: f32,
    pub traveled: f32,
    pub despawn_timer: Timer,
}

impl Arrow {
    pub fn new(color: PlayerColor, damage: f32, start: Vec3, destination: Vec3) -> Self {
        Arrow {
            color,
            damage,
            start,
            destination,
            total_distance: start.distance(destination),
            traveled: 0.,
            despawn_timer: Timer::new(Duration::from_secs(ARROW_ON_GROUND_SECS), TimerMode::Once),
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
    mut commands: Commands,
    mut unit_q: Query<(Entity, &Transform, &mut Unit)>,
    mut cycle_completed_msg: MessageReader<CycleCompletedEvent>,
    mut apply_damage_msg: MessageWriter<ApplyDamageMsg>,
    assets: Local<WorldAssets>,
) {
    // Apply damage after the attacking animation finished
    for msg in cycle_completed_msg.read() {
        let (unit, unit_t) = if let Ok((_, unit_t, unit)) = unit_q.get(msg.anim_entity) {
            (*unit, unit_t.clone())
        } else {
            continue;
        };

        match unit.action {
            Action::Attack(e) | Action::Heal(e) => {
                if let Ok((target_e, target_t, _)) = unit_q.get_mut(e) {
                    if unit.name == UnitName::Archer {
                        // Archers don't apply damage but spawn arrows at the end of the animation
                        let start_pos = Vec3::new(
                            unit_t.translation.x
                                + 0.25
                                    * RADIUS
                                    * if target_t.translation.x < unit_t.translation.x {
                                        -1.
                                    } else {
                                        1.
                                    },
                            unit_t.translation.y + 0.25 * RADIUS,
                            UNITS_Z + 0.1,
                        );

                        commands.spawn((
                            Sprite {
                                image: assets.image("arrow"),
                                ..default()
                            },
                            Transform {
                                translation: start_pos,
                                rotation: Quat::from_rotation_z(FRAC_PI_4),
                                scale: unit_t.scale,
                            },
                            Arrow::new(
                                unit.color,
                                unit.name.damage(),
                                start_pos,
                                target_t.translation,
                            ),
                            MapCmp,
                        ));
                    } else {
                        apply_damage_msg.write(ApplyDamageMsg::new(target_e, unit.name.damage()));
                    }
                }
            },
            _ => (),
        }
    }
}

pub fn apply_damage_message(
    mut unit_q: Query<(Entity, &mut Unit)>,
    mut apply_damage_msg: MessageReader<ApplyDamageMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
) {
    for msg in apply_damage_msg.read() {
        if let Ok((unit_e, mut unit)) = unit_q.get_mut(msg.entity) {
            unit.health = (unit.health - msg.damage).clamp(0., unit.name.health());
            if unit.health == 0. {
                despawn_msg.write(DespawnMsg(unit_e));
            }
        }
    }
}
