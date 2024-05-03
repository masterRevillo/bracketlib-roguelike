use std::iter::once_with;
use bracket_lib::color::{BLACK, OLIVE, PERU, YELLOW};
use bracket_lib::prelude::{BError, BTerm, BTermBuilder, FontCharType, GameState, main_loop, Point, RGB, to_cp437, VirtualKeyCode};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use crate::components::{BlocksTile, CombatStats, LeftMover, Monster, Name, Player, Position, Renderable, SufferDamage, Viewshed, WantsToMelee};
use crate::damage_system::DamageSystem;
use crate::map::Map;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::{player_input, try_move_player};
use crate::util::namegen::generate_ogur_name;
use crate::visibility_system::VisibilitySystem;

mod map;
mod components;
mod player;
mod rect;
mod visibility_system;
mod monster_ai_system;
mod map_indexing_system;
mod melee_combat_system;
mod damage_system;

mod util {
    pub mod namegen;
    pub mod string_utils;
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn }

struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut melee_combat_sys = MeleeCombatSystem{};
        melee_combat_sys.run_now(&self.ecs);
        let mut damage_system = DamageSystem{};
        damage_system.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
        }
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }
        DamageSystem::delete_the_dead(&mut self.ecs);
        let map = self.ecs.fetch::<Map>();
        map.draw_map(ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

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
    };
    state.ecs.register::<Position>();
    state.ecs.register::<Renderable>();
    state.ecs.register::<LeftMover>();
    state.ecs.register::<Player>();
    state.ecs.register::<Viewshed>();
    state.ecs.register::<Monster>();
    state.ecs.register::<Name>();
    state.ecs.register::<BlocksTile>();
    state.ecs.register::<CombatStats>();
    state.ecs.register::<WantsToMelee>();
    state.ecs.register::<SufferDamage>();
    let mut map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    state.ecs.insert(Point::new(player_x, player_y));
    let mut rng = RandomNumberGenerator::new();
    for room in map.rooms.iter().skip(1) {
        let (x,y) = room.center();

        let glyph: FontCharType;
        let color: RGB;
        let name: String;
        let combat_stats: CombatStats;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = to_cp437('b');
                color = RGB::named(PERU);
                name = generate_ogur_name();
                combat_stats = CombatStats{
                    max_hp: 12,
                    hp: 12,
                    defense: 1,
                    power: 3
                }
            }
            _ => {
                glyph = to_cp437('o');
                color = RGB::named(OLIVE);
                name = generate_ogur_name();
                combat_stats = CombatStats{
                    max_hp: 16,
                    hp: 16,
                    defense: 1,
                    power: 4
                }
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
            .with(BlocksTile{})
            .with(combat_stats)
            .build();
    }
    state.ecs.insert(map);
    let mut bterm = BTermBuilder::simple80x50()
        .with_title("Rusty Roguelike V2")
        .with_tile_dimensions(12, 12)
        .build()?;
    let player_entity = state.ecs
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
        .with(CombatStats{max_hp: 30, hp: 30, defense: 2, power: 5})
        .build();

    state.ecs.insert(player_entity);
    state.ecs.insert(RunState::PreRun);

    main_loop(bterm, state)
}


