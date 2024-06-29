use std::collections::HashMap;

use bracket_lib::geometry::Point;
use bracket_lib::prelude::{DistanceAlg, RandomNumberGenerator};
use specs::World;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::{find_most_distant_tile, generate_voroni_spawn_regions};
use crate::map_builders::MapBuilder;
use crate::spawner::spawn_region;
use crate::{DEBUGGING, SHOW_MAPGEN_VISUALIZATION};

#[derive(PartialEq, Copy, Clone)]
pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

pub struct VoronoiCellBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<(i32, i32)>>,
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm,
}

impl MapBuilder for VoronoiCellBuilder {
    fn build_map(&mut self, ecs: &mut World) {
        self.build()
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawn_region(ecs, self.starting_position.clone(), area.1, self.depth);
        }
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

impl VoronoiCellBuilder {
    pub fn pythagoras(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            n_seeds: 128,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        }
    }

    pub fn manhattan(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            n_seeds: 128,
            distance_algorithm: DistanceAlgorithm::Manhattan,
        }
    }

    pub fn chebyshev(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Chebyshev,
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        let mut voronoi_seeds: Vec<Point> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, self.map.width - 1);
            let vy = rng.roll_dice(1, self.map.height - 1);
            let candidate = Point::new(vx, vy);
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance: Vec<(usize, f32)> = vec![(0, 0.0f32); self.n_seeds];
        let mut voronoi_membership: Vec<Vec<i32>> =
            vec![vec![0; self.map.height as usize]; self.map.width as usize];

        for (y, col) in voronoi_membership.iter_mut().enumerate() {
            for (x, vid) in col.iter_mut().enumerate() {
                for (seed, pos) in voronoi_seeds.iter().enumerate() {
                    let distance = match self.distance_algorithm {
                        DistanceAlgorithm::Pythagoras => {
                            DistanceAlg::PythagorasSquared.distance2d(Point::new(x, y), *pos)
                        }
                        DistanceAlgorithm::Manhattan => {
                            DistanceAlg::Manhattan.distance2d(Point::new(x, y), *pos)
                        }
                        DistanceAlgorithm::Chebyshev => {
                            DistanceAlg::Chebyshev.distance2d(Point::new(x, y), *pos)
                        }
                    };
                    voronoi_distance[seed] = (seed, distance);
                }
                voronoi_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                *vid = voronoi_distance[0].0 as i32;
            }
        }

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let x = x as usize;
                let y = y as usize;
                let mut neighbors = 0;
                let my_seed = voronoi_membership[x][y];
                if voronoi_membership[x - 1][y] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[x + 1][y] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[x][y - 1] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[x][y + 1] != my_seed {
                    neighbors += 1;
                }

                if neighbors < 2 {
                    self.map.tiles[x][y] = TileType::Floor;
                }
            }
            self.take_snapshot();
        }

        if DEBUGGING {
            for pos in voronoi_seeds.iter() {
                self.map.tiles[pos.x as usize][pos.y as usize] = TileType::DownStairs;
                self.take_snapshot();
            }
        }

        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };

        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        let exit_tile = find_most_distant_tile(&mut self.map, start_idx);
        self.take_snapshot();

        self.map.tiles[exit_tile.0][exit_tile.1] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voroni_spawn_regions(&self.map, &mut rng);
    }
}
