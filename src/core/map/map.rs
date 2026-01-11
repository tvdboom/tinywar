use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MapSize {
    Small,
    Medium,
    Large,
}

impl MapSize {
    /// Building starting positions given by the tile position
    pub fn starting_tiles(&self) -> [TilePos; 2] {
        match self {
            Self::Small => [TilePos::new(3, 5), TilePos::new(16, 5)],
            _ => todo!(),
        }
    }
}

/// Metadata required to draw the map
#[derive(Resource, Debug)]
pub struct Map {
    size: MapSize,
}

impl Map {
    pub const TILE_SIZE: u32 = 64;

    pub fn new(size: &MapSize) -> Map {
        Self {
            size: *size,
        }
    }

    pub fn size(&self) -> UVec2 {
        match self.size {
            MapSize::Small => UVec2::new(20, 10),
            _ => todo!(),
        }
    }

    pub fn tile_to_world(&self, tile: &TilePos) -> Vec2 {
        let size = self.size();
        let half_w = size.x as f32 * 0.5;
        let half_h = size.y as f32 * 0.5;

        Vec2::new(
            (tile.x as f32 + 0.5 - half_w) * Self::TILE_SIZE as f32,
            (tile.y as f32 + 0.5 - half_h) * Self::TILE_SIZE as f32,
        )
    }

    pub fn starting_positions(&self) -> Vec<Vec2> {
        self.size.starting_tiles().iter().map(|p| self.tile_to_world(p)).collect()
    }
}
