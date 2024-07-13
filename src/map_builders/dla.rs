use std::collections::HashMap;

use bracket_lib::geometry::LineAlg;
use bracket_lib::prelude::{line2d, Point};
use bracket_lib::random::RandomNumberGenerator;
use specs::World;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::{
    find_most_distant_tile, generate_voroni_spawn_regions, paint, Symmetry,
};
use crate::map_builders::MapBuilder;
use crate::spawner::{spawn_region, SpawnList};
use crate::SHOW_MAPGEN_VISUALIZATION;

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgoritm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<(i32, i32)>>,
    algorithm: DLAAlgoritm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32,
    spawn_list: SpawnList
}

impl MapBuilder for DLABuilder {
    fn build_map(&mut self, _ecs: &mut World) {
        self.build()
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

impl DLABuilder {
    #[allow(dead_code)]
    pub fn new(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgoritm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
            spawn_list: Vec::new()
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
            symmetry: Symmetry::None,
            floor_percent: 0.25,
            spawn_list: Vec::new()
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
            symmetry: Symmetry::None,
            floor_percent: 0.25,
            spawn_list: Vec::new()
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
            symmetry: Symmetry::None,
            floor_percent: 0.25,
            spawn_list: Vec::new()
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
            symmetry: Symmetry::Horizontal,
            floor_percent: 0.25,
            spawn_list: Vec::new()
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
            symmetry: Symmetry::Both,
            floor_percent: 0.25,
            spawn_list: Vec::new()
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
            symmetry: Symmetry::Vertical,
            floor_percent: 0.25,
            spawn_list: Vec::new()
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
                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        prev_x,
                        prev_y,
                    );
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
                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        digger_x,
                        digger_y,
                    );
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
                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        prev_x,
                        prev_y,
                    );
                }
            }
            floor_tile_count = self.map.get_total_floor_tiles();
            self.take_snapshot();

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

}
