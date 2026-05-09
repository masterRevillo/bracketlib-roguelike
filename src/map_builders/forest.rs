use bracket_lib::random::RandomNumberGenerator;

use crate::map_builders::{
    area_starting_points::{AreaStartingPoint, XStart, YStart},
    cellular_automata::CellularAutomataBuilder,
    cull_unreachable::CullUnreachable,
    distant_exit::DistantExit,
    voronoi_spawning::VoronoiSpawning,
    BuilderChain,
};

pub fn forest_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPoint::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPoint::new(XStart::LEFT, YStart::CENTER));

    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain
}
