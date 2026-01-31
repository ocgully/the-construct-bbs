//! World management for Ultimo
//!
//! Handles world state, zones, and multiplayer visibility.

use super::data::{get_zone, TerrainType, Zone, ZoneType};
use super::state::{Position, VisiblePlayer};

/// Generate terrain for a zone
/// Returns a 2D grid of terrain types
pub fn generate_zone_terrain(zone: &Zone) -> Vec<Vec<TerrainType>> {
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    // Use zone key as seed for consistent generation
    let seed: u64 = zone.key.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64) * 31);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut terrain = vec![vec![TerrainType::Grass; zone.width as usize]; zone.height as usize];

    match zone.zone_type {
        ZoneType::Town => {
            // Towns have roads and buildings
            // Horizontal main road
            let road_y = zone.height as usize / 2;
            for x in 0..zone.width as usize {
                terrain[road_y][x] = TerrainType::Road;
            }
            // Vertical cross road
            let road_x = zone.width as usize / 2;
            for row in terrain.iter_mut() {
                row[road_x] = TerrainType::Road;
            }
            // Scatter buildings
            for _ in 0..20 {
                let bx = rng.gen_range(2..zone.width as usize - 2);
                let by = rng.gen_range(2..zone.height as usize - 2);
                if terrain[by][bx] == TerrainType::Grass {
                    terrain[by][bx] = TerrainType::Building;
                }
            }
        }
        ZoneType::Wilderness => {
            // Wilderness has trees, water, etc.
            // Add some forests
            for _ in 0..(zone.width * zone.height / 10) {
                let x = rng.gen_range(0..zone.width as usize);
                let y = rng.gen_range(0..zone.height as usize);
                terrain[y][x] = TerrainType::Forest;
            }
            // Add some water features
            for _ in 0..(zone.width * zone.height / 50) {
                let x = rng.gen_range(0..zone.width as usize);
                let y = rng.gen_range(0..zone.height as usize);
                terrain[y][x] = TerrainType::Water;
            }
            // Add some mountains
            for _ in 0..(zone.width * zone.height / 30) {
                let x = rng.gen_range(0..zone.width as usize);
                let y = rng.gen_range(0..zone.height as usize);
                terrain[y][x] = TerrainType::Mountain;
            }
        }
        ZoneType::Dungeon => {
            // Dungeons are mostly stone with corridors
            for row in terrain.iter_mut() {
                for cell in row.iter_mut() {
                    *cell = TerrainType::Wall;
                }
            }
            // Carve corridors
            let mut x = zone.width as usize / 2;
            let mut y = zone.height as usize / 2;
            for _ in 0..500 {
                terrain[y][x] = TerrainType::Stone;
                // Random walk
                match rng.gen_range(0..4) {
                    0 if x > 1 => x -= 1,
                    1 if x < zone.width as usize - 2 => x += 1,
                    2 if y > 1 => y -= 1,
                    3 if y < zone.height as usize - 2 => y += 1,
                    _ => {}
                }
            }
        }
        ZoneType::PvP => {
            // Arena is mostly open sand
            for row in terrain.iter_mut() {
                for cell in row.iter_mut() {
                    *cell = TerrainType::Sand;
                }
            }
            // Walls around edges
            for x in 0..zone.width as usize {
                terrain[0][x] = TerrainType::Wall;
                terrain[zone.height as usize - 1][x] = TerrainType::Wall;
            }
            for y in 0..zone.height as usize {
                terrain[y][0] = TerrainType::Wall;
                terrain[y][zone.width as usize - 1] = TerrainType::Wall;
            }
        }
        ZoneType::Housing => {
            // Housing district is mostly grass with roads
            for y in 0..zone.height as usize {
                if y % 10 == 0 {
                    for x in 0..zone.width as usize {
                        terrain[y][x] = TerrainType::Road;
                    }
                }
            }
            for x in 0..zone.width as usize {
                if x % 10 == 0 {
                    for row in terrain.iter_mut() {
                        row[x] = TerrainType::Road;
                    }
                }
            }
        }
    }

    // Mark exits
    for (_, exit_x, exit_y) in zone.exits {
        if *exit_x >= 0 && (*exit_x as usize) < zone.width as usize
            && *exit_y >= 0 && (*exit_y as usize) < zone.height as usize
        {
            terrain[*exit_y as usize][*exit_x as usize] = TerrainType::Door;
        }
    }

    terrain
}

/// Get a view of terrain around a position
/// Returns a viewport of specified size centered on position
pub fn get_terrain_view(
    zone: &Zone,
    center: &Position,
    view_width: u32,
    view_height: u32,
) -> Vec<Vec<TerrainType>> {
    let terrain = generate_zone_terrain(zone);

    let half_w = view_width as i32 / 2;
    let half_h = view_height as i32 / 2;

    let mut view = vec![vec![TerrainType::Wall; view_width as usize]; view_height as usize];

    for vy in 0..view_height as i32 {
        for vx in 0..view_width as i32 {
            let world_x = center.x - half_w + vx;
            let world_y = center.y - half_h + vy;

            if world_x >= 0 && world_x < zone.width as i32
                && world_y >= 0 && world_y < zone.height as i32
            {
                view[vy as usize][vx as usize] = terrain[world_y as usize][world_x as usize];
            }
        }
    }

    view
}

/// Check if a position is passable
pub fn is_passable(zone: &Zone, x: i32, y: i32) -> bool {
    if x < 0 || x >= zone.width as i32 || y < 0 || y >= zone.height as i32 {
        return false;
    }

    let terrain = generate_zone_terrain(zone);
    terrain[y as usize][x as usize].passable()
}

/// Calculate distance between two positions
pub fn distance(p1: &Position, p2: &Position) -> f32 {
    if p1.zone != p2.zone {
        return f32::MAX;
    }
    let dx = (p1.x - p2.x) as f32;
    let dy = (p1.y - p2.y) as f32;
    (dx * dx + dy * dy).sqrt()
}

/// Get nearby players in the same zone
pub fn get_nearby_players(
    current_player: &Position,
    all_players: &[(i64, String, Position, u32)], // (user_id, name, position, level)
    range: f32,
) -> Vec<VisiblePlayer> {
    all_players
        .iter()
        .filter(|(_, _, pos, _)| {
            pos.zone == current_player.zone && distance(current_player, pos) <= range
        })
        .map(|(_, name, pos, level)| VisiblePlayer {
            name: name.clone(),
            level: *level,
            x: pos.x,
            y: pos.y,
            guild: None,
        })
        .collect()
}

/// Get the danger level of a zone for a character
pub fn zone_danger_level(zone: &Zone, player_level: u32) -> &'static str {
    let level_diff = zone.min_level as i32 - player_level as i32;

    if level_diff <= -10 {
        "Trivial"
    } else if level_diff <= -5 {
        "Easy"
    } else if level_diff <= 0 {
        "Normal"
    } else if level_diff <= 5 {
        "Dangerous"
    } else if level_diff <= 10 {
        "Very Dangerous"
    } else {
        "DEADLY"
    }
}

/// Get all zones connected to a given zone
pub fn get_connected_zones(zone_key: &str) -> Vec<&'static Zone> {
    if let Some(zone) = get_zone(zone_key) {
        zone.exits
            .iter()
            .filter_map(|(exit_key, _, _)| get_zone(exit_key))
            .collect()
    } else {
        Vec::new()
    }
}

/// Find path between two positions (simplified A* for small areas)
#[allow(dead_code)]
pub fn find_path(zone: &Zone, start: &Position, end: &Position) -> Option<Vec<(i32, i32)>> {
    if start.zone != end.zone || start.zone != zone.key {
        return None;
    }

    // Simple BFS for pathfinding
    use std::collections::{HashSet, VecDeque};

    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut queue: VecDeque<(i32, i32, Vec<(i32, i32)>)> = VecDeque::new();

    queue.push_back((start.x, start.y, vec![]));
    visited.insert((start.x, start.y));

    let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    while let Some((x, y, path)) = queue.pop_front() {
        if x == end.x && y == end.y {
            return Some(path);
        }

        for (dx, dy) in &directions {
            let nx = x + dx;
            let ny = y + dy;

            if !visited.contains(&(nx, ny)) && is_passable(zone, nx, ny) {
                visited.insert((nx, ny));
                let mut new_path = path.clone();
                new_path.push((nx, ny));
                queue.push_back((nx, ny, new_path));
            }
        }

        // Limit search to prevent infinite loops
        if visited.len() > 1000 {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_terrain() {
        let zone = get_zone("britain").unwrap();
        let terrain = generate_zone_terrain(zone);

        assert_eq!(terrain.len(), zone.height as usize);
        assert_eq!(terrain[0].len(), zone.width as usize);
    }

    #[test]
    fn test_terrain_view() {
        let zone = get_zone("britain").unwrap();
        let pos = Position::new("britain", 30, 20);

        let view = get_terrain_view(zone, &pos, 21, 11);

        assert_eq!(view.len(), 11);
        assert_eq!(view[0].len(), 21);
    }

    #[test]
    fn test_distance() {
        let p1 = Position::new("britain", 0, 0);
        let p2 = Position::new("britain", 3, 4);
        let p3 = Position::new("other_zone", 3, 4);

        assert_eq!(distance(&p1, &p2), 5.0);
        assert_eq!(distance(&p1, &p3), f32::MAX);
    }

    #[test]
    fn test_zone_danger() {
        let zone = get_zone("dungeon_despise_deep").unwrap(); // min_level 15

        assert_eq!(zone_danger_level(zone, 20), "Easy");
        assert_eq!(zone_danger_level(zone, 15), "Normal");
        assert_eq!(zone_danger_level(zone, 10), "Dangerous");
        assert_eq!(zone_danger_level(zone, 1), "DEADLY");
    }

    #[test]
    fn test_connected_zones() {
        let connected = get_connected_zones("britain");
        assert!(!connected.is_empty());
        assert!(connected.iter().any(|z| z.key == "britain_outskirts"));
    }
}
