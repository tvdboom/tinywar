use crate::core::boosts::Boost;
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
                "The warrior is a balanced front-line fighter with solid health and damage. \
                With moderate speed and close-range attacks, warriors excel at holding the \
                line and engaging enemies in direct combat."
            },
            UnitName::Lancer => {
                "Lancers are swift and deadly units with. They sacrifice some durability for \
                superior speed and reduced spawning times, making them excellent for quick \
                strikes against enemy formations."
            },
            UnitName::Archer => {
                "Archers have low health and damage, but shoot fast arrows at enemies at a \
                distance. Their exceptional range allows them to harass foes from safety, \
                though they're vulnerable in close combat."
            },
            UnitName::Priest => {
                "Priests heal damaged units over a range. A priest cannot attack nor heal himself. \
                These fragile support units are slow-moving and defenseless, but their powerful \
                healing can turn the tide of battle."
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

    pub fn can_attack(&self) -> bool {
        !matches!(self, UnitName::Priest)
    }

    pub fn is_melee(&self) -> bool {
        !matches!(self, UnitName::Archer)
    }

    pub fn spawn_duration(&self) -> u64 {
        match self {
            UnitName::Warrior => 2500,
            UnitName::Lancer => 1800,
            UnitName::Archer => 3300,
            UnitName::Priest => 3400,
        }
    }

    pub fn speed(&self) -> f32 {
        match self {
            UnitName::Warrior => 30.,
            UnitName::Lancer => 35.,
            UnitName::Archer => 25.,
            UnitName::Priest => 25.,
        }
    }

    pub fn range(&self) -> f32 {
        match self {
            UnitName::Archer => 3.,
            UnitName::Priest => 3.,
            _ => 1.,
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            UnitName::Warrior => 130.,
            UnitName::Lancer => 100.,
            UnitName::Archer => 60.,
            UnitName::Priest => 40.,
        }
    }

    pub fn attack_damage(&self) -> f32 {
        match self {
            UnitName::Warrior => 15.,
            UnitName::Lancer => 15.,
            UnitName::Archer => 10.,
            UnitName::Priest => -30., // This is the healing done (negative damage)
        }
    }

    pub fn magic_damage(&self) -> f32 {
        match self {
            _ => 0.,
        }
    }

    pub fn armor(&self) -> f32 {
        match self {
            UnitName::Warrior => 7.,
            UnitName::Lancer => 4.,
            UnitName::Archer => 1.,
            UnitName::Priest => 0.,
        }
    }

    pub fn magic_resist(&self) -> f32 {
        match self {
            UnitName::Warrior => 3.,
            UnitName::Lancer => 3.,
            UnitName::Archer => 0.,
            UnitName::Priest => 12.,
        }
    }

    pub fn armor_pen(&self) -> f32 {
        match self {
            UnitName::Warrior => 5.,
            UnitName::Lancer => 8.,
            UnitName::Archer => 2.,
            UnitName::Priest => 0.,
        }
    }

    pub fn magic_pen(&self) -> f32 {
        match self {
            _ => 0.,
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
    pub on_building: Option<Entity>,
}

impl Unit {
    pub fn new(
        name: UnitName,
        player: &Player,
        path: Option<Path>,
        on_building: Option<Entity>,
    ) -> Self {
        Unit {
            name,
            color: player.color,
            action: Action::default(),
            health: name.health(),
            path: path.unwrap_or(*player.direction.paths().choose(&mut rng()).unwrap()),
            on_building,
        }
    }

    pub fn range(&self, player: &Player) -> f32 {
        let mut range = if self.on_building.is_some() {
            2. * self.name.range()
        } else {
            self.name.range()
        };

        if self.name == UnitName::Archer && player.has_boost(Boost::Longbow) {
            range *= 1.5;
        }

        range
    }
}
