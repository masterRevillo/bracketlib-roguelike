use std::cmp::{max, min};
use std::os::unix::raw::time_t;
use bracket_lib::algorithm_traits::SmallVec;

use bracket_lib::color::RGB;
use bracket_lib::geometry::Point;
use bracket_lib::prelude::{Algorithm2D, BaseMap, BTerm, DistanceAlg, RandomNumberGenerator, to_cp437};
use specs::prelude::*;
use specs::WorldExt;

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
    pub visible_tiles: Vec<Vec<bool>>,
    pub blocked: Vec<Vec<bool>>,
    pub tile_content: Vec<Vec<Vec<Entity>>>
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

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 / self.width;
        let y = idx as i32 % self.width;
        let w = self.width as usize;

        if self.is_exit_valid(x-1, y) { exits.push((idx-w, 1.0))};
        if self.is_exit_valid(x+1, y) { exits.push((idx+w, 1.0))};
        if self.is_exit_valid(x, y-1) { exits.push((idx-1, 1.0))};
        if self.is_exit_valid(x, y+1) { exits.push((idx+1, 1.0))};

        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45));}
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx+w)-1, 1.45));}
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx-w)-1, 1.45));}
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45));}

        exits
    }


    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 / w, idx1 % w);
        let p2 = Point::new(idx2 / w, idx2 % w);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (x as usize * self.width as usize) + y as usize
    }

    pub fn get_tile_at_pos(&self, x: i32, y: i32) -> TileType {
        self.tiles[x as usize][y as usize]
    }

    pub fn populate_blocked(&mut self) {
        for (i, row) in self.tiles.iter_mut().enumerate() {
            for (j, tile) in row.iter_mut().enumerate() {
                self.blocked[i][j] = *tile == TileType::Wall;
            }
        }
    }

    pub fn clear_content_index(&mut self) {
        for col in self.tile_content.iter_mut() {
            for content in col.iter_mut() {
                content.clear();
            }
        }
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
            if self.is_tile_in_bounds(x, y) {
                self.tiles[x as usize][y as usize] = TileType::Floor
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2) ..= max(y1, y2) {
            if self.is_tile_in_bounds(x, y) {
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
            blocked: vec![vec![false; 50]; 80],
            tile_content: vec![vec![Vec::new(); 50]; 80],
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

    pub fn is_tile_in_bounds(&self, x: i32, y: i32) -> bool {
        x > 0 && x < self.width && y > 0 && y < self.height
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if !self.is_tile_in_bounds(x, y) {return false;}
        !self.blocked[x as usize][y as usize]
    }
}
