use bracket_lib::prelude::{main_loop, BError, BTerm, BTermBuilder, GameState, Point};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

use crate::components::{
    AreaOfEffect, Artefact, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus,
    EntityMoved, EntryTrigger, Equippable, Equipped, Examinable, Hidden, HungerClock, InBackpack,
    InflictsDamage, Item, MagicMapper, MeleeAttackBonus, Monster, Name, ParticleLifetime, Player,
    Position, ProvidesFood, ProvidesHealing, Ranged, Renderable, SerializationHelper, SerializeMe,
    SingleActivation, SufferDamage, Viewshed, WantsToDropItem, WantsToMelee, WantsToPickUpItem,
    WantsToUnequipItem, WantsToUseItem,
};
use crate::damage_system::DamageSystem;
use crate::gamelog::GameLog;
use crate::gui::{
    drop_item_menu, ranged_target, show_inventory, GameOverResult, ItemMenuResult, MainMenuResult,
    MainMenuSelection,
};
use crate::hunger_system::HungerSystem;
use crate::inventory_system::{
    ItemCollectionSystem, ItemDropSystem, ItemUnequippingSystem, ItemUseSystem,
};
use crate::map::Map;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::particle_system::ParticleSpawnSystem;
use crate::player::player_input;
use crate::rex_assets::RexAssets;
use crate::spawner::player;
use crate::trigger_system::TriggerSystem;
use crate::visibility_system::VisibilitySystem;
use crate::RunState::MainMenu;

mod components;
mod damage_system;
mod gamelog;
mod gui;
mod hunger_system;
mod inventory_system;
mod map;
pub mod map_builders;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod particle_system;
mod player;
mod random_tables;
mod rect;
mod rex_assets;
mod saveload_system;
mod spawner;
mod trigger_system;
mod visibility_system;

mod util {
    pub mod namegen;
    pub mod string_utils;
}

const SHOW_MAPGEN_VISUALIZATION: bool = true;
const DEBUGGING: bool = true;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
    MainMenu { menu_selection: MainMenuSelection },
    SaveGame,
    NextLevel,
    ShowRemoveItem,
    GameOver,
    MagicMapReveal { row: i32 },
    MapGeneration,
}

struct State {
    ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut triggers = TriggerSystem {};
        triggers.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee_combat_sys = MeleeCombatSystem {};
        melee_combat_sys.run_now(&self.ecs);
        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut potion_use_system = ItemUseSystem {};
        potion_use_system.run_now(&self.ecs);
        let mut item_drop_system = ItemDropSystem {};
        item_drop_system.run_now(&self.ecs);
        let mut item_unequipping_system = ItemUnequippingSystem {};
        item_unequipping_system.run_now(&self.ecs);
        let mut particle_system = ParticleSpawnSystem {};
        particle_system.run_now(&self.ecs);
        let mut hunger_system = HungerSystem {};
        hunger_system.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn generate_world_map(&mut self, new_depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let mut builder = map_builders::random_builder(new_depth);
        builder.build_map(&mut self.ecs);
        self.mapgen_history = builder.get_snapshot_history();
        let player_start;
        {
            let mut map_resource = self.ecs.write_resource::<Map>();
            *map_resource = builder.get_map();
            player_start = builder.get_starting_position();
        }
        builder.spawn_entities(&mut self.ecs);
        let (player_x, player_y) = (player_start.x, player_start.y);
        let mut player_pos = self.ecs.write_resource::<Point>();
        *player_pos = Point::new(player_x, player_y);
        let mut position_component = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_component.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }
        let mut viewshed_comp = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_comp.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }
            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }
            let eq = equipped.get(entity);
            if let Some(eq) = eq {
                if eq.owner == *player_entity {
                    should_delete = false;
                }
            }
            if should_delete {
                to_delete.push(entity);
            }
        }
        to_delete
    }

    pub fn game_over_cleanup(&mut self) {
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Could not delete")
        }
        {
            let p_entity = player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = p_entity;
        }
        self.generate_world_map(1);
    }

    fn goto_next_level(&mut self) {
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        let current_depth;
        {
            let mut worldmap_resources = self.ecs.write_resource::<Map>();
            current_depth = worldmap_resources.depth;
        }
        self.generate_world_map(current_depth + 1);

        let player_entity = self.ecs.fetch::<Entity>();
        let mut gamelog = self.ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("You rest for a moment and then descend to the next level".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        match new_runstate {
            RunState::MainMenu { .. } => {}
            RunState::GameOver { .. } => {}
            _ => {
                let map = self.ecs.fetch::<Map>();
                map.draw_map(ctx);
                let positions = self.ecs.read_storage::<Position>();
                let renderables = self.ecs.read_storage::<Renderable>();
                let hidden = self.ecs.read_storage::<Hidden>();

                let mut to_render = (&positions, &renderables, !&hidden)
                    .join()
                    .collect::<Vec<_>>();
                to_render.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                for (pos, render, _h) in to_render.iter() {
                    if map.visible_tiles[pos.x as usize][pos.y as usize] {
                        ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                    }
                }
                ctx.print(70, 0, ctx.fps);
                gui::dwaw_ui(&self.ecs, ctx);
            }
        }
        match new_runstate {
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZATION {
                    new_runstate = self.mapgen_next_state.unwrap();
                }
                ctx.cls();
                self.mapgen_history[self.mapgen_index].draw_map(ctx);

                self.mapgen_timer += ctx.frame_time_ms;
                if self.mapgen_timer > 50.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    if self.mapgen_index >= self.mapgen_history.len() {
                        new_runstate = self.mapgen_next_state.unwrap();
                    }
                }
            }

            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    MainMenuResult::Selected { selected } => match selected {
                        MainMenuSelection::NewGame => new_runstate = RunState::PreRun,
                        MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            new_runstate = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        MainMenuSelection::Quit => {
                            std::process::exit(0);
                        }
                    },
                }
            }
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
                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        new_runstate = RunState::MagicMapReveal { row: 0 }
                    }
                    _ => new_runstate = RunState::MonsterTurn,
                }
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
                            new_runstate = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item,
                            };
                        } else {
                            let mut wants_to_use = self.ecs.write_storage::<WantsToUseItem>();
                            wants_to_use
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem { item, target: None },
                                )
                                .expect("Unable to consume potion");
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
                        drop.insert(
                            *self.ecs.fetch::<Entity>(),
                            WantsToDropItem { item: item_entity },
                        )
                        .expect("Unable to insert drop item");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = ranged_target(self, ctx, range);
                match result.0 {
                    ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let mut use_item = self.ecs.write_storage::<WantsToUseItem>();
                        use_item
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: result.1,
                                },
                            )
                            .expect("Cannot insert WantsToUseItem");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                new_runstate = RunState::MainMenu {
                    menu_selection: MainMenuSelection::LoadGame,
                };
            }
            RunState::NextLevel => {
                self.goto_next_level();
                new_runstate = RunState::PreRun;
            }
            RunState::ShowRemoveItem => {
                let result = gui::unequip_item_menu(self, ctx);
                match result.0 {
                    ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToUnequipItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUnequipItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    GameOverResult::NoSelection => {}
                    GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        new_runstate = RunState::MainMenu {
                            menu_selection: MainMenuSelection::NewGame,
                        };
                    }
                }
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..map.width {
                    map.revealed_tiles[x as usize][row as usize] = true;
                }
                if row == map.height - 1 {
                    new_runstate = RunState::MonsterTurn;
                } else {
                    new_runstate = RunState::MagicMapReveal { row: row + 1 }
                }
            }
        }
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }
        if DamageSystem::delete_the_dead(&mut self.ecs) {
            let mut mapindex = MapIndexingSystem {};
            mapindex.run_now(&self.ecs);
        }
    }
}

fn main() -> BError {
    println!("Hello, world!");
    let mut state = State {
        ecs: World::new(),
        mapgen_next_state: Some(MainMenu {
            menu_selection: MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };
    state.ecs.register::<Position>();
    state.ecs.register::<Renderable>();
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
    state.ecs.register::<SimpleMarker<SerializeMe>>();
    state.ecs.register::<SerializationHelper>();
    state.ecs.register::<Examinable>();
    state.ecs.register::<Equippable>();
    state.ecs.register::<Equipped>();
    state.ecs.register::<MeleeAttackBonus>();
    state.ecs.register::<DefenseBonus>();
    state.ecs.register::<WantsToUnequipItem>();
    state.ecs.register::<ParticleLifetime>();
    state.ecs.register::<HungerClock>();
    state.ecs.register::<ProvidesFood>();
    state.ecs.register::<MagicMapper>();
    state.ecs.register::<Hidden>();
    state.ecs.register::<EntryTrigger>();
    state.ecs.register::<EntityMoved>();
    state.ecs.register::<SingleActivation>();

    state.ecs.insert(particle_system::ParticleBuilder::new());
    state
        .ecs
        .insert(SimpleMarkerAllocator::<SerializeMe>::new());
    state.ecs.insert(RexAssets::new());
    state.ecs.insert(RandomNumberGenerator::new());

    state.ecs.insert(Map::new(1));
    state.ecs.insert(Point::new(0, 0));
    let player_entity = player(&mut state.ecs, 0, 0);
    state.ecs.insert(player_entity);
    state.ecs.insert(RunState::MapGeneration {});
    state.ecs.insert(GameLog {
        entries: vec!["Welcome to the Halls of Ruztoo".to_string()],
    });

    state.generate_world_map(1);

    let mut bterm = BTermBuilder::simple(100, 80)?
        .with_title("Rusty Roguelike V2")
        .with_tile_dimensions(12, 12)
        .with_fps_cap(120.)
        .build()?;
    // bterm.with_post_scanlines(true);

    main_loop(bterm, state)
}
