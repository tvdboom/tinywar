use crate::core::map::map::Path;
use crate::core::settings::PlayerColor;
use crate::core::units::units::UnitName;
use crate::core::utils::ClientId;
use bevy::prelude::*;
use std::collections::VecDeque;
use std::time::Duration;
use strum::IntoEnumIterator;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, serde::Deserialize))]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, serde::Deserialize))]
pub enum PlayerDirection {
    #[default]
    Any,
    Top,
    TopMid,
    Mid,
    MidBot,
    Bot,
}

impl PlayerDirection {
    pub fn image(&self) -> &str {
        match self {
            PlayerDirection::Any => "any arrow",
            PlayerDirection::Top | PlayerDirection::Bot => "top arrow",
            PlayerDirection::TopMid | PlayerDirection::MidBot => "top-mid arrow",
            PlayerDirection::Mid => "mid arrow",
        }
    }

    pub fn flip_y(&self) -> bool {
        matches!(self, PlayerDirection::Bot | PlayerDirection::MidBot)
    }

    pub fn next(&self) -> Self {
        match self {
            PlayerDirection::Any => PlayerDirection::Top,
            PlayerDirection::Top => PlayerDirection::TopMid,
            PlayerDirection::TopMid => PlayerDirection::Mid,
            PlayerDirection::Mid => PlayerDirection::MidBot,
            PlayerDirection::MidBot => PlayerDirection::Bot,
            PlayerDirection::Bot => PlayerDirection::Any,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            PlayerDirection::Any => PlayerDirection::Bot,
            PlayerDirection::Top => PlayerDirection::Any,
            PlayerDirection::TopMid => PlayerDirection::Top,
            PlayerDirection::Mid => PlayerDirection::TopMid,
            PlayerDirection::MidBot => PlayerDirection::Mid,
            PlayerDirection::Bot => PlayerDirection::MidBot,
        }
    }

    pub fn paths(&self) -> Vec<Path> {
        match self {
            Self::Any => Path::iter().collect(),
            Self::Top => vec![Path::Top],
            Self::TopMid => vec![Path::Top, Path::Mid],
            Self::Mid => vec![Path::Mid],
            Self::MidBot => vec![Path::Mid, Path::Bot],
            Self::Bot => vec![Path::Bot],
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, serde::Deserialize))]
pub struct QueuedUnit {
    pub unit: UnitName,
    pub timer: Timer,
}

impl QueuedUnit {
    pub fn new(unit: UnitName, millis: u64) -> QueuedUnit {
        Self {
            unit,
            timer: Timer::new(Duration::from_millis(millis), TimerMode::Once),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, serde::Deserialize))]
pub struct Player {
    pub id: ClientId,
    pub color: PlayerColor,
    pub side: Side,
    pub direction: PlayerDirection,
    pub queue: VecDeque<QueuedUnit>,
    pub queue_default: UnitName,
}

impl Player {
    pub fn new(id: ClientId, color: PlayerColor, side: Side) -> Self {
        Self {
            id,
            color,
            side,
            direction: PlayerDirection::default(),
            queue: VecDeque::new(),
            queue_default: UnitName::default(),
        }
    }

    pub fn is_human(&self) -> bool {
        self.id == 0 || (self.id > 10 && self.id < ClientId::MAX)
    }
}

#[derive(Resource, Clone, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, serde::Deserialize))]
pub struct Players {
    pub me: Player,
    pub enemy: Player,
}

impl Players {
    pub fn get(&self, id: ClientId) -> &Player {
        if self.me.id == id {
            &self.me
        } else {
            &self.enemy
        }
    }

    pub fn get_mut(&mut self, id: ClientId) -> &mut Player {
        if self.me.id == id {
            &mut self.me
        } else {
            &mut self.enemy
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Player> {
        [&mut self.me, &mut self.enemy].into_iter()
    }
}
