use bracket_lib::prelude::RandomNumberGenerator;

use crate::components::Position;
use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::random_tables::EntryType;
use crate::spawner::spawn_room;

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.door(rng, build_data);
    }
}

impl DoorPlacement {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn door(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(halls_original) = &build_data.corridors {
            let halls = halls_original.clone();
            for hall in halls.iter() {
                if hall.len() > 2 {
                    if self.door_possible(build_data, hall[0].0, hall[0].1) {
                        build_data.spawn_list.push(((hall[0].0 as i32, hall[0].1 as i32), EntryType::Door));

                    }
                }
            }
        } else {
            let tiles = build_data.map.tiles.clone();
            for (x, row) in tiles.iter().enumerate() {
                for (y, tile) in row.iter().enumerate() {
                    if *tile == TileType::Floor && self.door_possible(build_data, x, y) && rng.roll_dice(1,3)==1{
                        build_data.spawn_list.push(((x as i32, y as i32), EntryType::Door));
                    }
                }
            }
        }
    }

    fn door_possible(&self, build_data: &mut BuilderMap, x: usize, y: usize) -> bool {
        if build_data.map.tiles[x][y] == TileType::Floor &&
            (x > 1 && build_data.map.tiles[x-1][y] == TileType::Floor) &&
            (x < (build_data.map.width - 2) as usize && build_data.map.tiles[x+1][y] == TileType::Floor) &&
            (y > 1 && build_data.map.tiles[x][y-1] == TileType::Wall) &&
            (x < (build_data.map.height - 2) as usize && build_data.map.tiles[x][y+1] == TileType::Wall) {
            return true;
        }

        if build_data.map.tiles[x][y] == TileType::Floor &&
            (x > 1 && build_data.map.tiles[x-1][y] == TileType::Wall) &&
            (x < (build_data.map.width - 2) as usize && build_data.map.tiles[x+1][y] == TileType::Wall) &&
            (y > 1 && build_data.map.tiles[x][y-1] == TileType::Floor) &&
            (x < (build_data.map.height - 2) as usize && build_data.map.tiles[x][y+1] == TileType::Floor) {
            return true;
        }
        for spawn in build_data.spawn_list.iter() {
            if spawn.0.0 == x as i32 && spawn.0.1 == y as i32 {
                return false;
            }
        }

        false
    }
}
