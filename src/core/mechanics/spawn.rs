use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::map::systems::MapCmp;
use crate::core::map::utils::SpriteFrameLens;
use crate::core::mechanics::combat::Arrow;
use crate::core::multiplayer::EntityMap;
use crate::core::player::Players;
use crate::core::settings::PlayerColor;
use crate::core::units::buildings::{Building, BuildingName};
use crate::core::units::units::{Action, Unit, UnitName};
use crate::utils::NameFromEnum;
use bevy::color::palettes::css::{BLACK, LIME};
use bevy::color::Color;
use bevy::ecs::children;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy_tweening::{RepeatCount, Tween, TweenAnim};
use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

#[derive(Component)]
pub struct HealthWrapperCmp;

#[derive(Component)]
pub struct HealthCmp;

#[derive(Message)]
pub struct SpawnBuildingMsg {
    pub color: PlayerColor,
    pub building: BuildingName,
    pub position: Vec2,
    pub is_base: bool,
    pub health: f32,
    pub with_units: bool,
    pub entity: Option<Entity>,
}

#[derive(Message)]
pub struct SpawnUnitMsg {
    pub color: PlayerColor,
    pub unit: UnitName,
    pub position: Option<Vec2>,
    pub on_building: Option<Entity>,
    pub entity: Option<Entity>,
}

impl SpawnUnitMsg {
    pub fn new(color: PlayerColor, unit: UnitName) -> Self {
        Self {
            color,
            unit,
            position: None,
            on_building: None,
            entity: None,
        }
    }
}

#[derive(Message)]
pub struct SpawnArrowMsg {
    pub color: PlayerColor,
    pub damage: f32,
    pub start: Vec2,
    pub destination: Vec2,
    pub entity: Option<Entity>,
}

#[derive(Message)]
pub struct DespawnMsg(pub Entity);

pub fn spawn_building_message(
    mut commands: Commands,
    #[cfg(not(target_arch = "wasm32"))] mut entity_map: ResMut<EntityMap>,
    mut spawn_building_msg: MessageReader<SpawnBuildingMsg>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    assets: Local<WorldAssets>,
) {
    for msg in spawn_building_msg.read() {
        let size = msg.building.size();

        let id = commands
            .spawn((
                Sprite {
                    image: assets.image(format!(
                        "{}-{}",
                        msg.color.to_name(),
                        msg.building.to_name()
                    )),
                    custom_size: Some(size),
                    ..default()
                },
                Transform {
                    translation: msg.position.extend(BUILDINGS_Z),
                    scale: Vec3::splat(BUILDING_SCALE),
                    ..default()
                },
                Building::new(msg.building, msg.color, msg.is_base, msg.health),
                MapCmp,
                children![(
                    Sprite {
                        color: Color::from(BLACK),
                        custom_size: Some(Vec2::new(0.5 * size.x, 15.)),
                        ..default()
                    },
                    Transform::from_xyz(0., size.y * 0.5, EXPLOSION_Z - 0.2),
                    Visibility::Hidden,
                    HealthWrapperCmp,
                    children![(
                        Sprite {
                            color: Color::from(LIME),
                            custom_size: Some(Vec2::new(0.49 * size.x, 13.)),
                            ..default()
                        },
                        Transform::from_xyz(0., 0., EXPLOSION_Z - 0.1),
                        HealthCmp,
                    )],
                )],
            ))
            .id();

        if msg.with_units {
            for pos in msg.building.units() {
                spawn_unit_msg.write(SpawnUnitMsg {
                    color: msg.color,
                    unit: UnitName::Archer,
                    position: Some(msg.position + pos),
                    on_building: Some(id),
                    entity: None,
                });
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(entity) = msg.entity {
            entity_map.insert(entity, id);
        }
    }
}

pub fn spawn_unit_message(
    mut commands: Commands,
    building_q: Query<(&Transform, &Building)>,
    players: Res<Players>,
    #[cfg(not(target_arch = "wasm32"))] mut entity_map: ResMut<EntityMap>,
    mut spawn_unit_msg: MessageReader<SpawnUnitMsg>,
    assets: Local<WorldAssets>,
) {
    for msg in spawn_unit_msg.read() {
        let action = Action::default();

        let atlas = assets.atlas(format!(
            "{}-{}-{}",
            msg.color.to_name(),
            msg.unit.to_name(),
            action.to_name()
        ));

        // Determine the spawning translation
        // If not provided, use the default position at the door of the base
        let translation = if let Some(pos) = msg.position {
            Some(pos.extend(UNITS_Z))
        } else {
            building_q
                .iter()
                .find(|(_, b)| b.color == msg.color && b.is_base)
                .map(|(t, _)| Vec3::new(t.translation.x, t.translation.y - 70., UNITS_Z))
        };

        if let Some(translation) = translation {
            let id = commands
                .spawn((
                    Sprite {
                        image: atlas.image,
                        texture_atlas: Some(atlas.atlas),
                        custom_size: Some(Vec2::splat(msg.unit.size())),
                        flip_x: players.me.color != msg.color,
                        ..default()
                    },
                    Transform {
                        translation,
                        scale: Vec3::splat(UNIT_SCALE),
                        ..default()
                    },
                    TweenAnim::new(
                        Tween::new(
                            EaseFunction::Linear,
                            Duration::from_millis(FRAME_RATE * msg.unit.frames(action) as u64),
                            SpriteFrameLens(atlas.last_index),
                        )
                        .with_repeat_count(RepeatCount::Infinite),
                    ),
                    Unit::new(msg.unit, players.get_by_color(msg.color), msg.on_building),
                    MapCmp,
                    children![(
                        Sprite {
                            color: Color::from(BLACK),
                            custom_size: Some(4. + HEALTH_SIZE),
                            ..default()
                        },
                        Transform::from_xyz(0., HEALTH_SIZE.x * 0.75, 0.1),
                        Visibility::Hidden,
                        HealthWrapperCmp,
                        children![(
                            Sprite {
                                color: Color::from(LIME),
                                custom_size: Some(HEALTH_SIZE),
                                ..default()
                            },
                            Transform::from_xyz(0., 0., 0.2),
                            HealthCmp,
                        )],
                    )],
                ))
                .id();

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(entity) = msg.entity {
                entity_map.insert(entity, id);
            }
        }
    }
}

pub fn spawn_arrow_message(
    mut commands: Commands,
    #[cfg(not(target_arch = "wasm32"))] mut entity_map: ResMut<EntityMap>,
    mut spawn_arrow_msg: MessageReader<SpawnArrowMsg>,
    assets: Local<WorldAssets>,
) {
    for msg in spawn_arrow_msg.read() {
        let id = commands
            .spawn((
                Sprite {
                    image: assets.image("arrow"),
                    ..default()
                },
                Transform {
                    translation: msg.start.extend(ARROW_Z),
                    rotation: Quat::from_rotation_z(FRAC_PI_4),
                    scale: Vec3::splat(UNIT_SCALE),
                },
                Arrow::new(msg.color, msg.damage, msg.start, msg.destination),
                MapCmp,
            ))
            .id();

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(entity) = msg.entity {
            entity_map.insert(entity, id);
        }
    }
}

pub fn despawn_message(
    mut commands: Commands,
    unit_q: Query<(Entity, &Unit)>,
    mut despawn_message: MessageReader<DespawnMsg>,
) {
    for msg in despawn_message.read() {
        // Try since there can be multiple messages to despawn the same entity
        commands.entity(msg.0).try_despawn();

        // Despawn any units on top of this building
        for (unit_e, unit) in unit_q.iter() {
            if unit.on_building == Some(msg.0) {
                commands.entity(unit_e).despawn();
            }
        }
    }
}
