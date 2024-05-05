use bracket_lib::prelude::console;
use specs::prelude::*;
use crate::components::{CombatStats, Name, Player, SufferDamage};
use crate::gamelog::GameLog;

pub struct DamageSystem{}

impl <'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }
        damage.clear()
    }

}

impl DamageSystem {

    pub fn delete_the_dead(ecs: &mut World) {
        let mut dead: Vec<Entity> = Vec::new();
        {
            let combat_stats = ecs.read_storage::<CombatStats>();
            let players = ecs.read_storage::<Player>();
            let entities = ecs.entities();
            let names = ecs.read_storage::<Name>();
            let mut gamelog = ecs.write_resource::<GameLog>();
            for (entity, stats) in (&entities, &combat_stats).join() {
                if stats.hp < 1 {
                    if stats.hp < 1 {
                        let player = players.get(entity);
                        match player {
                            None => {
                                let name = names.get(entity);
                                if let Some(name) = name {
                                    gamelog.entries.push(format!("{} is dead", name.name));
                                }
                                dead.push(entity)
                            },
                            Some(_) => gamelog.entries.push("You died!".to_string())
                        }
                    }

                }
            }
        }
        for victim in dead {
            ecs.delete_entity(victim).expect("Unable to delete");
        }
    }
}
