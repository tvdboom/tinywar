use crate::core::boosts::Boost;
use crate::core::constants::UNIT_DEFAULT_SIZE;
use crate::core::map::map::Lane;
use crate::core::mechanics::combat::Projectile;
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
    Bear,
    Gnome,
    Goblin,
    Hammerhead,
    Minotaur,
    Shark,
    Skull,
    Snake,
    Spider,
    Turtle,
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
                "Priests heal damaged units over a range. A priest cannot heal himself. These \
                fragile support units are slow-moving and defenseless, but their powerful healing \
                can turn the tide of battle. Priests do not attack."
            },
            UnitName::Bear => {
                "A massive forest bully that crushes enemies with its enormous, powerful claws. \
                Bears are often summoned by priests to defend them in close combat."
            },
            UnitName::Gnome => {
                "A small and fragile magical creature that attacks with a wooden hammer."
            },
            UnitName::Goblin => {
                "A bad-tempered goblin that uses its spear to pierce through any armor."
            },
            UnitName::Hammerhead => {
                "A magical sea creature that strikes with a heavy oar, as surprising as it is \
                lethal."
            },
            UnitName::Minotaur => {
                "A giant magical brute with a giant hammer, delivering strikes with overwhelming \
                force. Minotaurs are strong both in offense and defense."
            },
            UnitName::Shark => {
                "A long-range magical predator that launches harpoons with deadly precision."
            },
            UnitName::Skull => {
                "Skulls are fragile units used primarily as fodder or to overwhelm enemies \
                with in huge numbers."
            },
            UnitName::Snake => {
                "A venomous serpent that strikes to infect its victims with deadly toxins."
            },
            UnitName::Spider => {
                "A giant arachnid that bites and poisons its victims with every strike."
            },
            UnitName::Turtle => {
                "Turtles are slow and have low damage, but are incredibly resilient. Their \
                high armor and magic resist make them the perfect units to block a path."
            },
        }
    }

    pub fn key(&self) -> KeyCode {
        match self {
            UnitName::Warrior => KeyCode::KeyZ,
            UnitName::Lancer => KeyCode::KeyX,
            UnitName::Archer => KeyCode::KeyC,
            UnitName::Priest => KeyCode::KeyV,
            _ => KeyCode::KeyB,
        }
    }

    pub fn size(&self) -> f32 {
        match self {
            UnitName::Lancer | UnitName::Minotaur | UnitName::Turtle => 320.,
            UnitName::Bear | UnitName::Goblin => 256.,
            _ => UNIT_DEFAULT_SIZE,
        }
    }

    pub fn frames(&self, action: Action) -> u32 {
        match self {
            UnitName::Warrior => match action {
                Action::Idle => 8,
                Action::Run => 6,
                Action::Guard => 6,
                Action::Attack(_) => 8,
                _ => 0,
            },
            UnitName::Lancer => match action {
                Action::Idle => 12,
                Action::Run => 6,
                Action::Attack(_) => 9,
                _ => 0,
            },
            UnitName::Archer => match action {
                Action::Idle => 6,
                Action::Run => 4,
                Action::Attack(_) => 6, // Skip last 2 frames to spawn arrow at end of animation
                _ => 0,
            },
            UnitName::Priest => match action {
                Action::Idle => 6,
                Action::Run => 4,
                Action::Heal(_) => 11,
                _ => 0,
            },
            UnitName::Bear => match action {
                Action::Idle => 8,
                Action::Run => 5,
                Action::Attack(_) => 9,
                _ => 0,
            },
            UnitName::Gnome => match action {
                Action::Idle => 8,
                Action::Run => 6,
                Action::Attack(_) => 7,
                _ => 0,
            },
            UnitName::Goblin => match action {
                Action::Idle => 7,
                Action::Run => 6,
                Action::Attack(_) => 8,
                _ => 0,
            },
            UnitName::Hammerhead => match action {
                Action::Idle => 8,
                Action::Run => 6,
                Action::Attack(_) => 6,
                _ => 0,
            },
            UnitName::Minotaur => match action {
                Action::Idle => 16,
                Action::Run => 8,
                Action::Guard => 11,
                Action::Attack(_) => 12,
                _ => 0,
            },
            UnitName::Shark => match action {
                Action::Idle => 8,
                Action::Run => 6,
                Action::Attack(_) => 4, // Skip last 4 frames to spawn arrow at end of animation
                _ => 0,
            },
            UnitName::Skull => match action {
                Action::Idle => 8,
                Action::Run => 6,
                Action::Guard => 7,
                Action::Attack(_) => 7,
                _ => 0,
            },
            UnitName::Snake => match action {
                Action::Idle => 8,
                Action::Run => 8,
                Action::Attack(_) => 6,
                _ => 0,
            },
            UnitName::Spider => match action {
                Action::Idle => 8,
                Action::Run => 5,
                Action::Attack(_) => 8,
                _ => 0,
            },
            UnitName::Turtle => match action {
                Action::Idle => 10,
                Action::Run => 7,
                Action::Guard => 6,
                Action::Attack(_) => 10,
                _ => 0,
            },
        }
    }

    pub fn is_basic_unit(self) -> bool {
        matches!(self, UnitName::Warrior | UnitName::Lancer | UnitName::Archer | UnitName::Priest)
    }

    pub fn can_attack(&self) -> bool {
        self.frames(Action::Attack(Entity::PLACEHOLDER)) > 0
    }

    pub fn can_guard(&self) -> bool {
        self.frames(Action::Guard) > 0
    }

    pub fn is_melee(&self) -> bool {
        self.range() == 1.
    }

    pub fn spawn_duration(&self) -> u64 {
        match self {
            UnitName::Warrior => 2500,
            UnitName::Lancer => 1800,
            UnitName::Archer => 3300,
            UnitName::Priest => 3400,
            UnitName::Bear => 3400,
            UnitName::Gnome => 1000,
            UnitName::Goblin => 2000,
            UnitName::Hammerhead => 1900,
            UnitName::Minotaur => 8900,
            UnitName::Shark => 3500,
            UnitName::Skull => 800,
            UnitName::Snake => 500,
            UnitName::Spider => 2500,
            UnitName::Turtle => 7500,
        }
    }

    pub fn speed(&self) -> f32 {
        match self {
            UnitName::Warrior => 30.,
            UnitName::Lancer => 35.,
            UnitName::Archer => 25.,
            UnitName::Priest => 25.,
            UnitName::Bear => 40.,
            UnitName::Gnome => 40.,
            UnitName::Goblin => 35.,
            UnitName::Hammerhead => 35.,
            UnitName::Minotaur => 25.,
            UnitName::Shark => 25.,
            UnitName::Skull => 40.,
            UnitName::Snake => 45.,
            UnitName::Spider => 30.,
            UnitName::Turtle => 15.,
        }
    }

    pub fn range(&self) -> f32 {
        match self {
            UnitName::Archer => 3.,
            UnitName::Priest => 3.,
            UnitName::Shark => 3.,
            _ => 1.,
        }
    }

    pub fn projectile(&self) -> Option<Projectile> {
        match self {
            UnitName::Archer => Some(Projectile::Arrow),
            UnitName::Shark => Some(Projectile::Harpoon),
            _ => None,
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            UnitName::Warrior => 130.,
            UnitName::Lancer => 100.,
            UnitName::Archer => 60.,
            UnitName::Priest => 40.,
            UnitName::Bear => 200.,
            UnitName::Gnome => 60.,
            UnitName::Goblin => 100.,
            UnitName::Hammerhead => 100.,
            UnitName::Minotaur => 200.,
            UnitName::Shark => 60.,
            UnitName::Skull => 60.,
            UnitName::Snake => 45.,
            UnitName::Spider => 100.,
            UnitName::Turtle => 350.,
        }
    }

    pub fn physical_damage(&self) -> f32 {
        match self {
            UnitName::Warrior => 15.,
            UnitName::Lancer => 15.,
            UnitName::Archer => 10.,
            UnitName::Priest => -30., // This is the healing done (negative damage)
            UnitName::Bear => 20.,
            UnitName::Gnome => 7.,
            UnitName::Goblin => 15.,
            UnitName::Skull => 8.,
            UnitName::Turtle => 5.,
            _ => 0.,
        }
    }

    pub fn magic_damage(&self) -> f32 {
        match self {
            UnitName::Hammerhead => 15.,
            UnitName::Minotaur => 30.,
            UnitName::Shark => 10.,
            UnitName::Skull => 2.,
            UnitName::Snake => 8.,
            UnitName::Spider => 18.,
            UnitName::Turtle => 5.,
            _ => 0.,
        }
    }

    pub fn armor(&self) -> f32 {
        match self {
            UnitName::Warrior => 5.,
            UnitName::Lancer => 3.,
            UnitName::Archer => 1.,
            UnitName::Priest => 0.,
            UnitName::Bear => 10.,
            UnitName::Gnome => 1.,
            UnitName::Goblin => 4.,
            UnitName::Hammerhead => 3.,
            UnitName::Minotaur => 12.,
            UnitName::Shark => 0.,
            UnitName::Skull => 0.,
            UnitName::Snake => 0.,
            UnitName::Spider => 5.,
            UnitName::Turtle => 20.,
        }
    }

    pub fn magic_resist(&self) -> f32 {
        match self {
            UnitName::Warrior => 3.,
            UnitName::Lancer => 3.,
            UnitName::Archer => 0.,
            UnitName::Priest => 12.,
            UnitName::Bear => 6.,
            UnitName::Gnome => 1.,
            UnitName::Goblin => 4.,
            UnitName::Hammerhead => 7.,
            UnitName::Minotaur => 12.,
            UnitName::Shark => 2.,
            UnitName::Skull => 0.,
            UnitName::Snake => 0.,
            UnitName::Spider => 2.,
            UnitName::Turtle => 20.,
        }
    }

    pub fn armor_pen(&self) -> f32 {
        match self {
            UnitName::Warrior => 5.,
            UnitName::Lancer => 8.,
            UnitName::Archer => 2.,
            UnitName::Bear => 9.,
            UnitName::Gnome => 1.,
            UnitName::Goblin => 12.,
            UnitName::Hammerhead => 8.,
            UnitName::Minotaur => 10.,
            UnitName::Shark => 5.,
            UnitName::Spider => 3.,
            _ => 0.,
        }
    }

    pub fn magic_pen(&self) -> f32 {
        match self {
            UnitName::Hammerhead => 8.,
            UnitName::Minotaur => 10.,
            UnitName::Shark => 5.,
            UnitName::Spider => 3.,
            _ => 0.,
        }
    }
}

#[derive(EnumDiscriminants, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[strum_discriminants(name(ActionKind), derive(EnumIter))]
pub enum Action {
    #[default]
    Idle,
    Run,
    Guard,
    Attack(Entity),
    Heal(Entity),
}

impl ActionKind {
    pub fn to_action(self) -> Action {
        match self {
            ActionKind::Idle => Action::Idle,
            ActionKind::Run => Action::Run,
            ActionKind::Guard => Action::Guard,
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
    pub lane: Lane,
    pub on_building: Option<Entity>,
}

impl Unit {
    pub fn new(
        name: UnitName,
        player: &Player,
        lane: Option<Lane>,
        on_building: Option<Entity>,
    ) -> Self {
        Unit {
            name,
            color: player.color,
            action: Action::default(),
            health: name.health(),
            lane: lane.unwrap_or(*player.direction.lanes().choose(&mut rng()).unwrap()),
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
