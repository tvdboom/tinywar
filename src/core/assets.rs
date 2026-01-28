use crate::core::settings::PlayerColor;
use crate::core::units::buildings::BuildingName;
use crate::core::units::units::{ActionKind, UnitName};
use crate::utils::NameFromEnum;
use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;
use std::path::Path;
use strum::IntoEnumIterator;

#[derive(Clone)]
pub struct TextureInfo {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Clone)]
pub struct AtlasInfo {
    pub image: Handle<Image>,
    pub atlas: TextureAtlas,
    pub last_index: usize,
}

pub struct WorldAssets {
    pub audio: HashMap<&'static str, Handle<AudioSource>>,
    pub fonts: HashMap<&'static str, Handle<Font>>,
    pub images: HashMap<&'static str, Handle<Image>>,
    pub textures: HashMap<&'static str, TextureInfo>,
    pub atlas: HashMap<&'static str, AtlasInfo>,
}

impl WorldAssets {
    fn get_asset<'a, T: Clone>(
        &self,
        map: &'a HashMap<&str, T>,
        name: impl Into<String>,
        asset_type: &str,
    ) -> &'a T {
        let name = name.into().clone();
        map.get(name.as_str()).unwrap_or_else(|| panic!("No asset for {asset_type} {name}."))
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

    pub fn atlas(&self, name: impl Into<String>) -> AtlasInfo {
        self.get_asset(&self.atlas, name, "atlas").clone()
    }
}

impl FromWorld for WorldAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();

        let audio = HashMap::from([
            ("music", assets.load("audio/music.ogg")),
            ("message", assets.load("audio/message.ogg")),
            ("button", assets.load("audio/button.ogg")),
            ("click", assets.load("audio/click.ogg")),
            ("error", assets.load("audio/error.ogg")),
            ("defeat", assets.load("audio/defeat.ogg")),
            ("victory", assets.load("audio/victory.ogg")),
            ("explosion", assets.load("audio/explosion.ogg")),
        ]);

        let fonts = HashMap::from([
            ("bold", assets.load("fonts/FiraSans-Bold.ttf")),
            ("medium", assets.load("fonts/FiraMono-Medium.ttf")),
        ]);

        let mut images: HashMap<&'static str, Handle<Image>> = HashMap::from([
            // Icons
            ("mute", assets.load("images/icons/mute.png")),
            ("sound", assets.load("images/icons/sound.png")),
            ("music", assets.load("images/icons/music.png")),
            ("any arrow", assets.load("images/icons/any arrow.png")),
            ("top arrow", assets.load("images/icons/top arrow.png")),
            ("top-mid arrow", assets.load("images/icons/top-mid arrow.png")),
            ("mid arrow", assets.load("images/icons/mid arrow.png")),
            // Background
            ("bg", assets.load("images/bg/bg.png")),
            ("victory", assets.load("images/bg/victory.png")),
            ("defeat", assets.load("images/bg/defeat.png")),
            // Ui
            ("banner", assets.load("images/ui/banner.png")),
            ("swords1", assets.load("images/ui/swords1.png")),
            ("swords2", assets.load("images/ui/swords2.png")),
            ("swords3", assets.load("images/ui/swords3.png")),
            ("small ribbons", assets.load("images/ui/small ribbons.png")),
            ("large ribbons", assets.load("images/ui/large ribbons.png")),
            // Units
            ("arrow", assets.load("images/units/arrow.png")),
            // Effects
            ("heal", assets.load("images/effects/heal.png")),
            ("explosion1", assets.load("images/effects/explosion1.png")),
            ("explosion2", assets.load("images/effects/explosion2.png")),
            ("fire1", assets.load("images/effects/fire1.png")),
            ("fire2", assets.load("images/effects/fire2.png")),
            ("fire3", assets.load("images/effects/fire3.png")),
            // Boosts
            ("boost", assets.load("images/boosts/boost.png")),
            ("selected boost", assets.load("images/boosts/selected boost.png")),
            ("active boost", assets.load("images/boosts/active boost.png")),
            ("longbow", assets.load("images/boosts/longbow.png")),
        ]);

        let mut atlas: HashMap<&'static str, AtlasInfo> = HashMap::new();

        let mut actions = vec![];
        for color in PlayerColor::iter() {
            for building in BuildingName::iter() {
                let name =
                    Box::leak(Box::new(format!("{}-{}", color.to_name(), building.to_name())))
                        .as_str();

                images.insert(
                    name,
                    assets.load(format!(
                        "images/buildings/{}/{}.png",
                        color.to_name(),
                        building.to_name()
                    )),
                );
            }

            for unit in UnitName::iter() {
                let name =
                    Box::leak(Box::new(format!("{}-{}", color.to_name(), unit.to_name()))).as_str();

                images.insert(
                    name,
                    assets.load(format!("images/units/{}/{}.png", color.to_name(), unit.to_name())),
                );

                for action in ActionKind::iter() {
                    let path = format!(
                        "assets/images/units/{}/{}_{}.png",
                        color.to_name(),
                        unit.to_name(),
                        action.to_name()
                    );

                    if Path::new(&path).exists() {
                        let name = Box::leak(Box::new(format!(
                            "{}-{}-{}",
                            color.to_name(),
                            unit.to_name(),
                            action.to_name()
                        )))
                        .as_str();

                        images.insert(name, assets.load(path.replace("assets/", "")));

                        actions.push((name, action, unit));
                    }
                }
            }
        }

        let mut texture = world.get_resource_mut::<Assets<TextureAtlasLayout>>().unwrap();
        let swords1 = TextureAtlasLayout::from_grid(UVec2::new(105, 128), 1, 5, None, None);
        let small_ribbons = TextureAtlasLayout::from_grid(UVec2::new(64, 64), 5, 10, None, None);
        let large_ribbons = TextureAtlasLayout::from_grid(UVec2::new(64, 128), 7, 5, None, None);

        let textures = HashMap::from([
            (
                "swords1",
                TextureInfo {
                    image: images["swords1"].clone(),
                    layout: texture.add(swords1),
                },
            ),
            (
                "small ribbons",
                TextureInfo {
                    image: images["small ribbons"].clone(),
                    layout: texture.add(small_ribbons),
                },
            ),
            (
                "large ribbons",
                TextureInfo {
                    image: images["large ribbons"].clone(),
                    layout: texture.add(large_ribbons),
                },
            ),
        ]);

        // Add atlas separately since it requires mutable access to world
        for (name, action, unit) in actions {
            let layout = TextureAtlasLayout::from_grid(
                UVec2::splat(unit.size() as u32),
                unit.frames(action.to_action()),
                1,
                None,
                None,
            );

            atlas.insert(
                name,
                AtlasInfo {
                    image: images[name].clone(),
                    atlas: TextureAtlas {
                        layout: texture.add(layout),
                        index: 0,
                    },
                    last_index: unit.frames(action.to_action()) as usize,
                },
            );
        }

        let heal = TextureAtlasLayout::from_grid(UVec2::splat(192), 11, 1, None, None);
        let explosion1 = TextureAtlasLayout::from_grid(UVec2::splat(192), 8, 1, None, None);
        let explosion2 = TextureAtlasLayout::from_grid(UVec2::splat(192), 10, 1, None, None);
        let fire1 = TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 1, None, None);
        let fire2 = TextureAtlasLayout::from_grid(UVec2::splat(64), 10, 1, None, None);
        let fire3 = TextureAtlasLayout::from_grid(UVec2::splat(64), 12, 1, None, None);
        atlas.extend([
            (
                "heal",
                AtlasInfo {
                    image: images["heal"].clone(),
                    atlas: TextureAtlas {
                        layout: texture.add(heal),
                        index: 0,
                    },
                    last_index: 11,
                },
            ),
            (
                "explosion1",
                AtlasInfo {
                    image: images["explosion1"].clone(),
                    atlas: TextureAtlas {
                        layout: texture.add(explosion1),
                        index: 0,
                    },
                    last_index: 7,
                },
            ),
            (
                "explosion2",
                AtlasInfo {
                    image: images["explosion2"].clone(),
                    atlas: TextureAtlas {
                        layout: texture.add(explosion2),
                        index: 0,
                    },
                    last_index: 9,
                },
            ),
            (
                "fire1",
                AtlasInfo {
                    image: images["fire1"].clone(),
                    atlas: TextureAtlas {
                        layout: texture.add(fire1),
                        index: 0,
                    },
                    last_index: 7,
                },
            ),
            (
                "fire2",
                AtlasInfo {
                    image: images["fire2"].clone(),
                    atlas: TextureAtlas {
                        layout: texture.add(fire2),
                        index: 0,
                    },
                    last_index: 9,
                },
            ),
            (
                "fire3",
                AtlasInfo {
                    image: images["fire3"].clone(),
                    atlas: TextureAtlas {
                        layout: texture.add(fire3),
                        index: 0,
                    },
                    last_index: 11,
                },
            ),
        ]);

        Self {
            audio,
            fonts,
            images,
            textures,
            atlas,
        }
    }
}
