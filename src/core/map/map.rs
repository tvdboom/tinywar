use crate::core::assets::WorldAssets;
use bevy::asset::Handle;
use bevy::image::Image;
use bevy_ecs_tilemap::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MapSize {
    Small,
    Medium,
    Large,
}

fn parse_map(map_str: &str) -> Vec<Vec<u32>> {
    let mut rows: Vec<Vec<u32>> = map_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split_whitespace().map(|num| num.parse::<u32>().unwrap()).collect())
        .collect();

    // Reverse order since bevy_ecs_tilemap start bottom-left
    // and the str representation starts top-left
    rows.reverse();

    rows
}

/// A tile layer on the map
#[derive(Debug)]
pub struct Layer {
    pub texture: Handle<Image>,
    pub tile_size: TilemapTileSize,
    pub grid: Vec<Vec<u32>>,
    pub animation: Option<u32>,
}

/// Metadata required to draw the map
#[derive(Debug)]
pub struct Map {
    pub size: TilemapSize,
    pub grid_size: TilemapGridSize,
    pub map_type: TilemapType,
    pub layers: Vec<Layer>,
}

impl Map {
    pub fn new(map_size: &MapSize, assets: &WorldAssets) -> Map {
        // Lower grass layer
        let layer1 = Layer {
            texture: assets.image("tiles0"),
            tile_size: TilemapTileSize::new(64., 64.),
            grid: parse_map(
                "
                  0  1  1  1  1  1 28 28 28 28 28  1  1  1  1  1  2
                  9 10 10 10 10 11  4  4  4  4  4  9 10 10 10 10 11
                  9 10 10 10 10 11  4  4  4  4  4  9 10 10 10 10 11
                  9 10 10 10 10 11  4  4  4  4  4  9 10 10 10 10 11
                  9 10 10 10 10 10  1  1  1  1  1 10 10 10 10 10 11
                  9 10 10 10 10 10 19 19 19 19 19 10 10 10 10 10 11
                  9 10 10 10 10 11  4  4  4  4  4  9 10 10 10 10 11
                  9 10 10 10 10 11  4  4  4  4  4  9 10 10 10 10 11
                  9 10 10 10 10 11  4  4  4  4  4  9 10 10 10 10 11
                 18 19 19 19 19 19 28 28 28 28 28 19 19 19 19 19 20",
            ),
            animation: None,
        };

        // The foam layer
        let layer0 = Layer {
            texture: assets.image("foam"),
            tile_size: TilemapTileSize::new(192., 192.),
            grid: layer1
                .grid
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|&v| {
                            if v == 4 {
                                u32::MAX // u32::MAX used as marker to skip the tile
                            } else {
                                0
                            }
                        })
                        .collect()
                })
                .collect(),
            animation: Some(15),
        };

        Self {
            size: TilemapSize::new(layer1.grid[0].len() as u32, layer1.grid.len() as u32),
            grid_size: TilemapGridSize::new(64., 64.),
            map_type: TilemapType::Square,
            layers: vec![layer0, layer1],
        }
    }
}
