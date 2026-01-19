use crate::core::constants::{ARROW_SPEED, CAPPED_DELTA_SECS_SPEED, RADIUS, UNITS_Z};
use crate::core::map::map::Map;
use crate::core::mechanics::combat::{ApplyDamageMsg, Arrow};
use crate::core::mechanics::spawn::DespawnMsg;
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::units::units::{Action, Unit, UnitName};
use crate::utils::scale_duration;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use std::collections::{HashMap, HashSet, VecDeque};

/// Get all tiles at <= `distance` from `pos`
fn get_tiles_at_distance(pos: &TilePos, distance: u32) -> Vec<TilePos> {
    let mut visited: HashSet<TilePos> = HashSet::new();
    let mut queue: VecDeque<(TilePos, u32)> = VecDeque::new();
    let mut result = Vec::new();

    visited.insert(*pos);
    queue.push_back((*pos, 0));

    while let Some((current, dist)) = queue.pop_front() {
        if dist == distance {
            result.push(current);
            continue;
        }

        if dist > distance {
            continue;
        }

        for neighbor in Map::get_neighbors(&current, false) {
            if visited.insert(neighbor) {
                queue.push_back((neighbor, dist + 1));
            }
        }
    }

    result
}

/// Return the next tile to walk to, which is the one following the closest tile
fn next_tile_on_path<'a>(pos: &Vec3, path: &'a Vec<TilePos>) -> &'a TilePos {
    path.iter()
        .enumerate()
        .min_by_key(|(_, tile)| {
            let tile_pos = Map::tile_to_world(tile);
            let dx = pos.x - tile_pos.x;
            let dy = pos.y - tile_pos.y;
            (dx * dx + dy * dy) as i32
        })
        .map(|(idx, _)| path.get(idx + 1).unwrap_or(path.last().unwrap()))
        .unwrap()
}

fn move_unit(
    unit_e: Entity,
    unit: &mut Unit,
    unit_t: &mut Transform,
    unit_s: &mut Sprite,
    positions: &HashMap<TilePos, Vec<(Entity, Vec3, Unit)>>,
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

    let target_tile = next_tile_on_path(&unit_t.translation, &path);
    let target_pos = Map::tile_to_world(&target_tile).extend(unit_t.translation.z);

    // Check units in this tile + two adjacent tiles
    for tile in std::iter::once(tile).chain(get_tiles_at_distance(&tile, 2)) {
        if let Some(units) = positions.get(&tile) {
            for (other_e, other_pos, other) in units {
                let distance = unit_t.translation.distance(*other_pos);

                // Skip if self or too far to interact
                if unit_e == *other_e || distance > unit.name.range() * RADIUS {
                    continue;
                }

                // Possible interactions are:
                // - Priest with unhealthy ally -> heal
                // - Non-priest with enemy -> attack
                unit.action = match (unit.name, unit.color == other.color) {
                    (UnitName::Priest, true) if other.health < other.name.health() => {
                        Action::Heal(*other_e)
                    },
                    (u, false) if u == UnitName::Archer => Action::Attack(*other_e),
                    (u, false) if u != UnitName::Priest && distance <= 2. * RADIUS => {
                        Action::Attack(*other_e)
                    },
                    _ => continue,
                };

                return;
            }
        }
    }

    let mut next_pos = unit_t.translation
        + (target_pos - unit_t.translation).normalize()
            * unit.name.speed()
            * settings.speed
            * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED);
    let next_tile = Map::world_to_tile(&next_pos);

    if tile == next_tile || Map::is_walkable(&next_tile) {
        // Check if the tile below is walkable. If not, restrict movement to the top part
        if !Map::is_walkable(&TilePos::new(next_tile.x, next_tile.y + 1)) {
            let bottom_limit = Map::tile_to_world(&next_tile).y - Map::TILE_SIZE as f32 * 0.25;
            if next_pos.y < bottom_limit {
                next_pos.y = bottom_limit;
            }
        }

        // Change the direction the unit is facing
        unit_s.flip_x = next_pos.x < unit_t.translation.x;

        unit_t.translation = next_pos;
        unit.action = Action::Run;
    } else {
        unit.action = Action::Idle;
    }
}

fn move_arrow(
    arrow_e: Entity,
    arrow: &mut Arrow,
    arrow_t: &mut Transform,
    arrow_s: &mut Sprite,
    apply_damage_msg: &mut MessageWriter<ApplyDamageMsg>,
    despawn_msg: &mut MessageWriter<DespawnMsg>,
    positions: &HashMap<TilePos, Vec<(Entity, Vec3, Unit)>>,
    settings: &Settings,
    images: &Assets<Image>,
    time: &Time,
) {
    // Resolve arrow hitting an enemy
    let speed = ARROW_SPEED * settings.speed * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED);
    arrow.traveled += speed;

    // Calculate progress (0.0 to 1.0)
    let progress = (arrow.traveled / arrow.total_distance).min(1.0);

    // Check if arrow reached destination
    if progress >= 1.0 {
        if let Some(image) = images.get(&arrow_s.image) {
            // Hide the point to look as if the arrow is stuck in the ground
            arrow_s.rect = Some(Rect {
                min: Vec2::ZERO,
                max: Vec2::new(image.width() as f32 * 0.65, image.height() as f32),
            });

            // Place ground arrows behind units
            arrow_t.translation.z = UNITS_Z - 0.1;
        }

        arrow.despawn_timer.tick(scale_duration(time.delta(), settings.speed));
        if arrow.despawn_timer.just_finished() {
            despawn_msg.write(DespawnMsg(arrow_e));
        }
        return;
    }

    // Linear interpolation between start and destination
    let horizontal_pos = arrow.start.lerp(arrow.destination, progress);

    // Parabolic arc: height = progress * (1 - progress) * 4 * arc_factor
    // Arc height is proportional to distance (20% of total distance as max height)
    let arc_height = progress * (1. - progress) * 4. * arrow.total_distance * 0.2;

    // Set new position with arc
    arrow_t.translation = horizontal_pos + Vec3::Y * arc_height;

    // Check if the arrow hit someone (in this or adjacent tiles)
    let tile = Map::world_to_tile(&arrow_t.translation);
    for tile in std::iter::once(tile).chain(Map::get_neighbors(&tile, false)) {
        if let Some(units) = positions.get(&tile) {
            for (other_e, other_pos, other_unit) in units {
                if other_unit.color != arrow.color
                    && arrow_t.translation.distance(*other_pos) < RADIUS * 0.4
                {
                    apply_damage_msg.write(ApplyDamageMsg::new(*other_e, arrow.damage));
                    despawn_msg.write(DespawnMsg(arrow_e));
                    return;
                }
            }
        }
    }

    // Calculate velocity direction for rotation (take a small step ahead to determine angle)
    let next_progress = ((arrow.traveled + 1.) / arrow.total_distance).min(1.);
    let next_horizontal = arrow.start.lerp(arrow.destination, next_progress);
    let next_arc_height = next_progress * (1. - next_progress) * 4.0 * arrow.total_distance * 0.2;
    let next_pos = next_horizontal + Vec3::Y * next_arc_height;

    let velocity = next_pos - arrow_t.translation;
    if velocity.length() > 0.01 {
        let angle = velocity.y.atan2(velocity.x);
        arrow_t.rotation = Quat::from_rotation_z(angle);
    }
}

pub fn apply_movement(
    mut unit_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Unit)>,
    mut arrow_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Arrow), Without<Unit>>,
    mut apply_damage_msg: MessageWriter<ApplyDamageMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    settings: Res<Settings>,
    map: Res<Map>,
    players: Res<Players>,
    images: Res<Assets<Image>>,
    time: Res<Time>,
) {
    // Build spatial hashmap: tile -> positions + unit
    let positions: HashMap<TilePos, Vec<(Entity, Vec3, Unit)>> =
        unit_q.iter().fold(HashMap::new(), |mut acc, (e, t, _, u)| {
            let tile = Map::world_to_tile(&t.translation);
            acc.entry(tile).or_default().push((e, t.translation, u.clone()));
            acc
        });

    // Move units
    for (unit_e, mut unit_t, mut unit_s, mut unit) in
        unit_q.iter_mut().filter(|(_, _, _, u)| matches!(u.action, Action::Idle | Action::Run))
    {
        move_unit(
            unit_e,
            &mut unit,
            &mut unit_t,
            &mut unit_s,
            &positions,
            &settings,
            &map,
            &players,
            &time,
        );
    }

    // Move arrows
    for (arrow_e, mut arrow_t, mut arrow_s, mut arrow) in &mut arrow_q {
        move_arrow(
            arrow_e,
            &mut arrow,
            &mut arrow_t,
            &mut arrow_s,
            &mut apply_damage_msg,
            &mut despawn_msg,
            &positions,
            &settings,
            &images,
            &time,
        )
    }
}
