use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::{BUILDING_SCALE, EXPLOSION_Z, FRAME_RATE};
use crate::core::map::systems::MapCmp;
use crate::core::map::utils::SpriteFrameLens;
use crate::core::menu::systems::Host;
use crate::core::units::buildings::Building;
use bevy::prelude::*;
use bevy_tweening::{CycleCompletedEvent, Delay, Tween, TweenAnim};
use rand::{rng, Rng};
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use {
    crate::core::multiplayer::EntityMap,
    crate::core::network::{ServerMessage, ServerSendMsg},
};

#[derive(Component)]
pub struct ExplosionCmp;

#[derive(Message)]
pub struct ExplosionMsg(pub Entity);

pub fn explosion_message(
    mut commands: Commands,
    building_q: Query<(&Transform, &Building)>,
    host: Option<Res<Host>>,
    #[cfg(not(target_arch = "wasm32"))] entity_map: Res<EntityMap>,
    #[cfg(not(target_arch = "wasm32"))] mut server_send_msg: MessageWriter<ServerSendMsg>,
    mut explosion_msg: MessageReader<ExplosionMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    assets: Res<WorldAssets>,
) {
    for ExplosionMsg(entity) in explosion_msg.read() {
        #[cfg(not(target_arch = "wasm32"))]
        let entity = if host.is_some() {
            server_send_msg.write(ServerSendMsg::new(ServerMessage::Explosion(*entity), None));
            entity
        } else {
            entity_map.get_by_left(entity).unwrap_or(entity)
        };

        if let Ok((building_t, building)) = building_q.get(*entity) {
            let mut rng = rng();
            play_audio_msg.write(PlayAudioMsg::new("explosion"));

            let size = building.name.size() * BUILDING_SCALE;
            for _ in 0..20 {
                let atlas = assets.atlas(format!("explosion{}", rng.random_range(1..3)));

                commands.spawn((
                    Sprite {
                        image: atlas.image,
                        texture_atlas: Some(atlas.atlas),
                        ..default()
                    },
                    Transform {
                        translation: (building_t.translation.truncate()
                            + Vec2::new(
                                rng.random_range(-0.4 * size.x..0.4 * size.x),
                                rng.random_range(-0.3 * size.y..0.3 * size.y),
                            ))
                        .extend(EXPLOSION_Z),
                        scale: Vec3::splat(rng.random_range(1.0..1.5)),
                        ..default()
                    },
                    TweenAnim::new(
                        Delay::new(Duration::from_millis(rng.random_range(1..1500))).then(
                            Tween::new(
                                EaseFunction::Linear,
                                Duration::from_millis(FRAME_RATE * atlas.last_index as u64),
                                SpriteFrameLens(atlas.last_index),
                            )
                            .with_cycle_completed_event(true),
                        ),
                    ),
                    ExplosionCmp,
                    MapCmp,
                ));
            }
        }
    }
}

pub fn update_explosions(
    mut commands: Commands,
    explosion_q: Query<Entity, With<ExplosionCmp>>,
    mut cycle_completed_msg: MessageReader<CycleCompletedEvent>,
) {
    for msg in cycle_completed_msg.read() {
        if let Ok(entity) = explosion_q.get(msg.anim_entity) {
            commands.entity(entity).despawn();
        }
    }
}
