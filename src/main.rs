use std::cmp::{max, min};

use bracket_lib::color::{BLACK, RED, YELLOW};
use bracket_lib::prelude::{BError, BTerm, BTermBuilder, GameState, main_loop, RGB, to_cp437, VirtualKeyCode};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use crate::components::{LeftMover, Player, Position, Renderable, Viewshed};
use crate::map::{Map, TileType};
use crate::visibility_system::VisibilitySystem;

mod map;
mod components;
mod player;
mod rect;
mod visibility_system;


struct State {
    ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print(70, 0, ctx.fps);
        self.run_systems();
        player_input(self, ctx);
        let map = self.ecs.fetch::<Map>();
        map.draw_map(ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn main() -> BError {
    println!("Hello, world!");
    let mut state = State{
        ecs: World::new()
    };
    state.ecs.register::<Position>();
    state.ecs.register::<Renderable>();
    state.ecs.register::<LeftMover>();
    state.ecs.register::<Player>();
    state.ecs.register::<Viewshed>();
    let mut map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    state.ecs.insert(map);
    let mut bterm = BTermBuilder::simple80x50()
        .with_title("Rusty Roguelike V2")
        .with_tile_dimensions(12, 12)
        .build()?;
    state.ecs
        .create_entity()
        .with(Position{x: player_x, y: player_y})
        .with(
            Renderable{
                glyph: to_cp437('@'),
                fg: RGB::named(YELLOW),
                bg: RGB::named(BLACK),
            }
        )
        .with(Player{})
        .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true})
        .build();

    main_loop(bterm, state)
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewseheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, viewshed, pos) in (&mut players, &mut viewseheds, &mut positions).join() {
        if map.get_tile_at_pos(pos.x + delta_x, pos.y + delta_y) != TileType::Wall {
            pos.x = min(map.width-1, max(0, pos.x + delta_x));
            pos.y = min(map.height-1, max(0, pos.y + delta_y));

            viewshed.dirty = true;
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut BTerm) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        }
    }
}
