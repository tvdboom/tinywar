use bevy::color::Color;
use bevy::prelude::Vec2;

/// Menu
pub const SUBTITLE_TEXT_SIZE: f32 = 10.;
pub const TITLE_TEXT_SIZE: f32 = 15.;
pub const BUTTON_TEXT_SIZE: f32 = 20.;
pub const NORMAL_BUTTON_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON_COLOR: Color = Color::srgb_u8(59, 66, 82);
pub const PRESSED_BUTTON_COLOR: Color = Color::srgb_u8(95, 131, 175);
pub const DISABLED_BUTTON_COLOR: Color = Color::srgb(0.8, 0.5, 0.5);

/// Camera
pub const MAX_MAP_OFFSET: f32 = 1.8;
pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 1.4;
pub const ZOOM_FACTOR: f32 = 1.1;
pub const LERP_FACTOR: f32 = 0.05;

/// Map
pub const WATER_COLOR: Color = Color::srgb_u8(71, 171, 169);
pub const MAP_Z: f32 = 0.;
pub const BUILDINGS_Z: f32 = 1.;
pub const UNITS_Z: f32 = 2.;

/// Game settings
pub const MAX_QUEUE_LENGTH: usize = 10;
pub const MIN_GAME_SPEED: f32 = 0.25;
pub const MAX_GAME_SPEED: f32 = 16.;
pub const FRAME_RATE: u64 = 100;
pub const CAPPED_DELTA_SECS_SPEED: f32 = 0.05;

/// Units
pub const UNIT_DEFAULT_SIZE: f32 = 192.;
pub const RADIUS: f32 = UNIT_DEFAULT_SIZE * UNIT_SCALE * 0.5;
pub const BUILDING_SCALE: f32 = 0.7;
pub const UNIT_SCALE: f32 = 0.5;
pub const HEALTH_SIZE: Vec2 = Vec2::new(75., 15.);
pub const INNER_HEALTH_SIZE: Vec2 = Vec2::new(HEALTH_SIZE.x * 0.92, HEALTH_SIZE.y * 0.75);
pub const ARROW_SPEED: f32 = 160.;
pub const ARROW_ON_GROUND_SECS: u64 = 2;
