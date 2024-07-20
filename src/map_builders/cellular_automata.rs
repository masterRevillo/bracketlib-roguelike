use bracket_lib::random::RandomNumberGenerator;
use specs::WorldExt;

use crate::map::TileType;
use crate::map_builders::{BuilderMap, InitialMapBuilder};

pub struct CellularAutomataBuilder {
}

impl InitialMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl CellularAutomataBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    pub fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {

        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                build_data.map.tiles[x as usize][y as usize] = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                }
            }
        }
        build_data.take_snapshot();

        for _i in 0..15 {
            let mut new_tiles = build_data.map.tiles.clone();
            for y in 1..(build_data.map.height - 1) as usize {
                for x in 1..(build_data.map.width - 1) as usize {
                    let mut neighbors = 0;
                    if build_data.map.tiles[x][y - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x][y + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x - 1][y] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x + 1][y] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x + 1][y - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x + 1][y + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x - 1][y - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x - 1][y + 1] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        new_tiles[x][y] = TileType::Wall;
                    } else {
                        new_tiles[x][y] = TileType::Floor;
                    }
                }
            }
            build_data.map.tiles = new_tiles.clone();
            build_data.take_snapshot();
        }
    }
}
