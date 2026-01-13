use crate::core::assets::WorldAssets;
use crate::core::constants::{FRAME_RATE, HEALTH_BAR_SIZE};
use crate::core::map::utils::SpriteFrameLens;
use crate::core::mechanics::spawn::{UnitHealthCmp, UnitHealthWrapperCmp};
use crate::core::units::units::Unit;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy_tweening::{RepeatCount, Tween, TweenAnim};
use std::time::Duration;

pub fn update_units(
    mut commands: Commands,
    mut unit_q: Query<(Entity, &mut Sprite, &Unit)>,
    mut wrapper_q: Query<(Entity, &mut Visibility), With<UnitHealthWrapperCmp>>,
    mut health_q: Query<(&mut Transform, &mut Sprite), (With<UnitHealthCmp>, Without<Unit>)>,
    children_q: Query<&Children>,
    assets: Local<WorldAssets>,
) {
    for (unit_e, mut unit_s, unit) in &mut unit_q {
        // Update the animation
        let atlas = assets.atlas(format!(
            "{}-{}-{}",
            unit.color.to_name(),
            unit.name.to_name(),
            unit.action.to_name()
        ));

        if unit_s.image != atlas.image {
            unit_s.image = atlas.image;
            commands.entity(unit_e).insert(TweenAnim::new(
                Tween::new(
                    EaseFunction::Linear,
                    Duration::from_millis(FRAME_RATE * unit.name.frames(unit.action) as u64),
                    SpriteFrameLens(atlas.last_index),
                )
                .with_repeat_count(RepeatCount::Infinite),
            ));
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
                                size.x = HEALTH_BAR_SIZE.x * unit.health / unit.name.health();
                                health_t.translation.x = (size.x - HEALTH_BAR_SIZE.x) * 0.5;
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
