mod map;
mod components;
mod player;
mod rect;
use rect::Rect;

use std::cmp::{max, min};
use bracket_lib::color::{BLACK, RED, YELLOW};
use bracket_lib::prelude::{BError, BTerm, BTermBuilder, FontCharType, GameState, main_loop, RGB, to_cp437, VirtualKeyCode};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;
use specs_derive::Component;
use crate::map::{draw_map, new_map_rooms_and_corridors};

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    fg: RGB,
    bg: RGB
}

#[derive(Component)]
struct LeftMover {}

#[derive(Component)]
struct Player {}

struct LeftWalker{}
impl<'a> System<'a> for LeftWalker {
    type SystemData = (
        ReadStorage<'a, LeftMover>,
        WriteStorage<'a, Position>
    );

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for(_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 { pos.x = 79}
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}


struct State {
    ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker{};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print(70, 0, ctx.fps);
        self.run_systems();
        player_input(self, ctx);
        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

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
    let (rooms, map) = new_map_rooms_and_corridors();
    state.ecs.insert(map);
    let (player_x, player_y) = rooms[0].center();
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
        .build();

    let mut rng = RandomNumberGenerator::new();

    for i in 0..10 {
        state.ecs
            .create_entity()
            .with(Position{x: i * 7, y: 20})
            .with(
                Renderable{
                    glyph: rng.range(0, 255),
                    fg: RGB::named(RED),
                    bg: RGB::named(BLACK),
                }
            )
            .with(LeftMover{})
            .build();
    }

    main_loop(bterm, state)
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y +delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
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
