use bracket_lib::prelude::{console, XpFile};
use specs::World;

use crate::components::Position;
use crate::map::{Map, TileType};
use crate::map_builders::common::find_most_distant_tile;
use crate::map_builders::MapBuilder;
use crate::map_builders::prefab_builder::prefab_levels::PrefabLevel;
use crate::map_builders::prefab_builder::prefab_sections::{HorizontalPlacement, NESTED_ROOMS, PrefabSection, UNDERGROUND_FORT, VerticalPlacement};
use crate::random_tables::EntryType;
use crate::SHOW_MAPGEN_VISUALIZATION;
use crate::spawner::SpawnList;

mod prefab_levels;
mod prefab_sections;

#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel{ template: &'static str },
    Constant{ level: PrefabLevel },
    Sectional{ section: PrefabSection }
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    previous_builder: Option<Box<dyn MapBuilder>>,
    spawn_list: SpawnList
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
        Self { map: Map::new(depth),
            starting_position: Position{ x: 0, y: 0},
            depth,
            history: Vec::new(),
            mode: PrefabMode::Sectional { section: NESTED_ROOMS },
            previous_builder,
            spawn_list: Vec::new()
        }
    }

    fn build(&mut self, ecs: &mut World) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, ecs),
        }
        self.take_snapshot();

        if self.starting_position.x == 0 {
            self.starting_position = Position{ x: self.map.width/2, y: self.map.height/2 };
            while self.map.tiles[self.starting_position.x as usize][self.starting_position.y as usize]
                != TileType::Floor
            {
                self.starting_position.x -= 1;
                if self.starting_position.x < 1 {
                    self.starting_position.x = self.map.width - 2;
                    self.starting_position.y -= 1;
                }
                if self.starting_position.y < 1 {
                    self.starting_position.y = self.map.height-2;
                }
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
            ' ' => self.map.tiles[x][y] = TileType::Floor,
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
            'q'  => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::ChickenLeg));
            }
            'u'  => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::GobletOfWine));
            }
            '!' => {
                self.map.tiles[x][y] = TileType::Floor;
                self.spawn_list.push(((x as i32, y as i32), EntryType::HealthPotion));
            }
            w => {console::log(format!("unknown glyph loading map {}", w))}
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
            HorizontalPlacement::Center => chunk_x = (self.map.width/2) - (section.width as i32/2),
            HorizontalPlacement::Right => chunk_x = (self.map.width-1) - section.width as i32,
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (self.map.height/2) - (section.height as i32/2),
            VerticalPlacement::Bottom => chunk_y = (self.map.height-1) - section.height as i32,
        }

        console::log(format!("selected position of prefab: {}, {}", chunk_x, chunk_y));

        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map(ecs);
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map().clone();
        for e in prev_builder.get_spawn_list().iter() {
            let (x,y) = e.0;
            if x < chunk_x || x > (chunk_x + section.width as i32) ||
                y < chunk_y || y > (chunk_y + section.height as i32) {
                self.spawn_list.push(
                    (e.0, e.1)
                );
            }
        }
        self.take_snapshot();

        let mut i = 0;
        for ty in 0..section.height {
            for tx  in 0..section.width {
                if tx < self.map.width as usize && ty < self.map.height as usize && i < string_vec.len() {
                    // console::log(format!("placing char: {}", string_vec[i]));
                    self.char_to_map(string_vec[i], tx + chunk_x as usize, ty + chunk_y as usize);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }
}
