use bracket_lib::noise::{FastNoise, NoiseType};
use bracket_lib::prelude::RandomNumberGenerator;

use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct NoiseVegitationBuilder {}

impl MetaMapBuilder for NoiseVegitationBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl NoiseVegitationBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        for y in 0..build_data.height {
            for x in 0..build_data.width {
                let noise = build_data.map.noise[x as usize][y as usize];
                let tile = build_data.map.tiles[x as usize][y as usize];
                if tile == TileType::Floor {
                    if noise > 0.999 {
                        build_data.map.tiles[x as usize][y as usize] = TileType::Moss
                    } else if noise < -0.99 {
                        build_data.map.tiles[x as usize][y as usize] = TileType::Grass
                    }
                }
            }
        }
    }
}
