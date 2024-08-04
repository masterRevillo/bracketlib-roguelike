use std::cmp::{max, min};

use bracket_lib::prelude::{BTerm, Point, to_cp437, VirtualKeyCode};
use specs::{Join, World};
use specs::prelude::*;

use crate::{RunState, State};
use crate::components::{BlocksTile, BlocksVisibility, CombatStats, Door, EntityMoved, HungerClock, HungerState, Item, Monster, Player, Position, Renderable, Viewshed, WantsToMelee, WantsToPickUpItem};
use crate::gamelog::GameLog;
use crate::map::{Map, TileType};

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
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),
            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::Escape => return RunState::SaveGame,
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }
            VirtualKeyCode::R => return RunState::ShowRemoveItem,
            _ => { return RunState::AwaitingInput }
        }
    }
    RunState::PlayerTurn
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let map = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        for entity_id in map.tile_content[tile.x as usize][tile.y as usize].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {}
                Some(_) => { can_heal = false;}
            }
        }
    }

    if can_heal {
        let hunger_clock = ecs.read_storage::<HungerClock>();
        let hc = hunger_clock.get(*player_entity);
        if let Some(hc) = hc {
            match hc.state {
                HungerState::Hungry | HungerState::Starving => can_heal = false,
                _ => {}

            }
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}

fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    if map.tiles[player_pos.x as usize][player_pos.y as usize] == TileType::DownStairs{
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here".to_string());
        false
    }
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.write_storage::<Player>();
    let mut viewseheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let map = ecs.fetch::<Map>();

    for (entity, _p, viewshed, pos) in (&entities, &players, &mut viewseheds, &mut positions)
        .join() {
            let (dest_x, dest_y) = (pos.x + delta_x, pos.y + delta_y);
            if !map.is_tile_in_bounds(dest_x, dest_y) {return;}

            for potential_target in map.tile_content[dest_x as usize][dest_y as usize].iter() {
                let target = combat_stats.get(*potential_target);
                if let Some(_t) = target {
                    wants_to_melee.insert(entity, WantsToMelee{target: *potential_target})
                        .expect("Failed to add target");
                    return;
                }
                let door = doors.get_mut(*potential_target);
                if let Some(door) = door {
                    door.open = true;
                    blocks_visibility.remove(*potential_target);
                    blocks_movement.remove(*potential_target);
                    let glyph = renderables.get_mut(*potential_target).unwrap();
                    glyph.glyph = to_cp437('/');
                    viewshed.dirty = true;
                }
            }
            if !map.blocked[dest_x as usize][dest_y as usize] {
                pos.x = min(map.width-1, max(0, dest_x));
                pos.y = min(map.height-1, max(0, dest_y));
                let mut ppos = ecs.write_resource::<Point>();
                ppos.x = pos.x;
                ppos.y = pos.y;

                viewshed.dirty = true;
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
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
