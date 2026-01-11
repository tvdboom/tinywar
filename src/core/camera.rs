use crate::core::constants::{LERP_FACTOR, MAX_MAP_OFFSET, MAX_ZOOM, MIN_ZOOM, ZOOM_FACTOR};
use crate::core::map::map::Map;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};

#[derive(Component)]
pub struct MainCamera;

pub fn clamp_to_rect(pos: Vec2, view_size: Vec2, bounds: Rect) -> Vec2 {
    let min_x = bounds.min.x + view_size.x * 0.5;
    let min_y = bounds.min.y + view_size.y * 0.5;
    let max_x = bounds.max.x - view_size.x * 0.5;
    let max_y = bounds.max.y - view_size.y * 0.5;

    if min_x > max_x || min_y > max_y {
        Vec2::new((bounds.min.x + bounds.max.x) * 0.5, (bounds.min.y + bounds.max.y) * 0.5)
    } else {
        Vec2::new(pos.x.clamp(min_x, max_x), pos.y.clamp(min_y, max_y))
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off, MainCamera));
}

pub fn move_camera(
    mut commands: Commands,
    ui_q: Query<&Interaction, With<Node>>,
    camera_q: Single<
        (&Camera, &GlobalTransform, &mut Transform, &mut Projection),
        With<MainCamera>,
    >,
    mut scroll_msg: MessageReader<MouseWheel>,
    mut motion_ev: MessageReader<MouseMotion>,
    map: Res<Map>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<(Entity, &Window)>,
) {
    let (camera, global_t, mut camera_t, mut projection) = camera_q.into_inner();
    let (window_e, window) = *window;

    let Projection::Orthographic(projection) = &mut *projection else {
        panic!("Expected Orthographic projection.");
    };

    for msg in scroll_msg.read() {
        // Get cursor position in window space
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert to world space
            if let Ok(world_pos) = camera.viewport_to_world_2d(global_t, cursor_pos) {
                let scale_change = if msg.y > 0. {
                    1. / ZOOM_FACTOR
                } else {
                    ZOOM_FACTOR
                };

                let new_scale = (projection.scale * scale_change).clamp(MIN_ZOOM, MAX_ZOOM);

                // Adjust camera position to keep focus on the cursor
                let shift = (world_pos - camera_t.translation.truncate())
                    * (1. - new_scale / projection.scale);
                camera_t.translation += shift.extend(0.);

                projection.scale = new_scale;
            }
        }
    }

    // Only act if not hovering a UI element
    if !ui_q.iter().any(|i| *i != Interaction::None) {
        if mouse.pressed(MouseButton::Left) {
            commands.entity(window_e).insert(Into::<CursorIcon>::into(SystemCursorIcon::Grab));
            for msg in motion_ev.read() {
                commands
                    .entity(window_e)
                    .insert(Into::<CursorIcon>::into(SystemCursorIcon::Grabbing));
                if msg.delta.x.is_nan() || msg.delta.y.is_nan() {
                    continue;
                }
                camera_t.translation.x -= msg.delta.x * projection.scale;
                camera_t.translation.y += msg.delta.y * projection.scale;
            }
        } else {
            commands.entity(window_e).insert(Into::<CursorIcon>::into(SystemCursorIcon::Default));
        }
    }

    let mut position = camera_t.translation.truncate();

    // Compute the camera's current view size based on projection
    let view_size = projection.area.max - projection.area.min;

    // Clamp camera position within bounds
    let size = map.size().as_vec2() * Map::TILE_SIZE as f32;
    position = position.lerp(
        clamp_to_rect(
            position,
            view_size,
            Rect {
                min: Vec2::new(-size.x * 0.5, -size.y * 0.5) * MAX_MAP_OFFSET,
                max: Vec2::new(size.x * 0.5, size.y * 0.5) * MAX_MAP_OFFSET,
            },
        ),
        LERP_FACTOR,
    );

    camera_t.translation = position.extend(camera_t.translation.z);
}

pub fn move_camera_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<(&mut Transform, &Projection), With<MainCamera>>,
) {
    let (mut camera_t, projection) = camera_q.single_mut().unwrap();

    let scale = if let Projection::Orthographic(projection) = projection {
        projection.scale
    } else {
        1.0
    };

    let transform = 10. * scale;
    if keyboard.pressed(KeyCode::KeyA) {
        camera_t.translation.x -= transform;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        camera_t.translation.x += transform;
    }
    if keyboard.pressed(KeyCode::KeyW) {
        camera_t.translation.y += transform;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        camera_t.translation.y -= transform;
    }
}

pub fn reset_camera(mut camera_q: Query<(&mut Transform, &mut Projection), With<MainCamera>>) {
    let (mut camera_t, mut projection) = camera_q.single_mut().unwrap();
    camera_t.translation = Vec3::new(0., 0., 1.);

    if let Projection::Orthographic(projection) = &mut *projection {
        projection.scale = 1.;
    }
}
