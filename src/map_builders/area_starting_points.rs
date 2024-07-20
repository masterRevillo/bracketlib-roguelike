use bracket_lib::prelude::DistanceAlg::PythagorasSquared;
use bracket_lib::prelude::{Point, RandomNumberGenerator};
use crate::components::Position;
use crate::map::TileType;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::map_builders::common::paint;

pub enum XStart {
    LEFT,
    CENTER,
    RIGHT
}

pub enum YStart {
    TOP,
    CENTER,
    BOTTOM
}

pub struct AreaStartingPoint {
    x: XStart,
    y: YStart
}

impl MetaMapBuilder for AreaStartingPoint {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl AreaStartingPoint {
    pub fn new(x: XStart, y: YStart) -> Box<Self> {
        Box::new(Self { x, y })
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let seed_x = match self.x {
            XStart::LEFT => 1,
            XStart::CENTER => build_data.map.width / 2,
            XStart::RIGHT => build_data.map.width - 2
        };

        let seed_y = match self.y {
            YStart::TOP => 1,
            YStart::CENTER => build_data.map.height / 2,
            YStart::BOTTOM => build_data.map.height - 2
        };

        let mut available_floors: Vec<((usize, usize), f32)> = Vec::new();
        for (x, row) in build_data.map.tiles.iter().enumerate() {
            for (y, tile) in row.iter().enumerate() {
                if *tile == TileType::Floor {
                    available_floors.push(
                        ((x, y),
                        PythagorasSquared.distance2d (
                            Point::new(x as i32, y as i32),
                            Point::new(seed_x, seed_y)
                        )
                    ));
                }
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }
        available_floors.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

        let start_x = available_floors[0].0.0 as i32;
        let start_y = available_floors[0].0.1 as i32;

        build_data.starting_position = Some(Position{ x: start_x, y: start_y})
    }
}
