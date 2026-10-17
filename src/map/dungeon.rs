use serde::{Deserialize, Serialize};
use specs::{Entity, Join, World, WorldExt};
use std::collections::HashMap;

use crate::{
    components::{OtherLevelPosition, Position},
    map::Map,
};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps: HashMap<i32, Map>,
}

impl MasterDungeonMap {
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
        }
    }

    pub fn store_map(&mut self, map: &Map) {
        self.maps.insert(map.depth, map.clone());
    }

    pub fn get_map(&self, depth: i32) -> Option<Map> {
        if self.maps.contains_key(&depth) {
            let mut result = self.maps[&depth].clone();
            result.tile_content =
                vec![vec![Vec::new(); result.height as usize]; result.width as usize];
            Some(result)
        } else {
            None
        }
    }
}

/// Moves entities from the current level (stored with Position components) to storage by
/// removing the component and adding an OtherLevelPosition for each, tagging the level
pub fn freeze_level_entities(ecs: &mut World) {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player = ecs.fetch::<Entity>();
    let map_depth = ecs.fetch::<Map>().depth;

    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &positions).join() {
        if entity != *player {
            other_level_positions
                .insert(
                    entity,
                    OtherLevelPosition {
                        x: pos.x,
                        y: pos.y,
                        depth: map_depth,
                    },
                )
                .expect("Could not insert");
            pos_to_delete.push(entity);
        }
    }

    for p in pos_to_delete.iter() {
        positions.remove(*p);
    }
}

/// Move entities stored on another level into the current by removing their OtherLevelPosition
/// comp and replacing with a Position comp
pub fn thaw_level_entities(ecs: &mut World) {
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player = ecs.fetch::<Entity>();
    let map_depth = ecs.fetch::<Map>().depth;

    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &other_level_positions).join() {
        if entity != *player && pos.depth == map_depth {
            positions
                .insert(entity, Position { x: pos.x, y: pos.y })
                .expect("Could not insert");
            pos_to_delete.push(entity);
        }
    }

    for p in pos_to_delete.iter() {
        other_level_positions.remove(*p);
    }
}
