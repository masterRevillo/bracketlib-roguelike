use std::collections::HashMap;

use bracket_lib::random::RandomNumberGenerator;
use specs::{World, WorldExt};

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::{find_most_distant_tile, generate_voroni_spawn_regions};
use crate::map_builders::MapBuilder;
use crate::spawner::spawn_region;
use crate::SHOW_MAPGEN_VISUALIZATION;

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

impl MapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, ecs: &mut World) {
        self.build(ecs)
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
