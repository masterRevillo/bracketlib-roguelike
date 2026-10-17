use core::f32;
use std::collections::HashSet;
use std::{mem, usize};

use bracket_lib::algorithm_traits::{Algorithm2D, BaseMap, SmallVec};
use bracket_lib::color::RGB;
use bracket_lib::geometry::{DistanceAlg, Point};
use bracket_lib::random::RandomNumberGenerator;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::{Entity, World};

pub use tiletype::{tile_opaque, tile_walkable, TileType};

use crate::components::{Position, Viewshed};
use crate::map::dungeon::MasterDungeonMap;
use crate::map::tiletype::tile_cost;
use crate::{map_builders, MAP_X, MAP_Y};

pub mod dungeon;
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
    pub outdoors: bool,
    pub light: Vec<Vec<RGB>>,

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
            outdoors: true,
            light: vec![vec![RGB::from_f32(0., 0., 0.); height as usize]; width as usize],
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

    pub fn find_first_tile_xy(&self, tile_type: TileType) -> Option<(usize, usize)> {
        let mut res = None;
        for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().enumerate() {
                if *tile == tile_type {
                    res = Some((i, j));
                }
            }
        }
        res
    }

    pub fn count_direct_neighbors(&self, x: usize, y: usize, tile_type: TileType) -> i32 {
        let mut neighbors = 0;
        if x > 0 && self.tiles[x - 1][y] == tile_type {
            neighbors += 1;
        }
        if x < (self.width - 2) as usize && self.tiles[x + 1][y] == tile_type {
            neighbors += 1;
        }
        if y > 0 && self.tiles[x][y - 1] == tile_type {
            neighbors += 1;
        }
        if y < (self.height - 2) as usize && self.tiles[x][y + 1] == tile_type {
            neighbors += 1;
        }
        neighbors
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

pub fn level_transition(ecs: &mut World, new_depth: i32, offset: i32) -> Option<Vec<Map>> {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();

    if dungeon_master.get_map(new_depth).is_some() {
        mem::drop(dungeon_master);
        transition_to_existing_map(ecs, new_depth, offset);
        None
    } else {
        mem::drop(dungeon_master);
        Some(transition_to_new_map(ecs, new_depth))
    }
}

/// Generates a new map
/// Uses the map builder to construct a new level, and sets an UpStairs at the players starting
/// position. Then, the generated map is copied into the ecs from the map builder, along with
/// the starting position.
/// Entities are then spawned using the builder.
/// Then the players position is set in the ecs (set as the Point resource, and also the
/// Position storage). Then the viewshed is marked dirty
fn transition_to_new_map(ecs: &mut World, new_depth: i32) -> Vec<Map> {
    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    let mut builder = map_builders::level_builder(new_depth, &mut rng, MAP_X, MAP_Y);
    builder.build_map(&mut rng);

    if new_depth > 1 {
        if let Some(pos) = &builder.build_data.starting_position {
            builder.build_data.map.tiles[pos.x as usize][pos.y as usize] = TileType::UpStairs;
        }
    }
    let mapgen_history = builder.build_data.history.clone();

    let player_start;
    {
        let mut map_resource = ecs.write_resource::<Map>();
        *map_resource = builder.build_data.map.clone();
        player_start = builder
            .build_data
            .starting_position
            .as_mut()
            .unwrap()
            .clone();
    }
    drop(rng);
    builder.spawn_entities(ecs);
    let (player_x, player_y) = (player_start.x, player_start.y);
    let mut player_pos = ecs.write_resource::<Point>();
    *player_pos = Point::new(player_x, player_y);
    let mut position_component = ecs.write_storage::<Position>();
    let player_entity = ecs.fetch::<Entity>();
    let player_pos_comp = position_component.get_mut(*player_entity);
    if let Some(player_pos_comp) = player_pos_comp {
        player_pos_comp.x = player_x;
        player_pos_comp.y = player_y;
    }
    let mut viewshed_comp = ecs.write_storage::<Viewshed>();
    let vs = viewshed_comp.get_mut(*player_entity);
    if let Some(vs) = vs {
        vs.dirty = true;
    }

    let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
    dungeon_master.store_map(&builder.build_data.map);

    mapgen_history
}

/// Pulls an existing map out of the master dungeon map list. It sets the ecs's map resource to
/// the map pulled from the list. It then determines the stair type based on the offset (which
/// direction the player is going, up or down). It then finds the desired stairs type  on the map
/// and places the player at it.
/// (I dont remember why there is a Point resource in the ecs, but also a position). Finally,
/// it marks the viewshed as dirty
fn transition_to_existing_map(ecs: &mut World, new_depth: i32, offset: i32) {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    let map = dungeon_master.get_map(new_depth).unwrap();
    let mut worldmap_resource = ecs.write_resource::<Map>();
    let player_entity = ecs.fetch::<Entity>();

    let down_stairs = map
        .find_first_tile_xy(if offset < 0 {
            TileType::DownStairs
        } else {
            TileType::UpStairs
        })
        .expect("Could not find down stairs on map");

    let mut player_pos = ecs.write_resource::<Point>();
    *player_pos = Point::new(down_stairs.0 as i32, down_stairs.1 as i32);
    let mut position_comp = ecs.write_storage::<Position>();
    let player_pos_comp = position_comp.get_mut(*player_entity);
    if let Some(position) = player_pos_comp {
        position.x = down_stairs.0 as i32;
        position.y = down_stairs.1 as i32;
    }

    *worldmap_resource = map;

    let mut viewshed_comp = ecs.write_storage::<Viewshed>();
    let vs = viewshed_comp.get_mut(*player_entity);
    if let Some(vs) = vs {
        vs.dirty = true;
    }
}
