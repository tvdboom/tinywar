use crate::core::camera::MainCamera;
use crate::core::constants::{MAP_Z, MAX_ZOOM};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TilemapAnchor};

#[derive(Component)]
pub struct MapCmp;

pub fn draw_map(
    mut commands: Commands,
    mut camera_q: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    assets: Res<AssetServer>,
) {
    let (mut camera_t, mut projection) = camera_q.single_mut().unwrap();
    camera_t.translation = Vec3::new(0., 0., 1.);

    if let Projection::Orthographic(projection) = &mut *projection {
        projection.scale = MAX_ZOOM;
    }

    commands.spawn((
        TiledMap(assets.load("map/map.tmx")),
        TilemapAnchor::Center,
        Transform::from_xyz(0., 0., MAP_Z),
        MapCmp,
    ));
}
