use std::cmp::{max, min};
use crate::map::{Map, TileType};
use crate::rect::Rect;

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for x in room.x1 +1..= room.x2 {
        for y in room.y1 +1 ..= room.y2 {
            map.tiles[x as usize][y as usize] = TileType::Floor
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2) ..= max(x1, x2) {
        if map.is_tile_in_bounds(x, y) {
            map.tiles[x as usize][y as usize] = TileType::Floor
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2) ..= max(y1, y2) {
        if map.is_tile_in_bounds(x, y) {
            map.tiles[x as usize][y as usize] = TileType::Floor
        }
    }
}
