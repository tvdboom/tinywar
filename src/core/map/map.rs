use crate::core::assets::WorldAssets;
use bevy::asset::Handle;
use bevy::image::Image;
use bevy_ecs_tilemap::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum MapSize {
    Small,
    Medium,
    Large,
}

pub struct MapLayer {
    pub texture: Handle<Image>,
    pub tile_size: TilemapTileSize,
    pub grid: Vec<Vec<u32>>,
    pub animation: u32,
}

pub struct Map {
    pub size: TilemapSize,
    pub grid_size: TilemapGridSize,
    pub map_type: TilemapType,
    pub layers: Vec<MapLayer>,
}

impl Map {
    pub fn new(map_size: &MapSize, assets: &WorldAssets) -> Map {
        let size = match map_size {
            MapSize::Small => 5,
            MapSize::Medium => 30,
            MapSize::Large => 50,
        };

        // Lower grass layer
        let layer1 = MapLayer {
            texture: assets.image("tiles0"),
            tile_size: TilemapTileSize::new(64., 64.),
            grid: vec![vec![]],
            animation: 0,
        };

        // The foam layer
        let layer0 = MapLayer {
            texture: assets.image("foam"),
            tile_size: TilemapTileSize::new(192., 192.),
            grid: vec![vec![]],
            animation: 15,
        };

        Self {
            size: TilemapSize::new(size, size),
            grid_size: TilemapGridSize::new(64., 64.),
            map_type: TilemapType::Square,
            layers: vec![layer0],
        }
    }
}
