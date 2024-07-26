use std::cmp::{max, min};
use bracket_lib::random::RandomNumberGenerator;

use crate::map::{Map, TileType};
use crate::map_builders::area_starting_points::AreaStartingPoint;
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::BuilderChain;
use crate::map_builders::distant_exit::DistantExit;
use crate::map_builders::room_based_spawner::RoomBasedSpawner;
use crate::map_builders::room_based_stairs::RoomBasedStairs;
use crate::map_builders::room_based_starting_position::RoomBasedStartingPosition;
use crate::map_builders::room_corner_rounding::RoomCornerRounding;
use crate::map_builders::room_corridors_bsp::BSPCorridors;
use crate::map_builders::room_corridors_dogleg::DoglegCorridors;
use crate::map_builders::room_exploder::RoomExploder;
use crate::map_builders::room_sorter::{RoomSort, RoomSorter};
use crate::map_builders::simple_map::SimpleMapBuilder;
use crate::map_builders::voronoi_spawning::VoronoiSpawning;
use crate::rect::Rect;

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


pub fn draw_corridor(map: &mut Map, x1: i32, y1: i32, x2: i32, y2: i32) {
    let mut x = x1;
    let mut y = y1;

    while x != x2 || y != y2 {
        if x < x2 {
            x += 1;
        } else if x > x2 {
            x -= 1;
        } else if y < y2 {
            y += 1;
        } else if y > y2 {
            y -= 1;
        }
        map.tiles[x as usize][y as usize] = TileType::Floor;
    }
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
