use core::f32;
use std::collections::HashSet;
use std::usize;

use bracket_lib::algorithm_traits::{Algorithm2D, BaseMap, SmallVec};
use bracket_lib::geometry::{DistanceAlg, Point};
use serde::{Deserialize, Serialize};
use specs::Entity;

pub use tiletype::{tile_opaque, tile_walkable, TileType};

use crate::map::tiletype::tile_cost;

pub mod themes;
pub mod tiletype;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<Vec<TileType>>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<Vec<bool>>,
    pub visible_tiles: Vec<Vec<bool>>,
    pub blocked: Vec<Vec<bool>>,
    pub depth: i32,
    pub bloodstains: HashSet<(i32, i32)>,
    pub view_blocked: HashSet<(i32, i32)>,
    pub name: String,
    pub noise: Vec<Vec<f32>>,
    pub w_noise: Vec<Vec<f32>>,
    pub n_height: Vec<Vec<f32>>,
    pub n_temp: Vec<Vec<f32>>,
    pub n_humid: Vec<Vec<f32>>,
    pub n_biome: Vec<Vec<f32>>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Vec<Entity>>>,
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        tile_opaque(self.tiles[idx % self.width as usize][idx / self.width as usize])
            || self
                .view_blocked
                .contains(&(idx as i32 % self.width, idx as i32 / self.width))
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;
        let tt = self.tiles[x as usize][y as usize];

        // cardinal directions:
        // west
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, tile_cost(tt)))
        };
        // east
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, tile_cost(tt)))
        };
        // north
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, tile_cost(tt)))
        };
        // south
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, tile_cost(tt)))
        };

        // diagonals:
        // north west
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, tile_cost(tt) * 1.45));
        }
        // north east
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, tile_cost(tt) * 1.45));
        }
        // south west
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, tile_cost(tt) * 1.45));
        }
        // south east
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, tile_cost(tt) * 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let _h = self.height as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Map {
    pub fn new<S: ToString>(new_depth: i32, width: i32, height: i32, name: S) -> Map {
        Map {
            tiles: vec![vec![TileType::Wall; height as usize]; width as usize],
            width,
            height,
            revealed_tiles: vec![vec![false; height as usize]; width as usize],
            visible_tiles: vec![vec![false; height as usize]; width as usize],
            blocked: vec![vec![false; height as usize]; width as usize],
            tile_content: vec![vec![Vec::new(); height as usize]; width as usize],
            depth: new_depth,
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
            name: name.to_string(),
            noise: vec![vec![0.; height as usize]; width as usize],
            w_noise: vec![vec![0.; height as usize]; width as usize],
            n_height: vec![vec![0.; height as usize]; width as usize],
            n_temp: vec![vec![0.; height as usize]; width as usize],
            n_humid: vec![vec![0.; height as usize]; width as usize],
            n_biome: vec![vec![0.; height as usize]; width as usize],
        }
    }
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn get_tile_at_pos(&self, x: i32, y: i32) -> TileType {
        self.tiles[x as usize][y as usize]
    }

    pub fn populate_blocked(&mut self) {
        for (i, row) in self.tiles.iter_mut().enumerate() {
            for (j, tile) in row.iter_mut().enumerate() {
                self.blocked[i][j] = !tile_walkable(*tile);
            }
        }
    }

    pub fn clear_content_index(&mut self) {
        for col in self.tile_content.iter_mut() {
            for content in col.iter_mut() {
                content.clear();
            }
        }
    }

    pub fn is_tile_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width - 1 && y >= 0 && y < self.height - 1
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if !self.is_tile_in_bounds(x, y) {
            return false;
        }
        !self.blocked[x as usize][y as usize]
    }

    pub fn get_total_floor_tiles(&self) -> usize {
        self.tiles
            .iter()
            .map(|x| x.iter().filter(|y| **y == TileType::Floor).count())
            .sum()
    }
}
