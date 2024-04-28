use std::cmp::{max, min};

use bracket_lib::color::RGB;
use bracket_lib::geometry::Point;
use bracket_lib::prelude::{Algorithm2D, BaseMap, BTerm, RandomNumberGenerator, to_cp437};
use specs::{World, WorldExt};
use specs::prelude::*;

use crate::components::{Player, Viewshed};
use crate::rect::Rect;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

pub struct Map {
    pub tiles: Vec<Vec<TileType>>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<Vec<bool>>,
    pub visible_tiles: Vec<Vec<bool>>
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx % self.width as usize][idx / self.width as usize] == TileType::Wall
    }
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (x as usize * self.width as usize) + y as usize
    }

    pub fn get_tile_at_pos(&self, x: i32, y: i32) -> TileType {
        self.tiles[x as usize][y as usize]
    }

    pub fn apply_room_to_map(&mut self, room: &Rect) {
        for x in room.x1 +1..= room.x2 {
            for y in room.y1 +1 ..= room.y2 {
                self.tiles[x as usize][y as usize] = TileType::Floor
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2) ..= max(x1, x2) {
            if x > 0 && x < self.width && y > 0 && y < self.height {
                self.tiles[x as usize][y as usize] = TileType::Floor
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2) ..= max(y1, y2) {
            if x > 0 && x < self.width && y > 0 && y < self.height {
                self.tiles[x as usize][y as usize] = TileType::Floor
            }
        }
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![vec![TileType::Wall; 50]; 80],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![vec![false; 50]; 80],
            visible_tiles: vec![vec![false; 50]; 80],
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 3;
        const MAX_SIZE: i32 = 15;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - w - 1);
            let y = rng.roll_dice(1, 50 - h - w) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok =true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false}
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
                map.rooms.push(new_room)
            }
        }
        map
    }


    pub fn draw_map(&self, ctx: &mut BTerm) {
        for x in 0..self.width {
            for y in 0..self.height {
                if self.revealed_tiles[x as usize][y as usize] {
                    let glyph;
                    let mut fg;
                    match self.tiles[x as usize][y as usize] {
                        TileType::Floor => {
                            glyph = to_cp437('.');
                            fg = RGB::from_f32(0.5, 0.5, 0.5);
                        }
                        TileType::Wall => {
                            glyph = to_cp437('#');
                            fg = RGB::from_f32(0.6, 0.3, 0.1);
                        }
                    }
                    if !self.visible_tiles[x as usize][y as usize] { fg = fg.to_greyscale() }
                    ctx.set(x, y, fg, RGB::from_f32(0.,0.,0.), glyph)
                }
            }
        }

    }
}

// generates a room with borders and 400 randomly placed walls
// pub fn new_map_test() -> Vec<TileType> {
//     let mut map = vec![TileType::Floor; 80*50];
//
//     for x in 0..80 {
//         map[xy_idx(x, 0)] = TileType::Wall;
//         map[xy_idx(x, 49)] = TileType::Wall;
//     }
//     for y in 0..50 {
//         map[xy_idx(0, y)] = TileType::Wall;
//         map[xy_idx(79, y)] = TileType::Wall;
//     }
//     let mut rng = RandomNumberGenerator::new();
//
//     for _i in 0..400 {
//         let x = rng.roll_dice(1, 79);
//         let y = rng.roll_dice(1, 49);
//         let idx = xy_idx(x, y);
//         if idx != xy_idx(40, 25) {
//             map[idx] = TileType::Wall;
//         }
//     }
//     map
// }
