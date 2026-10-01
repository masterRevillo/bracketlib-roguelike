use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::map::Map;

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
