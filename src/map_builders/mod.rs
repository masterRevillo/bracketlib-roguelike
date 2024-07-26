use std::any::{Any, type_name_of_val};
use bracket_lib::prelude::console;
use bracket_lib::random::RandomNumberGenerator;
use specs::{Builder, World};

use crate::components::Position;
use crate::map_builders::area_starting_points::{AreaStartingPoint, XStart, YStart};
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::cellular_automata::CellularAutomataBuilder;
use crate::map_builders::cull_unreachable::CullUnreachable;
use crate::map_builders::distant_exit::DistantExit;
use crate::map_builders::dla::DLABuilder;
use crate::map_builders::drunkards::DrunkardsWalkBuilder;
use crate::map_builders::maze::MazeBuilder;
use crate::map_builders::prefab_builder::prefab_levels::WFC_POPULATED;
use crate::map_builders::prefab_builder::prefab_sections::UNDERGROUND_FORT;
use crate::map_builders::prefab_builder::PrefabBuilder;
use crate::map_builders::room_based_spawner::RoomBasedSpawner;
use crate::map_builders::room_based_stairs::RoomBasedStairs;
use crate::map_builders::room_based_starting_position::RoomBasedStartingPosition;
use crate::map_builders::room_corner_rounding::RoomCornerRounding;
use crate::map_builders::room_corridors_bsp::BSPCorridors;
use crate::map_builders::room_corridors_dogleg::DoglegCorridors;
use crate::map_builders::room_exploder::RoomExploder;
use crate::map_builders::room_sorter::RoomSorter;
use crate::map_builders::simple_map::SimpleMapBuilder;
use crate::map_builders::voronoi::VoronoiCellBuilder;
use crate::map_builders::voronoi_spawning::VoronoiSpawning;
use crate::map_builders::waveform_collapse::WaveformCollapseBuilder;
use crate::rect::Rect;
use crate::spawner::{spawn_debug_items, spawn_entity, SpawnList};

use super::{Map, SHOW_MAPGEN_VISUALIZATION};

mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod dla;
mod drunkards;
mod maze;
mod simple_map;
mod voronoi;
mod waveform_collapse;
mod prefab_builder;
mod room_based_spawner;
mod room_based_starting_position;
mod room_based_stairs;
mod area_starting_points;
mod cull_unreachable;
mod voronoi_spawning;
mod distant_exit;
mod room_exploder;
mod room_corner_rounding;
mod room_corridors_dogleg;
mod room_corridors_bsp;
mod room_sorter;

pub struct BuilderMap {
    pub spawn_list: SpawnList,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>
}

impl BuilderMap {
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

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap
}

impl BuilderChain {
    pub fn new(depth: i32) -> Self {
        Self {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(depth),
                starting_position: None,
                rooms: None,
                history: Vec::new()
            }
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("Only one starting builder is allowed")
        };
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder)
    }

    pub fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Missing starting map builder"),
            Some(starter) => {
                starter.build_map(rng, &mut self.build_data);
            }
        }

        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
            console::log(format!("Adding metabuilder {}", type_name_of_val(metabuilder)))
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawn_entity(ecs, &(&entity.0, &entity.1));
        }
        match &self.build_data.starting_position {
            None => {},
            Some(p) => spawn_debug_items(ecs, p)
        }
    }

}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}
pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub fn random_initial_builder(rng: &mut RandomNumberGenerator) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 20);
    let mut result: (Box<dyn InitialMapBuilder>, bool);
    result = match builder {
        1 => (BspDungeonBuilder::new(), true),
        2 => (BspInteriorBuilder::new(), true),
        3 => (CellularAutomataBuilder::new(), false),
        4 => (DrunkardsWalkBuilder::open_area(), false),
        5 => (DrunkardsWalkBuilder::open_halls(), false),
        6 => (DrunkardsWalkBuilder::winding_passages(), false),
        7 => (DrunkardsWalkBuilder::fat_passages(), false),
        8 => (DrunkardsWalkBuilder::symmetrical_passages(), false),
        9 => (DrunkardsWalkBuilder::crazy_beer_goggles(), false),
        10 => (MazeBuilder::new(), false),
        11 => (DLABuilder::walk_inward(), false),
        12 => (DLABuilder::walk_outward(), false),
        13 => (DLABuilder::insectoid(), false),
        14 => (DLABuilder::central_attractor(), false),
        15 => (DLABuilder::walk_inwards_symmetry(), false),
        16 => (DLABuilder::walk_outward_symmetry(), false),
        17 => (VoronoiCellBuilder::pythagoras(), false),
        18 => (VoronoiCellBuilder::manhattan(), false),
        19 => (VoronoiCellBuilder::chebyshev(), false),
        20 => (PrefabBuilder::constant(WFC_POPULATED), false),
        _ => (SimpleMapBuilder::new(), true),
    };
    result
}

fn random_room_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 3);
    match build_roll {
        1 => builder.start_with(SimpleMapBuilder::new()),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new())
    }

    if build_roll != 3 {
        match rng.roll_dice(1, 5) {
            1 => builder.with(RoomSorter::leftmost()),
            2 => builder.with(RoomSorter::rightmost()),
            3 => builder.with(RoomSorter::topmost()),
            4 => builder.with(RoomSorter::bottommost()),
            _ => builder.with(RoomSorter::central()),
        }

        match rng.roll_dice(1,2) {
            1 => builder.with(DoglegCorridors::new()),
            _ => builder.with(BSPCorridors::new())
        }

        match rng.roll_dice(1, 6) {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounding::new()),
            _ => {}
        }
    }

    match rng.roll_dice(1, 2) {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPoint::new(start_x, start_y))
        }
    }

    match rng.roll_dice(1, 2) {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new())
    }

    match rng.roll_dice(1, 2) {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new())
    }
}

fn random_start_position(rng: &mut RandomNumberGenerator) -> (XStart, YStart) {
    let x = match rng.roll_dice(1, 3) {
        1 => XStart::LEFT,
        2 => XStart::RIGHT,
        _ => XStart::CENTER,
    };
    let y = match rng.roll_dice(1, 3) {
        1 => YStart::TOP,
        2 => YStart::BOTTOM,
        _ => YStart::CENTER,
    };
    (x, y)
}

fn random_shape_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
   match rng.roll_dice(1, 16) {
       1 => builder.start_with(CellularAutomataBuilder::new()),
       2 => builder.start_with(DrunkardsWalkBuilder::open_area()),
       3 => builder.start_with(DrunkardsWalkBuilder::open_halls()),
       4 => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
       5 => builder.start_with(DrunkardsWalkBuilder::symmetrical_passages()),
       6 => builder.start_with(DrunkardsWalkBuilder::crazy_beer_goggles()),
       7 => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
       8 => builder.start_with(MazeBuilder::new()),
       9 => builder.start_with(DLABuilder::central_attractor()),
       10 => builder.start_with(DLABuilder::insectoid()),
       11 => builder.start_with(DLABuilder::walk_inward()),
       12 => builder.start_with(DLABuilder::walk_outward()),
       13 => builder.start_with(VoronoiCellBuilder::pythagoras()),
       14 => builder.start_with(VoronoiCellBuilder::manhattan()),
       _ => builder.start_with(PrefabBuilder::constant(WFC_POPULATED)),
   }
    builder.with(AreaStartingPoint::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());

    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPoint::new(start_x, start_y));
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}

pub fn random_builder(depth: i32, rng: &mut RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(depth);
    match rng.roll_dice(1, 2) {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder)
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());
    }
    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(UNDERGROUND_FORT));
    }
    builder.with(PrefabBuilder::vaults());

    builder
}
