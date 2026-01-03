use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::{TileBundle, TileTextureIndex};
use bevy_tweening::Lens;

/// Tween: sprite texture cycle
#[derive(Debug, Clone, Copy)]
pub struct TileTextureLens(pub u32);

impl Lens<TileTextureIndex> for TileTextureLens {
    fn lerp(&mut self, mut target: Mut<TileTextureIndex>, ratio: f32) {
        target.0 = (ratio * self.0 as f32) as u32 % self.0;
    }
}
