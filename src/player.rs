use std::cmp::{max, min};

use bracket_lib::prelude::{BTerm, Point, VirtualKeyCode};
use specs::{Join, World};
use specs::prelude::*;

use crate::components::{Player, Position, Viewshed};
use crate::map::Map;
use crate::{RunState, State};


pub fn player_input(gs: &mut State, ctx: &mut BTerm) -> RunState {
    match ctx.key {
        None => { return RunState::Paused }
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
            _ => { return RunState::Paused }
        }
    }
    RunState::Running
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewseheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, viewshed, pos) in (&mut players, &mut viewseheds, &mut positions).join() {
        let (dest_x, dest_y) = (pos.x + delta_x, pos.y + delta_y);
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
