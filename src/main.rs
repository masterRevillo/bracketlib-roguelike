use bracket_lib::prelude::{BError, BTerm, BTermBuilder, GameState, main_loop, Point};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use crate::components::{AreaOfEffect, Artefact, BlocksTile, CombatStats, Confusion, Consumable, InBackpack, InflictsDamage, Item, LeftMover, Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SufferDamage, Viewshed, WantsToDropItem, WantsToMelee, WantsToPickUpItem, WantsToUseItem};
use crate::damage_system::DamageSystem;
use crate::gamelog::GameLog;
use crate::gui::{drop_item_menu, ItemMenuResult, ranged_target, show_inventory};
use crate::inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};
use crate::map::Map;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::player_input;
use crate::spawner::{player, spawn_room};
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
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity }
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
        let mut potion_use_system = ItemUseSystem {};
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
                        let item = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item);
                        if let Some(is_item_ranged) = is_item_ranged {
                            new_runstate = RunState::ShowTargeting {range: is_item_ranged.range, item };
                        } else {
                            let mut wants_to_use = self.ecs.write_storage::<WantsToUseItem>();
                            wants_to_use.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item , target: None}).expect("Unable to consume potion");
                            new_runstate = RunState::PlayerTurn;
                        }
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
            RunState::ShowTargeting {range, item} => {
                let result = ranged_target(self, ctx, range);
                match result.0 {
                    ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let mut use_item = self.ecs.write_storage::<WantsToUseItem>();
                        use_item.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item, target: result.1})
                            .expect("Cannot insert WantsToUseItem");
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
    state.ecs.register::<ProvidesHealing>();
    state.ecs.register::<InBackpack>();
    state.ecs.register::<WantsToPickUpItem>();
    state.ecs.register::<WantsToUseItem>();
    state.ecs.register::<WantsToDropItem>();
    state.ecs.register::<Artefact>();
    state.ecs.register::<Consumable>();
    state.ecs.register::<Ranged>();
    state.ecs.register::<InflictsDamage>();
    state.ecs.register::<AreaOfEffect>();
    state.ecs.register::<Confusion>();
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


