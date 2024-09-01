use std::collections::HashMap;
use std::hash::Hash;

use bracket_lib::color::{BLACK, RGB, YELLOW};
use bracket_lib::prelude::to_cp437;
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};

use crate::components::{Attribute, Attributes, HungerClock, HungerState, Name, Player, Pool, Pools, Position, Renderable, SerializeMe, Skill, Skills, Viewshed};
use crate::DEBUGGING;
use crate::gamesystem::{attr_bonus, mana_at_level, player_hp_at_level};
use crate::map::Map;
use crate::map::tiletype::TileType;
use crate::random_tables::{EntityType, RandomTable};
use crate::random_tables::EntityType::*;
use crate::raws::rawmaster::{get_spawn_table_for_depth, spawn_named_entity, SpawnType};
use crate::raws::RAWS;
use crate::rect::Rect;

const MAX_MONSTERS: i32 = 4;

pub type SpawnList = Vec<((i32, i32), EntityType)>;

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    let mut skills = Skills{
        skills: HashMap::new()
    };
    skills.skills.insert(Skill::Melee, 1);
    skills.skills.insert(Skill::Defense, 1);
    skills.skills.insert(Skill::Magic, 1);
    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 32,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(HungerClock {
            state: HungerState::WellFed,
            hunger_points: 20,
        })
        .with(
            Attributes{
                might: Attribute {
                    base: 11, modifiers: 0, bonus: attr_bonus(11)
                },
                fitness: Attribute {
                    base: 11, modifiers: 0, bonus: attr_bonus(11)
                },
                quickness: Attribute {
                    base: 11, modifiers: 0, bonus: attr_bonus(11)
                },
                intelligence: Attribute {
                    base: 11, modifiers: 0, bonus: attr_bonus(11)
                },
            }
        )
        .with(skills)
        .with(Pools {
           hit_points: Pool {
               current: player_hp_at_level(11, 1),
               max: player_hp_at_level(11, 1)
           },
            mana: Pool {
                current: mana_at_level(11, 1),
                max: mana_at_level(11, 1)
            },
            xp: 0,
            level: 1
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn room_table(level: i32) -> RandomTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(),  level)
}

pub fn spawn_entity(ecs: &mut World, spawn: &(&(i32, i32), &EntityType)) {
    // let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    // let builder = ecs.create_entity();
    let coords = spawn.0;

    let item_result = spawn_named_entity(
        &RAWS.lock().unwrap(),
        spawn.1,
        SpawnType::AtPosition {x:coords.0, y:coords.1},
        ecs
    );
    if item_result.is_some() {
        return;
    }
    match spawn.1 {
        None => {}
        // Bisat => bisat(ecs, coords.0, coords.1),
        // Ogur => ogur(ecs, coords.0, coords.1),
        // HealthPotion => health_potion(ecs, coords.0, coords.1),
        // FireballScroll => fireball_stroll(ecs, coords.0, coords.1),
        // ConfusionScroll => confusion_stroll(ecs, coords.0, coords.1),
        // MagicMissileScroll => magic_missile_stroll(ecs, coords.0, coords.1),
        // Sandwich => food(
        //     ecs,
        //     to_cp437('='),
        //     "Sandwich",
        //     RGB::named(CHOCOLATE3),
        //     2,
        //     200,
        //     coords.0,
        //     coords.1,
        // ),
        // ChickenLeg => food(
        //     ecs,
        //     to_cp437('q'),
        //     "Chicken Leg",
        //     RGB::named(CHOCOLATE3),
        //     3,
        //     150,
        //     coords.0,
        //     coords.1,
        // ),
        // GobletOfWine => food(
        //     ecs,
        //     to_cp437('u'),
        //     "Goblet of Wine",
        //     RGB::named(MAROON),
        //     1,
        //     25,
        //     coords.0,
        //     coords.1,
        // ),
        // Artefact => artefact(ecs, coords.0, coords.1),
        // Dagger => dagger(ecs, coords.0, coords.1),
        // Shield => shield(ecs, coords.0, coords.1),
        // Longsword => longsword(ecs, coords.0, coords.1),
        // TowerShield => tower_shield(ecs, coords.0, coords.1),
        // MagicMappingScroll => magic_mapping_stroll(ecs, coords.0, coords.1),
        // BearTrap => bear_trap(ecs, coords.0, coords.1),
        // Door => door(ecs, coords.0, coords.1),
        _ => {}
    }
}

pub fn spawn_room(map: &Map, rng: &mut RandomNumberGenerator, room: &Rect, map_level: i32, spawn_list: &mut SpawnList) {
    let mut possible_targets: Vec<(i32, i32)> = Vec::new();
    {
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                if map.tiles[x as usize][y as usize] == TileType::Floor {
                    possible_targets.push((x, y))
                }
            }
        }
    }
    spawn_region(map, rng, &possible_targets, map_level, spawn_list);

    // let spawn_table = room_table(map_level);
    // let mut spawn_points: HashMap<(i32, i32), EntryType> = HashMap::new();
    // {
    //     let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    //     let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_level - 1) - 3;
    //     for _i in 0..num_spawns {
    //         let mut added = false;
    //         let mut tries = 0;
    //         while !added && tries < 20 {
    //             let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
    //             let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));
    //             if !spawn_points.contains_key(&(x, y)) {
    //                 spawn_points.insert((x, y), spawn_table.roll(&mut rng));
    //                 added = true;
    //             } else {
    //                 tries += 1;
    //             }
    //         }
    //     }
    // }
    // for spawn in spawn_points.iter() {
    //     console::log(format!(
    //         "spawning stuff at {:?} in room with boundaries x{},x{},y{},y{}",
    //         spawn.0, room.x1, room.x2, room.y1, room.y2
    //     ));
    //     spawn_entity(ecs, &spawn);
    // }
}

pub fn spawn_debug_items(ecs: &mut World, starting_position: &Position) {
    if DEBUGGING {
        spawn_entity(
            ecs,
            &(&(starting_position.x, starting_position.y), &MagicMappingScroll),
        );
    }
}

pub fn spawn_region(
    _map: &Map,
    rng: &mut RandomNumberGenerator,
    area: &[(i32, i32)],
    map_depth: i32,
    spawn_list: &mut SpawnList
) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<(i32, i32), EntityType> = HashMap::new();
    let mut areas: Vec<(i32, i32)> = Vec::from(area);

    {
        let num_spawns = i32::min(
            areas.len() as i32,
            rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3,
        );
        if num_spawns == 0 {
            return;
        }
        for _i in 0..num_spawns {
            let idx = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };
            spawn_points.insert(areas[idx], spawn_table.roll(rng));
            areas.remove(idx);
        }
    }

    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, *spawn.1))
    }
}
