use bracket_lib::prelude::RandomNumberGenerator;
use crate::components::Position;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::spawner::spawn_room;

pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomBasedStartingPosition {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {

        if let Some(rooms) =&build_data.rooms {
            let starting_pos = rooms[0].center();
            build_data.starting_position = Some(Position{ x: starting_pos.0, y: starting_pos.1 });
        } else {
            panic!("Room based starting position requires list of rooms to be present ")
        }
    }
}
