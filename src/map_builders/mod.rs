mod bsp_dungeon;
mod bsp_interior;
mod cellular_automata;
mod common;
mod simple_map;

use super::Map;
use crate::components::Position;
use crate::map_builders::bsp_dungeon::BspDungeonBuilder;
use crate::map_builders::bsp_interior::BspInteriorBuilder;
use crate::map_builders::cellular_automata::CellularAutomataBuilder;
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
    // let mut rng = RandomNumberGenerator::new();
    // let builder = rng.roll_dice(1, 2);
    // match builder {
    //     1 => Box::new(BspDungeonBuilder::new(new_depth)),
    //     _ => Box::new(SimpleMapBuilder::new(new_depth)),
    // }
    Box::new(CellularAutomataBuilder::new(new_depth))
    // Box::new(SimpleMapBuilder::new(new_depth))
}
