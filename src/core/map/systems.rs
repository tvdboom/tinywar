use crate::core::assets::WorldAssets;
use crate::core::map::utils::TileTextureLens;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_tweening::{Delay, RepeatCount, Tween, TweenAnim};
use rand::{rng, Rng};
use std::time::Duration;

#[derive(Component)]
pub struct MapCmp;

pub fn draw_map(mut commands: Commands, assets: Local<WorldAssets>) {
    let mut rng = rng();

    let map_size = TilemapSize::new(10, 10);

    // Layer 1
    let texture = assets.image("foam");

    let mut tile_storage = TileStorage::empty(map_size);
    let entity = commands.spawn_empty().id();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos::new(x, y);
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(entity),
                        ..default()
                    },
                    TweenAnim::new(
                        Delay::new(Duration::from_millis(rng.random_range(1..500))).then(
                            Tween::new(
                                EaseFunction::Linear,
                                Duration::from_millis(1250),
                                TileTextureLens(15),
                            )
                            .with_repeat_count(RepeatCount::Infinite),
                        ),
                    ),
                    MapCmp,
                ))
                .id();

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize::new(192., 192.);

    commands.entity(entity).insert(TilemapBundle {
        grid_size: TilemapGridSize::new(64., 64.),
        map_type: TilemapType::Square,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..default()
    });

    // Layer 2
    let texture = assets.image("tiles1");

    let mut tile_storage = TileStorage::empty(map_size);
    let entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(10),
        map_size,
        TilemapId(entity),
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize::new(64., 64.);

    let texture = assets.image("tiles1");

    commands.entity(entity).insert(TilemapBundle {
        grid_size: tile_size.into(),
        map_type: TilemapType::Square,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture),
        tile_size,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0., 0., 1.0),
        ..default()
    });
}
