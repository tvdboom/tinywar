#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod utils;

use bevy::asset::AssetMetaCheck;
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution};
use bevy::winit::WINIT_WINDOWS;
use bevy_ecs_tiled::prelude::TiledPlugin;
use bevy_kira_audio::AudioPlugin;
#[cfg(not(target_arch = "wasm32"))]
use bevy_renet::{
    netcode::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};
use bevy_tweening::TweeningPlugin;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::panic;
use std::sync::Mutex;

use crate::core::GamePlugin;
use crate::utils::NameFromEnum;

pub const TITLE: &str = "TinyWar";

#[allow(dead_code)]
static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

fn main() {
    #[cfg(not(debug_assertions))]
    init_panic_logger();

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.into(),
                    mode: WindowMode::Windowed,
                    position: WindowPosition::Automatic,
                    resolution: WindowResolution::new(1600, 900),

                    // Tells Wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,

                    // Don't override browser's default behavior (ctrl+5, etc...)
                    prevent_default_event_handling: true,

                    ..default()
                }),
                ..default()
            })
            // Disable loading of asset meta since that fails on itch.io
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins((AudioPlugin, TweeningPlugin, TiledPlugin::default()))
    .add_plugins(GamePlugin);

    #[cfg(target_os = "windows")]
    app.add_systems(Startup, set_window_icon);

    #[cfg(not(target_arch = "wasm32"))]
    app
        // Networking: systems are disabled until server/client resource is added
        .add_plugins((
            RenetServerPlugin,
            NetcodeServerPlugin,
            RenetClientPlugin,
            NetcodeClientPlugin,
        ));

    app.run();
}

#[allow(dead_code)]
fn init_panic_logger() {
    panic::set_hook(Box::new(|info| {
        let mut guard = LOG_FILE.lock().unwrap();

        if guard.is_none() {
            *guard = OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("{}-logs.txt", TITLE.to_lowername()))
                .ok();
        }

        if let Some(file) = guard.as_mut() {
            let _ = writeln!(file, "=== PANIC ===");
            let _ = writeln!(file, "{}", info);
            let _ = writeln!(file);
        }
    }));
}

#[cfg(target_os = "windows")]
fn set_window_icon(_: NonSendMarker) {
    use winit::window::Icon;

    let image = image::open("assets/images/icons/favicon.png").unwrap().into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    let icon = Icon::from_rgba(rgba, width, height).unwrap();

    WINIT_WINDOWS.with_borrow(|windows| {
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    });
}
