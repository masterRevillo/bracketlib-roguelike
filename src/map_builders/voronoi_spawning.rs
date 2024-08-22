use std::collections::HashMap;

use bracket_lib::noise::{CellularDistanceFunction, FastNoise, NoiseType};
use bracket_lib::prelude::RandomNumberGenerator;

use crate::map::tiletype::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::spawner::spawn_region;

pub struct VoronoiSpawning {}

impl MetaMapBuilder for VoronoiSpawning {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl VoronoiSpawning {

    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut noise_areas: HashMap<i32, Vec<(i32, i32)>> = HashMap::new();
        let mut noise = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(CellularDistanceFunction::Manhattan);

        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                if build_data.map.tiles[x as usize][y as usize] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 1024.0;
                    let cell_value = cell_value_f as i32;

                    if noise_areas.contains_key(&cell_value) {
                        noise_areas.get_mut(&cell_value).unwrap().push((x, y));
                    } else {
                        noise_areas.insert(cell_value, vec![(x, y)]);
                    }
                }
            }
        }
        for area in noise_areas.iter() {
            spawn_region(&build_data.map, rng, area.1, build_data.map.depth, &mut build_data.spawn_list);
        }
    }
}
