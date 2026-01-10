use crate::core::audio::PlayAudioMsg;
use crate::core::constants::MAX_QUEUE_LENGTH;
use crate::core::mechanics::spawn::SpawnUnitMsg;
use crate::core::player::{Players, QueuedUnit};
use crate::core::settings::Settings;
use crate::core::units::units::UnitName;
use crate::utils::scale_duration;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::rng;
use strum::IntoEnumIterator;

#[derive(Message)]
pub struct QueueUnitMsg {
    pub id: ClientId,
    pub unit: UnitName,
}

impl QueueUnitMsg {
    pub fn new(id: ClientId, unit: UnitName) -> Self {
        Self {
            id,
            unit,
        }
    }
}

pub fn queue_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    players: Res<Players>,
    mut queue_unit_msg: MessageWriter<QueueUnitMsg>,
) {
    for unit in UnitName::iter() {
        if keyboard.just_pressed(unit.key()) {
            queue_unit_msg.write(QueueUnitMsg::new(players.me.id, unit));
        }
    }
}

pub fn queue_message(
    mut queue_unit_msg: MessageReader<QueueUnitMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut players: ResMut<Players>,
) {
    for msg in queue_unit_msg.read() {
        let player = players.get_mut(msg.id);

        if player.queue.len() < MAX_QUEUE_LENGTH {
            player.queue.push_back(QueuedUnit::new(msg.unit, msg.unit.spawn_duration()));
        } else if player.is_human() {
            play_audio_msg.write(PlayAudioMsg::new("error"));
        }
    }
}

pub fn queue_resolve(
    mut players: ResMut<Players>,
    mut queue_unit_msg: MessageWriter<QueueUnitMsg>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    settings: Res<Settings>,
    time: Res<Time>,
) {
    for player in players.iter_mut() {
        let mut spawn = None;
        if let Some(queue) = player.queue.front_mut() {
            queue.timer.tick(scale_duration(time.delta(), settings.speed));

            if queue.timer.just_finished() {
                spawn = Some(queue.unit);
            }
        } else if player.is_human() {
            queue_unit_msg.write(QueueUnitMsg::new(player.id, player.queue_default));
        } else {
            // Spawn units randomly with inverse probability to their spawning time
            let units: Vec<UnitName> = UnitName::iter().collect();

            let weights: Vec<f64> = units.iter().map(|u| 1.0 / u.spawn_duration() as f64).collect();

            let dist = WeightedIndex::new(&weights).unwrap();
            let unit = units[dist.sample(&mut rng())];

            queue_unit_msg.write(QueueUnitMsg::new(player.id, unit));
        }

        if let Some(unit) = spawn {
            spawn_unit_msg.write(SpawnUnitMsg::new(player.id, unit));
            player.queue.pop_front();
            player.queue_default = unit;
        }
    }
}
