use bracket_lib::random::RandomNumberGenerator;
use specs::World;

use crate::components::Position;
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::cellular_automata::CellularAutomataBuilder;
use crate::map_builders::dla::DLABuilder;
use crate::map_builders::drunkards::DrunkardsWalkBuilder;
use crate::map_builders::maze::MazeBuilder;
use crate::map_builders::prefab_builder::prefab_sections::UNDERGROUND_FORT;
use crate::map_builders::prefab_builder::PrefabBuilder;
use crate::map_builders::simple_map::SimpleMapBuilder;
use crate::map_builders::voronoi::VoronoiCellBuilder;
use crate::map_builders::waveform_collapse::WaveformCollapseBuilder;
use crate::spawner::{spawn_debug_items, spawn_entity, SpawnList};

use super::Map;

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

pub trait MapBuilder {
    fn build_map(&mut self, ecs: &mut World);
    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawn_entity(ecs, &(&entity.0, &entity.1));
        }
        spawn_debug_items(ecs, self.get_starting_position());
    }
    fn get_spawn_list(&self) -> &SpawnList;
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 17);
    let mut result: Box<dyn MapBuilder>;
    result = match builder {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(BspInteriorBuilder::new(new_depth)),
        3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
        5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        7 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
        8 => Box::new(DrunkardsWalkBuilder::symmetrical_passages(new_depth)),
        9 => Box::new(DrunkardsWalkBuilder::crazy_beer_goggles(new_depth)),
        10 => Box::new(MazeBuilder::new(new_depth)),
        11 => Box::new(DLABuilder::walk_inward(new_depth)),
        12 => Box::new(DLABuilder::walk_outward(new_depth)),
        13 => Box::new(DLABuilder::insectoid(new_depth)),
        14 => Box::new(DLABuilder::central_attractor(new_depth)),
        15 => Box::new(DLABuilder::walk_inwards_symmetry(new_depth)),
        16 => Box::new(DLABuilder::walk_outward_symmetry(new_depth)),
        17 => Box::new(VoronoiCellBuilder::pythagoras(new_depth)),
        18 => Box::new(VoronoiCellBuilder::manhattan(new_depth)),
        19 => Box::new(VoronoiCellBuilder::chebyshev(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    };
    if rng.roll_dice(1,3) == 1 {
        result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    }
    if rng.roll_dice(1,4) == 1 {
        result = Box::new(PrefabBuilder::sectional(new_depth, UNDERGROUND_FORT, result));
    }
    result = Box::new(PrefabBuilder::vaults(new_depth, result));

    result
}
