use crate::core::boosts::Boost;
use crate::core::constants::RADIUS;
use crate::core::mechanics::effects::EffectMsg;
use crate::core::mechanics::spawn::{DespawnMsg, SpawnArrowMsg};
use crate::core::player::{Player, Players, Strategy};
use crate::core::settings::PlayerColor;
use crate::core::states::GameState;
use crate::core::units::buildings::Building;
use crate::core::units::units::{Action, Unit, UnitName};
use bevy::prelude::*;
use bevy_tweening::{CycleCompletedEvent, TweenAnim};
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

#[derive(Component, Deref, DerefMut)]
pub struct BuildingDestroyCmp(pub Timer);

impl Default for BuildingDestroyCmp {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1500), TimerMode::Once))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProjectileMode {
    Parabolic,
    Straight,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Projectile {
    Arrow,
    Bone,
    Harpoon,
    Magic,
}

impl Projectile {
    pub fn angle(&self) -> f32 {
        match self {
            Projectile::Arrow => 0.,
            Projectile::Bone => 0.,
            Projectile::Harpoon => -FRAC_PI_4,
            Projectile::Magic => 0.,
        }
    }

    pub fn animation(&self) -> bool {
        matches!(self, Projectile::Bone | Projectile::Magic)
    }

    pub fn mode(&self) -> ProjectileMode {
        match self {
            Projectile::Arrow => ProjectileMode::Parabolic,
            Projectile::Bone => ProjectileMode::Straight,
            Projectile::Harpoon => ProjectileMode::Parabolic,
            Projectile::Magic => ProjectileMode::Straight,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Arrow {
    pub color: PlayerColor,
    pub projectile: Projectile,
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

    pub fn new(
        color: PlayerColor,
        projectile: Projectile,
        damage: f32,
        start: Vec2,
        destination: Vec2,
    ) -> Self {
        Arrow {
            color,
            projectile,
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
    let mut attack_damage = unit.name.physical_damage();
    let mut magic_damage = unit.name.magic_damage();

    if attacker.has_boost(Boost::MagicSwap) {
        magic_damage += attack_damage;
        attack_damage = 0.;
    }

    if attacker.has_boost(Boost::MagicPower) {
        magic_damage *= 2.;
    }

    let effective_armor = (armor
        - unit.name.armor_pen()
        - if attacker.has_boost(Boost::Penetration) {
            5.
        } else {
            0.
        })
    .max(0.);
    let effective_mr = (magic_resist - unit.name.magic_pen()).max(0.);

    let mitigate = |dmg, def| dmg * (10. / (10. + def));

    let physical_taken = mitigate(attack_damage, effective_armor);
    let magical_taken = mitigate(magic_damage, effective_mr);

    let mut damage = physical_taken + magical_taken;

    damage *= match unit.name {
        UnitName::Warrior if attacker.has_boost(Boost::Warrior) => 1.5,
        UnitName::Lancer if attacker.has_boost(Boost::Lancer) => 1.6,
        UnitName::Archer if attacker.has_boost(Boost::Arrows) => 1.3,
        _ => 1.,
    };

    damage *= if defender.has_boost(Boost::ArmorGain) && !is_building {
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
    mut commands: Commands,
    entity_q: Query<(Entity, &Transform, Option<&Unit>), Or<(With<Unit>, With<Building>)>>,
    mut unit_q: Query<(Entity, &Transform, &mut Sprite, &Unit)>,
    players: Res<Players>,
    mut cycle_completed_msg: MessageReader<CycleCompletedEvent>,
    mut spawn_arrow_msg: MessageWriter<SpawnArrowMsg>,
    mut apply_damage_msg: MessageWriter<ApplyDamageMsg>,
) {
    // Apply damage after the attacking animation finished
    for msg in cycle_completed_msg.read() {
        if let Ok((unit_e, unit_t, mut unit_s, unit)) = unit_q.get_mut(msg.anim_entity) {
            let attacker = players.get_by_color(unit.color);
            let defender = players.get_by_side(attacker.side.opposite());

            match unit.action {
                Action::Attack(e) | Action::Heal(e) => {
                    if let Ok((target_e, target_t, target)) = entity_q.get(e) {
                        let (armor, mr, is_building) = if let Some(target) = target {
                            let mut armor = target.name.armor();
                            let mut mr = target.name.magic_resist();

                            if target.action == Action::Guard {
                                armor *= 2.;
                                mr *= 2.;
                            }

                            if defender.strategy == Strategy::Berserk
                                && target.on_building.is_none()
                            {
                                armor /= 2.;
                                mr /= 2.;
                            }

                            (armor, mr, false)
                        } else {
                            (0., 0., true) // Buildings have no armor nor magic resist
                        };

                        let damage = if unit.name == UnitName::Priest {
                            unit.name.physical_damage()
                                * if attacker.has_boost(Boost::Meditation) {
                                    1.7
                                } else {
                                    1.0
                                }
                        } else {
                            calculate_damage(unit, armor, mr, is_building, attacker, defender)
                        };

                        if let Some(projectile) = unit.name.projectile() {
                            // These units don't apply damage but spawn projectiles at the end of the animation
                            spawn_arrow_msg.write(SpawnArrowMsg {
                                color: unit.color,
                                projectile,
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
                Action::Guard if unit.name == UnitName::Turtle => {
                    // Turtles stay in the shell during guard
                    if let Some(atlas) = &mut unit_s.texture_atlas {
                        atlas.index = 5;
                        commands.entity(unit_e).remove::<TweenAnim>();
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
    mut effect_msg: MessageWriter<EffectMsg>,
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
                    effect_msg.write(EffectMsg::explosion(building_e));

                    if building.is_base {
                        next_game_state.set(GameState::EndGame);
                    }
                }
            }
        }
    }
}
