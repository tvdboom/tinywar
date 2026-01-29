use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::boosts::{Boost, CardCmp};
use crate::core::constants::{BUTTON_TEXT_SIZE, MAX_BOOSTS};
use crate::core::map::systems::MapCmp;
use crate::core::map::ui::systems::UiCmp;
use crate::core::map::utils::UiScaleLens;
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::player::{Players, SelectedBoost};
use crate::core::settings::{GameMode, Settings};
use crate::core::states::GameState;
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
    settings: Res<Settings>,
    players: Res<Players>,
    mut next_game_state: ResMut<NextState<GameState>>,
    assets: Local<WorldAssets>,
    window: Single<&Window>,
) {
    // Skip boost selection if already at max. number of boosts
    if players.me.boosts.len() == MAX_BOOSTS {
        if settings.game_mode == GameMode::SinglePlayer {
            next_game_state.set(GameState::Playing);
        } else {
            next_game_state.set(GameState::AfterBoostSelection);
        }
        return;
    }

    // The possible boosts to select are those that aren't drained nor in the current selected list
    let boosts = Boost::iter()
        .filter(|t| {
            !players.me.drained_boosts.contains(t)
                && !players.me.boosts.iter().map(|b| b.name).contains(t)
        })
        .choose_multiple(&mut rng(), 3);

    commands.spawn((add_root_node(false), CardCmp, MapCmp)).with_children(|parent| {
        parent
            .spawn(Node {
                top: Val::Percent(7.),
                width: Val::Percent(80.),
                height: Val::Percent(60.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                margin: UiRect::ZERO.with_top(Val::Percent(5.)),
                ..default()
            })
            .with_children(|parent| {
                for boost in boosts.into_iter() {
                    parent.spawn((Node {
                                width: Val::Percent(23.),
                                height: Val::Percent(100.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                margin: UiRect::horizontal(Val::Percent(3.)),
                                ..default()
                            },
                            ImageNode::new(assets.image("boost")),
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
                                    top: Val::Percent(26.),
                                    height: Val::Percent(30.),
                                    width: Val::Percent(85.),
                                    ..default()
                                },
                                ImageNode::new(assets.image(boost.to_lowername())),
                                children![(
                                    Node {
                                        bottom: Val::Percent(0.),
                                        right: Val::Percent(3.),
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
                                    top: Val::Percent(32.),
                                    height: Val::Percent(30.),
                                    width: Val::Percent(60.),
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
                        .observe(move |
                            trigger: On<Pointer<Click>>,
                            settings: Res<Settings>,
                            mut players: ResMut<Players>,
                            mut play_audio_msg: MessageWriter<PlayAudioMsg>,
                            mut next_game_state: ResMut<NextState<GameState>>| {
                            if trigger.event.button == PointerButton::Primary {
                                play_audio_msg.write(PlayAudioMsg::new("button"));

                                players.me.boosts.push(SelectedBoost::new(boost));
                                if boost.is_draining() {
                                    players.me.drained_boosts.push(boost);
                                }

                                if settings.game_mode == GameMode::SinglePlayer {
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
    assets: Local<WorldAssets>,
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
