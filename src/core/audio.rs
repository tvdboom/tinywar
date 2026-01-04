use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::core::assets::WorldAssets;
use crate::core::constants::{NORMAL_BUTTON_COLOR, PRESSED_BUTTON_COLOR};
use crate::core::menu::settings::SettingsBtn;
use crate::core::settings::Settings;
use crate::core::states::AudioState;

#[derive(Resource, Default)]
pub struct PlayingAudio(pub HashMap<&'static str, Handle<AudioInstance>>);

impl PlayingAudio {
    pub const DEFAULT_VOLUME: f32 = -30.;
    pub const TWEEN: AudioTween = AudioTween::new(Duration::from_secs(2), AudioEasing::OutPowi(2));
}

#[derive(Message, Clone)]
pub struct PlayAudioMsg {
    pub name: &'static str,
    pub volume: f32,
    pub is_background: bool,
}

impl PlayAudioMsg {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            volume: PlayingAudio::DEFAULT_VOLUME,
            is_background: false,
        }
    }

    pub fn background(mut self) -> Self {
        self.is_background = true;
        self
    }
}

#[derive(Message, Clone)]
pub struct PauseAudioMsg {
    pub name: &'static str,
}

impl PauseAudioMsg {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
        }
    }
}

#[derive(Message, Clone)]
pub struct StopAudioMsg {
    pub name: &'static str,
}

impl StopAudioMsg {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
        }
    }
}

#[derive(Message, Clone)]
pub struct MuteAudioMsg;

#[derive(Component)]
pub struct MusicBtnCmp;

#[derive(Message)]
pub struct ChangeAudioMsg(pub Option<AudioState>);

pub fn setup_audio(mut commands: Commands, assets: Local<WorldAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(3.),
                height: Val::Percent(3.),
                right: Val::Percent(0.),
                top: Val::Percent(2.),
                ..default()
            },
            ZIndex(5),
        ))
        .with_children(|parent| {
            parent.spawn((ImageNode::new(assets.image("no-music")), MusicBtnCmp)).observe(
                |_: On<Pointer<Click>>, mut commands: Commands| {
                    commands.queue(|w: &mut World| {
                        w.write_message(ChangeAudioMsg(None));
                    })
                },
            );
        });
}

pub fn update_audio(
    mut change_audio_msg: MessageReader<ChangeAudioMsg>,
    mut btn_q: Query<&mut ImageNode, With<MusicBtnCmp>>,
    mut settings_btn: Query<(&mut BackgroundColor, &SettingsBtn)>,
    mut settings: ResMut<Settings>,
    audio_state: Res<State<AudioState>>,
    mut next_audio_state: ResMut<NextState<AudioState>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut pause_audio_msg: MessageWriter<PauseAudioMsg>,
    mut stop_audio_msg: MessageWriter<StopAudioMsg>,
    mut mute_audio_msg: MessageWriter<MuteAudioMsg>,
    assets: Local<WorldAssets>,
) {
    for ev in change_audio_msg.read() {
        settings.audio = ev.0.unwrap_or(match *audio_state.get() {
            AudioState::Mute => AudioState::NoMusic,
            AudioState::NoMusic => AudioState::Sound,
            AudioState::Sound => AudioState::Mute,
        });

        if let Ok(mut node) = btn_q.single_mut() {
            node.image = match settings.audio {
                AudioState::Mute => {
                    mute_audio_msg.write(MuteAudioMsg);
                    next_audio_state.set(AudioState::Mute);
                    assets.image("mute")
                },
                AudioState::NoMusic => {
                    pause_audio_msg.write(PauseAudioMsg::new("music"));
                    stop_audio_msg.write(StopAudioMsg::new("drums"));
                    next_audio_state.set(AudioState::NoMusic);
                    assets.image("no-music")
                },
                AudioState::Sound => {
                    play_audio_msg.write(PlayAudioMsg::new("music").background());
                    next_audio_state.set(AudioState::Sound);
                    assets.image("sound")
                },
            };
        }

        for (mut bgcolor, setting) in &mut settings_btn {
            if matches!(setting, SettingsBtn::Mute | SettingsBtn::NoMusic | SettingsBtn::Sound) {
                bgcolor.0 = if (*setting == SettingsBtn::Mute && settings.audio == AudioState::Mute)
                    || (*setting == SettingsBtn::NoMusic && settings.audio == AudioState::NoMusic)
                    || (*setting == SettingsBtn::Sound && settings.audio == AudioState::Sound)
                {
                    PRESSED_BUTTON_COLOR
                } else {
                    NORMAL_BUTTON_COLOR
                };
            }
        }
    }
}

pub fn toggle_audio(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut change_audio_msg: MessageWriter<ChangeAudioMsg>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        change_audio_msg.write(ChangeAudioMsg(None));
    }
}

pub fn play_music(mut play_audio_msg: MessageWriter<PlayAudioMsg>) {
    play_audio_msg.write(PlayAudioMsg::new("music").background());
}

pub fn play_audio(
    mut play_audio_msg: MessageReader<PlayAudioMsg>,
    audio_state: Res<State<AudioState>>,
    mut playing_audio: ResMut<PlayingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    audio: Res<Audio>,
    assets: Local<WorldAssets>,
) {
    for message in play_audio_msg.read() {
        if *audio_state.get() != AudioState::Mute {
            let mut new_sound = false;

            if let Some(handle) = playing_audio.0.get(message.name) {
                if let Some(instance) = audio_instances.get_mut(handle) {
                    if matches!(
                        instance.state(),
                        PlaybackState::Paused { .. } | PlaybackState::Pausing { .. }
                    ) {
                        if !message.is_background || *audio_state.get() != AudioState::NoMusic {
                            instance.resume(PlayingAudio::TWEEN);
                        }
                    } else if !message.is_background
                        || !matches!(
                            instance.state(),
                            PlaybackState::Playing { .. }
                                | PlaybackState::WaitingToResume { .. }
                                | PlaybackState::Resuming { .. }
                        )
                    {
                        new_sound = true; // Audio finished playing
                    }
                }
            } else if message.is_background {
                if *audio_state.get() != AudioState::NoMusic {
                    playing_audio.0.insert(
                        message.name,
                        audio
                            .play(assets.audio(message.name))
                            .fade_in(PlayingAudio::TWEEN)
                            .with_volume(message.volume)
                            .looped()
                            .handle(),
                    );
                }
            } else {
                new_sound = true;
            }

            if new_sound {
                playing_audio.0.insert(
                    message.name,
                    audio.play(assets.audio(message.name)).with_volume(message.volume).handle(),
                );
            }
        }
    }
}

pub fn pause_audio(
    mut pause_audio_msg: MessageReader<PauseAudioMsg>,
    playing_audio: Res<PlayingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for message in pause_audio_msg.read() {
        if let Some(handle) = playing_audio.0.get(message.name) {
            if let Some(instance) = audio_instances.get_mut(handle) {
                instance.pause(PlayingAudio::TWEEN);
            }
        }
    }
}

pub fn stop_audio(
    mut stop_audio_msg: MessageReader<StopAudioMsg>,
    mut playing_audio: ResMut<PlayingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for message in stop_audio_msg.read() {
        if let Some(handle) = playing_audio.0.get(message.name) {
            if let Some(instance) = audio_instances.get_mut(handle) {
                instance.stop(PlayingAudio::TWEEN);
                playing_audio.0.remove(message.name);
            }
        }
    }
}

pub fn mute_audio(
    mut mute_audio_msg: MessageReader<MuteAudioMsg>,
    playing_audio: Res<PlayingAudio>,
    mut pause_audio_msg: MessageWriter<PauseAudioMsg>,
    mut stop_audio_msg: MessageWriter<StopAudioMsg>,
) {
    for _ in mute_audio_msg.read() {
        for name in playing_audio.0.keys() {
            if *name == "music" {
                pause_audio_msg.write(PauseAudioMsg::new(name));
            } else {
                stop_audio_msg.write(StopAudioMsg::new(name));
            }
        }
    }
}
