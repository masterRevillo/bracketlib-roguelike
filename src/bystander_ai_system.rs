use bracket_lib::prelude::Point;
use bracket_lib::random::RandomNumberGenerator;
use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::components::{Bystander, EntityMoved, Name, Position, Quips, Viewshed};
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::RunState;

pub struct BystanderAI {}

impl<'a> System<'a> for BystanderAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Bystander>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadExpect<'a, Point>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            runstate,
            entities,
            mut viewshed,
            bystander,
            mut position,
            mut entity_moved,
            mut rng,
            player_pos,
            mut gamelog,
            mut quips,
            names
        ) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed, _bystander, mut pos) in (&entities, &mut viewshed, &bystander, &mut position).join() {
            let quip = quips.get_mut(entity);
            if let Some(quip) = quip {
                if !quip.available.is_empty() && viewshed.visible_tiles.contains(&player_pos) && rng.roll_dice(1, 6) == 1 {
                    let name = names.get(entity);
                    let q_idx = if quip.available.len() == 1 {
                        0
                    } else {
                        (rng.roll_dice(1, quip.available.len() as i32)-1) as usize
                    };
                    gamelog.entries.push(
                        format!("{} says \"{}\"", name.unwrap().name, quip.available[q_idx])
                    );
                    quip.available.remove(q_idx);
                }
            }

            let mut x = pos.x;
            let mut y = pos.y;
            let move_roll = rng.roll_dice(1, 5);
            match move_roll {
                1 => x -= 1,
                2 => x += 1,
                3 => y -= 1,
                4 => y += 1,
                _ => {}
            }

            if x > 0 && x < map.width-1 && y > 0 && y < map.height-1 {
                if !map.blocked[x as usize][y as usize] {
                    map.blocked[pos.x as usize][pos.y as usize] = false;
                    pos.x = x;
                    pos.y = y;
                    entity_moved.insert(entity, EntityMoved{}).expect("unable to insert entity moved");
                    map.blocked[x as usize][y as usize] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
