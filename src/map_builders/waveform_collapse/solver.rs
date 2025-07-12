use std::collections::HashSet;

use bracket_lib::prelude::console;
use bracket_lib::random::RandomNumberGenerator;

use crate::map::{Map, TileType};
use crate::map_builders::waveform_collapse::common::MapChunk;
use crate::map_builders::waveform_collapse::constraints::build_patterns;

pub struct Solver {
    constraints: Vec<MapChunk>,
    chunk_size: i32,
    chunks: Vec<Option<usize>>, // represents the index of chunks that we have selected to use
    chunks_x: usize,            // number of chunks in the x direction
    chunks_y: usize,            // number of chunks in the y direction
    remaining: Vec<(usize, i32)>, // (index, # of neighbors)
    pub possible: bool,
}

impl Solver {
    pub fn new(constraints: Vec<MapChunk>, chunk_size: i32, map: &Map) -> Self {
        let chunks_x = (map.width / chunk_size) as usize;
        let chunks_y = (map.height / chunk_size) as usize;
        let mut remaining: Vec<(usize, i32)> = Vec::new();
        // fill the remaining vector with all the chunks
        // the dimensions of the array are the number of chunks in the x * number of chunks in the y
        for i in 0..(chunks_x * chunks_y) {
            remaining.push((i, 0));
        }
        Self {
            constraints,
            chunk_size,
            chunks: vec![None; chunks_x * chunks_y],
            chunks_x,
            chunks_y,
            remaining,
            possible: true,
        }
    }

    fn chunk_idx(&self, x: usize, y: usize) -> usize {
        (y * self.chunks_x) + x
    }

    // Here chunk_x and chunk_y are the coordinates of the chunk in the grid
    fn count_neighbors(&self, chunk_x: usize, chunk_y: usize) -> i32 {
        let mut neighbors = 0;
        if chunk_x > 0 {
            let left_idx = self.chunk_idx(chunk_x - 1, chunk_y);
            if self.chunks[left_idx].is_some() {
                neighbors += 1;
            }
        }
        if chunk_x < self.chunks_x - 1 {
            let right_idx = self.chunk_idx(chunk_x + 1, chunk_y);
            if self.chunks[right_idx].is_some() {
                neighbors += 1;
            }
        }
        if chunk_y > 0 {
            let up_idx = self.chunk_idx(chunk_x, chunk_y - 1);
            if self.chunks[up_idx].is_some() {
                neighbors += 1;
            }
        }
        if chunk_y < self.chunks_y - 1 {
            let down_idx = self.chunk_idx(chunk_x, chunk_y + 1);
            if self.chunks[down_idx].is_some() {
                neighbors += 1;
            }
        }

        neighbors
    }

    pub fn iteration(&mut self, map: &mut Map, rng: &mut RandomNumberGenerator) -> bool {
        if self.remaining.is_empty() {
            return true;
        }
        let mut remain_copy = self.remaining.clone();
        let mut neighbors_exist = false;
        for r in remain_copy.iter_mut() {
            let idx = r.0;
            let chunk_x = idx % self.chunks_x;
            let chunk_y = idx / self.chunks_x;
            let neighbor_count = self.count_neighbors(chunk_x, chunk_y);
            if neighbor_count > 0 {
                neighbors_exist = true;
            }
            *r = (r.0, neighbor_count);
        }
        // Sort array with the most neighbors first
        remain_copy.sort_by(|a, b| b.1.cmp(&a.1));
        self.remaining = remain_copy;

        // if neighbors exist for any of the remaining tiles, select the first one, which should
        // have the most neighbors. If none remaining have neighbors, pick a random index
        let remaining_index = if !neighbors_exist {
            (rng.roll_dice(1, self.remaining.len() as i32) - 1) as usize
        } else {
            0usize
        };

        let chunk_index = self.remaining[remaining_index].0;
        self.remaining.remove(remaining_index);

        // These are chunk coodrinates in the grid, not dimee
        let chunk_x = chunk_index % self.chunks_x;
        let chunk_y = chunk_index / self.chunks_x;

        let mut neighbors = 0;
        let mut options: Vec<Vec<usize>> = Vec::new();

        // check the neighbors of the chunk to see if they are already set
        if chunk_x > 0 {
            let left_idx = self.chunk_idx(chunk_x - 1, chunk_y);
            match self.chunks[left_idx] {
                None => {}
                Some(nt) => {
                    neighbors += 1;
                    options.push(self.constraints[nt].compatible_with[3].clone());
                }
            }
        }
        if chunk_x < self.chunks_x - 1 {
            let right_idx = self.chunk_idx(chunk_x + 1, chunk_y);
            match self.chunks[right_idx] {
                None => {}
                Some(nt) => {
                    neighbors += 1;
                    options.push(self.constraints[nt].compatible_with[2].clone());
                }
            }
        }
        if chunk_y > 0 {
            let up_idx = self.chunk_idx(chunk_x, chunk_y - 1);
            match self.chunks[up_idx] {
                None => {}
                Some(nt) => {
                    neighbors += 1;
                    options.push(self.constraints[nt].compatible_with[1].clone());
                }
            }
        }
        if chunk_y < self.chunks_y - 1 {
            let down_idx = self.chunk_idx(chunk_x, chunk_y + 1);
            match self.chunks[down_idx] {
                None => {}
                Some(nt) => {
                    neighbors += 1;
                    options.push(self.constraints[nt].compatible_with[0].clone());
                }
            }
        }

        if neighbors == 0 {
            // there is nothing nearby, so we can use any pattern here
            // randomly select one, determine its x and y bounds within the map, and place it on
            // the map
            let new_chunk_idx = (rng.roll_dice(1, self.constraints.len() as i32) - 1) as usize;
            self.chunks[chunk_index] = Some(new_chunk_idx);
            let left_x = chunk_x as i32 * self.chunk_size;
            let right_x = (chunk_x as i32 + 1) * self.chunk_size;
            let top_y = chunk_y as i32 * self.chunk_size;
            let bottom_y = (chunk_y as i32 + 1) * self.chunk_size;

            let mut i: usize = 0;
            for y in top_y..bottom_y {
                for x in left_x..right_x {
                    let tile = self.constraints[new_chunk_idx].pattern[i];
                    map.tiles[x as usize][y as usize] = tile;
                    i += 1;
                }
            }
        } else {
            // there are neighbors, so we try to be compatible with them
            // build out a list of possible chunks based on the compatible neighbors built above.
            // If there are possibilities, select one at random if there is more than 1.
            let mut options_to_check: HashSet<usize> = HashSet::new();
            for o in options.iter() {
                for i in o.iter() {
                    options_to_check.insert(*i);
                }
            }
            let mut possible_options: Vec<usize> = Vec::new();
            for new_chunk_idx in options_to_check.iter() {
                let mut possible = true;
                for o in options.iter() {
                    if !o.contains(new_chunk_idx) {
                        possible = false;
                    }
                }
                if possible {
                    possible_options.push(*new_chunk_idx);
                }
            }
            if possible_options.is_empty() {
                console::log("Map is not possible. Stopping");
                self.possible = false;
                return true;
            } else {
                let new_chunk_idx = if possible_options.len() == 1 {
                    0
                } else {
                    rng.roll_dice(1, possible_options.len() as i32) - 1
                };
                self.chunks[chunk_index] = Some(new_chunk_idx as usize);
                let left_x = chunk_x as i32 * self.chunk_size;
                let right_x = (chunk_x as i32 + 1) * self.chunk_size;
                let top_y = chunk_y as i32 * self.chunk_size;
                let bottom_y = (chunk_y as i32 + 1) * self.chunk_size;

                let mut i: usize = 0;
                for y in top_y..bottom_y {
                    for x in left_x..right_x {
                        let tile = self.constraints[new_chunk_idx as usize].pattern[i];
                        map.tiles[x as usize][y as usize] = tile;
                        i += 1;
                    }
                }
            }
        }

        false
    }
}
