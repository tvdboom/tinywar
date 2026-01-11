use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::MAX_QUEUE_LENGTH;
use crate::core::map::systems::MapCmp;
use crate::core::mechanics::queue::QueueUnitMsg;
use crate::core::menu::utils::add_text;
use crate::core::player::Players;
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::GameState;
use crate::core::units::units::{Unit, UnitName};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy_tweening::{PlaybackState, TweenAnim};
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Component)]
pub struct UiCmp;

#[derive(Component)]
pub struct AdvanceBannerCmp(pub bool);

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
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            let mut spawn = |index, width, component| {
                let mut p = parent.spawn((
                    Node {
                        height: Val::Percent(100.),
                        width,
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

                if let Some(me) = component {
                    p.insert((
                        AdvanceBannerCmp(me),
                        children![(
                            Node {
                                justify_content: JustifyContent::End,
                                align_items: if me {
                                    AlignItems::Start
                                } else {
                                    AlignItems::End
                                },
                                ..default()
                            },
                            add_text("0%", "bold", 20., &assets, &window),
                        )],
                    ));
                }
            };

            // Own banner
            let me = players.me.color.index();
            spawn(me * 7, Val::Auto, None);
            spawn(1 + me * 7, Val::Auto, None);
            spawn(3 + me * 7, Val::Percent(45.), Some(true));

            // Enemy banner
            let enemy = players.enemy.color.index();
            spawn(3 + enemy * 7, Val::Percent(45.), Some(false));
            spawn(5 + enemy * 7, Val::Auto, None);
            spawn(6 + enemy * 7, Val::Auto, None);
        });

    // Draw units
    commands
        .spawn((
            Node {
                top: Val::Percent(15.),
                left: Val::Percent(2.),
                height: Val::Percent(90.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                ..default()
            },
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            for unit in UnitName::iter() {
                parent
                    .spawn((
                        Node {
                            height: Val::Percent(12.),
                            aspect_ratio: Some(1.),
                            ..default()
                        },
                        ImageNode::new(assets.image(format!(
                            "{}-{}",
                            PlayerColor::Blue.to_name(),
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
                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                    .observe(
                        |event: On<Pointer<Click>>,
                         btn_q: Query<&ShopButtonCmp>,
                         players: Res<Players>,
                         mut queue_unit_msg: MessageWriter<QueueUnitMsg>,
                         mut play_audio_msg: MessageWriter<PlayAudioMsg>| {
                            if event.button == PointerButton::Primary {
                                let unit = btn_q.get(event.entity).unwrap().0;
                                queue_unit_msg.write(QueueUnitMsg::new(players.me.id, unit));
                                play_audio_msg.write(PlayAudioMsg::new("button"));
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

pub fn update_ui(
    unit_q: Query<(&Transform, &Unit)>,
    mut advance_q: Query<(&mut Node, &AdvanceBannerCmp)>,
    mut label_q: Query<(&mut Text, &ShopLabelCmp)>,
    mut queue_q: Query<(&mut Node, &mut SwordQueueCmp), Without<AdvanceBannerCmp>>,
    mut images_q: Query<(Entity, &mut ImageNode, &QueueButtonCmp)>,
    mut progress_wrapper_q: Query<(Entity, &mut Visibility), With<QueueProgressWrapperCmp>>,
    mut progress_inner_q: Query<
        &mut Node,
        (With<QueueProgressCmp>, Without<SwordQueueCmp>, Without<AdvanceBannerCmp>),
    >,
    mut anim_q: Query<&mut TweenAnim>,
    mut speed_q: Single<&mut Text, (With<SpeedCmp>, Without<ShopLabelCmp>)>,
    children_q: Query<&Children>,
    settings: Res<Settings>,
    players: Res<Players>,
    game_state: Res<State<GameState>>,
    assets: Local<WorldAssets>,
) {
    // Update the shop labels
    let (mut me, mut enemy) = (0., 0.);
    let mut counts = HashMap::new();
    for (unit_t, unit) in unit_q.iter() {
        if unit.color == players.me.color {
            me += unit_t.translation.x;
            *counts.entry(unit.name).or_insert(0) += 1;
        } else {
            enemy -= unit_t.translation.x;
        }
    }

    let frac = if me > 0. || enemy > 0. {
        me.min(enemy) / me.max(enemy)
    } else {
        1. // No units on the board (start of game)
    };
    for (mut node, banner) in &mut advance_q {
        if banner.0 {
            node.width = if me > enemy {
                Val::Percent(45. * (1. + frac))
            } else {
                Val::Percent(45. * frac)
            };
        } else {
            node.width = if me > enemy {
                Val::Percent(frac)
            } else {
                Val::Percent(45. * frac)
            };
        }
    }

    for (mut text, label) in label_q.iter_mut() {
        text.0 = counts.get(&label.0).unwrap_or(&0).to_string();
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

    // Play/pause tween animations
    anim_q.iter_mut().for_each(|mut t| match game_state.get() {
        GameState::Playing => {
            t.playback_state = PlaybackState::Playing;
            t.speed = settings.speed as f64;
        },
        _ => t.playback_state = PlaybackState::Paused,
    });

    // Update speed indicator
    speed_q.as_mut().0 = format!(
        "{}x{}",
        settings.speed,
        match game_state.get() {
            GameState::Playing => "",
            _ => " - paused",
        },
    );
}
