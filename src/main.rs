use std::iter::once_with;
use bracket_lib::color::{BLACK, OLIVE, PERU, YELLOW};
use bracket_lib::prelude::{BError, BTerm, BTermBuilder, FontCharType, GameState, main_loop, Point, RGB, to_cp437, VirtualKeyCode};
use bracket_lib::prelude::LineAlg::Vector;
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use crate::components::{BlocksTile, CombatStats, InBackpack, Item, LeftMover, Monster, Name, Player, Position, Potion, Renderable, SufferDamage, Viewshed, WantsToConsumePotion, WantsToDropItem, WantsToMelee, WantsToPickUpItem};
use crate::damage_system::DamageSystem;
use crate::gamelog::GameLog;
use crate::gui::{drop_item_menu, ItemMenuResult, show_inventory};
use crate::inventory_system::{ItemCollectionSystem, ItemDropSystem, PotionUseSystem};
use crate::map::Map;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::{player_input, try_move_player};
use crate::spawner::{player, random_monster, spawn_room};
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
mod gui;
mod gamelog;
mod spawner;
mod inventory_system;

mod util {
    pub mod namegen;
    pub mod string_utils;
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem
}

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
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut potion_use_system = PotionUseSystem{};
        potion_use_system.run_now(&self.ecs);
        let mut item_drop_system = ItemDropSystem{};
        item_drop_system.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        DamageSystem::delete_the_dead(&mut self.ecs);
        {
            let map = self.ecs.fetch::<Map>();
            map.draw_map(ctx);
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();

            let mut to_render = (&positions, &renderables).join().collect::<Vec<_>>();
            to_render.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
            for (pos, render) in to_render.iter() {
                if map.visible_tiles[pos.x as usize][pos.y as usize] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
        }

        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = show_inventory(self, ctx);
                match result.0 {
                    ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut consume = self.ecs.write_storage::<WantsToConsumePotion>();
                        consume.insert(*self.ecs.fetch::<Entity>(), WantsToConsumePotion{potion: item_entity}).expect("Unable to consume potion");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = drop_item_menu(self, ctx);
                match result.0 {
                    ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut drop = self.ecs.write_storage::<WantsToDropItem>();
                        drop.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{item: item_entity}).expect("Unable to insert drop item");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
        }
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }
        ctx.print(70, 0, ctx.fps);
        gui::dwaw_ui(&self.ecs, ctx);
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
    state.ecs.register::<Item>();
    state.ecs.register::<Potion>();
    state.ecs.register::<InBackpack>();
    state.ecs.register::<WantsToPickUpItem>();
    state.ecs.register::<WantsToConsumePotion>();
    state.ecs.register::<WantsToDropItem>();
    let mut map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = player(&mut state.ecs, player_x, player_y);

    state.ecs.insert(player_entity);
    state.ecs.insert(Point::new(player_x, player_y));
    let rng = RandomNumberGenerator::new();
    state.ecs.insert(rng);
    for room in map.rooms.iter().skip(1) {
        spawn_room(&mut state.ecs, room);
    }
    state.ecs.insert(map);
    let mut bterm = BTermBuilder::simple(100, 80)?
        .with_title("Rusty Roguelike V2")
        .with_tile_dimensions(12, 12)
        .with_fps_cap(120.)
        .build()?;
    bterm.with_post_scanlines(true);
    state.ecs.insert(RunState::PreRun);
    state.ecs.insert(GameLog{entries: vec!["Welcome to the Halls of Ruztoo".to_string()]});

    main_loop(bterm, state)
}


