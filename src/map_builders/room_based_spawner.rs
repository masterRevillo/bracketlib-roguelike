use bracket_lib::prelude::RandomNumberGenerator;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::spawner::spawn_room;

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomBasedSpawner {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(rooms) =&build_data.rooms {
            for room in rooms.iter().skip(1) {
                spawn_room(&build_data.map, rng, room, build_data.map.depth, &mut build_data.spawn_list);
            }
        } else {
            panic!("Room based spawning requires list of rooms to be present")
        }
    }
}
