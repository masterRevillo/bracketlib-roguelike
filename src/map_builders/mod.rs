mod simple_map;
mod common;

use specs::World;
use crate::components::Position;
use crate::map_builders::simple_map::SimpleMapBuilder;
use super::Map;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(new_depth))
}