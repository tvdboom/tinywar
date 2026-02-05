use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::boosts::{ActivateBoostMsg, Boost};
use crate::core::constants::{MAX_BOOSTS, MAX_QUEUE_LENGTH};
use crate::core::map::systems::MapCmp;
use crate::core::mechanics::queue::QueueUnitMsg;
use crate::core::menu::utils::add_text;
use crate::core::player::{Players, Side, Strategy};
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::GameState;
use crate::core::units::units::{Action, Unit, UnitName};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Component)]
pub struct UiCmp;

#[derive(Component, Deref)]
pub struct AdvanceBannerCmp(pub Side);

#[derive(Component)]
pub struct StrategyAdvanceBannerCmp;

#[derive(Component)]
pub struct TextAdvanceBannerCmp;

#[derive(Component)]
pub struct DirectionCmp;

#[derive(Component)]
pub struct BoostBoxCmp {
    pub n: usize,
    pub color: PlayerColor,
}

impl BoostBoxCmp {
    pub fn new(n: usize, color: PlayerColor) -> Self {
        BoostBoxCmp {
            n,
            color,
        }
    }
}

#[derive(Component)]
pub struct BoostBoxImageCmp;

#[derive(Component)]
pub struct BoostBoxTimerCmp;

#[derive(Component)]
pub struct HoverBoxCmp;

#[derive(Component)]
pub struct HoverBoxBoostCmp;

#[derive(Component)]
pub struct HoverBoxBoostLabelCmp;

#[derive(Component)]
pub struct ShopButtonCmp {
    pub unit: UnitName,
    pub is_basic: bool,
}

impl ShopButtonCmp {
    pub fn new(unit: UnitName, is_basic: bool) -> Self {
        Self {
            unit,
            is_basic,
        }
    }
}

#[derive(Component, Deref)]
pub struct StrategyButtonCmp(pub Strategy);

#[derive(Component)]
pub struct StrategyHoverCmp;

#[derive(Component)]
pub struct StrategyProgressWrapperCmp;

#[derive(Component)]
pub struct StrategyProgressCmp;

#[derive(Component, Deref)]
pub struct ShopLabelCmp(pub UnitName);

#[derive(Component, Deref)]
pub struct SwordQueueCmp(pub usize);

#[derive(Component, Deref)]
pub struct QueueButtonCmp(pub usize);

#[derive(Component)]
pub struct QueueProgressWrapperCmp;

#[derive(Component)]
pub struct QueueProgressCmp;

#[derive(Component)]
pub struct SpeedCmp;

pub fn draw_ui(
    mut commands: Commands,
    players: Res<Players>,
    settings: Res<Settings>,
    window: Single<&Window>,
    assets: Res<WorldAssets>,
) {
    // Draw advance
    let texture = assets.texture("large ribbons");
    commands
        .spawn((
            Node {
                top: Val::Percent(3.),
                left: Val::Percent(15.),
                width: Val::Percent(70.),
                height: Val::Percent(10.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            Pickable::IGNORE,
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            let mut spawn = |index: usize, width: Val, component: Option<Side>| {
                let mut p = parent.spawn((
                    Node {
                        width,
                        height: Val::Percent(100.),
                        align_items: AlignItems::Center,
                        flex_direction: component
                            .map(|side| {
                                if side == Side::Right {
                                    FlexDirection::RowReverse
                                } else {
                                    FlexDirection::Row
                                }
                            })
                            .unwrap_or_default(),
                        justify_content: component
                            .map(|side| {
                                if side == Side::Left {
                                    JustifyContent::Start
                                } else {
                                    JustifyContent::End
                                }
                            })
                            .unwrap_or_default(),
                        ..default()
                    },
                    ImageNode::from_atlas_image(
                        texture.image.clone(),
                        TextureAtlas {
                            layout: texture.layout.clone(),
                            index,
                        },
                    ),
                ));

                if let Some(side) = component {
                    p.insert((
                        AdvanceBannerCmp(side),
                        ZIndex(1),
                        children![
                            (
                                Node {
                                    height: Val::Percent(50.),
                                    aspect_ratio: Some(1.0),
                                    margin: UiRect::horizontal(Val::Percent(2.)),
                                    ..default()
                                },
                                ImageNode::new(assets.image("attack")),
                                StrategyAdvanceBannerCmp,
                            ),
                            (
                                Node {
                                    ..default()
                                },
                                TextLayout::new_with_justify(if side == Side::Left {
                                    Justify::Left
                                } else {
                                    Justify::Right
                                }),
                                add_text("0%", "bold", 12., &assets, &window),
                                TextAdvanceBannerCmp,
                                GlobalZIndex(2), // On top of other color banner
                            )
                        ],
                    ));
                }
            };

            // Own banner
            let left = players.get_by_side(Side::Left).color.index();
            spawn(left * 7, Val::Auto, None);
            spawn(1 + left * 7, Val::Auto, None);
            spawn(3 + left * 7, Val::Percent(45.), Some(Side::Left));

            // Enemy banner
            let right = players.get_by_side(Side::Right).color.index();
            spawn(3 + right * 7, Val::Percent(45.), Some(Side::Right));
            spawn(5 + right * 7, Val::Auto, None);
            spawn(6 + right * 7, Val::Auto, None);
        });

    // Draw boosts
    commands
        .spawn((
            Node {
                top: Val::Percent(12.),
                right: Val::Percent(20.),
                width: Val::Percent(60.),
                height: Val::Percent(13.),
                position_type: PositionType::Absolute,
                ..default()
            },
            Pickable::IGNORE,
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            for side in Side::iter() {
                let player = players.get_by_side(side);
                for i in 0..MAX_BOOSTS {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(10.),
                                height: Val::Percent(100.),
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,
                                margin: UiRect::horizontal(Val::Percent(1.)).with_left(if side == Side::Right && i == 0 { Val::Percent(6.) } else { Val::Percent(1.) }),
                                ..default()
                            },
                            ImageNode::new(assets.image("selected boost")),
                            GlobalZIndex(1),
                            Visibility::Hidden,
                            BoostBoxCmp::new(if player.side == Side::Left { i } else { MAX_BOOSTS - 1 - i }, player.color),
                            children![
                                (
                                    Node {
                                        top: Val::Percent(28.5),
                                        left: Val::Percent(6.),
                                        height: Val::Percent(57.5),
                                        width: Val::Percent(87.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    ImageNode::new(assets.image("longbow")),
                                    GlobalZIndex(0),
                                    BoostBoxImageCmp,
                                    children![(
                                        Node {
                                            bottom: Val::Percent(2.),
                                            right: Val::Percent(9.),
                                            position_type: PositionType::Absolute,
                                            ..default()
                                        },
                                        BoostBoxTimerCmp,
                                        add_text("", "bold", 12., &assets, &window,),
                                    )],
                                ),
                                (
                                    Node {
                                        top: Val::Percent(5.),
                                        right: Val::Percent(-250.),
                                        width: Val::Percent(270.),
                                        height: Val::Percent(120.),
                                        position_type: PositionType::Absolute,
                                        padding: UiRect::horizontal(Val::Percent(40.))
                                            .with_top(Val::Percent(20.)),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image("banner")),
                                    Pickable::IGNORE,
                                    GlobalZIndex(2),
                                    Visibility::Hidden,
                                    HoverBoxBoostCmp,
                                    children![(
                                        TextColor(Color::BLACK),
                                        add_text("", "bold", 10., &assets, &window),
                                        HoverBoxBoostLabelCmp,
                                    )],
                                )
                            ],
                        ))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(
                            |event: On<Pointer<Over>>,
                             mut box_q: Query<&mut Visibility, With<HoverBoxBoostCmp>>,
                             children_q: Query<&Children>| {
                                for child in children_q.iter_descendants(event.entity) {
                                    if let Ok(mut v) = box_q.get_mut(child) {
                                        *v = Visibility::Inherited;
                                    }
                                }
                            },
                        )
                        .observe(
                            |event: On<Pointer<Out>>,
                             mut box_q: Query<&mut Visibility, With<HoverBoxBoostCmp>>,
                             children_q: Query<&Children>| {
                                for child in children_q.iter_descendants(event.entity) {
                                    if let Ok(mut v) = box_q.get_mut(child) {
                                        *v = Visibility::Hidden;
                                    }
                                }
                            },
                        )
                        .observe(
                            |event: On<Pointer<Click>>,
                             box_q: Query<&BoostBoxCmp>,
                             mut players: ResMut<Players>,
                             game_state: Res<State<GameState>>,
                             mut play_audio_msg: MessageWriter<PlayAudioMsg>,
                             mut activate_boost_msg: MessageWriter<ActivateBoostMsg>| {
                                if event.button == PointerButton::Primary && *game_state.get() == GameState::Playing {
                                    if let Ok(bbox) = box_q.get(event.entity) {
                                        let color = players.me.color;
                                            if color == bbox.color {
                                                if let Some(boost) = players.me.boosts.get_mut(bbox.n) {
                                                if !boost.active {
                                                    boost.active = true;
                                                    activate_boost_msg.write(ActivateBoostMsg::new(boost.name, color));
                                                } else {
                                                    // Finish the boost early
                                                    boost.timer.finish();
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    play_audio_msg.write(PlayAudioMsg::new("error"));
                                }
                            },
                        );
                }
            }
        });

    // Draw direction
    commands
        .spawn((
            Node {
                top: Val::Percent(5.),
                left: Val::Percent(2.),
                width: Val::Percent(9.),
                height: Val::Percent(8.),
                ..default()
            },
            ImageNode {
                image: assets.image(players.me.direction.image()),
                flip_x: players.me.side == Side::Right,
                flip_y: players.me.direction.flip_y(),
                ..default()
            },
            DirectionCmp,
            UiCmp,
            MapCmp,
        ))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(
            |event: On<Pointer<Click>>,
             mut players: ResMut<Players>,
             mut play_audio_msg: MessageWriter<PlayAudioMsg>| {
                if event.button == PointerButton::Primary {
                    players.me.direction = players.me.direction.next();
                    play_audio_msg.write(PlayAudioMsg::new("click"));
                } else if event.button == PointerButton::Secondary {
                    players.me.direction = players.me.direction.previous();
                    play_audio_msg.write(PlayAudioMsg::new("click"));
                };
            },
        );

    // Draw shop
    let texture = assets.texture("small ribbons");
    commands
        .spawn((
            Node {
                left: Val::Percent(2.),
                width: Val::Percent(7.),
                height: Val::Percent(75.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            Pickable::IGNORE,
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            let mut spawn = |idx| {
                parent.spawn((
                    Node {
                        width: Val::Percent(100.),
                        aspect_ratio: Some(1.0),
                        ..default()
                    },
                    ImageNode::from_atlas_image(
                        texture.image.clone(),
                        TextureAtlas {
                            layout: texture.layout.clone(),
                            index: idx + players.me.color.index() * 10,
                        },
                    ),
                    UiTransform::from_rotation(Rot2::degrees(90.)),
                ));
            };

            // Spawn banner
            for idx in [0, 2, 2, 2, 9] {
                spawn(idx);
            }

            parent
                .spawn(Node {
                    width: Val::Percent(80.),
                    height: Val::Percent(70.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    for unit in UnitName::iter().filter(|u| u.is_basic_unit()).chain([UnitName::Turtle]) {
                        parent.spawn((Node {
                                display: if unit.is_basic_unit() {
                                    Display::Flex
                                } else {
                                    Display::None
                                },
                                width: Val::Percent(100.),
                                aspect_ratio: Some(1.),
                                ..default()
                            },
                            ImageNode::new(assets.image(format!(
                                "{}-{}",
                                players.me.color.to_name(),
                                unit.to_name()
                            ))),
                            ShopButtonCmp::new(unit, unit.is_basic_unit()),
                            children![
                                (
                                    Node {
                                        top: Val::Percent(15.),
                                        left: Val::Percent(70.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    add_text("0", "bold", 12., &assets, &window),
                                    ShopLabelCmp(unit),
                                ),
                                (
                                    Node {
                                        bottom: Val::Percent(15.),
                                        left: Val::Percent(70.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    add_text(
                                        unit.key().to_name().chars().last().unwrap().to_string(),
                                        "bold",
                                        12.,
                                        &assets,
                                        &window,
                                    )
                                ),
                            ],
                        ))
                        .with_children(|parent| {
                            // Spawn hover box
                            parent.spawn((
                                Node {
                                    top: Val::Percent(-140.),
                                    left: Val::Percent(80.),
                                    width: Val::Percent(600.),
                                    height: Val::Percent(700.),
                                    position_type: PositionType::Absolute,
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(Val::Percent(75.)),
                                    ..default()
                                },
                                ImageNode::new(assets.image("banner")),
                                Pickable::IGNORE,
                                GlobalZIndex(2), // On top of queue
                                Visibility::Hidden,
                                HoverBoxCmp,
                                ))
                                .with_children(|parent| {
                                parent
                                    .spawn((Node {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        flex_direction: FlexDirection::Column,
                                        margin: UiRect::ZERO.with_bottom(Val::Percent(4.)),
                                        ..default()
                                    }, children![(
                                        Node {
                                            margin: UiRect::vertical(Val::Percent(2.)),
                                            ..default()
                                        },
                                        TextColor(Color::BLACK),
                                        add_text(unit.to_title(), "bold", 18., &assets, &window),
                                    ),
                                    (
                                        TextColor(Color::BLACK),
                                        add_text(unit.description(), "bold", 8., &assets, &window),
                                    )]));

                                parent
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::FlexStart,
                                        margin: UiRect::ZERO.with_left(Val::Percent(5.)),
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        let attributes = [
                                            ("Health", unit.health().to_string()),
                                            (if unit.physical_damage() >= 0. {"Physical damage"} else {"Healing"}, unit.physical_damage().abs().to_string()),
                                            ("Magic damage", unit.magic_damage().to_string()),
                                            (if unit.physical_damage() >= 0. {"Attack speed"} else {"Healing speed"}, format!("{:.1}", 10. / unit.frames(if unit.physical_damage() > 0. {Action::Attack(Entity::PLACEHOLDER)} else {Action::Heal(Entity::PLACEHOLDER)}) as f32)),
                                            ("Armor", unit.armor().to_string()),
                                            ("Magic resist", unit.magic_resist().to_string()),
                                            ("Armor penetration", unit.armor_pen().to_string()),
                                            ("Magic penetration", unit.magic_pen().to_string()),
                                            ("Can guard", if unit.can_guard() {"Yes".to_owned()} else {"No".to_owned()}),
                                            ("Movement speed", unit.speed().to_string()),
                                            ("Attack range", unit.range().to_string()),
                                            ("Spawn duration", format!("{:.1}s", unit.spawn_duration() as f32 / 1000.)),
                                        ];

                                        for (k, v) in attributes.iter() {
                                            parent.spawn((
                                                Node {
                                                    width: Val::Percent(100.),
                                                    margin: UiRect::ZERO.with_bottom(Val::Percent(1.)),
                                                    ..default()
                                                },
                                                children![
                                                    (
                                                        Node {
                                                            width: Val::Percent(5.),
                                                            aspect_ratio: Some(1.0),
                                                            margin: UiRect::ZERO.with_right(Val::Percent(2.)),
                                                            ..default()
                                                        },
                                                        ImageNode::new(assets.image(k.to_lowercase())),
                                                    ),
                                                    (
                                                        TextColor(Color::BLACK),
                                                        add_text(
                                                            format!("{k}: {v}"),
                                                            "bold",
                                                            8.,
                                                            &assets,
                                                            &window,
                                                        ),
                                                    )
                                                ],
                                            ));
                                        }
                                    });
                            });
                        })
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(|event: On<Pointer<Over>>, mut box_q: Query<&mut Visibility, With<HoverBoxCmp>>, children_q: Query<&Children>| {
                            for child in children_q.iter_descendants(event.entity) {
                                if let Ok(mut v) = box_q.get_mut(child) {
                                    *v = Visibility::Inherited;
                                }
                            }
                        })
                        .observe(|event: On<Pointer<Out>>, mut box_q: Query<&mut Visibility, With<HoverBoxCmp>>, children_q: Query<&Children>| {
                            for child in children_q.iter_descendants(event.entity) {
                                if let Ok(mut v) = box_q.get_mut(child) {
                                    *v = Visibility::Hidden;
                                }
                            }
                        })
                        .observe(
                            |event: On<Pointer<Click>>,
                             btn_q: Query<&ShopButtonCmp>,
                             players: Res<Players>,
                             mut queue_unit_msg: MessageWriter<QueueUnitMsg>,
                             mut play_audio_msg: MessageWriter<PlayAudioMsg>| {
                                if event.button == PointerButton::Primary {
                                    let button = btn_q.get(event.entity).unwrap();
                                    queue_unit_msg.write(QueueUnitMsg::new(players.me.id, button.unit));
                                    play_audio_msg.write(PlayAudioMsg::new("button"));
                                }
                            },
                        );
                    }
                });
        });

    // Draw strategies
    let texture = assets.texture("small ribbons");
    commands
        .spawn((
            Node {
                right: Val::Percent(2.),
                width: Val::Percent(5.),
                height: Val::Percent(50.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            Pickable::IGNORE,
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            let mut spawn = |idx| {
                parent.spawn((
                    Node {
                        width: Val::Percent(100.),
                        aspect_ratio: Some(1.0),
                        ..default()
                    },
                    ImageNode::from_atlas_image(
                        texture.image.clone(),
                        TextureAtlas {
                            layout: texture.layout.clone(),
                            index: idx + players.me.color.index() * 10,
                        },
                    ),
                    UiTransform::from_rotation(Rot2::degrees(90.)),
                ));
            };

            // Spawn banner
            for idx in [0, 2, 2, 2, 9] {
                spawn(idx);
            }

            parent
                .spawn(Node {
                    width: Val::Percent(60.),
                    height: Val::Percent(70.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    for strategy in Strategy::iter() {
                        parent
                            .spawn((
                                Node {
                                    width: Val::Percent(100.),
                                    aspect_ratio: Some(1.0),
                                    margin: UiRect::all(Val::Percent(15.)),
                                    ..default()
                                },
                                ImageNode::new(assets.image(strategy.to_lowername())),
                                StrategyButtonCmp(strategy),
                                children![(
                                    Node {
                                        bottom: Val::Percent(-5.),
                                        left: Val::Percent(70.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    add_text(
                                        strategy
                                            .key()
                                            .to_name()
                                            .chars()
                                            .last()
                                            .unwrap()
                                            .to_string(),
                                        "bold",
                                        10.,
                                        &assets,
                                        &window,
                                    )
                                ),(
                                    Node {
                                        bottom: Val::Percent(0.),
                                        left: Val::Percent(0.),
                                        width: Val::Percent(100.),
                                        height: Val::Percent(20.),
                                        position_type: PositionType::Absolute,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::BLACK),
                                    Visibility::Hidden,
                                    ZIndex(2),
                                    StrategyProgressWrapperCmp,
                                    children![(
                                        Node {
                                            width: Val::Percent(95.),
                                            height: Val::Percent(76.),
                                            left: Val::Percent(3.),
                                            ..default()
                                        },
                                        BackgroundColor(players.me.color.color()),
                                        StrategyProgressCmp,
                                    )]
                                )],
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Node {
                                        top: Val::Percent(-50.),
                                        left: Val::Percent(-540.),
                                        width: Val::Percent(550.),
                                        height: Val::Percent(540.),
                                        align_items: AlignItems::Center,
                                        position_type: PositionType::Absolute,
                                        flex_direction: FlexDirection::Column,
                                        padding: UiRect::horizontal(Val::Percent(70.)).with_top(Val::Percent(60.)),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image("banner")),
                                    Pickable::IGNORE,
                                    GlobalZIndex(2),
                                    Visibility::Hidden,
                                    StrategyHoverCmp,
                                    children![
                                        (
                                            Node {
                                                margin: UiRect::vertical(Val::Percent(2.)),
                                                ..default()
                                            },
                                            TextColor(Color::BLACK),
                                            add_text(
                                                strategy.to_name(),
                                                "bold",
                                                12.,
                                                &assets,
                                                &window
                                            ),
                                        ),
                                        (
                                            TextColor(Color::BLACK),
                                            add_text(
                                                strategy.description(),
                                                "bold",
                                                8.,
                                                &assets,
                                                &window
                                            ),
                                        )
                                    ],
                                ));
                            })
                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                            .observe(
                                |event: On<Pointer<Over>>,
                                 mut hover_q: Query<&mut Visibility, With<StrategyHoverCmp>>,
                                 children_q: Query<&Children>| {
                                    for child in children_q.iter_descendants(event.entity) {
                                        if let Ok(mut v) = hover_q.get_mut(child) {
                                            *v = Visibility::Inherited;
                                        }
                                    }
                                },
                            )
                            .observe(
                                |event: On<Pointer<Out>>,
                                 mut hover_q: Query<&mut Visibility, With<StrategyHoverCmp>>,
                                 children_q: Query<&Children>| {
                                    for child in children_q.iter_descendants(event.entity) {
                                        if let Ok(mut v) = hover_q.get_mut(child) {
                                            *v = Visibility::Hidden;
                                        }
                                    }
                                },
                            )
                            .observe(
                                |event: On<Pointer<Click>>,
                                 btn_q: Query<&StrategyButtonCmp>,
                                 mut players: ResMut<Players>,
                                 mut play_audio_msg: MessageWriter<PlayAudioMsg>| {
                                    // Remove unit from queue if clicked
                                    if event.button == PointerButton::Primary {
                                        if let Ok(button) = btn_q.get(event.entity) {
                                            if **button != players.me.strategy && players.me.strategy_timer.is_finished() {
                                                play_audio_msg.write(PlayAudioMsg::new("click"));
                                                players.me.strategy = **button;
                                                players.me.strategy_timer.reset();
                                            } else {
                                                play_audio_msg.write(PlayAudioMsg::new("error"));
                                            }
                                        }
                                    }
                                },
                            );
                    }
                });
        });

    // Draw queue
    let texture = assets.texture("swords1");
    commands
        .spawn((
            Node {
                left: Val::Percent(10.),
                bottom: Val::Percent(6.),
                width: Val::Percent(88.),
                height: Val::Percent(15.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                ..default()
            },
            Pickable::IGNORE,
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    height: Val::Percent(100.),
                    ..default()
                },
                ImageNode::from_atlas_image(
                    texture.image,
                    TextureAtlas {
                        layout: texture.layout,
                        index: players.me.color.index(),
                    },
                ),
            ));

            for i in 0..MAX_QUEUE_LENGTH {
                parent
                    .spawn((
                        Node {
                            height: Val::Percent(100.),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ImageNode::new(assets.image("swords2")),
                        SwordQueueCmp(i),
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn((
                                Node {
                                    height: Val::Percent(80.),
                                    aspect_ratio: Some(1.0),
                                    ..default()
                                },
                                ImageNode::new(assets.image(format!(
                                    "{}-{}",
                                    PlayerColor::Blue.to_name(),
                                    UnitName::default().to_name()
                                ))),
                                QueueButtonCmp(i),
                                children![(
                                    Node {
                                        top: Val::Percent(70.),
                                        left: Val::Percent(20.),
                                        width: Val::Percent(60.),
                                        height: Val::Percent(12.),
                                        position_type: PositionType::Absolute,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::BLACK),
                                    Visibility::Hidden,
                                    QueueProgressWrapperCmp,
                                    children![(
                                        Node {
                                            width: Val::Percent(95.),
                                            height: Val::Percent(75.),
                                            left: Val::Percent(3.),
                                            ..default()
                                        },
                                        BackgroundColor(players.me.color.color()),
                                        QueueProgressCmp,
                                    )]
                                )],
                            ))
                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                            .observe(
                                |event: On<Pointer<Click>>,
                                 btn_q: Query<&QueueButtonCmp>,
                                 mut players: ResMut<Players>| {
                                    // Remove unit from queue if clicked
                                    if event.button == PointerButton::Primary {
                                        if let Ok(button) = btn_q.get(event.entity) {
                                            players.me.queue.remove(**button);
                                        }
                                    }
                                },
                            );
                    });
            }

            parent.spawn((
                Node {
                    height: Val::Percent(100.),
                    ..default()
                },
                ImageNode::new(assets.image("swords3")),
            ));
        });

    // Draw speed indicator
    commands.spawn((
        Node {
            bottom: Val::Px(10.),
            left: Val::Px(10.),
            position_type: PositionType::Absolute,
            ..default()
        },
        add_text(format!("{}x", settings.speed), "medium", 10., &assets, &window),
        Pickable::IGNORE, // Don't block camera movement
        SpeedCmp,
        UiCmp,
        MapCmp,
    ));
}

/// Updates the advance banner, shop, direction and strategy
pub fn update_ui(
    unit_q: Query<(&Transform, &Unit)>,
    mut direction_q: Query<&mut ImageNode, With<DirectionCmp>>,
    mut advance_q: Query<(Entity, &mut Node, &AdvanceBannerCmp)>,
    mut image_q: Query<
        &mut ImageNode,
        (With<StrategyAdvanceBannerCmp>, Without<DirectionCmp>, Without<ShopButtonCmp>),
    >,
    mut text_q: Query<&mut Text, With<TextAdvanceBannerCmp>>,
    mut btn_q: Query<
        (&mut Node, &mut ImageNode, &mut ShopButtonCmp),
        (Without<DirectionCmp>, Without<AdvanceBannerCmp>),
    >,
    mut label_q: Query<(&mut Text, &ShopLabelCmp), Without<TextAdvanceBannerCmp>>,
    strategy_q: Query<(Entity, &StrategyButtonCmp)>,
    mut wrapper_q: Query<&mut Visibility, With<StrategyProgressWrapperCmp>>,
    mut progress_q: Query<
        &mut Node,
        (With<StrategyProgressCmp>, Without<AdvanceBannerCmp>, Without<ShopButtonCmp>),
    >,
    children_q: Query<&Children>,
    players: Res<Players>,
    assets: Res<WorldAssets>,
) {
    let (mut me, mut enemy) = (50., 50.); // Start with prior
    let (mut power_me, mut power_enemy) = (0., 0.);
    let mut counts = HashMap::new();
    for (t, unit) in unit_q.iter() {
        let mut x = t.translation.x;

        let (side, acc) = if unit.color == players.me.color {
            *counts.entry(unit.name).or_insert(0) += 1;
            power_me += unit.name.spawn_duration() as f32 * unit.health / unit.name.health();
            (&players.me.side, &mut me)
        } else {
            power_enemy += unit.name.spawn_duration() as f32 * unit.health / unit.name.health();
            (&players.enemy.side, &mut enemy)
        };

        x = match side {
            Side::Left if x > 0. => x,
            Side::Right if x < 0. => -x,
            _ => continue,
        };

        *acc += (1. / (1. + (-1.5 * x).exp()) - 0.5) * 20.;
    }

    let total = me + enemy;
    let me_score = if total > 0. {
        me / total
    } else {
        0.5
    };
    let enemy_score = 1. - me_score;

    for (entity, mut node, banner) in &mut advance_q {
        let (n, power, strategy) = if banner.0 == players.me.side {
            (me_score, power_me, players.me.strategy)
        } else {
            (enemy_score, power_enemy, players.enemy.strategy)
        };

        node.width = Val::Percent(90. * n);

        for child in children_q.iter_descendants(entity) {
            if let Ok(mut image) = image_q.get_mut(child) {
                image.image = assets.image(strategy.to_lowername());
            }

            if let Ok(mut text) = text_q.get_mut(child) {
                text.0 = format!("{:.0}%\n{:.1}k", 100. * n, power / 1000.);
            }
        }
    }

    // Update the shop
    if let Some((mut node, mut image, mut btn)) = btn_q.iter_mut().find(|(_, _, b)| !b.is_basic) {
        node.display = if let Some(unit) =
            UnitName::iter().find(|u| !u.is_basic_unit() && players.me.can_queue(*u))
        {
            image.image =
                assets.image(format!("{}-{}", players.me.color.to_name(), unit.to_name()));
            btn.unit = unit;
            Display::Flex
        } else {
            Display::None
        }
    }

    for (mut text, label) in label_q.iter_mut() {
        text.0 = counts.get(&*label).unwrap_or(&0).to_string();
    }

    // Update the direction
    for mut image in &mut direction_q {
        image.image = assets.image(players.me.direction.image());
        image.flip_y = players.me.direction.flip_y()
    }

    // Update the strategies
    let timer = &players.me.strategy_timer;
    for (entity, strategy) in strategy_q.iter() {
        for child in children_q.iter_descendants(entity) {
            if let Ok(mut v) = wrapper_q.get_mut(child) {
                *v = if **strategy == players.me.strategy && !timer.is_finished() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }

            if let Ok(mut node) = progress_q.get_mut(child) {
                let frac = 1. - timer.elapsed_secs() / timer.duration().as_secs_f32();
                node.width = Val::Percent(95. * frac);
            }
        }
    }
}

/// Updates the boosts, queue and speed indicator
pub fn update_ui2(
    mut boost_q: Query<(Entity, &mut Visibility, &mut ImageNode, &BoostBoxCmp)>,
    mut image_q: Query<
        &mut ImageNode,
        (With<BoostBoxImageCmp>, Without<BoostBoxCmp>, Without<QueueButtonCmp>),
    >,
    mut timer_q: Query<&mut Text, With<BoostBoxTimerCmp>>,
    mut label_q: Query<&mut Text, (With<HoverBoxBoostLabelCmp>, Without<BoostBoxTimerCmp>)>,
    mut queue_q: Query<(&mut Node, &mut SwordQueueCmp), Without<BoostBoxCmp>>,
    mut images_q: Query<
        (Entity, &mut ImageNode, &QueueButtonCmp),
        (Without<BoostBoxCmp>, Without<BoostBoxImageCmp>),
    >,
    mut progress_wrapper_q: Query<
        &mut Visibility,
        (With<QueueProgressWrapperCmp>, Without<BoostBoxCmp>),
    >,
    mut progress_inner_q: Query<
        &mut Node,
        (With<QueueProgressCmp>, Without<BoostBoxCmp>, Without<SwordQueueCmp>),
    >,
    mut speed_q: Query<
        &mut Text,
        (With<SpeedCmp>, Without<HoverBoxBoostLabelCmp>, Without<BoostBoxTimerCmp>),
    >,
    children_q: Query<&Children>,
    settings: Res<Settings>,
    players: Res<Players>,
    game_state: Res<State<GameState>>,
    assets: Res<WorldAssets>,
) {
    // Update the boosts
    for (box_e, mut box_v, mut image, bbox) in &mut boost_q {
        let player = players.get_by_color(bbox.color);

        // For enemy, only show active boosts (no gaps)
        let boost = if player != players.me {
            player.boosts.iter().filter(|b| b.active).nth(bbox.n)
        } else {
            player.boosts.get(bbox.n)
        };

        *box_v = if let Some(boost) = boost {
            image.image = assets.image(if player != players.me {
                "enemy boost"
            } else if boost.active {
                "active boost"
            } else {
                "selected boost"
            });

            for child in children_q.iter_descendants(box_e) {
                if let Ok(mut image) = image_q.get_mut(child) {
                    image.image = assets.image(boost.name.to_lowername());
                }

                if let Ok(mut text) = timer_q.get_mut(child) {
                    **text = if boost.name.duration() == 0 {
                        "".to_owned()
                    } else {
                        format!("{}s", boost.timer.remaining().as_secs())
                    };
                }

                if let Ok(mut text) = label_q.get_mut(child) {
                    **text = boost.name.description().to_string();
                }
            }

            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    }

    // Update the queue
    for (mut node, queue) in &mut queue_q {
        node.display = if **queue == 0
            || players.me.queue.get(**queue).is_some()
            || (**queue == 1 && players.me.has_boost(Boost::DoubleQueue))
        {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Update the image
    for (entity, mut node, button) in &mut images_q {
        if let Some(queue) = players.me.queue.get(button.0) {
            node.image =
                assets.image(format!("{}-{}", players.me.color.to_name(), queue.unit.to_name()));

            // Update progress bar
            for child in children_q.iter_descendants(entity) {
                let frac = 1. - queue.timer.elapsed_secs() / queue.timer.duration().as_secs_f32();

                if let Ok(mut bar_v) = progress_wrapper_q.get_mut(child) {
                    *bar_v = if frac == 1. {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    };
                }

                if let Ok(mut node) = progress_inner_q.get_mut(child) {
                    node.width = Val::Percent(95. * frac); // 95 is original length of bar
                }
            }
        }
    }

    // Update speed indicator
    if let Ok(mut text) = speed_q.single_mut() {
        **text = format!(
            "{}x{}",
            settings.speed,
            match game_state.get() {
                GameState::Playing => "",
                _ => " - paused",
            },
        );
    }
}
