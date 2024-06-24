use std::collections::HashMap;

use bracket_lib::geometry::LineAlg;
use bracket_lib::prelude::{console, line2d, Point};
use bracket_lib::random::RandomNumberGenerator;
use specs::World;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::MapBuilder;
use crate::spawner::spawn_region;
use crate::SHOW_MAPGEN_VISUALIZATION;

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgoritm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

#[derive(PartialEq, Copy, Clone)]
pub enum DLASymmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<(i32, i32)>>,
    algorithm: DLAAlgoritm,
    brush_size: i32,
    symmetry: DLASymmetry,
    floor_percent: f32,
}

impl MapBuilder for DLABuilder {
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

impl DLABuilder {
    pub fn new(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgoritm::WalkOutwards,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn walk_inward(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgoritm::WalkInwards,
            brush_size: 1,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn walk_outward(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgoritm::WalkOutwards,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn central_attractor(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgoritm::CentralAttractor,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn insectoid(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgoritm::CentralAttractor,
            brush_size: 2,
            symmetry: DLASymmetry::Horizontal,
            floor_percent: 0.25,
        }
    }

    pub fn walk_inwards_symmetry(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgoritm::WalkInwards,
            brush_size: 2,
            symmetry: DLASymmetry::Both,
            floor_percent: 0.25,
        }
    }

    pub fn walk_outward_symmetry(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgoritm::WalkOutwards,
            brush_size: 2,
            symmetry: DLASymmetry::Vertical,
            floor_percent: 0.25,
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let (x, y) = (
            self.starting_position.x as usize,
            self.starting_position.y as usize,
        );
        self.take_snapshot();
        self.map.tiles[x][y] = TileType::Floor;
        self.map.tiles[x - 1][y] = TileType::Floor;
        self.map.tiles[x + 1][y] = TileType::Floor;
        self.map.tiles[x][y - 1] = TileType::Floor;
        self.map.tiles[x][y + 1] = TileType::Floor;

        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = self.map.get_total_floor_tiles();
        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgoritm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    while self.map.tiles[digger_x as usize][digger_y as usize] == TileType::Wall {
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
                                if digger_x < self.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < self.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }
                    }
                    self.paint(prev_x, prev_y);
                }
                DLAAlgoritm::WalkOutwards => {
                    let mut digger_x = self.starting_position.x;
                    let mut digger_y = self.starting_position.y;
                    while self.map.tiles[digger_x as usize][digger_y as usize] == TileType::Floor {
                        let stagger_dir = rng.roll_dice(1, 4);
                        match stagger_dir {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < self.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < self.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }
                    }
                    self.paint(digger_x, digger_y);
                }
                DLAAlgoritm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;

                    let mut path = line2d(
                        LineAlg::Bresenham,
                        Point::new(digger_x, digger_y),
                        Point::new(self.starting_position.x, self.starting_position.y),
                    );

                    while self.map.tiles[digger_x as usize][digger_y as usize] == TileType::Wall
                        && !path.is_empty()
                    {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                    }
                    self.paint(prev_x, prev_y);
                }
            }
            floor_tile_count = self.map.get_total_floor_tiles();
            self.take_snapshot();
        }
    }

    fn paint(&mut self, x: i32, y: i32) {
        match self.symmetry {
            DLASymmetry::None => self.apply_paint(x, y),
            DLASymmetry::Horizontal => {
                let center_x = self.map.width / 2;
                if x == center_x {
                    self.apply_paint(x, y);
                } else {
                    let dist_x = i32::abs(center_x - x);
                    self.apply_paint(center_x + dist_x, y);
                    self.apply_paint(center_x - dist_x, y);
                }
            }
            DLASymmetry::Vertical => {
                let center_y = self.map.height / 2;
                if y == center_y {
                    self.apply_paint(x, y);
                } else {
                    let dist_y = i32::abs(center_y - y);
                    self.apply_paint(x, center_y + dist_y);
                    self.apply_paint(x, center_y - dist_y);
                }
            }
            DLASymmetry::Both => {
                let center_x = self.map.width / 2;
                let center_y = self.map.height / 2;
                if x == center_x && y == center_y {
                    self.apply_paint(x, y);
                } else {
                    let dist_x = i32::abs(center_x - x);
                    self.apply_paint(center_x + dist_x, y);
                    self.apply_paint(center_x - dist_x, y);
                    let dist_y = i32::abs(center_y - y);
                    self.apply_paint(x, center_y + dist_y);
                    self.apply_paint(x, center_y - dist_y);
                }
            }
        }
    }

    fn apply_paint(&mut self, x: i32, y: i32) {
        match self.brush_size {
            1 => {
                self.map.tiles[x as usize][y as usize] = TileType::Floor;
            }
            _ => {
                let half_brush_size = self.brush_size / 2;
                for brush_y in y - half_brush_size..y + half_brush_size {
                    for brush_x in x - half_brush_size..x + half_brush_size {
                        if brush_x > 1
                            && brush_x < self.map.width - 1
                            && brush_y > 1
                            && brush_y < self.map.height - 1
                        {
                            self.map.tiles[brush_x as usize][brush_y as usize] = TileType::Floor;
                        }
                    }
                }
            }
        }
    }
}
