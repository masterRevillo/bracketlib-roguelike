use std::collections::HashSet;

use bracket_lib::prelude::console;

use crate::map::Map;
use crate::map::tiletype::TileType;
use crate::map_builders::waveform_collapse::common::{MapChunk, tile_idx_in_chunk};

pub fn build_patterns(map: &Map, chunk_size: i32, include_flipping: bool, dedupe: bool) -> Vec<Vec<TileType>> {
    let chunks_x = map.width / chunk_size;
    let chunks_y = map.height / chunk_size;
    let mut patterns = Vec::new();

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
            let mut pattern: Vec<TileType> = Vec::new();
            let start_x = cx * chunk_size;
            let end_x = (cx+1) * chunk_size;
            let start_y = cy * chunk_size;
            let end_y = (cy+1) * chunk_size;

            for y in start_y .. end_y {
                for x in start_x .. end_x {
                    pattern.push(map.tiles[x as usize][y as usize])
                }
            }
            patterns.push(pattern);
            if include_flipping {
                // horizontal flip
                pattern = Vec::new();
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        pattern.push(map.tiles[(end_x - (x+1)) as usize][y as usize]);
                    }
                }
                patterns.push(pattern);

                // vertical flip
                pattern = Vec::new();
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        pattern.push(map.tiles[x as usize][(end_y - (y+1)) as usize]);
                    }
                }
                pattern = Vec::new();

                //both
                for y in start_y .. end_y {
                    for x in start_x .. end_x {
                        pattern.push(map.tiles[(end_x - (x+1)) as usize][(end_y - (y+1)) as usize]);
                    }
                }
                patterns.push(pattern);
            }
        }
    }

    if dedupe {
        console::log(format!("Before deduping; {} patterns", patterns.len()));
        let set: HashSet<Vec<TileType>> = patterns.drain(..).collect();
        patterns.extend(set.into_iter());
        console::log(format!("After deduping; {} patterns", patterns.len()));
    }


    patterns
}

pub fn render_pattern_to_map(map: &mut Map, chunk: &MapChunk, chunk_size: i32, start_x: i32, start_y: i32) {
    let mut i = 0usize;
    for tile_y in 0..chunk_size {
        for tile_x in 0..chunk_size {
            map.tiles[(tile_x + start_x) as usize][(tile_y + start_y) as usize] = chunk.pattern[i];
            map.visible_tiles[(tile_x + start_x) as usize][(tile_y + start_y) as usize] = true;
            i += 1;
        }
    }
    let start_x: usize = start_x as usize;
    let start_y: usize = start_y as usize;
    let chunk_size: usize = chunk_size as usize;
    for (x, northbound) in chunk.exits[0].iter().enumerate() {
        if *northbound {
            map.tiles[start_x + x][start_y] = TileType::DownStairs;
        }
    }
    for (x, southbound) in chunk.exits[1].iter().enumerate() {
        if *southbound {
            map.tiles[start_x + x][start_y + chunk_size -1] = TileType::DownStairs;
        }
    }
    for (x, westbound) in chunk.exits[2].iter().enumerate() {
        if *westbound {
            map.tiles[start_x][start_y + x] = TileType::DownStairs;
        }
    }
    for (x, eastbound) in chunk.exits[3].iter().enumerate() {
        if *eastbound {
            map.tiles[start_x + chunk_size -1][start_y + x] = TileType::DownStairs;
        }
    }
}

pub fn patterns_to_constraints(patterns: Vec<Vec<TileType>>, chunk_size: i32) -> Vec<MapChunk> {
    let mut constraints: Vec<MapChunk> = Vec::new();
    for p in patterns {
        let mut new_chunk = MapChunk {
            pattern: p,
            exits: [ Vec::new(), Vec::new(), Vec::new(), Vec::new() ],
            has_exits: true,
            compatible_with: [ Vec::new(), Vec::new(), Vec::new(), Vec::new() ],
        };
        for exit in new_chunk.exits.iter_mut() {
            for _i in 0..chunk_size {
                exit.push(false);
            }
        }
        let mut n_exits = 0;
        for x in 0..chunk_size {
            let north_idx = tile_idx_in_chunk(chunk_size, x, 0);
            if new_chunk.pattern[north_idx] == TileType::Floor {
                new_chunk.exits[0][x as usize] = true;
                n_exits += 1;
            }
            let south_idx = tile_idx_in_chunk(chunk_size, x, chunk_size-1);
            if new_chunk.pattern[south_idx] == TileType::Floor {
                new_chunk.exits[1][x as usize] = true;
                n_exits += 1;
            }
            let west_idx = tile_idx_in_chunk(chunk_size, 0, x);
            if new_chunk.pattern[west_idx] == TileType::Floor {
                new_chunk.exits[2][x as usize] = true;
                n_exits += 1;
            }
            let east_idx = tile_idx_in_chunk(chunk_size, chunk_size-1, x);
            if new_chunk.pattern[east_idx] == TileType::Floor {
                new_chunk.exits[3][x as usize] = true;
                n_exits += 1;
            }
        }
        if n_exits == 0 {
            new_chunk.has_exits = false;
        }
        constraints.push(new_chunk);
    }

    let ch = constraints.clone();
    for c in constraints.iter_mut() {
        for (j, potential) in ch.iter().enumerate() {
            // if there are no exits, it's compatible with all 4 sides of the current chunk
            if !c.has_exits || !potential.has_exits {
                for compat in c.compatible_with.iter_mut() {
                    compat.push(j);
                }
            } else {
                // look for exits on each side of the current chunk
                for (direction, exit_list) in c.exits.iter_mut().enumerate() {
                    let opposite = match direction {
                        0 => 1, // our north, their south
                        1 => 0, // our south, their north
                        2 => 3, // our west, their east
                        _ => 2 // our east, their west
                    };

                    let mut it_fits = false;
                    let mut has_any = false;
                    for (slot, can_enter) in exit_list.iter().enumerate() {
                        if *can_enter {
                            has_any = true;
                            if potential.exits[opposite][slot] {
                                it_fits = true;
                            }
                        }
                    }
                    if it_fits {
                        c.compatible_with[direction].push(j);
                    }
                    if !has_any {
                        // there are no exits on this side. We will match only if the
                        // other edge also has not exits
                        let matching_exit_count = potential.exits[opposite].iter()
                            .filter(|a| !**a).count();
                        if matching_exit_count == 0 {
                            c.compatible_with[direction].push(j);
                        }
                    }
                }
            }
        }
    }
    constraints
}
