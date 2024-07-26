use bracket_lib::geometry::LineAlg;
use bracket_lib::prelude::{line2d, Point};
use bracket_lib::random::RandomNumberGenerator;

use crate::components::Position;
use crate::map::TileType;
use crate::map_builders::{BuilderMap, InitialMapBuilder, MetaMapBuilder};
use crate::map_builders::common::{
    paint, Symmetry,
};

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgoritm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

pub struct DLABuilder {
    algorithm: DLAAlgoritm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32,
}

impl InitialMapBuilder for DLABuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl MetaMapBuilder for DLABuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data)
    }
}

impl DLABuilder {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            algorithm: DLAAlgoritm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn walk_inward() -> Box<Self> {
        Box::new(Self {
            algorithm: DLAAlgoritm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn heavy_erosion() -> Box<Self> {
        Box::new(Self {
            algorithm: DLAAlgoritm::WalkInwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.35,
        })
    }

    pub fn walk_outward() -> Box<Self> {
        Box::new(Self {
            algorithm: DLAAlgoritm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn central_attractor() -> Box<Self> {
        Box::new(Self {
            algorithm: DLAAlgoritm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn insectoid() -> Box<Self> {
        Box::new(Self {
            algorithm: DLAAlgoritm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::Horizontal,
            floor_percent: 0.25,
        })
    }

    pub fn walk_inwards_symmetry() -> Box<Self> {
        Box::new(Self {
            algorithm: DLAAlgoritm::WalkInwards,
            brush_size: 2,
            symmetry: Symmetry::Both,
            floor_percent: 0.25,
        })
    }

    pub fn walk_outward_symmetry() -> Box<Self> {
        Box::new(Self {
            algorithm: DLAAlgoritm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::Vertical,
            floor_percent: 0.25,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_position = Position {
            x: build_data.map.width / 2,
            y: build_data.map.height / 2,
        };
        let (x, y) = (
            starting_position.x as usize,
            starting_position.y as usize,
        );
        build_data.take_snapshot();
        build_data.map.tiles[x][y] = TileType::Floor;
        build_data.map.tiles[x - 1][y] = TileType::Floor;
        build_data.map.tiles[x + 1][y] = TileType::Floor;
        build_data.map.tiles[x][y - 1] = TileType::Floor;
        build_data.map.tiles[x][y + 1] = TileType::Floor;

        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data.map.get_total_floor_tiles();
        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgoritm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    while build_data.map.tiles[digger_x as usize][digger_y as usize] == TileType::Wall {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        let stagger_dir = rng.roll_dice(1, 4);
                        match stagger_dir {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < build_data.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < build_data.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }
                    }
                    paint(
                        &mut build_data.map,
                        self.symmetry,
                        self.brush_size,
                        prev_x,
                        prev_y,
                    );
                }
                DLAAlgoritm::WalkOutwards => {
                    let mut digger_x = starting_position.x;
                    let mut digger_y = starting_position.y;
                    while build_data.map.tiles[digger_x as usize][digger_y as usize] == TileType::Floor {
                        let stagger_dir = rng.roll_dice(1, 4);
                        match stagger_dir {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < build_data.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < build_data.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }
                    }
                    paint(
                        &mut build_data.map,
                        self.symmetry,
                        self.brush_size,
                        digger_x,
                        digger_y,
                    );
                }
                DLAAlgoritm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;

                    let mut path = line2d(
                        LineAlg::Bresenham,
                        Point::new(digger_x, digger_y),
                        Point::new(starting_position.x, starting_position.y),
                    );

                    while build_data.map.tiles[digger_x as usize][digger_y as usize] == TileType::Wall
                        && !path.is_empty()
                    {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                    }
                    paint(
                        &mut build_data.map,
                        self.symmetry,
                        self.brush_size,
                        prev_x,
                        prev_y,
                    );
                }
            }
            floor_tile_count = build_data.map.get_total_floor_tiles();
            build_data.take_snapshot();
        }
    }

}
