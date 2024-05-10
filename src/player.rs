use std::cmp::{max, min};
use std::fmt::format;
use std::io::read_to_string;

use bracket_lib::prelude::{BTerm, console, Point, VirtualKeyCode};
use specs::{Join, World};
use specs::prelude::*;

use crate::components::{CombatStats, Item, Player, Position, Viewshed, WantsToMelee, WantsToPickUpItem};
use crate::map::Map;
use crate::{RunState, State};
use crate::gamelog::GameLog;


pub fn player_input(gs: &mut State, ctx: &mut BTerm) -> RunState {
    match ctx.key {
        None => { return RunState::AwaitingInput }
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),

            VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            _ => { return RunState::AwaitingInput }
        }
    }
    RunState::MonsterTurn
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewseheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (entity, _p, viewshed, pos) in (&entities, &players, &mut viewseheds, &mut positions)
        .join() {
            let (dest_x, dest_y) = (pos.x + delta_x, pos.y + delta_y);
            if !map.is_tile_in_bounds(dest_x, dest_y) {return;}

            for potential_target in map.tile_content[dest_x as usize][dest_y as usize].iter() {
                let target = combat_stats.get(*potential_target);
                match target {
                    None => {}
                    Some(t) => {
                        wants_to_melee.insert(entity, WantsToMelee{target: *potential_target})
                            .expect("Failed to add target");
                        return;
                    }
                }
            }
            if !map.blocked[dest_x as usize][dest_y as usize] {
                pos.x = min(map.width-1, max(0, dest_x));
                pos.y = min(map.height-1, max(0, dest_y));
                let mut ppos = ecs.write_resource::<Point>();
                ppos.x = pos.x;
                ppos.y = pos.y;

                viewshed.dirty = true;
            }
        }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gameplog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _i, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gameplog.entries.push("There is nothing here to pick  up".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickUpItem>();
            pickup.insert(*player_entity, WantsToPickUpItem{  collected_by: *player_entity, item})
                .expect("Unable to insert want to pickup");
        }
    }
}
