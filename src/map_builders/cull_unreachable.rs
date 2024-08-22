use bracket_lib::prelude::{DijkstraMap, RandomNumberGenerator};
use crate::map::tiletype::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};

pub struct CullUnreachable {}

impl MetaMapBuilder for CullUnreachable {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl CullUnreachable {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        build_data.map.populate_blocked();
        let start_idx = build_data.map.xy_idx(
            starting_pos.x,
            starting_pos.y
        );
        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = DijkstraMap::new(build_data.map.width, build_data.map.height, &map_starts, &build_data.map, 1000.0);
        for (x, row) in build_data.map.tiles.iter_mut().enumerate() {
            for (y, tile) in row.iter_mut().enumerate() {
                if *tile == TileType::Floor {
                    let distance_to_start = dijkstra_map.map[y * build_data.map.width as usize + x];
                    if distance_to_start == f32::MAX {
                        *tile = TileType::Wall;
                    }
                }
            }
        }
    }
}
