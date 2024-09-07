use std::collections::HashSet;
use std::hash::Hash;

use bracket_lib::prelude::{a_star_search, console, DistanceAlg, Point};
use bracket_lib::random::RandomNumberGenerator;

use crate::components::Position;
use crate::map::TileType;
use crate::map::TileType::{Bridge, DownStairs, Floor, Road, Wall, WoodFloor};
use crate::map_builders::{BuilderChain, BuilderMap, InitialMapBuilder, random_start_position};
use crate::map_builders::area_starting_points::AreaStartingPoint;
use crate::map_builders::distant_exit::DistantExit;

#[derive(Debug)]
enum BuildingTag {
    Pub,
    Temple,
    Blacksmith,
    Clothier,
    Alchemist,
    PlayerHouse,
    Hovel,
    Abandoned,
    Unassigned,
}

pub fn level_builder(new_depth: i32, rng: &mut RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height);
    chain.start_with(TownBuilder::new());
    let (start_x, start_y) = random_start_position(rng);
    chain.with(AreaStartingPoint::new(start_x, start_y));
    chain.with(DistantExit::new());
    chain
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data);
    }
}

impl TownBuilder {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }

    pub fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.grass_layer(build_data);
        self.water_and_piers(rng, build_data);

        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &doors);

        build_data.map.tiles[(build_data.width - 5) as usize][wall_gap_y as usize] = DownStairs;

        let building_size = self.sort_buildings(&buildings);
        self.building_factory(rng, build_data, &buildings, &building_size);

        self.spawn_dockers(build_data, rng);
        self.spawn_townsfolk(build_data, rng, &mut available_building_tiles);

        for x in build_data.map.visible_tiles.iter_mut() {
            for t in x.iter_mut() {
                *t = true;
            }
        }
        build_data.take_snapshot();
    }

    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        for x in build_data.map.tiles.iter_mut() {
            for t in x.iter_mut() {
                *t = TileType::Grass
            }
        }
        build_data.take_snapshot();
    }

    fn water_and_piers(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535f32;
        let mut water_width: Vec<i32> = Vec::new();
        for y in 0..build_data.height {
            let n_water = (f32::sin(n) * 10.0) as i32 + 14 + rng.roll_dice(1, 4);
            water_width.push(n_water);
            n += 0.1;
            for x in 0..n_water {
                build_data.map.tiles[x as usize][y as usize] = TileType::DeepWater;
            }
            for x in n_water..n_water + rng.roll_dice(1, 3) {
                build_data.map.tiles[x as usize][y as usize] = TileType::ShallowWater;
            }
            build_data.take_snapshot();
        }
        for _i in 0..rng.roll_dice(1, 4) + 6 {
            let y = rng.roll_dice(1, build_data.height) - 1;
            for x in 2 + rng.roll_dice(1, 6)..water_width[y as usize] + 4 {
                build_data.map.tiles[x as usize][y as usize] = Bridge;
            }
        }
        build_data.take_snapshot();
    }


    fn town_walls(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) -> (HashSet<(i32, i32)>, i32) {
        let mut available_building_tiles: HashSet<(i32, i32)> = HashSet::new();
        let wall_gap_y = rng.roll_dice(1, build_data.height - 9) + 5;
        for y in 1..build_data.height - 2 {
            if !(y > wall_gap_y - 4 && y < wall_gap_y + 4) {
                build_data.map.tiles[30usize][y as usize] = Wall;
                build_data.map.tiles[29usize][y as usize] = Floor;
                build_data.map.tiles[(build_data.width - 2) as usize][y as usize] = Wall;
                for x in 31..build_data.width - 2 {
                    build_data.map.tiles[x as usize][y as usize] = TileType::Gravel;
                    if y > 2 && y < build_data.height - 1 {
                        available_building_tiles.insert((x, y));
                    }
                }
            } else {
                for x in 30..build_data.width {
                    build_data.map.tiles[x as usize][y as usize] = Road;
                }
            }
        }
        build_data.take_snapshot();

        for x in 30..build_data.width - 1 {
            build_data.map.tiles[x as usize][1usize] = Wall;
            build_data.map.tiles[x as usize][(build_data.height - 2) as usize] = Wall;
        }
        build_data.take_snapshot();

        (available_building_tiles, wall_gap_y)
    }

    fn buildings(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        available_building_tiles: &mut HashSet<(i32, i32)>,
    ) -> Vec<(i32, i32, i32, i32)> {
        let mut buildings: Vec<(i32, i32, i32, i32)> = Vec::new();
        let mut n_buildings = 0;
        while n_buildings < 12 {
            let bx = rng.roll_dice(1, build_data.map.width - 32) + 30;
            let by = rng.roll_dice(1, build_data.map.height) - 2;
            let bw = rng.roll_dice(1, 8) + 4;
            let bh = rng.roll_dice(1, 8) + 4;
            let mut possible = true;
            for y in by..by + bh {
                for x in bx..bx + bw {
                    if x < 0 || x > build_data.width - 1 || y < 0 || y > build_data.height - 1 {
                        possible = false;
                    } else {
                        if !available_building_tiles.contains(&(x, y)) {
                            possible = false;
                        }
                    }
                }
            }
            if possible {
                n_buildings += 1;
                buildings.push((bx, by, bw, bh));
                for y in by..by + bh {
                    for x in bx..bx + bw {
                        build_data.map.tiles[x as usize][y as usize] = WoodFloor;
                        available_building_tiles.remove(&(x, y));
                        available_building_tiles.remove(&(x - 1, y));
                        available_building_tiles.remove(&(x + 1, y));
                        available_building_tiles.remove(&(x, y - 1));
                        available_building_tiles.remove(&(x, y + 1));
                    }
                }
                build_data.take_snapshot();
            }
        }
        let mut mapclone = build_data.map.clone();
        for y in 2..(build_data.height - 2) as usize {
            for x in 32..(build_data.width - 2) as usize {
                if build_data.map.tiles[x][y] == WoodFloor {
                    let mut neighbors = 0;
                    if build_data.map.tiles[x - 1][y] != WoodFloor {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x + 1][y] != WoodFloor {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x][y - 1] != WoodFloor {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[x][y + 1] != WoodFloor {
                        neighbors += 1;
                    }
                    if neighbors > 0 {
                        mapclone.tiles[x][y] = Wall;
                    }
                }
            }
        }
        build_data.map = mapclone;
        build_data.take_snapshot();
        buildings
    }

    fn add_doors(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &mut Vec<(i32, i32, i32, i32)>,
        wall_gap_y: i32,
    ) -> Vec<(i32, i32)> {
        let mut doors = Vec::new();
        for building in buildings.iter() {
            let door_x = building.0 + 1 + rng.roll_dice(1, building.2 - 3);
            let cy = building.1 + (building.3 / 2);
            let door_y;
            if cy > wall_gap_y {
                door_y = building.1;
            } else {
                door_y = building.1 + building.3 - 1;
            }
            build_data.map.tiles[door_x as usize][door_y as usize] = Floor;
            build_data.spawn_list.push(((door_x, door_y), "Door".to_string()));
            doors.push((door_x, door_y));
        }
        build_data.take_snapshot();
        doors
    }

    fn add_paths(&mut self, build_data: &mut BuilderMap, doors: &Vec<(i32, i32)>) {
        let mut roads = Vec::new();
        for y in 0..build_data.height {
            for x in 0..build_data.width {
                if build_data.map.tiles[x as usize][y as usize] == Road {
                    roads.push((x, y));
                }
            }
        }
        build_data.map.populate_blocked();
        for dc in doors.iter() {
            let mut nearest_roads: Vec<(usize, f32)> = Vec::new();
            let door_pt = Point::new(dc.0, dc.1);
            for r in roads.iter() {
                nearest_roads.push((
                    build_data.map.xy_idx(r.0, r.1),
                    DistanceAlg::PythagorasSquared.distance2d(
                        door_pt,
                        Point::new(r.0, r.1),
                    )
                ));
            }
            nearest_roads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let destination = nearest_roads[0].0;
            let path = a_star_search(
                build_data.map.xy_idx(dc.0, dc.1),
                destination,
                &mut build_data.map,
            );
            if path.success {
                for step in path.steps.iter() {
                    let x = *step % build_data.width as usize;
                    let y = *step / build_data.width as usize;
                    build_data.map.tiles[x][y] = Road;
                    roads.push((x as i32, y as i32));
                }
            }
            build_data.take_snapshot();
        }
    }

    fn sort_buildings(&mut self, buildings: &[(i32, i32, i32, i32)]) -> Vec<(usize, i32, BuildingTag)> {
        let mut building_size: Vec<(usize, i32, BuildingTag)> = Vec::new();
        for (i, building) in buildings.iter().enumerate() {
            console::log(format!("pos: {},{}; size: {}x{}={}", building.0, building.1, building.2, building.3, building.2 * building.3));
            building_size.push((
                i,
                building.2 * building.3,
                BuildingTag::Unassigned
            ));
        }
        building_size.sort_by(|a, b| b.1.cmp(&a.1));
        building_size[0].2 = BuildingTag::Pub;
        console::log(format!("pub buidling index is {}", building_size[0].0));
        building_size[1].2 = BuildingTag::Temple;
        building_size[2].2 = BuildingTag::Blacksmith;
        building_size[3].2 = BuildingTag::Clothier;
        building_size[4].2 = BuildingTag::Alchemist;
        building_size[5].2 = BuildingTag::PlayerHouse;
        for b in building_size.iter_mut().skip(6) {
            b.2 = BuildingTag::Hovel;
        }
        let last_index = building_size.len() - 1;
        building_size[last_index].2 = BuildingTag::Abandoned;
        for b in building_size.iter() {
            console::log(format!("building with type {:?} has size {}", b.2, b.1))
        }
        building_size
    }

    fn building_factory(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &[(i32, i32, i32, i32)],
        building_index: &[(usize, i32, BuildingTag)],
    ) {
        for (_i, b_idx) in building_index.iter().enumerate() {
            let build_type = &b_idx.2;
            let building = buildings[b_idx.0];
            match build_type {
                BuildingTag::Pub => {
                    console::log(format!("building pub at {},{}, index is {}", building.0, building.1, b_idx.1));
                    self.build_pub(&building, build_data, rng)
                }
                BuildingTag::Temple => self.build_temple(&building, build_data, rng),
                BuildingTag::Blacksmith => self.build_smith(&building, build_data, rng),
                BuildingTag::Clothier => self.build_clothier(&building, build_data, rng),
                BuildingTag::Alchemist => self.build_alchemist(&building, build_data, rng),
                BuildingTag::PlayerHouse => self.build_player_house(&building, build_data, rng),
                BuildingTag::Hovel => self.build_hovel(&building, build_data, rng),
                BuildingTag::Abandoned => self.build_abandoned_house(&building, build_data, rng),
                _ => {}
            }
        }
    }

    fn random_building_spawn(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        to_place: &mut Vec<&str>,
        player_pos: (i32, i32),
    ) {
        for y in building.1..building.1 + building.3 {
            for x in building.0..building.0 + building.2 {
                if build_data.map.tiles[x as usize][y as usize] == WoodFloor
                    && (x, y) != player_pos
                    && rng.roll_dice(1, 3) == 1
                    && !to_place.is_empty() {
                    let entity_tag = to_place[0].to_string();
                    to_place.remove(0);
                    build_data.spawn_list.push(
                        ((x, y), entity_tag)
                    )
                }
            }
        }
    }
    fn build_pub(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        let (px, py) = (
            building.0 + (building.2 / 2),
            building.1 + (building.3 / 2)
        );
        build_data.starting_position = Some(Position { x: px, y: py });

        let mut to_place: Vec<&str> = vec![
            "Bartender", "ShadySalesman", "Patron", "Patron", "Keg", "Table", "Chair", "Table", "Chair",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, (px, py));
    }

    fn build_temple(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut to_place: Vec<&str> = vec![
            "Priest", "Parishioner", "Parishioner", "Chair", "Chair", "Candle", "Candle",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, (0, 0));
    }

    fn build_smith(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut to_place: Vec<&str> = vec![
            "Blacksmith", "Anvil", "WaterTrough", "WeaponRack", "ArmorStand",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, (0, 0));
    }

    fn build_clothier(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut to_place: Vec<&str> = vec![
            "Clothier", "Cabinet", "Table", "Loom", "HideRack",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, (0, 0));
    }

    fn build_alchemist(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut to_place: Vec<&str> = vec![
            "Alchemist", "ChemistrySet", "DeadThing", "Chair", "Table",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, (0, 0));
    }

    fn build_player_house(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut to_place: Vec<&str> = vec![
            "Mom", "Bed", "Cabinet", "Chair", "Table",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, (0, 0));
    }
    fn build_hovel(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut to_place: Vec<&str> = vec![
            "Peasant", "Bed", "Chair", "Table",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, (0, 0));
    }
    fn build_abandoned_house(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        for y in building.1..building.1 + building.3 {
            for x in building.0..building.0 + building.2 {
                if build_data.map.tiles[x as usize][y as usize] == WoodFloor
                    && rng.roll_dice(1, 2) == 1 {
                    build_data.spawn_list.push(((x, y), "Rat".to_string()))
                }
            }
        }
    }

    fn spawn_dockers(&mut self, build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        for (x, row) in build_data.map.tiles.iter().enumerate() {
            for (y, t) in row.iter().enumerate() {
                if *t == Bridge && rng.roll_dice(1, 6) == 1 {
                    let roll = rng.roll_dice(1, 3);
                    let et = match roll {
                        1 => "DockWorker",
                        2 => "Pirate",
                        _ => "Fisher"
                    };
                    build_data.spawn_list.push(((x as i32, y as i32), et.to_string()));
                }
            }
        }
    }

    fn spawn_townsfolk(
        &mut self,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        available_building_tiles: &mut HashSet<(i32, i32)>,
    ) {
        for (x, y) in available_building_tiles.iter() {
            if rng.roll_dice(1, 30) == 1 {
                let roll = rng.roll_dice(1, 4);
                let et = match roll {
                    1 => "Peasant",
                    2 => "Drunk",
                    3 => "DockWorker",
                    _ => "Fisher"
                };
                build_data.spawn_list.push(((*x, *y), et.to_string()));
            }
        }
    }
}
