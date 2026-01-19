use crate::core::constants::UNIT_DEFAULT_SIZE;
use crate::core::map::map::Path;
use crate::core::player::Player;
use crate::core::settings::PlayerColor;
use bevy::prelude::{Component, Entity, KeyCode};
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumDiscriminants, EnumIter};

#[derive(EnumIter, Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum UnitName {
    #[default]
    Warrior,
    Lancer,
    Archer,
    Priest,
}

impl UnitName {
    pub fn description(&self) -> &'static str {
        match self {
            UnitName::Warrior => {
                "\
                The warrior is a balanced front-line fighter with solid health and damage. \
                With moderate speed and close-range attacks, warriors excel at holding the \
                line and engaging enemies in direct combat."
            },
            UnitName::Lancer => {
                "\
                Lancers are swift and deadly units with. They sacrifice some durability for \
                superior speed and reduced spawning times, making them excellent for quick \
                strikes against enemy formations."
            },
            UnitName::Archer => {
                "\
                Archers have low health and damage, but shoot fast arrows at enemies at a \
                distance. Their exceptional range allows them to harass foes from safety, \
                though they're vulnerable in close combat and need protection from melee units."
            },
            UnitName::Priest => {
                "\
                Priests heal damaged units over a range. A priest cannot attack nor heal himself. \
                These fragile support units are slow-moving and defenseless, but their powerful \
                healing can turn the tide of battle by keeping your army in fighting condition."
            },
        }
    }

    pub fn key(&self) -> KeyCode {
        match self {
            UnitName::Warrior => KeyCode::KeyZ,
            UnitName::Lancer => KeyCode::KeyX,
            UnitName::Archer => KeyCode::KeyC,
            UnitName::Priest => KeyCode::KeyV,
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
                Action::Attack(_) => 8,
                _ => unreachable!(),
            },
            UnitName::Lancer => match action {
                Action::Idle => 12,
                Action::Run => 6,
                Action::Attack(_) => 9,
                _ => unreachable!(),
            },
            UnitName::Archer => match action {
                Action::Idle => 6,
                Action::Run => 4,
                Action::Attack(_) => 6, // Skip last 2 frames to spawn arrow at end of animation
                _ => unreachable!(),
            },
            UnitName::Priest => match action {
                Action::Idle => 6,
                Action::Run => 4,
                Action::Heal(_) => 11,
                _ => unreachable!(),
            },
        }
    }

    /// Spawning time in milliseconds
    pub fn spawn_duration(&self) -> u64 {
        match self {
            UnitName::Warrior => 2000,
            UnitName::Lancer => 2000,
            UnitName::Archer => 3000,
            UnitName::Priest => 4000,
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            UnitName::Warrior => 150.,
            UnitName::Lancer => 100.,
            UnitName::Archer => 60.,
            UnitName::Priest => 40.,
        }
    }

    pub fn speed(&self) -> f32 {
        match self {
            UnitName::Warrior => 20.,
            UnitName::Lancer => 25.,
            UnitName::Archer => 15.,
            UnitName::Priest => 10.,
        }
    }

    pub fn range(&self) -> f32 {
        match self {
            UnitName::Archer => 4.,
            UnitName::Priest => 4.,
            _ => 1.,
        }
    }

    pub fn damage(&self) -> f32 {
        match self {
            UnitName::Warrior => 15.,
            UnitName::Lancer => 15.,
            UnitName::Archer => 10.,
            UnitName::Priest => -30., // This is the healing done (negative damage)
        }
    }
}

#[derive(EnumDiscriminants, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[strum_discriminants(name(ActionKind), derive(EnumIter))]
pub enum Action {
    #[default]
    Idle,
    Run,
    Attack(Entity),
    Heal(Entity),
}

impl ActionKind {
    pub fn to_action(self) -> Action {
        match self {
            ActionKind::Idle => Action::Idle,
            ActionKind::Run => Action::Run,
            ActionKind::Attack => Action::Attack(Entity::PLACEHOLDER),
            ActionKind::Heal => Action::Heal(Entity::PLACEHOLDER),
        }
    }
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
