use bracket_lib::prelude::RandomNumberGenerator;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::map_builders::common::{apply_horizontal_tunnel, apply_vertical_tunnel};

pub struct DoglegCorridors {}

impl MetaMapBuilder for DoglegCorridors {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl DoglegCorridors {

    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("Corridors require list of rooms to be present")
        };
        for (i, room) in rooms.iter().enumerate() {
            if i > 0 {
                let (nx, ny) = room.center();
                let (px, py) = rooms[i-1].center();
                if rng.range(0,2) == 1 {
                    apply_horizontal_tunnel(&mut build_data.map, px, nx, py);
                    apply_vertical_tunnel(&mut build_data.map, py, ny, nx);
                } else {
                    apply_vertical_tunnel(&mut build_data.map, py, ny, px);
                    apply_horizontal_tunnel(&mut build_data.map, px, nx, ny);
                }
                build_data.take_snapshot();
            }
        }
    }

}
