use specs::prelude::*;

use crate::components::{BlocksTile, Position};
use crate::map::Map;

pub struct MapIndexingSystem {}

impl <'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for(entity, position) in (&entities, &position).join() {
            let _p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[position.x as usize][position.y as usize] = true;
            }
            map.tile_content[position.x as usize][position.y as usize].push(entity)
        }
    }
}
