use bracket_lib::prelude::RandomNumberGenerator;
use specs::prelude::*;

use crate::rect::Rect;

use super::{BuilderMap, InitialMapBuilder};

pub struct SimpleMapBuilder {
}

impl InitialMapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data)
    }
}

impl SimpleMapBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 3;
        const MAX_SIZE: i32 = 15;
        let mut rooms: Vec<Rect> = Vec::new();


        for i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, build_data.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, build_data.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                rooms.push(new_room);
            }
        }
        build_data.rooms = Some(rooms);
    }
}
