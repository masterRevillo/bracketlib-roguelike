use bracket_lib::prelude::{console, RandomNumberGenerator};

use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::map_builders::common::{paint, Symmetry};
use crate::rect::Rect;

pub struct RoomExploder {}

impl MetaMapBuilder for RoomExploder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomExploder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Rooms Explosions requires rooms to be present")
        }

        for room in rooms.iter() {
            let start = room.center();
            let n_diggers = rng.roll_dice(1, 20) - 5;
            if n_diggers > 0 {
                for _i in 0..n_diggers {
                    let mut d_x = start.0;
                    let mut d_y = start.1;
                    let mut d_life = 20;
                    let mut did_something = false;

                    while d_life > 0 {
                        if build_data.map.tiles[d_x as usize][d_y as usize] == TileType::Wall {
                            did_something = true;
                        }
                        paint(&mut build_data.map, Symmetry::None, 1, d_x, d_y);
                        build_data.map.tiles[d_x as usize][d_y as usize] = TileType::DownStairs;

                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => {
                                if d_x > 2 {
                                    d_x -= 1;
                                }
                            }
                            2 => {
                                if d_x < build_data.map.width - 2 {
                                    d_x += 1;
                                }
                            }
                            3 => {
                                if d_y > 2 {
                                    d_y -= 1;
                                }
                            }
                            _ => {
                                if d_y < build_data.map.height - 2 {
                                    d_y += 1;
                                }
                            }
                        }
                        d_life -= 1;
                    }
                    if did_something {
                        build_data.take_snapshot();
                    }

                    for x in build_data.map.tiles.iter_mut() {
                        for t in x.iter_mut() {
                            if *t == TileType::DownStairs {
                                *t = TileType::Floor;
                            }
                        }
                    }
                }
            }
        }
    }
}
