use std::collections::HashMap;

use bracket_lib::random::RandomNumberGenerator;
use specs::World;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::{
    find_most_distant_tile, generate_voroni_spawn_regions, paint, Symmetry,
};
use crate::map_builders::MapBuilder;
use crate::spawner::spawn_region;
use crate::SHOW_MAPGEN_VISUALIZATION;

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub dr_lifetime: i32,
    pub floor_percentage: f32,
    pub brush_size: i32,
    pub symmetry: Symmetry,
}

pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<(i32, i32)>>,
    settings: DrunkardSettings,
}

impl MapBuilder for DrunkardsWalkBuilder {
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

impl DrunkardsWalkBuilder {
    pub fn new(depth: i32, settings: DrunkardSettings) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings,
        }
    }

    pub fn open_area(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                dr_lifetime: 400,
                floor_percentage: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        }
    }

    pub fn open_halls(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 400,
                floor_percentage: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        }
    }

    pub fn fat_passages(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 100,
                floor_percentage: 0.4,
                brush_size: 2,
                symmetry: Symmetry::None,
            },
        }
    }

    pub fn winding_passages(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 100,
                floor_percentage: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        }
    }

    pub fn symmetrical_passages(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 50,
                floor_percentage: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Horizontal,
            },
        }
    }

    pub fn crazy_beer_goggles(depth: i32) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 50,
                floor_percentage: 0.4,
                brush_size: 2,
                symmetry: Symmetry::Both,
            },
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        self.map.tiles[self.starting_position.x as usize][self.starting_position.y as usize] =
            TileType::Floor;

        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.settings.floor_percentage * total_tiles as f32) as usize;
        let mut floor_tile_count = self.map.get_total_floor_tiles();

        let mut digger_count = 0;
        let mut active_digger_count = 0;

        while floor_tile_count < desired_floor_tiles {
            let mut did_something = false;
            let mut dr_x;
            let mut dr_y;
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    dr_x = self.starting_position.x;
                    dr_y = self.starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        dr_x = self.starting_position.x;
                        dr_y = self.starting_position.y;
                    } else {
                        dr_x = rng.roll_dice(1, self.map.width - 3) + 1;
                        dr_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    }
                }
            }
            let mut dr_life = self.settings.dr_lifetime;

            while dr_life > 0 {
                if self.map.tiles[dr_x as usize][dr_y as usize] == TileType::Wall {
                    did_something = true;
                }
                paint(
                    &mut self.map,
                    self.settings.symmetry,
                    self.settings.brush_size,
                    dr_x,
                    dr_y,
                );
                self.map.tiles[dr_x as usize][dr_y as usize] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => {
                        if dr_x > 2 {
                            dr_x -= 1;
                        }
                    }
                    2 => {
                        if dr_x < self.map.width - 2 {
                            dr_x += 1;
                        }
                    }
                    3 => {
                        if dr_y > 2 {
                            dr_y -= 1;
                        }
                    }
                    _ => {
                        if dr_y < self.map.height - 2 {
                            dr_y += 1;
                        }
                    }
                }
                dr_life -= 1;
            }
            if did_something {
                self.take_snapshot();
                active_digger_count += 1;
            }

            digger_count += 1;
            for x in self.map.tiles.iter_mut() {
                for t in x.iter_mut() {
                    if *t == TileType::DownStairs {
                        *t = TileType::Floor;
                    }
                }
            }
            floor_tile_count = self.map.get_total_floor_tiles();
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
