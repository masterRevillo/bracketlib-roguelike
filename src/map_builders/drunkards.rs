use bracket_lib::random::RandomNumberGenerator;

use crate::components::Position;
use crate::map::tiletype::TileType;
use crate::map_builders::{BuilderMap, InitialMapBuilder, MetaMapBuilder};
use crate::map_builders::common::{
    paint, Symmetry,
};

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
    settings: DrunkardSettings,
}

impl InitialMapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl MetaMapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl DrunkardsWalkBuilder {
    pub fn open_area() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                dr_lifetime: 400,
                floor_percentage: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        })
    }

    pub fn open_halls() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 400,
                floor_percentage: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        })
    }

    pub fn fat_passages() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 100,
                floor_percentage: 0.4,
                brush_size: 2,
                symmetry: Symmetry::None,
            },
        })
    }

    pub fn winding_passages() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 100,
                floor_percentage: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        })
    }

    pub fn symmetrical_passages() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 50,
                floor_percentage: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Horizontal,
            },
        })
    }

    pub fn crazy_beer_goggles() -> Box<Self> {
        Box::new(Self {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                dr_lifetime: 50,
                floor_percentage: 0.4,
                brush_size: 2,
                symmetry: Symmetry::Both,
            },
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_position = Position {
            x: build_data.map.width / 2,
            y: build_data.map.height / 2,
        };
        build_data.map.tiles[starting_position.x as usize][starting_position.y as usize] =
            TileType::Floor;

        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.settings.floor_percentage * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data.map.get_total_floor_tiles();

        let mut digger_count = 0;

        while floor_tile_count < desired_floor_tiles {
            let mut did_something = false;
            let mut dr_x;
            let mut dr_y;
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    dr_x = starting_position.x;
                    dr_y = starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        dr_x = starting_position.x;
                        dr_y = starting_position.y;
                    } else {
                        dr_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                        dr_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    }
                }
            }
            let mut dr_life = self.settings.dr_lifetime;

            while dr_life > 0 {
                if build_data.map.tiles[dr_x as usize][dr_y as usize] == TileType::Wall {
                    did_something = true;
                }
                paint(
                    &mut build_data.map, self.settings.symmetry, self.settings.brush_size, dr_x, dr_y,
                );
                build_data.map.tiles[dr_x as usize][dr_y as usize] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => {
                        if dr_x > 2 {
                            dr_x -= 1;
                        }
                    }
                    2 => {
                        if dr_x < build_data.map.width - 2 {
                            dr_x += 1;
                        }
                    }
                    3 => {
                        if dr_y > 2 {
                            dr_y -= 1;
                        }
                    }
                    _ => {
                        if dr_y < build_data.map.height - 2 {
                            dr_y += 1;
                        }
                    }
                }
                dr_life -= 1;
            }
            if did_something {
                build_data.take_snapshot();
            }

            digger_count += 1;
            for x in build_data.map.tiles.iter_mut() {
                for t in x.iter_mut() {
                    if *t == TileType::DownStairs {
                        *t = TileType::Floor;
                    }
                }
            }
            floor_tile_count = build_data.map.get_total_floor_tiles();
        }
    }
}
