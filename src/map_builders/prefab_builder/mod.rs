use std::collections::HashSet;

use bracket_lib::prelude::{console, XpFile};
use bracket_lib::random::RandomNumberGenerator;

use crate::components::Position;
use crate::map::tiletype::TileType;
use crate::map_builders::{BuilderMap, InitialMapBuilder, MetaMapBuilder};
use crate::map_builders::prefab_builder::prefab_levels::PrefabLevel;
use crate::map_builders::prefab_builder::prefab_rooms::{CHECKERBOARD, CHICKFILA, PrefabRoom, TRAP, WELL};
use crate::map_builders::prefab_builder::prefab_sections::{HorizontalPlacement, PrefabSection, VerticalPlacement};

pub mod prefab_levels;
pub mod prefab_sections;
pub mod prefab_rooms;

#[derive(PartialEq, Clone)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
    Constant { level: PrefabLevel },
    Sectional { section: PrefabSection },
    RoomVaults,
}

pub struct PrefabBuilder {
    mode: PrefabMode,
}

impl InitialMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
       self.build(rng, build_data);
    }
}

impl PrefabBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            mode: PrefabMode::RoomVaults,
        })
    }

    pub fn rex_level(template: &'static str) -> Box<Self> {
        Box::new(Self {
            mode: PrefabMode::RexLevel {template},
        })
    }

    pub fn constant(level: PrefabLevel) -> Box<Self> {
        Box::new(Self {
            mode: PrefabMode::Constant {level},
        })
    }

    pub fn sectional(section: PrefabSection) -> Box<Self> {
        Box::new(Self {
            mode: PrefabMode::Sectional {section},
        })
    }

    pub fn vaults() -> Box<Self> {
        Box::new(Self {
            mode: PrefabMode::RoomVaults,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template, build_data),
            PrefabMode::Constant { level } => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data)
        }
        build_data.take_snapshot();
    }

    fn char_to_map(&mut self, ch: char, x: usize, y: usize, build_data: &mut BuilderMap) {
        match ch {
            ' ' | '.' => build_data.map.tiles[x][y] = TileType::Floor,
            '#' => build_data.map.tiles[x][y] = TileType::Wall,
            '@' => {
                build_data.map.tiles[x][y] = TileType::Floor;
                build_data.starting_position = Some(Position { x: x as i32, y: y as i32 });
            }
            '>' => build_data.map.tiles[x][y] = TileType::DownStairs,
            'b' => {
                build_data.map.tiles[x][y] = TileType::Floor;
                build_data.spawn_list.push(((x as i32, y as i32), "Bisat".to_string()));
            }
            'o' => {
                build_data.map.tiles[x][y] = TileType::Floor;
                build_data.spawn_list.push(((x as i32, y as i32), "Ogur".to_string()));
            }
            '^' => {
                build_data.map.tiles[x][y] = TileType::Floor;
                build_data.spawn_list.push(((x as i32, y as i32), "Bear Trap".to_string()));
            }
            '=' | '%' => {
                build_data.map.tiles[x][y] = TileType::Floor;
                build_data.spawn_list.push(((x as i32, y as i32), "Sandwich".to_string()));
            }
            'q' => {
                build_data.map.tiles[x][y] = TileType::Floor;
                build_data.spawn_list.push(((x as i32, y as i32), "Chicken Leg".to_string()));
            }
            'u' => {
                build_data.map.tiles[x][y] = TileType::Floor;
                build_data.spawn_list.push(((x as i32, y as i32), "Goblet Of Wine".to_string()));
            }
            '!' => {
                build_data.map.tiles[x][y] = TileType::Floor;
                build_data.spawn_list.push(((x as i32, y as i32), "Health Potion".to_string()));
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

    fn load_rex_map(&mut self, path: &str, build_data: &mut BuilderMap) {
        let xp_file = XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < build_data.map.width as usize && y < build_data.map.height as usize {
                        self.char_to_map(cell.ch as u8 as char, x, y, build_data);
                    }
                }
            }
        }
    }

    fn load_ascii_map(&mut self, level: &PrefabLevel, build_data: &mut BuilderMap) {
        console::log(format!("loading map: {}", level.template));
        let mut string_vec = PrefabBuilder::read_ascii_to_vec(level.template);
        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize && i < string_vec.len() {
                    self.char_to_map(string_vec[i], tx, ty, build_data);
                }
                i += 1;
            }
        }
    }

    pub fn apply_sectional(&mut self, section: &PrefabSection, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);

        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (build_data.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (build_data.map.width - 1) - section.width as i32,
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (build_data.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (build_data.map.height - 1) - section.height as i32,
        }

        console::log(format!("selected position of prefab: {}, {}", chunk_x, chunk_y));

        self.apply_previous_iteration(
            |x, y| {
                x < chunk_x || x > (chunk_x + section.width as i32) ||
                    y < chunk_y || y > (chunk_y + section.height as i32)
            },
            rng,
            build_data
        );

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize && i < string_vec.len() {
                    // console::log(format!("placing char: {}", string_vec[i]));
                    self.char_to_map(string_vec[i], tx + chunk_x as usize, ty + chunk_y as usize, build_data);

                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }

    fn apply_previous_iteration<F>(&mut self, mut filter: F, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap)
        where F: FnMut(i32, i32) -> bool
    {
        build_data.spawn_list.retain(|(c, _type)| {
            filter(c.0, c.1)
        });
        build_data.take_snapshot();
    }

    fn apply_room_vaults(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut rng = RandomNumberGenerator::new();
        self.apply_previous_iteration( |_x, _y| true, &mut rng, build_data);

        let vault_roll = rng.roll_dice(1, 6) + build_data.map.depth;
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
            .filter(|v| { build_data.map.depth >= v.first_depth && build_data.map.depth <= v.last_depth })
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

            for y in 0..build_data.map.height as usize {
                for x in 0..build_data.map.width as usize {
                    if x + vault.width < (build_data.map.width - 2) as usize
                        && y + vault.height < (build_data.map.height - 2) as usize {
                        let mut possible = true;

                        for ty in 0..vault.height {
                            for tx in 0..vault.width {
                                if build_data.map.tiles[tx + x][ty + y] != TileType::Floor {
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

                build_data.spawn_list.retain(|e| {
                    let (x, y) = e.0;
                    (x as usize) < chunk_x || (x as usize) > chunk_x + vault.width
                        || (y as usize) < chunk_y || (y as usize) > chunk_y + vault.height
                });

                let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for ty in 0..vault.height {
                    for tx in 0..vault.width {
                        if i < string_vec.len() {
                            self.char_to_map(string_vec[i], tx + chunk_x, ty + chunk_y, build_data);
                            used_tiles.insert(
                                ((tx + chunk_x) as i32, (ty + chunk_y) as i32 )
                            );
                        }
                        i += 1;
                    }
                }
                build_data.take_snapshot();
                possible_vaults.remove(vault_idx);
            }
        }
    }
}
