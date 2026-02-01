use crate::core::boosts::Boost;
use crate::core::constants::RADIUS;
use crate::core::mechanics::explosion::ExplosionMsg;
use crate::core::mechanics::spawn::{DespawnMsg, SpawnArrowMsg};
use crate::core::player::{Player, Players};
use crate::core::settings::PlayerColor;
use crate::core::states::GameState;
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

fn calculate_damage(
    unit: &Unit,
    armor: f32,
    magic_resist: f32,
    is_building: bool,
    attacker: &Player,
    defender: &Player,
) -> f32 {
    let mut attack_damage = unit.name.attack_damage();
    let mut magic_damage = unit.name.magic_damage();

    if attacker.has_boost(Boost::MagicSwap) {
        magic_damage += attack_damage;
        attack_damage = 0.;
    }

    if attacker.has_boost(Boost::MagicPower) {
        magic_damage *= 2.;
    }

    let effective_armor = armor
        - unit.name.armor_pen()
        - if attacker.has_boost(Boost::Penetration) {
            5.
        } else {
            0.
        };
    let effective_mr = magic_resist - unit.name.magic_pen();

    let mitigate = |dmg, def| dmg * (100. / (100. + def));

    let physical_taken = mitigate(attack_damage, effective_armor);
    let magical_taken = mitigate(magic_damage, effective_mr);

    let mut damage = physical_taken + magical_taken;

    damage *= match unit.name {
        UnitName::Warrior if attacker.has_boost(Boost::Warrior) => 1.5,
        UnitName::Lancer if attacker.has_boost(Boost::Lancer) => 1.6,
        UnitName::Archer if attacker.has_boost(Boost::ArmorGain) => 1.3,
        UnitName::Priest if attacker.has_boost(Boost::Meditation) => 1.7,
        _ => 1.,
    };

    damage *= if defender.has_boost(Boost::ArmorGain) {
        0.7
    } else {
        1.0
    };

    if unit.on_building.is_some() && attacker.has_boost(Boost::BuildingsDefense) {
        damage *= 2.0;
    }

    damage = damage.max(5.);

    if is_building {
        if attacker.has_boost(Boost::Siege) {
            damage *= 1.5;
        }

        if defender.has_boost(Boost::BuildingsBlock) {
            damage = 0.;
        }
    } else if !unit.name.is_melee() && defender.has_boost(Boost::BlockRange) {
        damage = 0.;
    }

    damage
}

pub fn resolve_attack(
    entity_q: Query<(Entity, &Transform, Option<&Unit>), Or<(With<Unit>, With<Building>)>>,
    unit_q: Query<(&Transform, &Unit)>,
    players: Res<Players>,
    mut cycle_completed_msg: MessageReader<CycleCompletedEvent>,
    mut spawn_arrow_msg: MessageWriter<SpawnArrowMsg>,
    mut apply_damage_msg: MessageWriter<ApplyDamageMsg>,
) {
    // Apply damage after the attacking animation finished
    for msg in cycle_completed_msg.read() {
        if let Ok((unit_t, unit)) = unit_q.get(msg.anim_entity) {
            let attacker = players.get_by_color(unit.color);
            let defender = players.get_by_side(attacker.side.opposite());

            match unit.action {
                Action::Attack(e) | Action::Heal(e) => {
                    if let Ok((target_e, target_t, u)) = entity_q.get(e) {
                        let (armor, mr, is_building) = if let Some(u) = u {
                            (u.name.armor(), u.name.magic_resist(), false)
                        } else {
                            (0., 0., true) // Buildings have no armor nor magic resist
                        };

                        let damage =
                            calculate_damage(unit, armor, mr, is_building, attacker, defender);

                        if unit.name == UnitName::Archer {
                            // Archers don't apply damage but spawn arrows at the end of the animation
                            spawn_arrow_msg.write(SpawnArrowMsg {
                                color: unit.color,
                                damage,
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
                            apply_damage_msg.write(ApplyDamageMsg::new(target_e, damage));
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
    mut next_game_state: ResMut<NextState<GameState>>,
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

                    if building.is_base {
                        next_game_state.set(GameState::EndGame);
                    }
                }
            }
        }
    }
}
