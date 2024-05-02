use bracket_lib::prelude::{a_star_search, console, DistanceAlg, Point};
use specs::prelude::*;
use crate::components::{Monster, Name, Position, Viewshed};
use crate::map::Map;

pub struct MonsterAI{}

impl <'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position) = data;

        for (mut viewshed, _monster, name, mut pos) in (&mut viewshed,  &monster, &name, &mut position).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                let distance = DistanceAlg::Pythagoras.distance2d(
                    Point::new(pos.x, pos.y), *player_pos
                );
                if distance < 1.5 {
                    // todo attack
                    console::log(format!("{} screeches incoherently", name.name));
                    return;
                }
                let path = a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map
                );
                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 / map.width;
                    pos.y = path.steps[1] as i32 % map.width;
                    viewshed.dirty = true;

                }
            }
        }
    }
}
