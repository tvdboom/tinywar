use crate::core::constants::MAP_Z;
use crate::core::settings::Settings;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TilemapAnchor};

#[derive(Component)]
pub struct MapCmp;

pub fn draw_map(mut commands: Commands, settings: Res<Settings>, assets: Res<AssetServer>) {
    commands.spawn((
        TiledMap(assets.load("map/map.tmx")),
        TilemapAnchor::Center,
        Transform::from_xyz(0., 0., MAP_Z),
    ));
}
