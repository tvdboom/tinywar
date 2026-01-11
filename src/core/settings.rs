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

impl PlayerColor {
    pub fn color(self) -> Color {
        match self {
            Self::Black => Color::srgb_u8(214, 20, 41),
            Self::Blue => Color::srgb_u8(71, 149, 167),
            Self::Purple => Color::srgb_u8(163, 112, 150),
            Self::Red => Color::srgb_u8(222, 84, 84),
            Self::Yellow => Color::srgb_u8(220, 170, 70),
        }
    }

    /// Matches the index of the images on the UI
    pub fn index(self) -> usize {
        match self {
            Self::Black => 4,
            Self::Blue => 0,
            Self::Purple => 3,
            Self::Red => 1,
            Self::Yellow => 2,
        }
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub color: PlayerColor,
    pub enemy_color: PlayerColor,
    pub map_size: MapSize,
    pub speed: f32,
    pub audio: AudioState,
    pub autosave: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            color: PlayerColor::Blue,
            enemy_color: PlayerColor::Red,
            map_size: MapSize::Small,
            speed: 1.0,
            audio: AudioState::default(),
            autosave: true,
        }
    }
}
