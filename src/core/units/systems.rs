use crate::core::assets::WorldAssets;
use crate::core::constants::{FRAME_RATE, INNER_HEALTH_SIZE, RADIUS, UNIT_DEFAULT_SIZE};
use crate::core::map::utils::SpriteFrameLens;
use crate::core::mechanics::spawn::{UnitHealthCmp, UnitHealthWrapperCmp};
use crate::core::units::units::{Action, Unit};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy_tweening::{RepeatCount, Tween, TweenAnim};
use std::time::Duration;

#[derive(Component)]
pub struct IsHealing;

#[derive(Component)]
pub struct HealingAnimCmp;

pub fn update_units(
    mut commands: Commands,
    mut unit_q: Query<(Entity, &Transform, &mut Sprite, Option<&IsHealing>, &mut Unit)>,
    entity_q: Query<&Transform, Without<UnitHealthCmp>>,
    mut wrapper_q: Query<(Entity, &mut Visibility), With<UnitHealthWrapperCmp>>,
    mut health_q: Query<(&mut Transform, &mut Sprite), (With<UnitHealthCmp>, Without<Unit>)>,
    children_q: Query<&Children>,
    assets: Local<WorldAssets>,
) {
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
        // Check the action receiver still exists and is in range, else go back to idle
        unit.action = match unit.action {
            Action::Attack(e) | Action::Heal(e)
                if entity_q.get(e).is_ok_and(|t| {
                    unit_t.translation.distance(t.translation) <= unit.name.range() * RADIUS
                }) =>
            {
                unit.action
            },
            Action::Attack(_) | Action::Heal(_) => Action::Idle,
            a => a,
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
            commands
                .entity(unit_e)
                .remove::<IsHealing>()
                .remove_recursive::<Children, (Sprite, TweenAnim, HealingAnimCmp)>();
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
                                size.x = INNER_HEALTH_SIZE.x * unit.health / unit.name.health();
                                health_t.translation.x = (size.x - INNER_HEALTH_SIZE.x) * 0.5;
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
