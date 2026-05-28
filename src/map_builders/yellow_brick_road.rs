use std::i32;

use bracket_lib::{
    prelude::{a_star_search, DistanceAlg, Point},
    random::RandomNumberGenerator,
};

use crate::{
    map::{tile_walkable, TileType},
    map_builders::{BuilderMap, MetaMapBuilder},
};

pub struct YellowBrickRoad {}

impl MetaMapBuilder for YellowBrickRoad {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl YellowBrickRoad {
    pub fn new() -> Box<Self> {
        Box::new(YellowBrickRoad {})
    }

    fn find_exit(&self, build_data: &mut BuilderMap, seed_x: i32, seed_y: i32) -> (i32, i32) {
        let mut avail_floors: Vec<(usize, f32)> = Vec::new();

        for y in 0..build_data.height {
            for x in 0..build_data.width {
                if tile_walkable(build_data.map.tiles[x as usize][y as usize]) {
                    let idx = build_data.map.xy_idx(x, y);
                    avail_floors.push((
                        idx,
                        DistanceAlg::PythagorasSquared.distance2d(
                            Point::new(
                                idx as i32 % build_data.map.width,
                                idx as i32 / build_data.map.height,
                            ),
                            Point::new(seed_x, seed_y),
                        ),
                    ));
                }
            }
        }

        if avail_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        avail_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let end_x = avail_floors[0].0 as i32 % build_data.map.width;
        let end_y = avail_floors[0].0 as i32 / build_data.map.width;
        (end_x, end_y)
    }

    fn paint_road(&self, build_data: &mut BuilderMap, x: i32, y: i32) {
        if x < 1 || x > build_data.map.width - 2 || y < 1 || y > build_data.map.height - 2 {
            return;
        }
        if build_data.map.tiles[x as usize][y as usize] != TileType::DownStairs {
            build_data.map.tiles[x as usize][y as usize] = TileType::Road;
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let (end_x, end_y) = self.find_exit(
            build_data,
            build_data.map.width - 2,
            build_data.map.height / 2,
        );
        //build_data.map.tiles[end_x as usize][end_y as usize] = TileType::DownStairs;

        let exit_dir = rng.roll_dice(1, 2);
        let (seed_x, seed_y, stream_startx, stream_starty) = if exit_dir == 1 {
            (build_data.map.width - 1, 1, 0, build_data.height - 1)
        } else {
            (
                build_data.map.width - 1,
                build_data.height - 1,
                1,
                build_data.height - 1,
            )
        };

        let (stx, sty) = self.find_exit(build_data, seed_x, seed_y);
        let st_idx = build_data.map.xy_idx(stx, sty);
        build_data.take_snapshot();

        let (stream_x, stream_y) = self.find_exit(build_data, stream_startx, stream_starty);
        let stream_idx = build_data.map.xy_idx(stream_x, stream_y);
        let stream = a_star_search(st_idx, stream_idx, &mut build_data.map);
        for tile in stream.steps.iter() {
            let x = *tile % build_data.map.width as usize;
            let y = *tile / build_data.map.width as usize;
            if build_data.map.tiles[x][y] == TileType::Floor {
                build_data.map.tiles[x][y] = TileType::ShallowWater;
            }
        }
        build_data.map.tiles[stx as usize][sty as usize] = TileType::DownStairs;
        build_data.take_snapshot();

        build_data.map.populate_blocked();
        let path = a_star_search(
            build_data.map.xy_idx(starting_pos.x, starting_pos.y),
            build_data.map.xy_idx(end_x, end_y),
            &mut build_data.map,
        );

        for idx in path.steps.iter() {
            let x = *idx as i32 % build_data.map.width;
            let y = *idx as i32 / build_data.map.width;
            self.paint_road(build_data, x, y);
            self.paint_road(build_data, x + 1, y);
            self.paint_road(build_data, x - 1, y);
            self.paint_road(build_data, x, y + 1);
            self.paint_road(build_data, x, y - 1);
        }
    }
}
