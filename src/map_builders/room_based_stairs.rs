use bracket_lib::prelude::RandomNumberGenerator;

use crate::map::tiletype::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomBasedStairs {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(rooms) =&build_data.rooms {
            let stairs_position = rooms[rooms.len() -1].center();
            build_data.map.tiles[stairs_position.0 as usize][stairs_position.1 as usize] = TileType::DownStairs;
            build_data.take_snapshot();

        } else {
            panic!("Room based stairs requires list of rooms to be present ")
        }
    }
}
