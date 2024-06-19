use bracket_lib::noise::{CellularDistanceFunction, FastNoise, NoiseType};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use bracket_lib::prelude::{console, DijkstraMap};
use bracket_lib::random::RandomNumberGenerator;
use specs::{World, WorldExt};

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::MapBuilder;
use crate::spawner::spawn_region;
use crate::{DEBUGGING, SHOW_MAPGEN_VISUALIZATION};

const MIN_ROOM_SIZE: i32 = 8;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<(i32, i32)>>,
}

impl CellularAutomataBuilder {
    pub fn new(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    pub fn build(&mut self, ecs: &mut World) {
        let mut rng = RandomNumberGenerator::new();

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                self.map.tiles[x as usize][y as usize] = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                }
            }
        }
        self.take_snapshot();

        for _i in 0..15 {
            let mut new_tiles = self.map.tiles.clone();
            for y in 1..(self.map.height - 1) as usize {
                for x in 1..(self.map.width - 1) as usize {
                    let mut neighbors = 0;
                    if self.map.tiles[x][y - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[x][y + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[x - 1][y] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[x + 1][y] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[x + 1][y - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[x + 1][y + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[x - 1][y - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[x - 1][y + 1] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        new_tiles[x][y] = TileType::Wall;
                    } else {
                        new_tiles[x][y] = TileType::Floor;
                    }
                }
            }
            self.map.tiles = new_tiles.clone();
            self.take_snapshot();
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
        }

        self.map.populate_blocked();

        let map_starts: Vec<usize> = vec![self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y)];
        let dijkstra_map = DijkstraMap::new(
            self.map.width,
            self.map.height,
            &map_starts,
            &self.map,
            200.0,
        );
        let mut exit_tile = (
            self.starting_position.x as usize,
            self.starting_position.y as usize,
            0.0f32,
        );
        for (x, row) in self.map.tiles.iter_mut().enumerate() {
            for (y, tile) in row.iter_mut().enumerate() {
                if *tile == TileType::Floor {
                    let distance_to_start = dijkstra_map.map[y * self.map.width as usize + x];
                    console::log(format!(
                        "checking {} {} with val {}",
                        x, y, distance_to_start
                    ));
                    if distance_to_start == f32::MAX {
                        *tile == TileType::Wall;
                    } else {
                        if distance_to_start > exit_tile.2 {
                            exit_tile.0 = x;
                            exit_tile.1 = y;
                            exit_tile.2 = distance_to_start;
                            console::log(format!(
                                "Relocating exit tile: {} {} {}",
                                exit_tile.0, exit_tile.1, exit_tile.2
                            ));
                        }
                    }
                }
            }
        }
        self.take_snapshot();
        self.map.tiles[exit_tile.0][exit_tile.1] = TileType::DownStairs;

        let mut file = File::create("dijkstra_map.txt").unwrap();

        if DEBUGGING {
            for y in 0..self.map.height {
                let mut vec = Vec::new();
                for x in 0..self.map.width {
                    vec.push(dijkstra_map.map[(y * self.map.width) as usize + x as usize]);
                }
                file.write_all(format!("{:?}\n", vec).as_bytes()).unwrap();
            }
            ecs.insert(dijkstra_map)
        }
        self.take_snapshot();

        let mut noise = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(CellularDistanceFunction::Manhattan);

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                if self.map.tiles[x as usize][y as usize] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 1024.0;
                    let cell_value = cell_value_f as i32;

                    if self.noise_areas.contains_key(&cell_value) {
                        self.noise_areas.get_mut(&cell_value).unwrap().push((x, y));
                    } else {
                        self.noise_areas.insert(cell_value, vec![(x, y)]);
                    }
                }
            }
        }
    }
}

impl MapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, ecs: &mut World) {
        self.build(ecs)
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for area in self.noise_areas.iter() {
            spawn_region(ecs, area.1, self.depth);
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
