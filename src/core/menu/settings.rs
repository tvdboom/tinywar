use std::fmt::Debug;

use bevy::prelude::*;

use crate::core::assets::WorldAssets;
use crate::core::audio::ChangeAudioMsg;
use crate::core::constants::*;
use crate::core::map::map::MapSize;
use crate::core::menu::utils::add_text;
use crate::core::settings::{PlayerColor, Settings};
use crate::core::states::AudioState;
use crate::utils::NameFromEnum;

#[derive(Component, Clone, Debug, PartialEq)]
pub enum SettingsBtn {
    Blue,
    Black,
    Purple,
    Red,
    Yellow,
    Small,
    Medium,
    Large,
    Mute,
    NoMusic,
    Sound,
    True,
    False,
}

fn match_setting(button: &SettingsBtn, settings: &Settings) -> bool {
    match button {
        SettingsBtn::Blue => settings.color == PlayerColor::Blue,
        SettingsBtn::Black => settings.color == PlayerColor::Black,
        SettingsBtn::Purple => settings.color == PlayerColor::Purple,
        SettingsBtn::Red => settings.color == PlayerColor::Red,
        SettingsBtn::Yellow => settings.color == PlayerColor::Yellow,
        SettingsBtn::Small => settings.map_size == MapSize::Small,
        SettingsBtn::Medium => settings.map_size == MapSize::Medium,
        SettingsBtn::Large => settings.map_size == MapSize::Large,
        SettingsBtn::Mute => settings.audio == AudioState::Mute,
        SettingsBtn::NoMusic => settings.audio == AudioState::NoMusic,
        SettingsBtn::Sound => settings.audio == AudioState::Sound,
        SettingsBtn::True => settings.autosave == true,
        SettingsBtn::False => settings.autosave == false,
    }
}

pub fn recolor_label<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(On<Pointer<E>>, Query<(&mut BackgroundColor, &SettingsBtn)>, ResMut<Settings>) {
    move |ev, mut bgcolor_q, settings| {
        if let Ok((mut bgcolor, button)) = bgcolor_q.get_mut(ev.entity) {
            // Don't change the color of selected buttons
            if !match_setting(&button, &settings) {
                bgcolor.0 = color;
            }
        };
    }
}

pub fn on_click_label_button(
    event: On<Pointer<Click>>,
    mut btn_q: Query<(&mut BackgroundColor, &SettingsBtn)>,
    mut settings: ResMut<Settings>,
    mut change_audio_msg: MessageWriter<ChangeAudioMsg>,
) {
    match btn_q.get(event.entity).unwrap().1 {
        SettingsBtn::Blue => settings.color = PlayerColor::Blue,
        SettingsBtn::Black => settings.color = PlayerColor::Black,
        SettingsBtn::Purple => settings.color = PlayerColor::Purple,
        SettingsBtn::Red => settings.color = PlayerColor::Red,
        SettingsBtn::Yellow => settings.color = PlayerColor::Yellow,
        SettingsBtn::Small => settings.map_size = MapSize::Small,
        SettingsBtn::Medium => settings.map_size = MapSize::Medium,
        SettingsBtn::Large => settings.map_size = MapSize::Large,
        SettingsBtn::Mute => {
            settings.audio = AudioState::Mute;
            change_audio_msg.write(ChangeAudioMsg(Some(AudioState::Mute)));
        },
        SettingsBtn::NoMusic => {
            settings.audio = AudioState::NoMusic;
            change_audio_msg.write(ChangeAudioMsg(Some(AudioState::NoMusic)));
        },
        SettingsBtn::Sound => {
            settings.audio = AudioState::Sound;
            change_audio_msg.write(ChangeAudioMsg(Some(AudioState::Sound)));
        },
        SettingsBtn::True => settings.autosave = true,
        SettingsBtn::False => settings.autosave = false,
    }

    // Reset the color of the other buttons
    for (mut bgcolor, setting) in &mut btn_q {
        if !match_setting(setting, &settings) {
            bgcolor.0 = NORMAL_BUTTON_COLOR;
        }
    }
}

pub fn spawn_label(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    buttons: Vec<SettingsBtn>,
    settings: &Settings,
    assets: &WorldAssets,
    window: &Window,
) {
    parent.spawn(add_text(title, "bold", BUTTON_TEXT_SIZE, &assets, &window));

    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            padding: UiRect {
                top: Val::Percent(3.),
                left: Val::Percent(5.),
                right: Val::Percent(5.),
                bottom: Val::Percent(7.),
            },
            ..default()
        })
        .with_children(|parent| {
            for item in buttons.iter() {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(30.),
                            height: Val::Percent(100.),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::all(Val::Percent(1.)),
                            ..default()
                        },
                        BackgroundColor(if match_setting(item, settings) {
                            PRESSED_BUTTON_COLOR
                        } else {
                            NORMAL_BUTTON_COLOR
                        }),
                        item.clone(),
                        Button,
                    ))
                    .observe(recolor_label::<Over>(HOVERED_BUTTON_COLOR))
                    .observe(recolor_label::<Out>(NORMAL_BUTTON_COLOR))
                    .observe(recolor_label::<Press>(PRESSED_BUTTON_COLOR))
                    .observe(recolor_label::<Release>(HOVERED_BUTTON_COLOR))
                    .observe(on_click_label_button)
                    .with_children(|parent| {
                        parent.spawn(add_text(
                            item.to_title(),
                            "bold",
                            SUBTITLE_TEXT_SIZE,
                            assets,
                            window,
                        ));
                    });
            }
        });
}
