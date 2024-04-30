use std::cmp::{max, min};
use bracket_lib::prelude::Point;
use specs::{Join, World};
use crate::components::{Player, Position, Viewshed};
use crate::map::{Map, TileType};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewseheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, viewshed, pos) in (&mut players, &mut viewseheds, &mut positions).join() {
        if map.get_tile_at_pos(pos.x + delta_x, pos.y + delta_y) != TileType::Wall {
            pos.x = min(map.width-1, max(0, pos.x + delta_x));
            pos.y = min(map.height-1, max(0, pos.y + delta_y));
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}
