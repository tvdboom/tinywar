use crate::core::audio::PlayAudioMsg;
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::utils::scale_duration;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Component)]
pub struct CardCmp;

#[derive(Message, Deref)]
pub struct InitiateBoostMsg(pub Boost);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AfterBoostCount(pub usize);

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Boost {
    Longbow,
}

impl Boost {
    pub fn description(&self) -> &'static str {
        match self {
            Boost::Longbow => "Increase the range of your archers by 50%.",
        }
    }

    /// Whether this boost can only be selected once
    pub fn is_draining(&self) -> bool {
        match self {
            Boost::Longbow => false,
        }
    }

    pub fn duration(&self) -> u64 {
        match self {
            Boost::Longbow => 60,
        }
    }
}

pub fn check_boost_timer(
    mut play_audio_ev: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_settings: ResMut<Settings>,
    time: Res<Time>,
) {
    let time = scale_duration(time.delta(), game_settings.speed);
    game_settings.boost_timer.tick(time);

    if game_settings.boost_timer.is_finished() {
        play_audio_ev.write(PlayAudioMsg::new("message"));
        next_game_state.set(GameState::BoostSelection);
    }
}

pub fn update_boosts(settings: Res<Settings>, mut players: ResMut<Players>, time: Res<Time>) {
    players.me.boosts.retain_mut(|boost| {
        if boost.active {
            boost.timer.tick(scale_duration(time.delta(), settings.speed));

            if boost.timer.just_finished() {
                return false;
            }
        }

        true
    });
}

pub fn initiate_boost_message(mut initiate_boost_msg: MessageReader<InitiateBoostMsg>) {
    for msg in initiate_boost_msg.read() {
        match **msg {
            _ => (),
        }
    }
}

pub fn after_boost_check(
    server: Res<RenetServer>,
    mut trait_count: ResMut<AfterBoostCount>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if *game_state.get() == GameState::AfterBoostSelection
        && **trait_count == server.clients_id().len()
    {
        **trait_count = 0;
        next_game_state.set(GameState::Playing);
    }
}
