use super::Map;
use super::MapBuilder;
use crate::components::Position;
use crate::map::TileType;
use crate::map_builders::common::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel,
};
use crate::rect::Rect;
use crate::{spawner, SHOW_MAPGEN_VISUALIZATION};
use bracket_lib::prelude::RandomNumberGenerator;
use specs::prelude::*;

pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    rooms: Vec<Rect>,
    history: Vec<Map>,
}

impl SimpleMapBuilder {
    fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 3;
        const MAX_SIZE: i32 = 15;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - w - w - 1);
            let y = rng.roll_dice(1, self.map.height - h - w) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in self.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                apply_room_to_map(&mut self.map, &new_room);
                self.take_snapshot();

                if !self.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = self.rooms[self.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(&mut self.map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut self.map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut self.map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut self.map, prev_x, new_x, new_y);
                    }
                }
                self.rooms.push(new_room);
                self.take_snapshot();
            }
        }
        let stairs_position = self.rooms[self.rooms.len() - 1usize].center();
        self.map.tiles[stairs_position.0 as usize][stairs_position.1 as usize] =
            TileType::DownStairs;
        let start_pos = self.rooms[0].center();
        self.starting_position = Position {
            x: start_pos.0,
            y: start_pos.1,
        };
    }
    pub fn new(new_depth: i32) -> Self {
        SimpleMapBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            rooms: Vec::new(),
            history: Vec::new(),
        }
    }
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, _ecs: &mut World) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
    }

    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
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

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }
}
