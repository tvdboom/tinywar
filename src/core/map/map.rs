use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use pathfinding::prelude::astar;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Path {
    Top,
    Mid,
    Bot,
}

impl Path {
    /// Y-position of the tile that is used as waypoint (middle) of the path to follow
    pub fn waypoint(&self) -> u32 {
        match self {
            Path::Top => 1,
            Path::Mid => 6,
            Path::Bot => 10,
        }
    }
}

/// Metadata required to draw the map
#[derive(Resource, Debug)]
pub struct Map {
    paths: HashMap<Path, Vec<TilePos>>,
}

impl Default for Map {
    fn default() -> Self {
        let start = Self::STARTING_POSITIONS[0];
        let end = Self::STARTING_POSITIONS[1];

        let paths = Path::iter()
            .map(|path| {
                // Pick an intermediate waypoint in this lane, roughly in the middle
                let waypoint = TilePos::new((start.x + end.x) / 2, path.waypoint());

                // Compute two segments: start → waypoint → end
                let mut firs_segment = Self::find_path(&start, &waypoint);
                let mut second_segment = Self::find_path(&waypoint, &end);

                // Remove the waypoint (overlap) from second segment
                second_segment.remove(0);
                firs_segment.extend(second_segment);

                (path, firs_segment)
            })
            .collect();

        Self {
            paths,
        }
    }
}

impl Map {
    pub const TILE_SIZE: u32 = 64;
    pub const MAP_SIZE: UVec2 = UVec2::new(20, 12);

    // Rect that the map occupies in world coordinates
    pub const MAP_VIEW: Rect = Rect {
        min: Vec2::new(
            -(Self::MAP_SIZE.x as f32) * Self::TILE_SIZE as f32 * 0.5,
            -(Self::MAP_SIZE.y as f32) * Self::TILE_SIZE as f32 * 0.5,
        ),
        max: Vec2::new(
            Self::MAP_SIZE.x as f32 * Self::TILE_SIZE as f32 * 0.5,
            Self::MAP_SIZE.y as f32 * Self::TILE_SIZE as f32 * 0.5,
        ),
    };
    pub const STARTING_POSITIONS: [TilePos; 2] = [TilePos::new(3, 5), TilePos::new(16, 5)];

    const WALKABLE_BITS: [u32; 12] = [
        0b00000000000000000000,
        0b01111111111111111110,
        0b01111100000000111110,
        0b01111100000000111110,
        0b01111100000000111110,
        0b01111111111111111110,
        0b01111111111111111110,
        0b01111100000000111110,
        0b01111100000000111110,
        0b01111100000000111110,
        0b01111111111111111110,
        0b00000000000000000000,
    ];

    pub fn starting_positions() -> Vec<Vec2> {
        Self::STARTING_POSITIONS.iter().map(|p| Self::tile_to_world(p)).collect()
    }

    pub fn get_neighbors(pos: &TilePos) -> Vec<TilePos> {
        let moves = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)];

        moves
            .iter()
            .filter_map(|&(dx, dy)| {
                let x = pos.x as i32 + dx;
                let y = pos.y as i32 + dy;

                // Check map bounds
                if x < 0 || y < 0 || x >= Self::MAP_SIZE.x as i32 || y >= Self::MAP_SIZE.y as i32 {
                    return None;
                }

                let new_pos = TilePos::new(x as u32, y as u32);

                if !Self::is_walkable(&new_pos) {
                    return None;
                }

                // If diagonal, prevent cutting corners
                if dx != 0 && dy != 0 {
                    let pos1 = TilePos::new((pos.x as i32 + dx) as u32, pos.y); // Horizontal
                    let pos2 = TilePos::new(pos.x, (pos.y as i32 + dy) as u32); // Vertical
                    if !Self::is_walkable(&pos1) || !Self::is_walkable(&pos2) {
                        return None;
                    }
                }

                Some(new_pos)
            })
            .collect()
    }

    pub fn is_walkable(pos: &TilePos) -> bool {
        Self::WALKABLE_BITS[pos.y as usize] & (1 << (Self::MAP_SIZE.x - 1 - pos.x)) != 0
    }

    fn find_path(start: &TilePos, end: &TilePos) -> Vec<TilePos> {
        astar(
            start,
            |pos| Self::get_neighbors(pos).into_iter().map(|pos| (pos, 1)).collect::<Vec<_>>(),
            |pos| (start.x as i32 - pos.x as i32).abs() + (start.y as i32 - pos.y as i32).abs(),
            |pos| pos == end,
        )
        .map(|(path, _)| path)
        .expect(&format!("Unable to find a path from {start:?} to {end:?}."))
    }

    pub fn path(&self, path: &Path) -> Vec<TilePos> {
        self.paths.get(path).unwrap().clone()
    }

    pub fn tile_to_world(tile: &TilePos) -> Vec2 {
        Vec2::new(
            Map::MAP_VIEW.min.x + Self::TILE_SIZE as f32 * (tile.x as f32 + 0.5),
            Map::MAP_VIEW.max.y - Self::TILE_SIZE as f32 * (tile.y as f32 + 0.5),
        )
    }

    pub fn world_to_tile(pos: &Vec3) -> TilePos {
        let x = (pos.x - Self::MAP_VIEW.min.x) / Self::TILE_SIZE as f32;
        let y = (Self::MAP_VIEW.max.y - pos.y) / Self::TILE_SIZE as f32;
        TilePos::new(x as u32, y as u32)
    }
}
