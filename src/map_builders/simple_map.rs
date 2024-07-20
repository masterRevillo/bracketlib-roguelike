use bracket_lib::prelude::RandomNumberGenerator;
use specs::prelude::*;

use crate::map_builders::common::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel,
};
use crate::rect::Rect;

use super::{BuilderMap, InitialMapBuilder};

pub struct SimpleMapBuilder {
}

impl InitialMapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.rooms_and_corridors(rng, build_data)
    }
}

impl SimpleMapBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn rooms_and_corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
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
                apply_room_to_map(&mut build_data.map, &new_room);
                build_data.take_snapshot();

                if !rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, new_y);
                    }
                }
                rooms.push(new_room);
                build_data.take_snapshot();
            }
        }
        build_data.rooms = Some(rooms);
    }
}
