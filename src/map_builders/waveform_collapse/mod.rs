use bracket_lib::prelude::RandomNumberGenerator;

use crate::map::Map;
use crate::map_builders::{BuilderMap,  MetaMapBuilder};
use crate::map_builders::waveform_collapse::common::MapChunk;
use crate::map_builders::waveform_collapse::constraints::{build_patterns, patterns_to_constraints, render_pattern_to_map};
use crate::map_builders::waveform_collapse::solver::Solver;

mod constraints;
mod common;
mod solver;

#[derive(PartialEq, Clone, Copy)]
pub enum WaveformMode { TestMap, Derived }

pub struct WaveformCollapseBuilder {}

impl MetaMapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl WaveformCollapseBuilder {
    pub fn new() -> Box<WaveformCollapseBuilder> {
        Box::new(WaveformCollapseBuilder {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        const CHUNK_SIZE: i32 = 8;
        build_data.take_snapshot();

        let current_depth = build_data.map.depth;
        let patterns = build_patterns(&build_data.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE, build_data);

        build_data.map = Map::new(current_depth, build_data.width, build_data.height);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &build_data.map);
            while !solver.iteration(&mut build_data.map, rng) {
                build_data.take_snapshot();
            }
            build_data.take_snapshot();
            if solver.possible {
                break;
            }
        }
    }

    fn render_tile_gallery(&mut self, constraints: &Vec<MapChunk>, chunk_size: i32, build_data: &mut BuilderMap) {
        build_data.map = Map::new(0, build_data.width, build_data.height);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < constraints.len() {
            render_pattern_to_map(&mut build_data.map, &constraints[counter], chunk_size, x, y);
            build_data.take_snapshot();
            x += chunk_size + 1;
            if x + chunk_size > build_data.map.width {
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > build_data.map.height {
                    build_data.map = Map::new(0, 64, 64);

                    x = 1;
                    y = 1;
                }
            }
            counter += 1;
        }
        build_data.take_snapshot();
    }
}
