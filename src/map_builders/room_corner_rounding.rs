use bracket_lib::prelude::RandomNumberGenerator;
use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::map_builders::room_exploder::RoomExploder;
use crate::rect::Rect;

pub struct RoomCornerRounding {}

impl MetaMapBuilder for RoomCornerRounding {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl RoomCornerRounding {
    pub fn new() -> Box<RoomCornerRounding> {
        Box::new(RoomCornerRounding{})
    }

    fn fill_if_corner(&mut self, x: usize, y: usize, build_data: &mut BuilderMap) {
        let w = build_data.map.width;
        let h = build_data.map.height;
        let mut neighbor_walls = 0;
        if x > 0 && build_data.map.tiles[x-1][y] == TileType::Wall {
            neighbor_walls += 1;
        }
        if y > 0 && build_data.map.tiles[x][y-1] == TileType::Wall {
            neighbor_walls += 1;
        }
        if x < (w-2) as usize && build_data.map.tiles[x+1][y] == TileType::Wall {
            neighbor_walls += 1;
        }
        if y < (h-2) as usize && build_data.map.tiles[x][y+1] == TileType::Wall {
            neighbor_walls += 1;
        }

        if neighbor_walls == 2 {
            build_data.map.tiles[x][y] = TileType::Wall;
        }
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(room_builder) = &build_data.rooms {
            rooms = room_builder.clone();
        } else {
            panic!("Room corner rounding requires a list of rooms to be present")
        }

        for room in rooms.iter() {
            self.fill_if_corner((room.x1+1) as usize, (room.y1+1) as usize, build_data);
            self.fill_if_corner(room.x2 as usize, (room.y1+1) as usize, build_data);
            self.fill_if_corner((room.x1+1) as usize, room.y2 as usize, build_data);
            self.fill_if_corner((room.x2) as usize, room.y2 as usize, build_data);

            build_data.take_snapshot();
        }
    }
}
