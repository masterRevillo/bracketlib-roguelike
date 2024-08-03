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
        let mut corridors: Vec<Vec<(usize, usize)>> = Vec::new();
        for (i, room) in rooms.iter().enumerate() {
            if i > 0 {
                let (nx, ny) = room.center();
                let (px, py) = rooms[i-1].center();
                if rng.range(0,2) == 1 {
                    let mut c1 = apply_horizontal_tunnel(&mut build_data.map, px, nx, py);
                    let mut c2 = apply_vertical_tunnel(&mut build_data.map, py, ny, nx);
                    c1.append(&mut c2);
                    corridors.push(c1)
                } else {
                    let mut c1 = apply_vertical_tunnel(&mut build_data.map, py, ny, px);
                    let mut c2 = apply_horizontal_tunnel(&mut build_data.map, px, nx, ny);
                    c1.append(&mut c2);
                    corridors.push(c1)
                }
                build_data.take_snapshot();
            }
        }
        build_data.corridors = Some(corridors);
    }

}
