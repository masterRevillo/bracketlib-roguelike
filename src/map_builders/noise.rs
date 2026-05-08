use bracket_lib::noise::{FastNoise, NoiseType};
use bracket_lib::prelude::RandomNumberGenerator;

use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct NoiseBuilder {
    noise_type: NoiseType,
}

impl MetaMapBuilder for NoiseBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl NoiseBuilder {
    pub fn new(noise_type: NoiseType) -> Box<Self> {
        Box::new(Self { noise_type })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut noise = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(self.noise_type);
        noise.set_frequency(0.08);

        let mut n_height = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        n_height.set_noise_type(self.noise_type);
        n_height.set_frequency(0.01);

        let mut n_temp = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        n_temp.set_noise_type(self.noise_type);
        n_temp.set_frequency(0.01);

        let mut n_humid = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        n_humid.set_noise_type(self.noise_type);
        n_humid.set_frequency(0.01);

        let mut n_biome = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        n_biome.set_noise_type(self.noise_type);
        n_biome.set_frequency(0.01);

        for y in 0..build_data.height {
            for x in 0..build_data.width {
                build_data.map.noise[x as usize][y as usize] = noise.get_noise(x as f32, y as f32);
                build_data.map.n_height[x as usize][y as usize] =
                    n_height.get_noise(x as f32, y as f32);
                build_data.map.n_temp[x as usize][y as usize] =
                    n_temp.get_noise(x as f32, y as f32);
                build_data.map.n_humid[x as usize][y as usize] =
                    n_humid.get_noise(x as f32, y as f32);
                build_data.map.n_biome[x as usize][y as usize] =
                    n_biome.get_noise(x as f32, y as f32);
            }
        }
    }
}
