use crate::core::constants::HEALTH_BAR_SIZE;
use crate::core::mechanics::spawn::{UnitHealthCmp, UnitHealthWrapperCmp};
use crate::core::units::units::Unit;
use bevy::prelude::*;

pub fn update_units(
    unit_q: Query<(Entity, &Unit)>,
    mut wrapper_q: Query<(Entity, &mut Visibility), With<UnitHealthWrapperCmp>>,
    mut health_q: Query<(&mut Transform, &mut Sprite), With<UnitHealthCmp>>,
    children_q: Query<&Children>,
) {
    for (unit_e, unit) in unit_q.iter() {
        if let Ok((wrapper_e, mut wrapper_v)) = wrapper_q.get_mut(unit_e) {
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
