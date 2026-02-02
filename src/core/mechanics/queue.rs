use crate::core::audio::PlayAudioMsg;
use crate::core::boosts::Boost;
use crate::core::constants::MAX_QUEUE_LENGTH;
use crate::core::mechanics::spawn::SpawnUnitMsg;
use crate::core::menu::systems::Host;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::network::{ClientMessage, ClientSendMsg};
use crate::core::player::{Players, QueuedUnit};
use crate::core::settings::Settings;
use crate::core::units::units::UnitName;
use crate::core::utils::ClientId;
use crate::utils::scale_duration;
use bevy::prelude::*;
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

pub fn queue_message(
    mut queue_unit_msg: MessageReader<QueueUnitMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut players: ResMut<Players>,
) {
    for msg in queue_unit_msg.read() {
        let player = players.get_by_id_mut(msg.id);

        if player.queue.len() < MAX_QUEUE_LENGTH {
            player.queue.push_back(QueuedUnit::new(msg.unit, msg.unit.spawn_duration()));
        } else if player.is_human() {
            play_audio_msg.write(PlayAudioMsg::new("error"));
        }
    }
}

pub fn queue_resolve(
    mut players: ResMut<Players>,
    host: Option<Res<Host>>,
    mut queue_unit_msg: MessageWriter<QueueUnitMsg>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    #[cfg(not(target_arch = "wasm32"))] mut client_send_msg: MessageWriter<ClientSendMsg>,
    settings: Res<Settings>,
    time: Res<Time>,
) {
    let me = players.me.id;
    for player in players.iter_mut() {
        // Each player resolves its own queue
        if player.is_human() && player.id != me {
            continue;
        }

        let queue_boost = if player.has_boost(Boost::SpawnTime) {
            1.2
        } else {
            1.0
        };

        let mut spawns: Vec<(usize, UnitName)> = Vec::with_capacity(2);

        let max_slots = if player.has_boost(Boost::DoubleQueue) {
            2
        } else {
            1
        };

        for i in 0..max_slots {
            if let Some(queue) = player.queue.get_mut(i) {
                queue.timer.tick(scale_duration(time.delta(), settings.speed * queue_boost));

                if queue.timer.just_finished() {
                    spawns.push((i, queue.unit));
                }
            } else if player.is_human() {
                queue_unit_msg.write(QueueUnitMsg::new(player.id, player.queue_default));
            } else {
                // Spawn units randomly with inverse probability to their spawning time
                let units: Vec<UnitName> = UnitName::iter().collect();
                let weights: Vec<f64> =
                    units.iter().map(|u| 1.0 / u.spawn_duration() as f64).collect();

                let dist = WeightedIndex::new(&weights).unwrap();
                let unit = units[dist.sample(&mut rng())];

                queue_unit_msg.write(QueueUnitMsg::new(player.id, unit));
            }
        }

        for (i, unit) in spawns.iter().rev() {
            if host.is_some() {
                spawn_unit_msg.write(SpawnUnitMsg::new(player.color, *unit));
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                client_send_msg.write(ClientSendMsg::new(ClientMessage::SpawnUnit(*unit)));
            }

            player.queue.remove(*i);
            player.queue_default = *unit;
        }
    }
}
