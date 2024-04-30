use bracket_lib::color::{BLACK, OLIVE, PERU, YELLOW};
use bracket_lib::prelude::{BError, BTerm, BTermBuilder, FontCharType, GameState, main_loop, Point, RGB, to_cp437, VirtualKeyCode};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use crate::components::{LeftMover, Monster, Name, Player, Position, Renderable, Viewshed};
use crate::map::Map;
use crate::monster_ai_system::MonsterAI;
use crate::player::try_move_player;
use crate::util::namegen::generate_ogur_name;
use crate::visibility_system::VisibilitySystem;

mod map;
mod components;
mod player;
mod rect;
mod visibility_system;
mod monster_ai_system;

mod util {
    pub mod namegen;
    pub mod string_utils;
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

struct State {
    ecs: World,
    pub runstate: RunState
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }
        let map = self.ecs.fetch::<Map>();
        map.draw_map(ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            if map.visible_tiles[pos.x as usize][pos.y as usize] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
        ctx.print(70, 0, ctx.fps);
    }
}

fn main() -> BError {
    println!("Hello, world!");
    let mut state = State{
        ecs: World::new(),
        runstate: RunState::Running
    };
    state.ecs.register::<Position>();
    state.ecs.register::<Renderable>();
    state.ecs.register::<LeftMover>();
    state.ecs.register::<Player>();
    state.ecs.register::<Viewshed>();
    state.ecs.register::<Monster>();
    state.ecs.register::<Name>();
    let mut map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    state.ecs.insert(Point::new(player_x, player_y));
    let mut rng = RandomNumberGenerator::new();
    for room in map.rooms.iter().skip(1) {
        let (x,y) = room.center();

        let glyph: FontCharType;
        let color: RGB;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = to_cp437('b');
                color = RGB::named(PERU);
                name = generate_ogur_name()
            }
            _ => {
                glyph = to_cp437('o');
                color = RGB::named(OLIVE);
                name = generate_ogur_name()
            }
        }

        state.ecs.create_entity()
            .with(Position{x, y})
            .with(
                Renderable{
                    glyph,
                    fg: color,
                    bg: RGB::named(BLACK),
                }
            )
            .with(Viewshed{ visible_tiles: Vec::new(), range: 8, dirty: true})
            .with(Monster{})
            .with(Name{name})
            .build();
    }
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
        .with(Name{name: "Player".to_string()})
        .build();

    main_loop(bterm, state)
}


fn player_input(gs: &mut State, ctx: &mut BTerm) -> RunState {
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
            _ => { return RunState::Paused }
        }
    }
    RunState::Running
}
