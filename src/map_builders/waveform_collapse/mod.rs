use std::collections::HashMap;

use bracket_lib::prelude::RandomNumberGenerator;
use specs::World;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::{find_most_distant_tile, generate_voroni_spawn_regions};
use crate::map_builders::MapBuilder;
use crate::map_builders::waveform_collapse::common::MapChunk;
use crate::map_builders::waveform_collapse::constraints::{build_patterns, patterns_to_constraints, render_pattern_to_map};
use crate::map_builders::waveform_collapse::solver::Solver;
use crate::SHOW_MAPGEN_VISUALIZATION;
use crate::spawner::{spawn_region, SpawnList};

mod constraints;
mod common;
mod solver;

#[derive(PartialEq, Clone, Copy)]
pub enum WaveformMode { TestMap, Derived }

pub struct WaveformCollapseBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<(i32, i32)>>,
    derive_from: Option<Box<dyn MapBuilder>>,
    spawn_list: SpawnList
}

impl MapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self, ecs: &mut World) {
        self.build(ecs)
    }

    fn get_spawn_list(&self) -> &SpawnList {
        &self.spawn_list
    }


    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZATION {
            let mut snapshot = self.map.clone();
            for x in snapshot.revealed_tiles.iter_mut() {
                for v in x.iter_mut() {
                    *v = true;
                }
            }
            self.history.push(snapshot);
        }
    }
}

impl WaveformCollapseBuilder {
    pub fn new(depth: i32, derive_from: Option<Box<dyn MapBuilder>>) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            derive_from,
            spawn_list: Vec::new()
        }
    }

    #[allow(dead_code)]
    pub fn test_map(new_depth: i32) -> Self {
        WaveformCollapseBuilder::new(new_depth, None)
    }

    pub fn derived_map(new_depth: i32, builder: Box<dyn MapBuilder>) -> Self {
        WaveformCollapseBuilder::new(new_depth, Some(builder))
    }

    fn build(&mut self, ecs: &mut World) {
        let prebuilder = &mut self.derive_from.as_mut().unwrap();
        prebuilder.build_map(ecs);
        self.map = prebuilder.get_map();
        self.history.append(&mut prebuilder.get_snapshot_history());
        for y in self.map.tiles.iter_mut() {
            for t in y.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
        }
        self.take_snapshot();
        let mut rng = RandomNumberGenerator::new();

        const CHUNK_SIZE: i32 = 8;

        let patterns = build_patterns(&self.map,  CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE);

        self.map = Map::new(self.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &self.map);
            while !solver.iteration(&mut self.map, &mut rng) {
                self.take_snapshot();
            }
            self.take_snapshot();
            if solver.possible {
                break;
            }
        }

        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        while self.map.tiles[self.starting_position.x as usize][self.starting_position.y as usize]
            != TileType::Floor
        {
            self.starting_position.x -= 1;
            if self.starting_position.x < 1 {
                self.starting_position.x = self.map.width - 2;
                self.starting_position.y -= 1;
            }
            if self.starting_position.y < 1 {
                self.starting_position.y = self.map.height-2;
            }
        }

        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        let exit_tile = find_most_distant_tile(&mut self.map, start_idx);
        self.take_snapshot();

        self.map.tiles[exit_tile.0][exit_tile.1] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voroni_spawn_regions(&self.map, &mut rng);

        for area in self.noise_areas.iter() {
            spawn_region(&self.map, &mut rng, area.1, self.depth, &mut self.spawn_list);
        }
    }

    fn render_tile_gallery(&mut self, constraints: &Vec<MapChunk>, chunk_size: i32) {
        self.map = Map::new(0);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < constraints.len() {
            render_pattern_to_map(&mut self.map, &constraints[counter], chunk_size, x, y);
            self.take_snapshot();
            x += chunk_size + 1;
            if x + chunk_size > self.map.width {
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > self.map.height {
                    self.map = Map::new(0);

                    x = 1;
                    y = 1;
                }
            }
            counter += 1;
        }
        self.take_snapshot();
    }
}
