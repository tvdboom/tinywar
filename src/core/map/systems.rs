use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::camera::MainCamera;
use crate::core::constants::MAX_ZOOM;
use crate::core::map::map::Map;
use crate::core::map::ui::systems::UiCmp;
use crate::core::map::utils::UiScaleLens;
use crate::core::player::Players;
use crate::core::units::buildings::Building;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TilemapAnchor};
use bevy_tweening::{RepeatCount, RepeatStrategy, Tween, TweenAnim};
use std::time::Duration;

#[derive(Component)]
pub struct MapCmp;

pub fn draw_map(
    mut commands: Commands,
    mut camera_q: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    assets: Res<AssetServer>,
) {
    let (mut camera_t, mut projection) = camera_q.single_mut().unwrap();
    camera_t.translation = Vec3::new(0., 0., 1.);

    if let Projection::Orthographic(projection) = &mut *projection {
        projection.scale = MAX_ZOOM;
    }

    commands.spawn((
        TiledMap(assets.load("map/map.tmx")),
        TilemapAnchor::Center,
        Transform::from_translation(Map::POSITION),
        MapCmp,
    ));
}

pub fn setup_end_game(
    mut commands: Commands,
    building_q: Query<&Building>,
    players: Res<Players>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    assets: Local<WorldAssets>,
) {
    let status = if building_q.iter().any(|b| b.color == players.me.color && b.is_base) {
        "victory"
    } else {
        "defeat"
    };

    play_audio_msg.write(PlayAudioMsg::new(status));

    commands.spawn((
        ImageNode::new(assets.image(status)),
        TweenAnim::new(
            Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(6),
                UiScaleLens {
                    start: Vec2::ZERO,
                    end: Vec2::splat(0.6),
                },
            )
            .with_repeat(RepeatCount::Finite(2), RepeatStrategy::MirroredRepeat),
        ),
        Pickable::IGNORE,
        UiCmp,
        MapCmp,
    ));
}
