use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::{MAX_BOOSTS, MAX_QUEUE_LENGTH};
use crate::core::map::systems::MapCmp;
use crate::core::mechanics::queue::QueueUnitMsg;
use crate::core::menu::utils::add_text;
use crate::core::player::{Players, SelectedBoost, Side};
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::GameState;
use crate::core::units::units::{Action, Unit, UnitName};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use crate::core::boosts::InitiateBoostMsg;

#[derive(Component)]
pub struct UiCmp;

#[derive(Component, Deref)]
pub struct AdvanceBannerCmp(pub Side);

#[derive(Component)]
pub struct TextAdvanceBannerCmp;

#[derive(Component)]
pub struct DirectionCmp;

#[derive(Component, Deref)]
pub struct BoostBoxCmp(pub usize);

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
pub struct ShopButtonCmp(pub UnitName);

#[derive(Component)]
pub struct ShopLabelCmp(pub UnitName);

#[derive(Component)]
pub struct SwordQueueCmp(pub usize);

#[derive(Component)]
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
    assets: Local<WorldAssets>,
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
                        children![(
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            add_text("0%", "bold", 20., &assets, &window),
                            TextAdvanceBannerCmp,
                            GlobalZIndex(2), // On top of other color banner
                        )],
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

    // Draw units
    let texture = assets.texture("small ribbons");
    commands
        .spawn((
            Node {
                top: Val::Percent(12.),
                left: Val::Percent(2.),
                width: Val::Percent(7.),
                height: Val::Percent(70.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
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
                    for unit in UnitName::iter() {
                        parent.spawn((Node {
                                width: Val::Percent(100.),
                                aspect_ratio: Some(1.),
                                ..default()
                            },
                            ImageNode::new(assets.image(format!(
                                "{}-{}",
                                players.me.color.to_name(),
                                unit.to_name()
                            ))),
                            ShopButtonCmp(unit),
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
                                    top: Val::Percent(-40.),
                                    left: Val::Percent(80.),
                                    width: Val::Percent(600.),
                                    height: Val::Percent(550.),
                                    position_type: PositionType::Absolute,
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(Val::Percent(70.)),
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
                                    .spawn(Node {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        flex_direction: FlexDirection::Column,
                                        margin: UiRect::ZERO.with_bottom(Val::Percent(5.)),
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Node {
                                                margin: UiRect::ZERO.with_bottom(Val::Percent(2.)),
                                                ..default()
                                            },
                                            TextColor(Color::BLACK),
                                            add_text(unit.to_title(), "bold", 18., &assets, &window),
                                        ));
                                        parent.spawn((
                                            TextColor(Color::BLACK),
                                            add_text(unit.description(), "bold", 8., &assets, &window)),
                                        );
                                    });

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
                                            (if unit.damage() > 0. {"Damage"} else {"Healing"}, unit.damage().abs().to_string()),
                                            (if unit.damage() > 0. {"Attack speed"} else {"Healing speed"}, format!("{:.1}", 10. / unit.frames(if unit.damage() > 0. {Action::Attack(Entity::PLACEHOLDER)} else {Action::Heal(Entity::PLACEHOLDER)}) as f32)),
                                            ("Speed", unit.speed().to_string()),
                                            ("Range", unit.range().to_string()),
                                            ("Spawn time", format!("{}s", unit.spawn_duration() / 1000)),
                                        ];

                                        for (k, v) in attributes.iter() {
                                            // Skip default values
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::ZERO.with_bottom(Val::Percent(1.)),
                                                    ..default()
                                                },
                                                TextColor(Color::BLACK),
                                                add_text(
                                                    format!("{k}: {}", v),
                                                    "bold",
                                                    10.,
                                                    &assets,
                                                    &window,
                                                ),
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
                                    let unit = btn_q.get(event.entity).unwrap().0;
                                    queue_unit_msg.write(QueueUnitMsg::new(players.me.id, unit, false));
                                    play_audio_msg.write(PlayAudioMsg::new("button"));
                                }
                            },
                        );
                    }
                });
        });

    // Draw boosts
    commands
        .spawn((
            Node {
                top: Val::Percent(10.),
                right: Val::Percent(1.),
                width: Val::Percent(8.),
                height: Val::Percent(90.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Pickable::IGNORE,
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            for i in 0..MAX_BOOSTS {
                parent
                    .spawn((
                        Node {
                            display: Display::None,
                            width: Val::Percent(100.),
                            height: Val::Percent(20.),
                            align_items: AlignItems::Center,
                            justify_items: JustifyItems::Center,
                            margin: UiRect::ZERO.with_bottom(Val::Percent(5.)),
                            ..default()
                        },
                        ImageNode::new(assets.image("selected boost")),
                        BoostBoxCmp(i),
                        children![
                            (
                                Node {
                                    top: Val::Percent(28.),
                                    left: Val::Percent(11.),
                                    height: Val::Percent(57.),
                                    width: Val::Percent(80.),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                ImageNode::new(assets.image("longbow")),
                                BoostBoxImageCmp,
                                children![(
                                    Node {
                                        bottom: Val::Percent(0.),
                                        right: Val::Percent(5.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    BoostBoxTimerCmp,
                                    add_text("", "bold", 13., &assets, &window,),
                                )],
                            ),
                            (
                                Node {
                                    top: Val::Percent(10.),
                                    left: Val::Percent(-235.),
                                    width: Val::Percent(250.),
                                    height: Val::Percent(90.),
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
                         mut initiate_boost_msg: MessageWriter<InitiateBoostMsg>| {
                            if event.button == PointerButton::Primary && *game_state.get() == GameState::Playing {
                                if let Ok(bbox) = box_q.get(event.entity) {
                                    if let Some(boost) = players.me.boosts.get_mut(**bbox) {
                                        boost.active = true;
                                        initiate_boost_msg.write(InitiateBoostMsg(boost.name));
                                    }
                                }
                            }
                        },
                    );
            }
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
                                            players.me.queue.remove(button.0);
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

/// Updates the advance banner and shop labels
pub fn update_ui(
    unit_q: Query<(&Transform, &Unit)>,
    mut direction_q: Query<&mut ImageNode, With<DirectionCmp>>,
    mut advance_q: Query<(Entity, &mut Node, &AdvanceBannerCmp)>,
    mut text_q: Query<&mut Text, With<TextAdvanceBannerCmp>>,
    mut label_q: Query<(&mut Text, &ShopLabelCmp), Without<TextAdvanceBannerCmp>>,
    children_q: Query<&Children>,
    players: Res<Players>,
    assets: Local<WorldAssets>,
) {
    let (mut me, mut enemy) = (50., 50.); // Start with prior
    let mut counts = HashMap::new();
    for (t, unit) in unit_q.iter() {
        let mut x = t.translation.x;

        let (side, acc) = if unit.color == players.me.color {
            *counts.entry(unit.name).or_insert(0) += 1;
            (&players.me.side, &mut me)
        } else {
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
        let n = if banner.0 == players.me.side {
            me_score
        } else {
            enemy_score
        };

        node.width = Val::Percent(90. * n);

        if let Ok(children) = children_q.get(entity) {
            for &child in children {
                if let Ok(mut text) = text_q.get_mut(child) {
                    text.0 = format!("{:.0}%", 100. * n);
                }
            }
        }
    }

    for (mut text, label) in label_q.iter_mut() {
        text.0 = counts.get(&label.0).unwrap_or(&0).to_string();
    }

    // Update the direction
    for mut image in &mut direction_q {
        image.image = assets.image(players.me.direction.image());
        image.flip_y = players.me.direction.flip_y()
    }
}

/// Updates the boosts, queue and speed indicator
pub fn update_ui2(
    mut boost_q: Query<(Entity, &mut Node, &mut ImageNode, &BoostBoxCmp)>,
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
    mut progress_wrapper_q: Query<(Entity, &mut Visibility), With<QueueProgressWrapperCmp>>,
    mut progress_inner_q: Query<&mut Node, (Without<BoostBoxCmp>, Without<SwordQueueCmp>)>,
    mut speed_q: Query<&mut Text, (Without<HoverBoxBoostLabelCmp>, Without<BoostBoxTimerCmp>)>,
    children_q: Query<&Children>,
    settings: Res<Settings>,
    players: Res<Players>,
    game_state: Res<State<GameState>>,
    assets: Local<WorldAssets>,
) {
    // Update the boosts
    let boosts: Vec<&SelectedBoost> = players.me.boosts.iter().rev().collect();
    for (box_e, mut node, mut image, bbox) in &mut boost_q {
        // Update the nth box, which corresponds to the nth boost
        node.display = if let Some(boost) = boosts.get(**bbox) {
            image.image = assets.image(if boost.active {
                "active boost"
            } else {
                "selected boost"
            });

            for child in children_q.iter_descendants(box_e) {
                if let Ok(mut image) = image_q.get_mut(child) {
                    image.image = assets.image(boost.name.to_lowername());
                }

                if let Ok(mut text) = timer_q.get_mut(child) {
                    **text = format!("{}s", boost.timer.duration().as_secs())
                }

                if let Ok(mut text) = label_q.get_mut(child) {
                    **text = boost.name.description().to_string();
                }
            }

            Display::Flex
        } else {
            Display::None
        }
    }

    // Update the queue
    for (mut node, queue) in &mut queue_q {
        node.display = if queue.0 == 0 || players.me.queue.get(queue.0).is_some() {
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
            if let Ok(children) = children_q.get(entity) {
                for &child in children {
                    if let Ok((bar_e, mut bar_v)) = progress_wrapper_q.get_mut(child) {
                        let frac =
                            1. - queue.timer.elapsed_secs() / queue.timer.duration().as_secs_f32();

                        *bar_v = if frac == 1. {
                            Visibility::Hidden
                        } else {
                            Visibility::Inherited
                        };

                        if let Ok(children) = children_q.get(bar_e) {
                            for &child in children {
                                if let Ok(mut node) = progress_inner_q.get_mut(child) {
                                    node.width = Val::Percent(95. * frac); // 95 is original length of bar
                                    break;
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    // Update speed indicator
    if let Ok(mut text) = speed_q.single_mut() {
        text.0 = format!(
            "{}x{}",
            settings.speed,
            match game_state.get() {
                GameState::Playing => "",
                _ => " - paused",
            },
        );
    }
}
