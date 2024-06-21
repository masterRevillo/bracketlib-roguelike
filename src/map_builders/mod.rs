mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod drunkards;
mod simple_map;

use super::Map;
use crate::components::Position;
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::cellular_automata::CellularAutomataBuilder;
use crate::map_builders::drunkards::{DrunkSpawnMode, DrunkardSettings, DrunkardsWalkBuilder};
use crate::map_builders::simple_map::SimpleMapBuilder;
use bracket_lib::prelude::console;
use bracket_lib::random::RandomNumberGenerator;
use specs::World;

pub trait MapBuilder {
    fn build_map(&mut self, ecs: &mut World);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 7);
    match builder {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(BspInteriorBuilder::new(new_depth)),
        3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
        5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth)),
    }
    Box::new(DrunkardsWalkBuilder::new(
        new_depth,
        DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            dr_lifetime: 100,
            floor_percentage: 0.4,
        },
    ))
}
