use bevy_ecs_tilemap::prelude::*;

pub enum MapSize {
    Small,
    Medium,
    Large,
}

pub struct Map;

impl Map {
    pub fn new(size: &MapSize) -> Map {
        Self
    }
}
