use std::fmt::Debug;

use crate::core::assets::WorldAssets;
use bevy::prelude::*;

#[derive(Component)]
pub struct TextSize(pub f32);

/// Change the background color of an entity
pub fn recolor<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(On<Pointer<E>>, Query<&mut BackgroundColor>) {
    move |ev, mut bgcolor_q| {
        if let Ok(mut bgcolor) = bgcolor_q.get_mut(ev.entity) {
            bgcolor.0 = color;
        };
    }
}

/// Add a root UI node that covers the whole screen
pub fn add_root_node(block: bool) -> (Node, Pickable) {
    (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(105.),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        if block {
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            }
        } else {
            Pickable::IGNORE
        },
    )
}

/// Add a standard text component
pub fn add_text(
    text: impl Into<String>,
    font: &str,
    font_size: f32,
    assets: &WorldAssets,
    window: &Window,
) -> (Text, TextFont, TextSize) {
    (
        Text::new(text),
        TextFont {
            font: assets.font(font),
            font_size: font_size * window.height() / 460.,
            ..default()
        },
        TextSize(font_size),
    )
}
