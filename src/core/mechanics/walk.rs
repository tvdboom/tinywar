use crate::core::constants::{CAPPED_DELTA_SECS_SPEED, UNIT_DEFAULT_SIZE, UNIT_SCALE};
use crate::core::map::map::Map;
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::units::units::{Action, Unit};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use std::collections::HashMap;

fn closest_tile_on_path<'a>(current_tile: &TilePos, path: &'a Vec<TilePos>) -> &'a TilePos {
    let mut nearest_idx = 0;
    let mut min_dist = f32::MAX;

    for (i, tile) in path.iter().enumerate() {
        let dx = current_tile.x as i32 - tile.x as i32;
        let dy = current_tile.y as i32 - tile.y as i32;
        let dist_sq = (dx * dx + dy * dy) as f32;

        if dist_sq < min_dist {
            min_dist = dist_sq;
            nearest_idx = i;
        }
    }

    // Return the next tile after the nearest one, or the last tile if at the end
    path.get(nearest_idx + 1).unwrap_or(path.last().unwrap())
}

pub fn run(
    unit: &mut Unit,
    unit_t: &mut Transform,
    unit_s: &mut Sprite,
    positions: &HashMap<TilePos, Vec<(Vec3, Unit)>>,
    settings: &Settings,
    map: &Map,
    players: &Players,
    time: &Time,
) {
    let tile = Map::world_to_tile(&unit_t.translation);
    let mut path = map.path(&unit.path);

    // Reverse paths for the enemy
    if players.me.color != unit.color {
        path.reverse();
    }

    if tile == *path.last().unwrap() {
        unit.action = Action::Idle;
        return;
    }

    let target_tile = closest_tile_on_path(&tile, &path);

    // Calculate the vector to the next location
    let target_pos = Map::tile_to_world(&target_tile).extend(unit_t.translation.z);
    let mut direction = target_pos - unit_t.translation;

    // Flip on direction before avoidance to avoid rapid, intermittent flipping
    unit_s.flip_x = direction.x < 0.;

    // Check units in this tile + adjacent tiles and apply avoidance factor to the direction
    let mut avoidance = Vec3::ZERO;
    for tile in std::iter::once(tile).chain(Map::get_neighbors(&tile)) {
        if let Some(units) = positions.get(&tile) {
            for (other_pos, _) in units {
                let delta = unit_t.translation - other_pos;
                let dist = delta.length();
                let radius = UNIT_DEFAULT_SIZE * 0.5 * UNIT_SCALE;
                if dist > 0. && dist < radius {
                    avoidance += delta.normalize() * (radius - dist);
                }
            }
        }
    }

    if avoidance != Vec3::ZERO {
        direction += 3. * avoidance;
    }

    let mut next_pos = unit_t.translation
        + direction.normalize()
            * unit.name.speed()
            * settings.speed
            * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED);
    let next_tile = Map::world_to_tile(&next_pos);

    if tile == next_tile || Map::is_walkable(&next_tile) {
        // Check if the tile below is walkable. If not, restrict movement to the top part
        if !Map::is_walkable(&TilePos::new(next_tile.x, next_tile.y - 1)) {
            let bottom_limit = Map::tile_to_world(&next_tile).y - Map::TILE_SIZE as f32 * 0.5; fix!!
            if next_pos.y < bottom_limit {
                next_pos.y = bottom_limit;
            }
        }

        unit_t.translation = next_pos;
        unit.action = Action::Run;
    } else {
        unit.action = Action::Idle;
    }
}

pub fn move_units(
    mut unit_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Unit)>,
    settings: Res<Settings>,
    map: Res<Map>,
    players: Res<Players>,
    time: Res<Time>,
) {
    // Build spatial hashmap: tile -> positions + radii
    let positions: HashMap<TilePos, Vec<(Vec3, Unit)>> =
        unit_q.iter().fold(HashMap::new(), |mut acc, (_, t, _, u)| {
            let tile = Map::world_to_tile(&t.translation);
            acc.entry(tile).or_default().push((t.translation, u.clone()));
            acc
        });

    for (_, mut unit_t, mut unit_s, mut unit) in &mut unit_q {
        run(&mut unit, &mut unit_t, &mut unit_s, &positions, &settings, &map, &players, &time);
    }
}
