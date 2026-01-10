use crate::core::assets::WorldAssets;
use crate::core::constants::{BUILDINGS_Z, FRAME_RATE, HEALTH_BAR_SIZE, UNITS_Z};
use crate::core::map::systems::MapCmp;
use crate::core::map::utils::SpriteFrameLens;
use crate::core::player::Players;
use crate::core::units::buildings::{Building, BuildingName};
use crate::core::units::units::{Action, Unit, UnitName};
use crate::utils::NameFromEnum;
use bevy::color::palettes::css::{BLACK, LIME};
use bevy::color::Color;
use bevy::ecs::children;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use bevy_tweening::{RepeatCount, Tween, TweenAnim};
use std::time::Duration;

#[derive(Component)]
pub struct UnitHealthWrapperCmp;

#[derive(Component)]
pub struct UnitHealthCmp;

#[derive(Message)]
pub struct SpawnBuildingMsg {
    pub id: ClientId,
    pub building: BuildingName,
    pub position: Vec2,
    pub is_base: bool,
}

impl SpawnBuildingMsg {
    pub fn new(id: ClientId, building: BuildingName, position: Vec2, is_base: bool) -> Self {
        Self {
            id,
            building,
            position,
            is_base,
        }
    }
}

#[derive(Message)]
pub struct SpawnUnitMsg {
    pub id: ClientId,
    pub unit: UnitName,
}

impl SpawnUnitMsg {
    pub fn new(id: ClientId, unit: UnitName) -> Self {
        Self {
            id,
            unit,
        }
    }
}

pub fn spawn_building_message(
    mut commands: Commands,
    players: Res<Players>,
    mut spawn_building_msg: MessageReader<SpawnBuildingMsg>,
    assets: Local<WorldAssets>,
) {
    for msg in spawn_building_msg.read() {
        let player = players.get(msg.id);

        commands.spawn((
            Sprite::from_image(assets.image(format!(
                "{}-{}",
                player.color.to_name(),
                msg.building.to_name()
            ))),
            Transform {
                translation: msg.position.extend(BUILDINGS_Z),
                scale: Vec3::splat(0.6),
                ..default()
            },
            Building::new(msg.building, player.color, msg.is_base),
            MapCmp,
        ));
    }
}

pub fn spawn_unit_message(
    mut commands: Commands,
    building_q: Query<(&Transform, &Building)>,
    players: Res<Players>,
    mut spawn_unit_msg: MessageReader<SpawnUnitMsg>,
    assets: Local<WorldAssets>,
) {
    for msg in spawn_unit_msg.read() {
        let player = players.get(msg.id);
        let action = Action::default();

        let texture = assets.texture(format!(
            "{}-{}-{}",
            player.color.to_name(),
            msg.unit.to_name(),
            action.to_name()
        ));

        // Spawn units at the door of the base
        if let Some((base_t, _)) =
            building_q.iter().find(|(_, b)| b.color == player.color && b.is_base)
        {
            commands.spawn((
                Sprite {
                    image: texture.image,
                    texture_atlas: Some(texture.atlas),
                    custom_size: Some(Vec2::splat(msg.unit.size())),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(
                        base_t.translation.x,
                        base_t.translation.y - 70.,
                        UNITS_Z,
                    ),
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                TweenAnim::new(
                    Tween::new(
                        EaseFunction::Linear,
                        Duration::from_millis(FRAME_RATE * msg.unit.frames(action) as u64),
                        SpriteFrameLens(texture.last_index),
                    )
                    .with_repeat_count(RepeatCount::Infinite),
                ),
                Unit::new(msg.unit, player.color),
                MapCmp,
                children![(
                    Sprite {
                        color: Color::from(BLACK),
                        custom_size: Some(HEALTH_BAR_SIZE),
                        ..default()
                    },
                    Transform::from_xyz(0., HEALTH_BAR_SIZE.x * 0.75, 0.1),
                    UnitHealthWrapperCmp,
                    children![(
                        Sprite {
                            color: Color::from(LIME),
                            custom_size: Some(HEALTH_BAR_SIZE * Vec2::new(0.9, 0.76)),
                            ..default()
                        },
                        Transform::from_xyz(0., 0., 0.2),
                        UnitHealthCmp,
                    )],
                )],
            ));
        }
    }
}
