use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::{EXPLOSION_Z, FRAME_RATE, RADIUS};
use crate::core::map::utils::SpriteFrameLens;
use crate::core::mechanics::spawn::{DespawnMsg, SpawnArrowMsg};
use crate::core::settings::PlayerColor;
use crate::core::units::buildings::Building;
use crate::core::units::units::{Action, Unit, UnitName};
use bevy::prelude::*;
use bevy_tweening::{CycleCompletedEvent, Delay, Tween, TweenAnim};
use rand::{rng, Rng};
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
    mut cycle_completed_msg: MessageReader<CycleCompletedEvent>,
    mut spawn_arrow_msg: MessageWriter<SpawnArrowMsg>,
    mut apply_damage_msg: MessageWriter<ApplyDamageMsg>,
) {
    // Apply damage after the attacking animation finished
    for msg in cycle_completed_msg.read() {
        if let Ok((unit_t, unit)) = unit_q.get(msg.anim_entity) {
            match unit.action {
                Action::Attack(e) | Action::Heal(e) => {
                    if let Ok((target_e, target_t)) = entity_q.get(e) {
                        if unit.name == UnitName::Archer {
                            // Archers don't apply damage but spawn arrows at the end of the animation
                            spawn_arrow_msg.write(SpawnArrowMsg {
                                color: unit.color,
                                damage: unit.name.damage(),
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
                                .write(ApplyDamageMsg::new(target_e, unit.name.damage()));
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
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    assets: Local<WorldAssets>,
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
                    let mut rng = rng();

                    play_audio_msg.write(PlayAudioMsg::new("explosion"));

                    // Insert destroy animation
                    commands
                        .entity(building_e)
                        .insert(BuildingDestroyCmp::default())
                        .with_children(|parent| {
                            let size = building.name.size();
                            for _ in 0..7 {
                                let atlas =
                                    assets.atlas(format!("explosion{}", rng.random_range(1..3)));

                                parent.spawn((
                                    Sprite {
                                        image: atlas.image,
                                        texture_atlas: Some(atlas.atlas),
                                        ..default()
                                    },
                                    Transform {
                                        translation: Vec3::new(
                                            rng.random_range(-0.4 * size.x..0.4 * size.x),
                                            rng.random_range(-0.3 * size.y..0.3 * size.y),
                                            EXPLOSION_Z,
                                        ),
                                        scale: Vec3::splat(rng.random_range(1.0..1.5)),
                                        ..default()
                                    },
                                    TweenAnim::new(
                                        Delay::new(Duration::from_millis(
                                            rng.random_range(1..1000),
                                        ))
                                        .then(Tween::new(
                                            EaseFunction::Linear,
                                            Duration::from_millis(
                                                FRAME_RATE * atlas.last_index as u64,
                                            ),
                                            SpriteFrameLens(atlas.last_index),
                                        )),
                                    ),
                                ));
                            }
                        });
                }
            }
        }
    }
}
