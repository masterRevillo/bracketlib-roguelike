use std::collections::HashSet;
use bracket_lib::prelude::{DistanceAlg, Point, RandomNumberGenerator};
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::map_builders::common::draw_corridor;

pub struct NearestCorridors {}

impl MetaMapBuilder for NearestCorridors {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.corridors(rng, build_data);
    }
}

impl NearestCorridors {

    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn corridors(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("Corridors require list of rooms to be present")
        };

        let mut corridors: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut connected: HashSet<usize> = HashSet::new();
        for (i, room) in rooms.iter().enumerate() {
            let mut room_distance: Vec<(usize, f32)> = Vec::new();
            let room_center = room.center();
            let room_center_pt = Point::new(room_center.0, room_center.1);
            for (j, other_room) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_center = other_room.center();
                    let other_center_pt = Point::new(other_center.0, other_center.1);
                    let distance = DistanceAlg::Pythagoras.distance2d(
                        room_center_pt, other_center_pt
                    );
                    room_distance.push((j, distance));
                }
            }
            if !room_distance.is_empty() {
                room_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
                let dest_center = rooms[room_distance[0].0].center();
                let corridor = draw_corridor(
                    &mut build_data.map,
                    room_center.0, room_center.1,
                    dest_center.0, dest_center.1
                );
                connected.insert(i);
                corridors.push(corridor);
                build_data.take_snapshot();
            }
        }
        build_data.corridors = Some(corridors);

    }

}
