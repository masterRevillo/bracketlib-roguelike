use bracket_lib::prelude::RandomNumberGenerator;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::spawner::{spawn_region, spawn_room};

pub struct CorridorSpawner {}

impl MetaMapBuilder for CorridorSpawner {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CorridorSpawner {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(corridors) =&build_data.corridors {
            for c in corridors.iter() {
                let depth = build_data.map.depth;
                let cor: Vec<(i32, i32)> = c.iter().map(|p|(p.0 as i32, p.1 as i32)).collect();
                spawn_region(
                    &build_data.map, rng, &cor, depth, &mut build_data.spawn_list
                );
            }
        } else {
            panic!("Room based spawning requires list of rooms to be present")
        }
    }
}
