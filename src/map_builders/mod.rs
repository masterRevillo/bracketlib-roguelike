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
use crate::map_builders::prefab_builder::prefab_sections::{NESTED_ROOMS, UNDERGROUND_FORT};
use crate::map_builders::prefab_builder::PrefabBuilder;
use crate::map_builders::room_based_spawner::RoomBasedSpawner;
use crate::map_builders::room_based_stairs::RoomBasedStairs;
use crate::map_builders::room_based_starting_position::RoomBasedStartingPosition;
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

pub fn random_builder(depth: i32, rng: &mut RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(depth);
    let (starter, has_rooms) = random_initial_builder(rng);
    builder.start_with(starter);
    if has_rooms {
        builder.with(RoomBasedSpawner::new());
        builder.with(RoomBasedStairs::new());
        builder.with(RoomBasedStartingPosition::new());
    } else {
        builder.with(AreaStartingPoint::new(XStart::CENTER, YStart::CENTER));
        builder.with(CullUnreachable::new());
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }
    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());
    }
    if rng.roll_dice(1, 20) == 1 {
        if rng.roll_dice(1, 2) == 1 {
            builder.with(PrefabBuilder::sectional(UNDERGROUND_FORT));
        } else {
            builder.with(PrefabBuilder::sectional(NESTED_ROOMS));
        }
    }
    builder.with(PrefabBuilder::vaults());
    builder
}
