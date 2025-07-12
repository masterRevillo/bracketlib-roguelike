use bracket_lib::prelude::{console, Point, RandomNumberGenerator};
use bracket_lib::prelude::DistanceAlg::PythagorasSquared;

use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::map_builders::room_sorter::RoomSort::{BOTTOMMOST, CENTRAL, LEFTMOST, RIGHTMOST, TOPMOST};
use crate::rect::Rect;

pub enum RoomSort {
    LEFTMOST,
    RIGHTMOST,
    TOPMOST,
    BOTTOMMOST,
    CENTRAL,
}

pub struct RoomSorter {
    sort_by: RoomSort,
}

impl MetaMapBuilder for RoomSorter {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.sorter(rng, build_data);
    }
}

impl RoomSorter {
    pub fn topmost() -> Box<Self> {
        Box::new(Self {
            sort_by: TOPMOST
        })
    }
    pub fn bottommost() -> Box<Self> {
        Box::new(Self {
            sort_by: BOTTOMMOST
        })
    }

    pub fn leftmost() -> Box<Self> {
        Box::new(Self {
            sort_by: LEFTMOST
        })
    }
    pub fn rightmost() -> Box<Self> {
        Box::new(Self {
            sort_by: RIGHTMOST
        })
    }

    pub fn central() -> Box<Self> {
        Box::new(Self {
            sort_by: CENTRAL
        })
    }

    fn sorter(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        console::log(format!("I am sorting, rooms is: {:?}", build_data.rooms));
        match self.sort_by {
            LEFTMOST => build_data.rooms.as_mut().unwrap().sort_by(|a, b| a.x1.cmp(&b.x1)),
            RIGHTMOST => build_data.rooms.as_mut().unwrap().sort_by(|a, b| a.x2.cmp(&b.x2)),
            TOPMOST => build_data.rooms.as_mut().unwrap().sort_by(|a, b| a.y1.cmp(&b.y1)),
            BOTTOMMOST => build_data.rooms.as_mut().unwrap().sort_by(|a, b| a.y2.cmp(&b.y2)),
            CENTRAL => {
                let map_center = Point::new(build_data.map.width / 2, build_data.map.height / 2);
                let center_sort = |a: &Rect, b: &Rect| {
                    let a_c = a.center();
                    let a_c_pt = Point::new(a_c.0, a_c.1);
                    let b_c = b.center();
                    let b_c_pt = Point::new(b_c.0, b_c.1);
                    let d_a = PythagorasSquared.distance2d(a_c_pt, map_center);
                    let d_b = PythagorasSquared.distance2d(b_c_pt, map_center);
                    d_a.partial_cmp(&d_b).unwrap()
                };

                build_data.rooms.as_mut().unwrap().sort_by(center_sort);
            }
        }
    }
}
