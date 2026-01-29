use crate::core::settings::PlayerColor;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum BuildingName {
    #[default]
    Barracks,
    Castle,
    Tower,
}

impl BuildingName {
    pub fn size(&self) -> Vec2 {
        match self {
            BuildingName::Barracks => Vec2::new(192., 256.),
            BuildingName::Castle => Vec2::new(320., 256.),
            BuildingName::Tower => Vec2::new(128., 256.),
        }
    }

    pub fn units(&self) -> Vec<Vec2> {
        match self {
            BuildingName::Barracks => {
                vec![Vec2::new(-25., 20.), Vec2::new(25., 20.)]
            },
            BuildingName::Castle => {
                vec![Vec2::new(-70., 35.), Vec2::new(0., 20.), Vec2::new(70., 35.)]
            },
            BuildingName::Tower => {
                vec![Vec2::new(0., 35.)]
            },
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            BuildingName::Barracks => 1000.,
            BuildingName::Castle => 2000.,
            BuildingName::Tower => 500.,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Building {
    pub name: BuildingName,
    pub color: PlayerColor,
    pub is_base: bool,
    pub health: f32,
}

impl Building {
    pub fn new(name: BuildingName, color: PlayerColor, is_base: bool) -> Self {
        Self {
            name,
            color,
            is_base,
            health: name.health(),
        }
    }
}
