use bracket_lib::color::{BLACK, GOLD, RGB};
use bracket_lib::prelude::{console, to_cp437, Point};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use crate::components::{
    Attributes, BlocksTile, Carnivore, Equipped, Fears, Herbivore, InBackpack, LootTable, Monster,
    Name, Player, Pools, Position, Renderable, SufferDamage,
};
use crate::gamelog::GameLog;
use crate::gamesystem::{mana_at_level, player_hp_at_level};
use crate::map::Map;
use crate::particle_system::ParticleBuilder;
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
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, ParticleBuilder>,
        ReadExpect<'a, Point>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut pools,
            mut damage,
            positions,
            mut map,
            entities,
            player,
            attributes,
            mut log,
            mut particles,
            player_pos,
        ) = data;
        let mut xp_gained = 0;

        for (entity, pools, damage) in (&entities, &mut pools, &damage).join() {
            for dmg in damage.amount.iter() {
                pools.hit_points.current -= dmg.0;
                let pos = positions.get(entity);
                if let Some(pos) = pos {
                    map.bloodstains.insert((pos.x, pos.y));
                }
                if pools.hit_points.current < 1 && dmg.1 {
                    xp_gained += pools.level * 100;
                }
            }
        }

        if xp_gained != 0 {
            let mut player_stats = pools.get_mut(*player).unwrap();
            let player_attr = attributes.get(*player).unwrap();
            player_stats.xp += xp_gained;
            if player_stats.xp >= player_stats.level * 1000 {
                // lvl up
                player_stats.level += 1;
                player_stats.hit_points.max = player_hp_at_level(
                    player_attr.fitness.base + player_attr.fitness.modifiers,
                    player_stats.level,
                );
                player_stats.hit_points.current = player_stats.hit_points.max;
                player_stats.mana.max = mana_at_level(
                    player_attr.intelligence.base + player_attr.intelligence.modifiers,
                    player_stats.level,
                );
                player_stats.mana.current = player_stats.mana.max;
                log.entries.push(format!(
                    "Congratulations! You are now level {}",
                    player_stats.level
                ));

                for i in 0..10 {
                    if player_pos.y - i > 1 {
                        particles.request(
                            player_pos.x,
                            player_pos.y - i,
                            RGB::named(GOLD),
                            RGB::named(BLACK),
                            to_cp437('░'),
                            200.0,
                        );
                    }
                }
            }
        }

        damage.clear()
    }
}

impl DamageSystem {
    pub fn delete_the_dead(ecs: &mut World) -> bool {
        let mut dead: Vec<Entity> = Vec::new();
        {
            let pools = ecs.write_storage::<Pools>();
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
                    console::log(format!(
                        "Found a dead thing with a loot table: {}",
                        &table.table
                    ));
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

        let mut pools = ecs.write_storage::<Pools>();
        let mut monsters = ecs.write_storage::<Monster>();
        let mut blockers = ecs.write_storage::<BlocksTile>();
        let mut herbivore = ecs.write_storage::<Herbivore>();
        let mut carnivore = ecs.write_storage::<Carnivore>();
        let mut fears = ecs.write_storage::<Fears>();
        for victim in &dead {
            monsters.remove(*victim);
            blockers.remove(*victim);
            pools.remove(*victim);
            herbivore.remove(*victim);
            carnivore.remove(*victim);
            fears.remove(*victim);
        }
        !&dead.is_empty()
    }
}
