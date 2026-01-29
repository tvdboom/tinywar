use crate::core::assets::WorldAssets;
use crate::core::constants::{FRAME_RATE, HEALTH_SIZE, RADIUS, UNIT_DEFAULT_SIZE};
use crate::core::map::utils::SpriteFrameLens;
use crate::core::mechanics::combat::BuildingDestroyCmp;
use crate::core::mechanics::spawn::{DespawnMsg, HealthCmp, HealthWrapperCmp};
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::core::units::buildings::Building;
use crate::core::units::units::{Action, Unit};
use crate::utils::{scale_duration, NameFromEnum};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_tweening::{RepeatCount, Tween, TweenAnim};
use itertools::Itertools;
use rand::{rng, Rng};
use std::time::Duration;

#[derive(Component)]
pub struct IsHealing;

#[derive(Component)]
pub struct HealingAnimCmp;

#[derive(Component)]
pub struct FireAnimCmp;

pub fn update_units(
    mut commands: Commands,
    mut unit_q: Query<(Entity, &Transform, &mut Sprite, Option<&IsHealing>, &mut Unit)>,
    building_q: Query<&Building>,
    healing_q: Query<&HealingAnimCmp>,
    mut wrapper_q: Query<(Entity, &mut Visibility), With<HealthWrapperCmp>>,
    mut health_q: Query<
        (&mut Transform, &mut Sprite),
        (With<HealthCmp>, Without<Unit>, Without<Building>),
    >,
    children_q: Query<&Children>,
    players: Res<Players>,
    assets: Local<WorldAssets>,
) {
    // Collect positions and health and for all units and buildings
    let units: HashMap<Entity, (Vec3, Unit)> =
        unit_q.iter().map(|(e, t, _, _, u)| (e, (t.translation, *u))).collect();

    // Get the entities of the units that are being healed
    let healed: Vec<Entity> = unit_q
        .iter()
        .filter_map(|(_, _, _, _, u)| {
            if let Action::Heal(e) = u.action {
                Some(e)
            } else {
                None
            }
        })
        .collect();

    for (unit_e, unit_t, mut unit_s, heal, mut unit) in &mut unit_q {
        let player = players.get_by_color(unit.color);

        // Check that the action receiver still exists and is in range, else go back to idle
        unit.action = match unit.action {
            Action::Attack(e) => {
                if building_q.get(e).is_ok() {
                    unit.action
                } else if let Some((pos, _)) = units.get(&e) {
                    if unit_t.translation.distance(*pos) <= unit.range(player) * RADIUS {
                        unit_s.flip_x = pos.x < unit_t.translation.x;
                        unit.action
                    } else {
                        Action::Idle
                    }
                } else {
                    Action::Idle // Target entity doesn't exist anymore
                }
            },
            Action::Heal(e) => units
                .get(&e)
                .filter(|(pos, target)| {
                    unit_t.translation.distance(*pos) <= unit.range(player) * RADIUS
                        && target.health < target.name.health()
                })
                .map(|(pos, _)| {
                    unit_s.flip_x = pos.x < unit_t.translation.x;
                    unit.action
                })
                .unwrap_or(Action::Idle),
            _ => unit.action,
        };

        // Update the action animation
        let atlas = assets.atlas(format!(
            "{}-{}-{}",
            unit.color.to_name(),
            unit.name.to_name(),
            unit.action.to_name()
        ));

        if unit_s.image != atlas.image {
            unit_s.image = atlas.image;
            unit_s.texture_atlas = Some(atlas.atlas);

            commands.entity(unit_e).insert(TweenAnim::new(
                Tween::new(
                    EaseFunction::Linear,
                    Duration::from_millis(FRAME_RATE * unit.name.frames(unit.action) as u64),
                    SpriteFrameLens(unit.name.frames(unit.action) as usize),
                )
                .with_repeat_count(RepeatCount::Infinite)
                .with_cycle_completed_event(matches!(
                    unit.action,
                    Action::Attack(_) | Action::Heal(_)
                )),
            ));
        }

        // Add/remove healing animation
        if healed.contains(&unit_e) {
            // Only add animation to those that don't have it yet
            if heal.is_none() {
                let atlas = assets.atlas("heal");
                commands.entity(unit_e).insert(IsHealing).with_children(|parent| {
                    parent.spawn((
                        Sprite {
                            image: atlas.image,
                            texture_atlas: Some(atlas.atlas),
                            custom_size: Some(Vec2::splat(UNIT_DEFAULT_SIZE)),
                            ..default()
                        },
                        TweenAnim::new(
                            Tween::new(
                                EaseFunction::Linear,
                                Duration::from_millis(FRAME_RATE * atlas.last_index as u64),
                                SpriteFrameLens(atlas.last_index),
                            )
                            .with_repeat_count(RepeatCount::Infinite),
                        ),
                        HealingAnimCmp,
                    ));
                });
            }
        } else if heal.is_some() {
            // Remove the healing animation from the entity
            for child in children_q.iter_descendants(unit_e) {
                if healing_q.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }

            // Remove the marker from the unit
            commands.entity(unit_e).remove::<IsHealing>();
        }

        // Update the health bar
        for child in children_q.iter_descendants(unit_e) {
            if let Ok((wrapper_e, mut wrapper_v)) = wrapper_q.get_mut(child) {
                // Show the health bar when the unit is damaged
                if unit.health < unit.name.health() {
                    *wrapper_v = Visibility::Inherited;

                    for child in children_q.iter_descendants(wrapper_e) {
                        if let Ok((mut health_t, mut health_s)) = health_q.get_mut(child) {
                            if let Some(size) = health_s.custom_size.as_mut() {
                                size.x = HEALTH_SIZE.x * unit.health / unit.name.health();
                                health_t.translation.x = (size.x - HEALTH_SIZE.x) * 0.5;
                            }
                        }
                    }
                } else {
                    *wrapper_v = Visibility::Hidden;
                }
            }
        }
    }
}

pub fn update_buildings(
    mut commands: Commands,
    mut building_q: Query<(Entity, &Transform, Option<&mut BuildingDestroyCmp>, &Building)>,
    mut wrapper_q: Query<(Entity, &mut Visibility), With<HealthWrapperCmp>>,
    mut health_q: Query<(&mut Transform, &mut Sprite), (With<HealthCmp>, Without<Building>)>,
    fire_q: Query<&Transform, (With<FireAnimCmp>, Without<Building>, Without<HealthCmp>)>,
    children_q: Query<&Children>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    settings: Res<Settings>,
    time: Res<Time>,
    assets: Local<WorldAssets>,
) {
    for (building_e, _, destroy, building) in &mut building_q {
        let b_size = building.name.size();

        // Update the destroy timer and despawn when finished
        if let Some(mut destroy) = destroy {
            destroy.tick(scale_duration(time.delta(), settings.speed));

            if destroy.just_finished() {
                despawn_msg.write(DespawnMsg(building_e));
                if building.is_base {
                    next_game_state.set(GameState::EndGame);
                }
                continue;
            }
        }

        // Update the health bar
        for child in children_q.iter_descendants(building_e) {
            if let Ok((wrapper_e, mut wrapper_v)) = wrapper_q.get_mut(child) {
                // Show the health bar when the unit is damaged
                if building.health < building.name.health() {
                    *wrapper_v = Visibility::Inherited;

                    for child in children_q.iter_descendants(wrapper_e) {
                        if let Ok((mut health_t, mut health_s)) = health_q.get_mut(child) {
                            if let Some(size) = health_s.custom_size.as_mut() {
                                size.x = 0.49 * b_size.x * building.health / building.name.health();
                                health_t.translation.x = (size.x - 0.49 * b_size.x) * 0.5;

                                if size.x == 0. {
                                    // Despawn the health bar when building starts exploding
                                    despawn_msg.write(DespawnMsg(wrapper_e));
                                }
                            }
                        }
                    }
                } else {
                    *wrapper_v = Visibility::Hidden;
                }
            }
        }

        // Update the fire animations
        let mut rng = rng();

        let damage = 1. - building.health / building.name.health();

        // Count existing fires
        let existing_fires = children_q
            .iter_descendants(building_e)
            .filter(|&child| fire_q.get(child).is_ok())
            .count();

        // Calculate desired number of fires based on damage
        let n_fires = match damage {
            d if d > 0.75 => 8,
            d if d > 0.5 => 5,
            d if d > 0.25 => 3,
            d if d > 0.05 => 1,
            _ => 0,
        };

        // Spawn new fires if needed
        if existing_fires < n_fires {
            commands.entity(building_e).with_children(|parent| {
                for _ in 0..n_fires - existing_fires {
                    let atlas = assets.atlas(format!("fire{}", rng.random_range(1..3)));

                    parent.spawn((
                        Sprite {
                            image: atlas.image,
                            texture_atlas: Some(atlas.atlas),
                            ..default()
                        },
                        Transform {
                            translation: Vec3::new(
                                rng.random_range(-0.4 * b_size.x..0.4 * b_size.x),
                                rng.random_range(-0.2 * b_size.y..0.2 * b_size.y),
                                0.1,
                            ),
                            scale: Vec3::splat(0.7 + damage + rng.random_range(0.0..0.2)),
                            ..default()
                        },
                        TweenAnim::new(
                            Tween::new(
                                EaseFunction::Linear,
                                Duration::from_millis(FRAME_RATE * atlas.last_index as u64),
                                SpriteFrameLens(atlas.last_index),
                            )
                            .with_repeat_count(RepeatCount::Infinite),
                        ),
                        FireAnimCmp,
                    ));
                }
            });
        } else if existing_fires > n_fires {
            // Despawn excess fires if building is healed
            for (entity, _) in children_q
                .iter_descendants(building_e)
                .filter_map(|e| fire_q.get(e).map(|t| (e, t.scale.length_squared())).ok())
                .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
                .take(existing_fires - n_fires)
            {
                commands.entity(entity).try_despawn();
            }
        }
    }
}
