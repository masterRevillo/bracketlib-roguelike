use bracket_lib::prelude::RandomNumberGenerator;

use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::map_builders::common::draw_corridor;

pub struct BSPCorridors {}

impl MetaMapBuilder for BSPCorridors {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.corridors(rng, build_data);
    }
}

impl BSPCorridors {

    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("Corridors require list of rooms to be present")
        };

        let mut corridors: Vec<Vec<(usize, usize)>> = Vec::new();
        for i in 0..rooms.len()-1 {
            let room = rooms[i];
            let next_room = rooms[i+1];

            let (start_x, start_y) = match room.radius {
                Some(r) => (
                    room.center().0 + (rng.roll_dice(1, i32::max(r as i32 - 1,1 ))),
                    room.center().1 + (rng.roll_dice(1, i32::max(r as i32 - 1, 1)))
                ),
                None => (
                    room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1),
                    room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1)
                )
            };
            let (end_x, end_y) = match next_room.radius {
                Some(r) => (
                    next_room.center().0 + (rng.roll_dice(1, i32::max(r as i32 - 1, 1))),
                    next_room.center().1 + (rng.roll_dice(1, i32::max(r as i32 - 1, 1)))
                ),
                None => (
                    next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1),
                    next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1)
                )
            };

            let corridor = draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            corridors.push(corridor);
            build_data.take_snapshot();
        }
        build_data.corridors = Some(corridors);
    }

}
