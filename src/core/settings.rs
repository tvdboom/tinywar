use crate::core::map::map::MapSize;
use crate::core::states::AudioState;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlayerColor {
    Black,
    Blue,
    Purple,
    Red,
    Yellow,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub color: PlayerColor,
    pub map_size: MapSize,
    pub speed: f32,
    pub audio: AudioState,
    pub autosave: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            color: PlayerColor::Blue,
            map_size: MapSize::Small,
            speed: 1.0,
            audio: AudioState::default(),
            autosave: true,
        }
    }
}
