use std::collections::HashSet;
use bracket_lib::prelude::{console, XpFile};
use bracket_lib::random::RandomNumberGenerator;
use specs::World;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::find_most_distant_tile;
use crate::map_builders::MapBuilder;
use crate::map_builders::prefab_builder::prefab_levels::PrefabLevel;
use crate::map_builders::prefab_builder::prefab_rooms::{CHECKERBOARD, CHICKFILA, PrefabRoom, TRAP, WELL};
use crate::map_builders::prefab_builder::prefab_sections::{HorizontalPlacement, PrefabSection, VerticalPlacement};
use crate::random_tables::EntryType;
use crate::SHOW_MAPGEN_VISUALIZATION;
use crate::spawner::SpawnList;

mod prefab_levels;
pub mod prefab_sections;
mod prefab_rooms;

#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
    Constant { level: PrefabLevel },
    Sectional { section: PrefabSection },
    RoomVaults,
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    previous_builder: Option<Box<dyn MapBuilder>>,
    spawn_list: SpawnList,
}

impl MapBuilder for PrefabBuilder {
    fn build_map(&mut self, ecs: &mut World) {
        self.build(ecs)
    }

    fn get_spawn_list(&self) -> &SpawnList {
        &self.spawn_list
    }

    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
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
}

impl PrefabBuilder {
    pub fn new(depth: i32, previous_builder: Option<Box<dyn MapBuilder>>) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            mode: PrefabMode::RoomVaults,
            previous_builder,
            spawn_list: Vec::new(),
        }
    }

    pub fn rex_level(depth: i32, template: &'static str) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            mode: PrefabMode::RexLevel {template},
            previous_builder: None,
            spawn_list: Vec::new(),
        }
    }

    pub fn constant(depth: i32, level: PrefabLevel) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            mode: PrefabMode::Constant {level},
            previous_builder: None,
            spawn_list: Vec::new(),
        }
    }

    pub fn sectional(depth: i32, section: PrefabSection, previous_builder: Box<dyn  MapBuilder>) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            mode: PrefabMode::Sectional {section},
            previous_builder: Some(previous_builder),
            spawn_list: Vec::new(),
        }
    }

    pub fn vaults(depth: i32, previous_builder: Box<dyn  MapBuilder>) -> Self {
        Self {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            depth,
            history: Vec::new(),
            mode: PrefabMode::RoomVaults,
            previous_builder: Some(previous_builder),
            spawn_list: Vec::new(),
        }
    }



    fn build(&mut self, ecs: &mut World) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, ecs),
            PrefabMode::RoomVaults => self.apply_room_vaults(ecs)
        }
        self.take_snapshot();

        if self.starting_position.x == 0 {
            self.starting_position = Position { x: self.map.width / 2, y: self.map.height / 2 };
            while self.map.tiles[self.starting_position.x as usize][self.starting_position.y as usize]
                != TileType::Floor
            {
                self.starting_position.x -= 1;
                if self.starting_position.x < 1 {
                    self.starting_position.x = self.map.width - 2;
                    self.starting_position.y -= 1;
                }
                if self.starting_position.y < 1 {
                    self.starting_position.y = self.map.height - 2;
                }
                console::log(format!("moving starting pos to {}, {}", self.starting_position.x, self.starting_position.y));
            }
        }
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        let _exit_tile = find_most_distant_tile(&mut self.map, start_idx);
        self.take_snapshot();
    }

    fn char_to_map(&mut self, ch: char, x: usize, y: usize) {
        match ch {
            ' ' | '.' => self.map.tiles[x][y] = TileType::Floor,
            '#' => self.map.tiles[x][y] = TileType::Wall,
            '@' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.starting_position = Position { x: x as i32, y: y as i32 };
            }
            '>' => self.map.tiles[x][y] = TileType::DownStairs,
            'b' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::Bisat));
            }
            'o' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::Ogur));
            }
            '^' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::BearTrap));
            }
            'o' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::Ogur));
            }
            '=' | '%' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::Sandwich));
            }
            'q' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::ChickenLeg));
            }
            'u' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::GobletOfWine));
            }
            '!' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::HealthPotion));
            }
            w => { console::log(format!("unknown glyph loading map {}", w)) }
        }
    }

    fn read_ascii_to_vec(ascii: &str) -> Vec<char> {
        console::log(format!("Reading ascii: {}", ascii));
        let mut string_vec: Vec<char> = ascii.chars().filter(|a| *a != '\r' && *a != '\n').collect();
        for c in string_vec.iter_mut() {
            if *c as u8 == 160u8 {
                *c = ' ';
            }
        }
        console::log(format!("filtered and converted: {:?}", string_vec));
        string_vec
    }

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        self.char_to_map(cell.ch as u8 as char, x, y);
                    }
                }
            }
        }
    }

    fn load_ascii_map(&mut self, level: &PrefabLevel) {
        console::log(format!("loading map: {}", level.template));
        let mut string_vec = PrefabBuilder::read_ascii_to_vec(level.template);
        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize && i < string_vec.len() {
                    self.char_to_map(string_vec[i], tx, ty);
                }
                i += 1;
            }
        }
    }

    pub fn apply_sectional(&mut self, section: &PrefabSection, ecs: &mut World) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);

        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (self.map.width - 1) - section.width as i32,
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (self.map.height - 1) - section.height as i32,
        }

        console::log(format!("selected position of prefab: {}, {}", chunk_x, chunk_y));

        self.apply_previous_iteration(
            ecs,
            |x, y, e| {
                x < chunk_x || x > (chunk_x + section.width as i32) ||
                    y < chunk_y || y > (chunk_y + section.height as i32)
            },
        );

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < self.map.width as usize && ty < self.map.height as usize && i < string_vec.len() {
                    // console::log(format!("placing char: {}", string_vec[i]));
                    self.char_to_map(string_vec[i], tx + chunk_x as usize, ty + chunk_y as usize);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }

    fn apply_previous_iteration<F>(&mut self, ecs: &mut World, mut filter: F)
        where F: FnMut(i32, i32, &((i32, i32), EntryType)) -> bool
    {
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map(ecs);
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map().clone();
        for e in prev_builder.get_spawn_list().iter() {
            let (x, y) = e.0;
            if filter(x, y, e) {
                self.spawn_list.push(
                    (e.0, e.1)
                );
            }
        }
        self.history.append(&mut prev_builder.get_snapshot_history());
        self.take_snapshot();
    }

    fn apply_room_vaults(&mut self, ecs: &mut World) {
        let mut rng = RandomNumberGenerator::new();
        self.apply_previous_iteration(ecs, |_x, _y, _e| true);

        let vault_roll = rng.roll_dice(1, 6) + self.depth;
        if vault_roll < 4 {
            return;
        }

        let master_vault_list = vec![
            TRAP,
            CHICKFILA,
            CHECKERBOARD,
            WELL
        ];
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| { self.depth >= v.first_depth && self.depth <= v.last_depth })
            .collect();

        if possible_vaults.is_empty() {
            return;
        }

        let n_valuts = i32::min(rng.roll_dice(1, 3), possible_vaults.len() as i32);
        let mut used_tiles: HashSet<(i32, i32)> = HashSet::new();

        for _i in 0..n_valuts {

            let vault_idx = if possible_vaults.len() == 1 { 0 } else { rng.roll_dice(1, possible_vaults.len() as i32 - 1) as usize };
            let vault = possible_vaults[vault_idx];

            let mut vault_position: Vec<Position> = Vec::new();

            for y in 0..self.map.height as usize {
                for x in 0..self.map.width as usize {
                    if x + vault.width < (self.map.width - 2) as usize
                        && y + vault.height < (self.map.height - 2) as usize {
                        let mut possible = true;

                        for ty in 0..vault.height {
                            for tx in 0..vault.width {
                                if self.map.tiles[tx + x][ty + y] != TileType::Floor {
                                    possible = false;
                                }
                                if used_tiles.contains(&((tx + x) as i32, (ty + y) as i32)) {

                                }
                            }
                        }

                        if possible {
                            vault_position.push(Position { x: x as i32, y: y as i32 })
                        }
                    }
                }
            }

            if !vault_position.is_empty() {
                let pos_idx = if vault_position.len() == 1 {
                    0
                } else {
                    (rng.roll_dice(1, vault_position.len() as i32) - 1) as usize
                };
                let pos = &vault_position[pos_idx];
                console::log(format!("picked a vault at {:?}", pos));

                let chunk_x = pos.x as usize;
                let chunk_y = pos.y as usize;

                self.spawn_list.retain(|e| {
                    let (x, y) = e.0;
                    (x as usize) < chunk_x || (x as usize) > chunk_x + vault.width
                        || (y as usize) < chunk_y || (y as usize) > chunk_y + vault.height
                });

                let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for ty in 0..vault.height {
                    for tx in 0..vault.width {
                        if i < string_vec.len() {
                            self.char_to_map(string_vec[i], tx + chunk_x, ty + chunk_y);
                            used_tiles.insert(
                                ((tx + chunk_x) as i32, (ty + chunk_y) as i32 )
                            );
                        }
                        i += 1;
                    }
                }
                self.take_snapshot();
                possible_vaults.remove(vault_idx);
            }
        }
    }
}
