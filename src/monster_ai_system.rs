use bracket_lib::prelude::{a_star_search, console, DistanceAlg, Point};
use specs::prelude::*;

use crate::components::{Confusion, Monster, Position, Viewshed, WantsToMelee};
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
        WriteStorage<'a, Confusion>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut confusion
        ) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed, _monster, mut pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
            let mut can_act = true;

            let is_confused = confusion.get_mut(entity);
            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confusion.remove(entity);
                }
                can_act = false;
            }
            if can_act {
                let distance = DistanceAlg::Pythagoras.distance2d(
                    Point::new(pos.x, pos.y), *player_pos
                );
                if distance < 1.5 {
                    wants_to_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Failed to insert attack");
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &mut *map
                    );
                    if path.success && path.steps.len() > 1 {
                        let (new_x, new_y) = (
                            path.steps[1] as i32 / map.width,
                            path.steps[1] as i32 % map.width
                        );
                        let dir = match (new_x - pos.x, new_y - pos.y) {
                            (-1, 0) => "west",
                            (-1, -1) => "northwest",
                            (0, -1) => "north",
                            (1, -1) => "northeast",
                            (1, 0) => "east",
                            (1, 1) => "southeast",
                            (0, 1) => "south",
                            (-1, 1) => "southwest",
                            _ => "wat"
                        };
                        console::log(format!("step: ({},{}), dir={}", new_x, new_y, dir));
                        let coord_path: Vec<_> = path.steps.iter().map(
                            |v| (v / map.width as usize, v % map.width as usize)
                        ).collect();
                        console::log(format!("the path: {:?}", coord_path));

                        pos.x = new_x;
                        pos.y = new_y;
                        map.blocked[pos.x as usize][pos.y as usize] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
