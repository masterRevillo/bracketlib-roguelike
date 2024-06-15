use bracket_lib::color::{
    BLACK, BLUE, CYAN, GREEN, GREY, MAGENTA, ORANGE, RED, RGB, WHEAT, WHITE, YELLOW,
};
use bracket_lib::prelude::{
    letter_to_option, to_cp437, BTerm, DijkstraMap, DistanceAlg, Point, VirtualKeyCode,
};
use bracket_lib::terminal::FontCharType;
use specs::prelude::*;
use std::path::Iter;

use crate::components::{
    CombatStats, Equipped, Hidden, HungerClock, HungerState, InBackpack, Name, Player, Position,
    Viewshed,
};
use crate::gamelog::GameLog;
use crate::map::{Map, MAPHEIGHT, MAPWIDTH};
use crate::rex_assets::RexAssets;
use crate::saveload_system::does_save_exist;
use crate::{RunState, State, DEBUGGING};

const GUIY: usize = MAPHEIGHT;
const GUIHEIGHT: usize = 6;
const GUIWIDTH: usize = MAPWIDTH - 1;

pub fn dwaw_ui(ecs: &World, ctx: &mut BTerm) {
    ctx.draw_box(
        0,
        MAPHEIGHT,
        GUIWIDTH,
        GUIHEIGHT,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();
    let log = ecs.fetch::<GameLog>();
    let map = ecs.fetch::<Map>();
    for (_p, stats, hc) in (&players, &combat_stats, &hunger).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(20, GUIY, RGB::named(YELLOW), RGB::named(BLACK), &health);

        ctx.draw_bar_horizontal(
            36,
            GUIY,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(RED),
            RGB::named(BLACK),
        );

        match match hc.state {
            HungerState::WellFed => Some((RGB::named(GREEN), "Well Fed")),
            HungerState::Normal => None,
            HungerState::Hungry => Some((RGB::named(ORANGE), "Hungry")),
            HungerState::Starving => Some((RGB::named(RED), "Starving")),
        } {
            Some(h) => ctx.print_color(GUIWIDTH - 10, GUIY - 1, h.0, RGB::named(BLACK), h.1),
            None => {}
        }
    }

    let mut y = GUIY + 1;
    for s in log.entries.iter().rev() {
        if y < GUIY + GUIHEIGHT {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    let dungeon_level = format!("Dungeon Level: {}", map.depth);
    ctx.print_color(
        2,
        GUIY,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        &dungeon_level,
    );

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(MAGENTA));
    draw_tooltips(ecs, ctx);
}

pub fn draw_tooltips(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();
    let dkm = ecs.try_fetch::<DijkstraMap>();

    let mouse_pos = ctx.mouse_pos();
    if !map.is_tile_in_bounds(mouse_pos.0, mouse_pos.1) {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, postion, _h) in (&names, &positions, !&hidden).join() {
        if postion.x == mouse_pos.0
            && postion.y == mouse_pos.1
            && map.visible_tiles[postion.x as usize][postion.y as usize]
        {
            tooltip.push(name.name.to_string());
        }
    }

    ctx.print(
        GUIWIDTH - 20,
        GUIY + 1,
        format!("Coordinates: ({},{})", mouse_pos.0, mouse_pos.1),
    );
    if let Some(dkm) = dkm {
        if DEBUGGING {
            ctx.print(
                GUIWIDTH - 25,
                GUIY + 2,
                format!(
                    "Dijkstra score: {}",
                    dkm.map[(mouse_pos.1 * map.width + mouse_pos.0) as usize]
                ),
            );
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_pos.0 > MAPWIDTH as i32 / 2 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(WHITE),
                        RGB::named(GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(WHITE),
                RGB::named(GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + 1 + i,
                        y,
                        RGB::named(WHITE),
                        RGB::named(GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(WHITE),
                RGB::named(GREY),
                &"<-".to_string(),
            );
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

const INVENTORY_X: usize = MAPWIDTH / 2 - 20;

pub fn show_inventory(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        INVENTORY_X,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
    ctx.print_color(
        INVENTORY_X + 3,
        y - 2,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Inventory",
    );
    ctx.print_color(
        INVENTORY_X + 3,
        y + count as i32 + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _p, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        ctx.set(
            INVENTORY_X + 2,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            to_cp437('('),
        );
        ctx.set(
            INVENTORY_X + 3,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            97 + j as FontCharType,
        );
        ctx.set(
            INVENTORY_X + 4,
            y,
            RGB::named(WHITE),
            RGB::named(BLACK),
            to_cp437(')'),
        );

        ctx.print(INVENTORY_X + 6, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    capture_item_options_selection(ctx, equippable, count as i32)
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        INVENTORY_X,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
    ctx.print_color(
        INVENTORY_X + 3,
        y - 2,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Drop Which Item?",
    );
    ctx.print_color(
        INVENTORY_X + 3,
        y + count as i32 + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _p, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        print_item_options_menu(&name.name.to_string(), y, j, ctx);
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut BTerm,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let player = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Select Target:",
    );

    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player);
    if let Some(visible) = visible {
        for point in visible.visible_tiles.iter() {
            let distance = DistanceAlg::Pythagoras.distance2d(*player_pos, *point);
            if distance <= range as f32 {
                ctx.set_bg(point.x, point.y, RGB::named(BLUE));
                available_cells.push(point);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for i in available_cells.iter() {
        if i.x == mouse_pos.0 && i.y == mouse_pos.1 {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(CYAN));
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_pos.0, mouse_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut BTerm) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();
    let save_exists = does_save_exist();

    let assets = gs.ecs.fetch::<RexAssets>();
    // ctx.render_xp_sprite(&assets.menu, -60, -50);
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.draw_box_double(34, 18, 31, 10, RGB::named(WHEAT), RGB::named(BLACK));
    ctx.print_color_centered(
        20,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "The Ruztoo Dungeon",
    );
    ctx.print_color_centered(21, RGB::named(CYAN), RGB::named(BLACK), "By Rev");
    ctx.print_color_centered(
        22,
        RGB::named(GREY),
        RGB::named(BLACK),
        "Use the arrow keys and Enter",
    );

    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        ctx.print_color_centered(
            24,
            get_option_color(selection, MainMenuSelection::NewGame),
            RGB::named(BLACK),
            "Begin New Game",
        );
        ctx.print_color_centered(
            25,
            get_option_color(selection, MainMenuSelection::LoadGame),
            RGB::named(BLACK),
            "Load Game",
        );
        ctx.print_color_centered(
            26,
            get_option_color(selection, MainMenuSelection::Quit),
            RGB::named(BLACK),
            "Quit",
        );

        match ctx.key {
            None => {
                return MainMenuResult::NoSelection {
                    selected: selection,
                }
            }
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    }
                }
                VirtualKeyCode::Up => {
                    let mut new_selection;
                    match selection {
                        MainMenuSelection::NewGame => new_selection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => new_selection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => new_selection = MainMenuSelection::LoadGame,
                    }
                    if new_selection == MainMenuSelection::LoadGame && !save_exists {
                        new_selection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection {
                        selected: new_selection,
                    };
                }
                VirtualKeyCode::Down => {
                    let mut new_selection;
                    match selection {
                        MainMenuSelection::NewGame => new_selection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => new_selection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => new_selection = MainMenuSelection::NewGame,
                    }
                    if new_selection == MainMenuSelection::LoadGame && !save_exists {
                        new_selection = MainMenuSelection::Quit;
                    }
                    return MainMenuResult::NoSelection {
                        selected: new_selection,
                    };
                }
                VirtualKeyCode::Return => {
                    return MainMenuResult::Selected {
                        selected: selection,
                    }
                }
                _ => {
                    return MainMenuResult::NoSelection {
                        selected: selection,
                    }
                }
            },
        }
    }
    MainMenuResult::NoSelection {
        selected: MainMenuSelection::NewGame,
    }
}

fn get_option_color(selection: MainMenuSelection, option: MainMenuSelection) -> RGB {
    if selection == option {
        RGB::named(MAGENTA)
    } else {
        RGB::named(WHITE)
    }
}

pub fn unequip_item_menu(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        INVENTORY_X,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );
    ctx.print_color(
        INVENTORY_X + 3,
        y - 2,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "Unequip Which Item?",
    );
    ctx.print_color(
        INVENTORY_X + 3,
        y + count as i32 + 1,
        RGB::named(YELLOW),
        RGB::named(BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _p, name) in (&entities, &backpack, &names)
        .join()
        .filter(|item| item.1.owner == *player_entity)
    {
        print_item_options_menu(&name.name.to_string(), y, j, ctx);
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    capture_item_options_selection(ctx, equippable, count as i32)
}

pub fn print_item_options_menu(name: &String, y: i32, j: i32, ctx: &mut BTerm) {
    ctx.set(
        INVENTORY_X + 2,
        y,
        RGB::named(WHITE),
        RGB::named(BLACK),
        to_cp437('('),
    );
    ctx.set(
        INVENTORY_X + 3,
        y,
        RGB::named(WHITE),
        RGB::named(BLACK),
        97 + j as FontCharType,
    );
    ctx.set(
        INVENTORY_X + 4,
        y,
        RGB::named(WHITE),
        RGB::named(BLACK),
        to_cp437(')'),
    );

    ctx.print(INVENTORY_X + 6, y, name.to_string());
}

pub fn capture_item_options_selection(
    ctx: &mut BTerm,
    options: Vec<Entity>,
    count: i32,
) -> (ItemMenuResult, Option<Entity>) {
    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count {
                    return (ItemMenuResult::Selected, Some(options[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut BTerm) -> GameOverResult {
    ctx.print_color_centered(25, RGB::named(YELLOW), RGB::named(BLACK), "RIP You");
    ctx.print_color_centered(
        27,
        RGB::named(MAGENTA),
        RGB::named(BLACK),
        "Press any key to return to the main menu",
    );

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}
