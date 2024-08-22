use bracket_lib::prelude::{console, DistanceAlg, Point, RandomNumberGenerator};

use crate::map::tiletype::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::rect::Rect;

pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer{
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data)
    }
}

impl RoomDrawer {

    pub fn new() -> Box<Self> {
        Box::new(Self { })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut rooms: Vec<Rect>;
        if let Some(r) = &build_data.rooms {
            rooms = r.clone();
        } else {
            panic!("Rooms must be present")
        }

        for room in rooms.iter_mut() {
            console::log(format!("printing rect with coords {}, {}, {}, {}", room.x1, room.x2, room.y1, room.y2));
            match rng.roll_dice(1,4) {
                1 => self.circle(build_data, room),
                _ => self.rectangle(build_data,room)
            }
            build_data.take_snapshot();
        }
        build_data.rooms = Some(rooms);
    }

    fn rectangle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        console::log("printing as rect");
        for y in room.y1+1..=room.y2 {
            for x in room.x1+1..=room.x2 {
                build_data.map.tiles[x as usize][y as usize] = TileType::Floor;
            }
        }
    }

    fn circle(&mut self, build_data: &mut BuilderMap, room: &mut Rect) {
        console::log("printing as circle");
        let radius = i32::min(room.x2 - room.x1, room.y2 - room.y1) as f32 / 2.0;
        room.radius = Some(radius);
        let center = room.center();
        let center_pt = Point::new(center.0, center.1);
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let distance = DistanceAlg::Pythagoras.distance2d(center_pt, Point::new(x, y));
                if distance <= radius {
                    build_data.map.tiles[x as usize][y as usize] = TileType::Floor;
                }
            }
        }
    }
}
