use bracket_lib::color::RGB;
use bracket_lib::prelude::to_cp437;
use specs::prelude::*;

use crate::components::{BlocksTile, Monster, Name, Player, Pools, Position, Renderable, SufferDamage};
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::RunState;

pub struct DamageSystem{}

impl <'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut pools, mut damage, positions, mut map, entities) = data;

        for (entity, pools, damage) in (&entities, &mut pools, &damage).join() {
            pools.hit_points.current -= damage.amount.iter().sum::<i32>();
            let pos = positions.get(entity);
            if let Some(pos) = pos {
                map.bloodstains.insert((pos.x, pos.y));
            }
        }
        damage.clear()
    }

}

impl DamageSystem {

    pub fn delete_the_dead(ecs: &mut World) -> bool {
        let mut dead: Vec<Entity> = Vec::new();
        let mut pools = ecs.write_storage::<Pools>();
        {
            let players = ecs.read_storage::<Player>();
            let entities = ecs.entities();
            let mut names = ecs.write_storage::<Name>();
            let mut renderables = ecs.write_storage::<Renderable>();
            let mut gamelog = ecs.write_resource::<GameLog>();
            for (entity, pools) in (&entities, &pools).join() {
                if pools.hit_points.current < 1 {
                    let player = players.get(entity);
                    match player {
                        None => {
                            let name = names.get_mut(entity);
                            if let Some(name) = name {
                                gamelog.entries.push(format!("{} is dead", name.name));
                                let mut corpse_name = "Remains of ".to_owned();
                                corpse_name.push_str(&name.name);
                                name.name = corpse_name;
                            }
                            dead.push(entity);
                            let r =  renderables.get_mut(entity);
                            if let Some(r) = r {
                                r.glyph = to_cp437('%');
                                r.fg = RGB::from_f32(0.75, 0., 0.);
                            }
                        },
                        Some(_) => {
                            let mut runstate = ecs.write_resource::<RunState>();
                            *runstate = RunState::GameOver;
                        }
                    }

                }
            }
        }
        let mut monsters = ecs.write_storage::<Monster>();
        let mut blockers = ecs.write_storage::<BlocksTile>();
        for victim in &dead {
            monsters.remove(*victim);
            blockers.remove(*victim);
            pools.remove(*victim);
        }
        !&dead.is_empty()
    }
}
