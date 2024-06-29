use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use bracket_lib::noise::{CellularDistanceFunction, FastNoise, NoiseType};
use bracket_lib::pathfinding::DijkstraMap;
use bracket_lib::random::RandomNumberGenerator;

use crate::map::{Map, TileType};
use crate::rect::Rect;
use crate::DEBUGGING;

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for x in room.x1 + 1..=room.x2 {
        for y in room.y1 + 1..=room.y2 {
            map.tiles[x as usize][y as usize] = TileType::Floor
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        if map.is_tile_in_bounds(x, y) {
            map.tiles[x as usize][y as usize] = TileType::Floor
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        if map.is_tile_in_bounds(x, y) {
            map.tiles[x as usize][y as usize] = TileType::Floor
        }
    }
}

pub fn find_most_distant_tile(map: &mut Map, start_idx: usize) -> (usize, usize) {
    map.populate_blocked();
    let map_starts: Vec<usize> = vec![start_idx];
    let dijkstra_map = DijkstraMap::new(map.width, map.height, &map_starts, map, 200.0);
    let mut exit_tile = (0, 0, 0.0f32);
    for (x, row) in map.tiles.iter_mut().enumerate() {
        for (y, tile) in row.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[y * map.width as usize + x];
                if distance_to_start == f32::MAX {
                    *tile = TileType::Wall;
                } else {
                    if distance_to_start > exit_tile.2 {
                        exit_tile.0 = x;
                        exit_tile.1 = y;
                        exit_tile.2 = distance_to_start;
                    }
                }
            }
        }
    }

    if DEBUGGING {
        let mut file = File::create("dijkstra_map.txt").unwrap();
        for y in 0..map.height {
            let mut vec = Vec::new();
            for x in 0..map.width {
                vec.push(dijkstra_map.map[(y * map.width) as usize + x as usize]);
            }
            file.write_all(format!("{:?}\n", vec).as_bytes()).unwrap();
        }
    }
    (exit_tile.0, exit_tile.1)
}

pub fn generate_voroni_spawn_regions(
    map: &Map,
    rng: &mut RandomNumberGenerator,
) -> HashMap<i32, Vec<(i32, i32)>> {
    let mut noise_areas: HashMap<i32, Vec<(i32, i32)>> = HashMap::new();
    let mut noise = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
    noise.set_noise_type(NoiseType::Cellular);
    noise.set_frequency(0.08);
    noise.set_cellular_distance_function(CellularDistanceFunction::Manhattan);

    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            if map.tiles[x as usize][y as usize] == TileType::Floor {
                let cell_value_f = noise.get_noise(x as f32, y as f32) * 1024.0;
                let cell_value = cell_value_f as i32;

                if noise_areas.contains_key(&cell_value) {
                    noise_areas.get_mut(&cell_value).unwrap().push((x, y));
                } else {
                    noise_areas.insert(cell_value, vec![(x, y)]);
                }
            }
        }
    }
    noise_areas
}

#[derive(PartialEq, Copy, Clone)]
pub enum Symmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

pub fn paint(map: &mut Map, mode: Symmetry, brush_size: i32, x: i32, y: i32) {
    match mode {
        Symmetry::None => apply_paint(map, brush_size, x, y),
        Symmetry::Horizontal => {
            let center_x = map.width / 2;
            if x == center_x {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_x = i32::abs(center_x - x);
                apply_paint(map, brush_size, center_x + dist_x, y);
                apply_paint(map, brush_size, center_x - dist_x, y);
            }
        }
        Symmetry::Vertical => {
            let center_y = map.height / 2;
            if y == center_y {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_y = i32::abs(center_y - y);
                apply_paint(map, brush_size, x, center_y + dist_y);
                apply_paint(map, brush_size, x, center_y - dist_y);
            }
        }
        Symmetry::Both => {
            let center_x = map.width / 2;
            let center_y = map.height / 2;
            if x == center_x && y == center_y {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_x = i32::abs(center_x - x);
                apply_paint(map, brush_size, center_x + dist_x, y);
                apply_paint(map, brush_size, center_x - dist_x, y);
                let dist_y = i32::abs(center_y - y);
                apply_paint(map, brush_size, x, center_y + dist_y);
                apply_paint(map, brush_size, x, center_y - dist_y);
            }
        }
    }
}

pub fn apply_paint(map: &mut Map, brush_size: i32, x: i32, y: i32) {
    match brush_size {
        1 => {
            map.tiles[x as usize][y as usize] = TileType::Floor;
        }
        _ => {
            let half_brush_size = brush_size / 2;
            for brush_y in y - half_brush_size..y + half_brush_size {
                for brush_x in x - half_brush_size..x + half_brush_size {
                    if brush_x > 1
                        && brush_x < map.width - 1
                        && brush_y > 1
                        && brush_y < map.height - 1
                    {
                        map.tiles[brush_x as usize][brush_y as usize] = TileType::Floor;
                    }
                }
            }
        }
    }
}
