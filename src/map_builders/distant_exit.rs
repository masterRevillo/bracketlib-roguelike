use bracket_lib::pathfinding::DijkstraMap;
use bracket_lib::prelude::RandomNumberGenerator;

use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct DistantExit {}

impl MetaMapBuilder for DistantExit {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl DistantExit {

    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(
            starting_pos.x,
            starting_pos.y
        );

        // First clear all existing exits
        for x in build_data.map.tiles.iter_mut() {
            for t in x.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
        }

        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = DijkstraMap::new(build_data.map.width, build_data.map.height, &map_starts, &build_data.map, 200.0);
        let mut exit_tile = (0, 0, 0.0f32);
        for (x, row) in build_data.map.tiles.iter_mut().enumerate() {
            for (y, tile) in row.iter_mut().enumerate() {
                if *tile == TileType::Floor {
                    let distance_to_start = dijkstra_map.map[y * build_data.map.width as usize + x];
                    if distance_to_start != f32::MAX {
                        if distance_to_start > exit_tile.2 {
                            exit_tile.0 = x;
                            exit_tile.1 = y;
                            exit_tile.2 = distance_to_start;
                        }
                    }
                }
            }
        }
        let stairs_x = exit_tile.0;
        let stairs_y = exit_tile.1;
        build_data.map.tiles[stairs_x][stairs_y] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
