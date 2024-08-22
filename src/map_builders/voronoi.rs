use bracket_lib::geometry::Point;
use bracket_lib::prelude::{DistanceAlg, RandomNumberGenerator};

use crate::DEBUGGING;
use crate::map::tiletype::TileType;
use crate::map_builders::{BuilderMap, InitialMapBuilder};

#[derive(PartialEq, Copy, Clone)]
pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

pub struct VoronoiCellBuilder {
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm,
}

impl InitialMapBuilder for VoronoiCellBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl VoronoiCellBuilder {
    pub fn pythagoras() -> Box<Self> {
        Box::new(Self {
            n_seeds: 128,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        })
    }

    pub fn manhattan() -> Box<Self> {
        Box::new(Self {
            n_seeds: 128,
            distance_algorithm: DistanceAlgorithm::Manhattan,
        })
    }

    pub fn chebyshev() -> Box<Self> {
        Box::new(Self {
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Chebyshev,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut voronoi_seeds: Vec<Point> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, build_data.map.width - 1);
            let vy = rng.roll_dice(1, build_data.map.height - 1);
            let candidate = Point::new(vx, vy);
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance: Vec<(usize, f32)> = vec![(0, 0.0f32); self.n_seeds];
        let mut voronoi_membership: Vec<Vec<i32>> =
            vec![vec![0; build_data.map.height as usize]; build_data.map.width as usize];

        for (x, row) in voronoi_membership.iter_mut().enumerate() {
            for (y, vid) in row.iter_mut().enumerate() {
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

        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
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
                    build_data.map.tiles[x][y] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        }

        if DEBUGGING {
            for pos in voronoi_seeds.iter() {
                build_data.map.tiles[pos.x as usize][pos.y as usize] = TileType::DownStairs;
                build_data.take_snapshot();
            }
        }
    }
}
