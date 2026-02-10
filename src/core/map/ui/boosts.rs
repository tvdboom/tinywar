use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::boosts::{ActivateBoostMsg, AfterBoostCount, Boost, CardCmp};
use crate::core::constants::BUTTON_TEXT_SIZE;
use crate::core::map::systems::MapCmp;
use crate::core::map::ui::systems::UiCmp;
use crate::core::map::utils::UiScaleLens;
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::player::{Player, Players, SelectedBoost};
use crate::core::settings::{GameMode, Settings};
use crate::core::states::GameState;
use crate::core::units::buildings::Building;
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy_tweening::{Tween, TweenAnim};
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::rng;
use std::time::Duration;
use strum::IntoEnumIterator;

pub fn setup_boost_selection(
    mut commands: Commands,
    building_q: Query<&Building>,
    settings: Res<Settings>,
    players: Res<Players>,
    mut play_audio_ev: MessageWriter<PlayAudioMsg>,
    assets: Res<WorldAssets>,
    window: Single<&Window>,
) {
    play_audio_ev.write(PlayAudioMsg::new("message"));

    // The possible boosts to select are those that aren't drained nor in the current selected list
    let boosts = |p: &Player, q: &Query<&Building>| -> Vec<Boost> {
        Boost::iter()
            .filter(|b| {
                b.condition(q.iter().filter(|b| b.color == players.me.color), p)
                    && !p.boosts.iter().map(|b| b.name).contains(b)
            })
            .collect()
    };

    let own_boosts = boosts(&players.me, &building_q).into_iter().sample(&mut rng(), 3);

    // Select a random boost for the NPC
    let enemy_boost = if settings.game_mode == GameMode::SinglePlayer {
        boosts(&players.enemy, &building_q).into_iter().choose(&mut rng()).unwrap()
    } else {
        Boost::ArmorGain // Random boost (never used)
    };

    commands.spawn((add_root_node(false), CardCmp, MapCmp)).with_children(|parent| {
        parent
            .spawn(Node {
                top: percent(7.),
                width: percent(80.),
                height: percent(60.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                margin: UiRect::ZERO.with_top(percent(5.)),
                ..default()
            })
            .with_children(|parent| {
                for boost in own_boosts.into_iter() {
                    parent.spawn((Node {
                                width: percent(23.),
                                height: percent(100.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                margin: UiRect::horizontal(percent(3.)),
                                ..default()
                            },
                            ImageNode::new(assets.image("boost")),
                            GlobalZIndex(1),
                            UiCmp, // Required to not pause animation
                            TweenAnim::new(
                                Tween::new(
                                    EaseFunction::Linear,
                                    Duration::from_millis(300),
                                    UiScaleLens {
                                        start: Vec2::ZERO,
                                        end: Vec2::splat(1.),
                                    },
                                )
                            ),
                            children![(
                                Node {
                                    top: percent(27.),
                                    height: percent(33.5),
                                    width: percent(85.),
                                    ..default()
                                },
                                ImageNode::new(assets.image(boost.to_lowername())),
                                GlobalZIndex(0),
                                children![(
                                    Node {
                                        bottom: percent(1.),
                                        right: percent(5.),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    add_text(
                                        if boost.duration() > 0 { format!("{}s", boost.duration()) } else { "".to_owned() },
                                        "bold",
                                        18.,
                                        &assets,
                                        &window,
                                    ),
                                )],
                            ),
                            (
                                Node {
                                    top: percent(34.),
                                    height: percent(30.),
                                    width: percent(70.),
                                    ..default()
                                },
                                TextColor(Color::BLACK),
                                add_text(
                                    boost.description(),
                                    "bold",
                                    10.,
                                    &assets,
                                    &window,
                                ),
                            )]
                        ))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(cursor::<Release>(SystemCursorIcon::Default))
                        .observe(move |
                            trigger: On<Pointer<Click>>,
                            settings: Res<Settings>,
                            mut players: ResMut<Players>,
                            mut boost_count: ResMut<AfterBoostCount>,
                            mut activate_boost_msg: MessageWriter<ActivateBoostMsg>,
                            mut play_audio_msg: MessageWriter<PlayAudioMsg>,
                            mut next_game_state: ResMut<NextState<GameState>>| {
                            if trigger.event.button == PointerButton::Primary {
                                play_audio_msg.write(PlayAudioMsg::new("button"));

                                players.me.boosts.push(SelectedBoost::new(boost));

                                if settings.game_mode == GameMode::SinglePlayer {
                                    players.enemy.boosts.push(SelectedBoost::new(enemy_boost).active());
                                    activate_boost_msg.write(ActivateBoostMsg::new(enemy_boost, players.enemy.color));
                                    next_game_state.set(GameState::Playing);
                                }  else if **boost_count == 1 {
                                    **boost_count = 0;
                                    next_game_state.set(GameState::Playing);
                                } else {
                                    next_game_state.set(GameState::AfterBoostSelection);
                                }
                            }});
                }
            });
    });
}

pub fn setup_after_boost(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    window: Single<&Window>,
) {
    commands.spawn((add_root_node(false), CardCmp, MapCmp)).with_children(|parent| {
        parent.spawn(add_text(
            "Waiting for other players to select a boost...",
            "bold",
            BUTTON_TEXT_SIZE,
            &assets,
            &window,
        ));
    });
}
