use bracket_lib::color::RGB;
use bracket_lib::prelude::to_cp437;
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;
use specs::rayon::vec;

use crate::components::{
    BlocksTile, Equipped, InBackpack, LootTable, Monster, Name, Player, Pools, Position,
    Renderable, SufferDamage,
};
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::raws::rawmaster::{get_item_drop, spawn_named_item, SpawnType};
use crate::raws::RAWS;
use crate::RunState;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
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
                            let r = renderables.get_mut(entity);
                            if let Some(r) = r {
                                r.glyph = to_cp437('%');
                                r.fg = RGB::from_f32(0.75, 0., 0.);
                            }
                        }
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

        // drop equipped items
        let mut to_spawn: Vec<(String, Position)> = Vec::new();
        {
            let mut to_drop: Vec<(Entity, Position)> = Vec::new();
            let entities = ecs.entities();
            let mut equipped = ecs.write_storage::<Equipped>();
            let mut carried = ecs.write_storage::<InBackpack>();
            let mut positions = ecs.write_storage::<Position>();
            let loot_tables = ecs.read_storage::<LootTable>();
            let mut rng = ecs.write_resource::<RandomNumberGenerator>();

            for victim in dead.iter() {
                for (entity, equipped) in (&entities, &equipped).join() {
                    if equipped.owner == *victim {
                        let pos = positions.get(*victim);

                        if let Some(pos) = pos {
                            to_drop.push((entity, pos.clone()));
                        }
                    }
                }
                if let Some(table) = loot_tables.get(*victim) {
                    let drop_finder = get_item_drop(&RAWS.lock().unwrap(), &mut rng, &table.table);
                    if let Some(tag) = drop_finder {
                        let pos = positions.get(*victim);
                        if let Some(pos) = pos {
                            to_spawn.push((tag, pos.clone()));
                        }
                    }
                }
            }
            for drop in to_drop.iter() {
                equipped.remove(drop.0);
                carried.remove(drop.0);
                positions
                    .insert(drop.0, drop.1.clone())
                    .expect("Failed to insert position");
            }
        }

        {
            for drop in to_spawn.iter() {
                spawn_named_item(
                    &RAWS.lock().unwrap(),
                    &drop.0,
                    SpawnType::AtPosition {
                        x: drop.1.x,
                        y: drop.1.y,
                    },
                    ecs,
                );
            }
        }

        for victim in &dead {
            monsters.remove(*victim);
            blockers.remove(*victim);
            pools.remove(*victim);
        }
        !&dead.is_empty()
    }
}
