use std::cmp::{max, min};
use std::collections::HashSet;

use bracket_lib::algorithm_traits::SmallVec;
use bracket_lib::color::RGB;
use bracket_lib::geometry::Point;
use bracket_lib::prelude::{Algorithm2D, BaseMap, BTerm, console, DistanceAlg, FontCharType, RandomNumberGenerator, to_cp437};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::WorldExt;

use crate::rect::Rect;

pub const MAPWIDTH: usize = 100;
pub const MAPHEIGHT: usize = 73;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;
#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<Vec<TileType>>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<Vec<bool>>,
    pub visible_tiles: Vec<Vec<bool>>,
    pub blocked: Vec<Vec<bool>>,
    pub depth: i32,
    pub bloodstains: HashSet<(i32, i32)>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
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

        // cardinal directions:
        // west
        if self.is_exit_valid(x-1, y) { exits.push((idx-w, 1.0))};
        // east
        if self.is_exit_valid(x+1, y) { exits.push((idx+w, 1.0))};
        // north
        if self.is_exit_valid(x, y-1) { exits.push((idx-1, 1.0))};
        // south
        if self.is_exit_valid(x, y+1) { exits.push((idx+1, 1.0))};

        // diagonals:
        // north west
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45));}
        // north east
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx+w)-1, 1.45));}
        // south west
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx-w)+1, 1.45));}
        // south east
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

    pub fn new_map_rooms_and_corridors(new_depth: i32) -> Map {
        let mut map = Map {
            tiles: vec![vec![TileType::Wall; MAPHEIGHT]; MAPWIDTH],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![vec![false; MAPHEIGHT]; MAPWIDTH],
            visible_tiles: vec![vec![false; MAPHEIGHT]; MAPWIDTH],
            blocked: vec![vec![false; MAPHEIGHT]; MAPWIDTH],
            tile_content: vec![vec![Vec::new(); MAPHEIGHT]; MAPWIDTH],
            depth: new_depth,
            bloodstains: HashSet::new(),
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 3;
        const MAX_SIZE: i32 = 15;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, MAPWIDTH as i32 - w - w - 1);
            let y = rng.roll_dice(1, MAPHEIGHT as i32 - h - w) - 1;
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
        let stairs_position = map.rooms[map.rooms.len() - 1usize].center();
        map.tiles[stairs_position.0 as usize][stairs_position.1 as usize] = TileType::DownStairs;
        map
    }


    pub fn draw_map(&self, ctx: &mut BTerm) {
        for x in 0..self.width {
            for y in 0..self.height {
                if self.revealed_tiles[x as usize][y as usize] {
                    let glyph;
                    let mut fg;
                    let mut bg = RGB::from_f32(0., 0.,0.);
                    match self.tiles[x as usize][y as usize] {
                        TileType::Floor => {
                            glyph = to_cp437('.');
                            fg = RGB::from_u8(170, 131, 96);
                            bg = RGB::from_u8(170, 131, 96);
                        }
                        TileType::Wall => {
                            glyph = self.wall_glyph(x, y);
                            fg = RGB::from_u8(127, 30, 20);
                        }
                        TileType::DownStairs => {
                            glyph = to_cp437('>');
                            fg = RGB::from_f32(0., 1.0, 1.0);
                        }
                    }
                    if self.bloodstains.contains(&(x, y)) { bg = RGB::from_f32(0.75, 0., 0.); }
                    if !self.visible_tiles[x as usize][y as usize] {
                        fg = fg.to_greyscale();
                        bg = RGB::from_f32(0., 0.,0.);
                    }
                    ctx.set(x, y, fg, bg, glyph)
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
    fn wall_glyph(&self, x: i32, y: i32) -> FontCharType {
        if x < 1 || x > self.width-2 || y < 1 || y > self.height- 2i32 { return 35; }
        let mut mask: u8 = 0;

        if self.is_revealed_and_wall(x, y - 1) { mask += 1; }
        if self.is_revealed_and_wall(x, y + 1) { mask += 2; }
        if self.is_revealed_and_wall(x - 1, y) { mask += 4; }
        if self.is_revealed_and_wall(x + 1, y) { mask += 8; }

        match mask {
            0 => { 9 }
            1 => { 186 }
            2 => { 186 }
            3 => { 186 }
            4 => { 205 }
            5 => { 188 }
            6 => { 187 }
            7 => { 185 }
            8 => { 205 }
            9 => { 200 }
            10 => { 201 }
            11 => { 204 }
            12 => { 205 }
            13 => { 202 }
            14 => { 203 }
            15 => { 206 }
            _ => { 35 }
        }
    }
    fn is_revealed_and_wall(&self, x: i32, y: i32) -> bool {
        self.tiles[x as usize][y as usize] == TileType::Wall
            && self.revealed_tiles[x as usize][y as usize]
    }
}
