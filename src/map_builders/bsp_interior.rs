use bracket_lib::prelude::RandomNumberGenerator;

use crate::map::tiletype::TileType;
use crate::map_builders::{BuilderMap, InitialMapBuilder, };
use crate::map_builders::common::draw_corridor;
use crate::rect::Rect;

const MIN_ROOM_SIZE: i32 = 10;

pub struct BspInteriorBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BspInteriorBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data)
    }
}

impl BspInteriorBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            rects: Vec::new()
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut rooms: Vec<Rect> = Vec::new();
        self.rects.clear();
        self.rects
            .push(Rect::new(1, 1, build_data.map.width - 2, build_data.map.height - 2));
        let first_room = self.rects[0];
        self.add_subrects(first_room, rng);

        let rooms_copy = self.rects.clone();
        for r in rooms_copy.iter() {
            let room = *r;
            rooms.push(room);
            for y in room.y1 + 1..=room.y2 {
                for x in room.x1 + 1..=room.x2 {
                    if x > 0 && x < build_data.map.width - 1 && y > 0 && y < build_data.map.height - 1 {
                        build_data.map.tiles[x as usize][y as usize] = TileType::Floor;
                    }
                }
            }
            build_data.take_snapshot();
        }

        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next_room = rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x =
                next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1);
            let end_y =
                next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1);
            draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }
        build_data.rooms = Some(rooms);
    }

    fn add_subrects(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) {
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }

        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            let h1 = Rect::new(rect.x1, rect.y1, half_width - 1, height);
            self.rects.push(h1);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h1, rng);
            }
            let h2 = Rect::new(rect.x1 + half_width, rect.y1, half_width, height);
            self.rects.push(h2);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h2, rng);
            }
        } else {
            let v1 = Rect::new(rect.x1, rect.y1, width, half_height - 1);
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v1, rng);
            }
            let v2 = Rect::new(rect.x1, rect.y1 + half_height, width, half_height);
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v2, rng);
            }
        }
    }
}
