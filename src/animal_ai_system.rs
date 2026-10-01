use std::usize;

use bracket_lib::prelude::{a_star_search, DijkstraMap, DistanceAlg, Point};
use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::components::{
    Carnivore, EntityMoved, Fears, Herbivore, Pools, Position, Species, Viewshed, WantsToMelee,
};
use crate::map::Map;
use crate::RunState;

pub struct AnimalAI {}

impl<'a> System<'a> for AnimalAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, Carnivore>,
        ReadStorage<'a, Herbivore>,
        ReadStorage<'a, Species>,
        ReadStorage<'a, Fears>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            runstate,
            entities,
            mut viewshed,
            pools,
            carnivore,
            herbivore,
            specieses,
            fears,
            mut melee,
            mut entity_moved,
            mut position,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        // fears logic
        for (e, view, fears, pos) in (&entities, &mut viewshed, &fears, &mut position).join() {
            let mut run_away_from: Vec<usize> = Vec::new();
            for other_tile in view.visible_tiles.iter() {
                for other_entity in
                    map.tile_content[other_tile.x as usize][other_tile.y as usize].iter()
                {
                    if specieses.get(*other_entity).map_or_else(
                        || false,
                        |s| {
                            return fears.fears.contains(&s.species);
                        },
                    ) {
                        run_away_from.push(map.xy_idx(other_tile.x, other_tile.y));
                    }
                }
            }

            if !run_away_from.is_empty() {
                let my_idx = map.xy_idx(pos.x, pos.y);
                map.populate_blocked();
                let flee_map = DijkstraMap::new(
                    map.width as usize,
                    map.height as usize,
                    &run_away_from,
                    &*map,
                    10.0,
                );
                let flee_target = DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map);
                if let Some(flee_target) = flee_target {
                    let x = flee_target as i32 % map.width;
                    let y = flee_target as i32 / map.width;
                    if !map.blocked[x as usize][y as usize] {
                        map.blocked[pos.x as usize][pos.y as usize] = false;
                        map.blocked[x as usize][y as usize] = true;
                        view.dirty = true;
                        pos.x = x;
                        pos.y = y;
                        entity_moved
                            .insert(e, EntityMoved {})
                            .expect("Cannot insert entity moved");
                    }
                }
            }
        }

        // Carnivore logic
        for (e, view, carnivore, pos) in
            (&entities, &mut viewshed, &carnivore, &mut position).join()
        {
            let mut run_towards: Vec<usize> = Vec::new();
            let mut attacked = false;
            for other_tile in view.visible_tiles.iter() {
                for other_entity in
                    map.tile_content[other_tile.x as usize][other_tile.y as usize].iter()
                {
                    let n = specieses
                        .get(*other_entity)
                        .map_or("".to_string(), |t| t.species.clone());

                    if carnivore.hunting.contains(&n) && pools.get(*other_entity).is_some() {
                        let dist = DistanceAlg::Pythagoras
                            .distance2d(Point::new(pos.x, pos.y), *other_tile);
                        if dist < 1.5 {
                            melee
                                .insert(
                                    e,
                                    WantsToMelee {
                                        target: *other_entity,
                                    },
                                )
                                .expect("Cannot insert melee");
                            attacked = true;
                        } else {
                            run_towards.push(map.xy_idx(other_tile.x, other_tile.y));
                        }
                    }
                }
            }
            if !run_towards.is_empty() && !attacked {
                let target = run_towards[0];
                // map.index_to_point2d(target);
                // let my_idx = map.xy_idx(pos.x, pos.y);
                map.populate_blocked();
                let path = a_star_search(map.xy_idx(pos.x, pos.y) as i32, target as i32, &mut *map);
                if path.success && path.steps.len() > 1 {
                    let (new_x, new_y) = (
                        path.steps[1] as i32 % map.width,
                        path.steps[1] as i32 / map.width,
                    );
                    map.blocked[pos.x as usize][pos.y as usize] = false;
                    map.blocked[new_x as usize][new_y as usize] = true;
                    view.dirty = true;
                    pos.x = new_x;
                    pos.y = new_y;
                    entity_moved
                        .insert(e, EntityMoved {})
                        .expect("Cannot insert entity moved");
                }
            }
        }
    }
}
