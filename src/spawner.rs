use bracket_lib::color;
use bracket_lib::color::{BLACK, MAGENTA, OLIVE, PERU, RGB, YELLOW};
use bracket_lib::prelude::{FontCharType, to_cp437};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;
use crate::components::{BlocksTile, CombatStats, Item, Monster, Name, Player, Position, Potion, Renderable, Viewshed};
use crate::rect::Rect;
use crate::util::namegen::generate_ogur_name;

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs
        .create_entity()
        .with(Position{x: player_x, y: player_y})
        .with(
            Renderable{
                glyph: to_cp437('@'),
                fg: RGB::named(YELLOW),
                bg: RGB::named(BLACK),
                render_order: 0
            }
        )
        .with(Player{})
        .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true})
        .with(Name{name: "Player".to_string()})
        .with(CombatStats{max_hp: 30, hp: 30, defense: 2, power: 5})
        .build()
}

pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1,2);
    }
    match roll {
        1 => { ogur(ecs, x, y) }
        _ => { bisat(ecs, x, y) }
    }
}

fn ogur(ecs: &mut World, x: i32, y: i32) {
    monster(
        ecs,
        x,
        y,
        to_cp437('o'),
        generate_ogur_name(),
        RGB::named(OLIVE),
        CombatStats{
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4
        }
    )
}

fn bisat(ecs: &mut World, x: i32, y: i32) {
    monster(
        ecs,
        x,
        y,
        to_cp437('b'),
        generate_ogur_name(),
        RGB::named(PERU),
        CombatStats{
            max_hp: 12,
            hp: 12,
            defense: 1,
            power: 3
        }
    )
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: FontCharType, name: S, color: RGB, cstats: CombatStats) {
    ecs.create_entity()
        .with(Position{x, y})
        .with(
            Renderable{
                glyph,
                fg: color,
                bg: RGB::named(BLACK),
                render_order: 1
            }
        )
        .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true})
        .with(Monster{})
        .with(Name{name: name.to_string()})
        .with(BlocksTile{})
        .with(cstats)
        .build();
}

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<(usize, usize)> = Vec::new();
    let mut item_spawn_points: Vec<(usize, usize)> = Vec::new();
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;
        for _i in 0 .. num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                if !monster_spawn_points.contains(&(x, y)) {
                    monster_spawn_points.push((x,y));
                    added = true;
                }
            }
        }
        for _ in 0 .. num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                if !item_spawn_points.contains(&(x, y)){
                    item_spawn_points.push((x, y));
                    added = true;
                }
            }
        }
    }
    for coords in monster_spawn_points.iter() {
        random_monster(ecs, coords.0 as i32, coords.1 as i32);
    }
    for coords in item_spawn_points.iter() {
        health_potion(ecs, coords.0 as i32, coords.1 as i32)
    }
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: to_cp437('¡'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2
        })
        .with(Name{ name: "Health Potion".to_string()})
        .with(Item{})
        .with(Potion{ heal_amount: 8})
        .build();
}
