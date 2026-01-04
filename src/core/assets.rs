use std::collections::HashMap;

use crate::core::settings::PlayerColor;
use crate::core::units::buildings::Building;
use crate::utils::NameFromEnum;
use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use strum::IntoEnumIterator;

#[derive(Clone)]
pub struct TextureInfo {
    pub image: Handle<Image>,
    pub atlas: TextureAtlas,
    pub last_index: usize,
}

pub struct WorldAssets {
    pub audio: HashMap<&'static str, Handle<AudioSource>>,
    pub fonts: HashMap<&'static str, Handle<Font>>,
    pub images: HashMap<&'static str, Handle<Image>>,
    pub textures: HashMap<&'static str, TextureInfo>,
}

impl WorldAssets {
    fn get_asset<'a, T: Clone>(
        &self,
        map: &'a HashMap<&str, T>,
        name: impl Into<String>,
        asset_type: &str,
    ) -> &'a T {
        let name = name.into().clone();
        map.get(name.as_str()).expect(&format!("No asset for {asset_type} {name}"))
    }

    pub fn audio(&self, name: impl Into<String>) -> Handle<AudioSource> {
        self.get_asset(&self.audio, name, "audio").clone()
    }

    pub fn font(&self, name: impl Into<String>) -> Handle<Font> {
        self.get_asset(&self.fonts, name, "font").clone()
    }

    pub fn image(&self, name: impl Into<String>) -> Handle<Image> {
        self.get_asset(&self.images, name, "image").clone()
    }

    pub fn texture(&self, name: impl Into<String>) -> TextureInfo {
        self.get_asset(&self.textures, name, "texture").clone()
    }
}

impl FromWorld for WorldAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();

        let audio = HashMap::from([]);

        let fonts = HashMap::from([
            ("bold", assets.load("fonts/FiraSans-Bold.ttf")),
            ("medium", assets.load("fonts/FiraMono-Medium.ttf")),
        ]);

        let mut images: HashMap<&'static str, Handle<Image>> = HashMap::from([
            // Icons
            ("mute", assets.load("images/icons/mute.png")),
            ("no-music", assets.load("images/icons/no-music.png")),
            ("sound", assets.load("images/icons/sound.png")),
            // Background
            ("bg", assets.load("images/bg/bg.png")),
            // Map
            ("tiles0", assets.load("images/map/tiles0.png")),
            ("foam", assets.load("images/map/foam.png")),
        ]);

        for color in PlayerColor::iter() {
            for building in Building::iter() {
                let name =
                    Box::leak(Box::new(format!("{}-{}", color.to_name(), building.to_name())))
                        .as_str();

                images.insert(
                    &name,
                    assets.load(&format!(
                        "images/buildings/{}/{}.png",
                        color.to_name(),
                        building.to_name()
                    )),
                );
            }
        }

        let mut texture = world.get_resource_mut::<Assets<TextureAtlasLayout>>().unwrap();

        let textures: HashMap<&'static str, TextureInfo> = HashMap::from([]);

        Self {
            audio,
            fonts,
            images,
            textures,
        }
    }
}
