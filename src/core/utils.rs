use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};
use std::fmt::Debug;

pub type ClientId = u64;

/// Generic system that despawns all entities with a specific component
pub fn despawn<T: Component>(mut commands: Commands, query_c: Query<Entity, With<T>>) {
    for entity in &query_c {
        commands.entity(entity).try_despawn();
    }
}

/// Set cursor icon on event
pub fn cursor<T: Debug + Clone + Reflect>(
    icon: SystemCursorIcon,
) -> impl FnMut(On<Pointer<T>>, Commands, Single<Entity, With<Window>>) {
    move |_: On<Pointer<T>>, mut commands: Commands, window_e: Single<Entity, With<Window>>| {
        commands.entity(*window_e).insert(CursorIcon::from(icon));
    }
}
