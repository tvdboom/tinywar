use crate::core::settings::PlayerColor;
use bevy::prelude::{Component, KeyCode};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum UnitName {
    #[default]
    Warrior,
    Lancer,
    Archer,
    Monk,
}

impl UnitName {
    pub fn key(&self) -> KeyCode {
        match self {
            UnitName::Warrior => KeyCode::KeyZ,
            UnitName::Lancer => KeyCode::KeyX,
            UnitName::Archer => KeyCode::KeyC,
            UnitName::Monk => KeyCode::KeyV,
        }
    }

    /// Spawning time in milliseconds
    pub fn spawn_duration(&self) -> u64 {
        match self {
            UnitName::Warrior => 2000,
            UnitName::Lancer => 2000,
            UnitName::Archer => 3000,
            UnitName::Monk => 4000,
        }
    }

    pub fn size(&self) -> f32 {
        match self {
            UnitName::Lancer => 320.,
            _ => 192.,
        }
    }

    pub fn frames(&self, action: Action) -> u32 {
        match self {
            UnitName::Warrior => match action {
                Action::Idle => 8,
            },
            UnitName::Lancer => match action {
                Action::Idle => 12,
            },
            UnitName::Archer => match action {
                Action::Idle => 6,
            },
            UnitName::Monk => match action {
                Action::Idle => 6,
            },
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            UnitName::Warrior => 150.,
            UnitName::Lancer => 100.,
            UnitName::Archer => 60.,
            UnitName::Monk => 40.,
        }
    }
}

#[derive(EnumIter, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Action {
    #[default]
    Idle,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub name: UnitName,
    pub color: PlayerColor,
    pub action: Action,
    pub health: f32,
}

impl Unit {
    pub fn new(name: UnitName, color: PlayerColor) -> Self {
        Unit {
            name,
            color,
            action: Action::default(),
            health: name.health(),
        }
    }
}
