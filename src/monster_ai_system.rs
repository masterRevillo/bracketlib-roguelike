use bracket_lib::prelude::{a_star_search, DistanceAlg, Point};
use specs::prelude::*;

use crate::components::{Monster, Position, Viewshed, WantsToMelee};
use crate::map::Map;
use crate::RunState;

pub struct MonsterAI{}

impl <'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, player_entity, runstate, entities, mut viewshed, monster, mut position, mut wants_to_melee) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed, _monster, mut pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
                let distance = DistanceAlg::Pythagoras.distance2d(
                    Point::new(pos.x, pos.y), *player_pos
                );
                if distance < 1.5 {
                    wants_to_melee.insert(entity, WantsToMelee{target: *player_entity}).expect("Failed to insert attack");
                }
                else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &mut *map
                    );
                    if path.success && path.steps.len() > 1 {
                        pos.x = path.steps[1] as i32 / map.width;
                        pos.y = path.steps[1] as i32 % map.width;
                        map.blocked[pos.x as usize][pos.y as usize] = true;
                        viewshed.dirty = true;

                    }
                }
        }
    }
}
