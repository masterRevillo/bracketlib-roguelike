use bracket_lib::{noise::NoiseType, random::RandomNumberGenerator};

use crate::{
    map::TileType,
    map_builders::{
        area_starting_points::{AreaStartingPoint, XStart, YStart},
        cull_unreachable::CullUnreachable,
        distant_exit::DistantExit,
        drunkards::DrunkardsWalkBuilder,
        noise::NoiseBuilder,
        noise_vegitation::NoiseVegitationBuilder,
        voronoi_spawning::VoronoiSpawning,
        BuilderChain, MetaMapBuilder,
    },
};

pub fn limestone_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Limestone Caverns");
    chain.start_with(DrunkardsWalkBuilder::winding_passages());
    chain.with(AreaStartingPoint::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPoint::new(XStart::LEFT, YStart::CENTER));
    chain.with(VoronoiSpawning::new());
    chain.with(CaveDecarator::new());
    chain.with(DistantExit::new());
    chain.with(NoiseBuilder::new(NoiseType::PerlinFractal));
    chain
}

pub struct CaveDecarator {}

impl MetaMapBuilder for CaveDecarator {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut super::BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveDecarator {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&self, rng: &mut RandomNumberGenerator, build_data: &mut super::BuilderMap) {
        let old_map = build_data.map.clone();

        for (x, row) in build_data.map.tiles.iter_mut().enumerate() {
            for (y, tile) in row.iter_mut().enumerate() {
                if *tile == TileType::Floor && rng.roll_dice(1, 6) == 1 {
                    *tile = TileType::Gravel;
                } else if *tile == TileType::Floor && rng.roll_dice(1, 10) == 1 {
                    *tile = TileType::ShallowWater;
                } else if *tile == TileType::Wall {
                    let neighbors = old_map.count_direct_neighbors(x, y, TileType::Wall);
                    if neighbors == 2 {
                        *tile = TileType::DeepWater;
                    } else if neighbors == 1 {
                        match rng.roll_dice(1, 4) {
                            1 => *tile = TileType::Stalactite,
                            2 => *tile = TileType::Stalagmite,
                            _ => {}
                        }
                    }
                }
            }
        }
        build_data.take_snapshot();
        build_data.map.outdoors = false;
    }
}
