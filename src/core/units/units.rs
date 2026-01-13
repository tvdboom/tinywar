use crate::core::map::map::Path;
use crate::core::player::Player;
use crate::core::settings::PlayerColor;
use bevy::prelude::{Component, KeyCode};
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use crate::core::constants::UNIT_DEFAULT_SIZE;

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
            _ => UNIT_DEFAULT_SIZE,
        }
    }

    pub fn frames(&self, action: Action) -> u32 {
        match self {
            UnitName::Warrior => match action {
                Action::Idle => 8,
                Action::Run => 6,
            },
            UnitName::Lancer => match action {
                Action::Idle => 12,
                Action::Run => 6,
            },
            UnitName::Archer => match action {
                Action::Idle => 6,
                Action::Run => 4,
            },
            UnitName::Monk => match action {
                Action::Idle => 6,
                Action::Run => 4,
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

    pub fn speed(&self) -> f32 {
        match self {
            UnitName::Warrior => 20.,
            UnitName::Lancer => 25.,
            UnitName::Archer => 15.,
            UnitName::Monk => 10.,
        }
    }
}

#[derive(EnumIter, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Action {
    #[default]
    Idle,
    Run,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub name: UnitName,
    pub color: PlayerColor,
    pub action: Action,
    pub health: f32,
    pub path: Path,
}

impl Unit {
    pub fn new(name: UnitName, player: &Player) -> Self {
        Unit {
            name,
            color: player.color,
            action: Action::default(),
            health: name.health(),
            path: *player.direction.paths().choose(&mut rng()).unwrap(),
        }
    }
}
