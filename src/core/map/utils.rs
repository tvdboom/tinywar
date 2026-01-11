use bevy::prelude::*;
use bevy_tweening::Lens;

/// Tween: sprite texture cycle
#[derive(Debug, Clone, Copy)]
pub struct SpriteFrameLens(pub usize);

impl Lens<Sprite> for SpriteFrameLens {
    fn lerp(&mut self, mut target: Mut<Sprite>, ratio: f32) {
        if let Some(texture) = &mut target.texture_atlas {
            texture.index = (ratio * self.0 as f32) as usize % self.0;
        }
    }
}
