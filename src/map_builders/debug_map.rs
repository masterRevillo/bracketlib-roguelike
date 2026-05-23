use bracket_lib::prelude::RandomNumberGenerator;

use crate::map::TileType;

use super::{BuilderMap, InitialMapBuilder};

pub struct DebugMapBuilder {}

impl InitialMapBuilder for DebugMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data)
    }
}

impl DebugMapBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        for y in 1..build_data.height - 1 {
            for x in 1..build_data.width - 1 {
                build_data.map.tiles[x as usize][y as usize] = TileType::Floor;
            }
        }
    }
}
