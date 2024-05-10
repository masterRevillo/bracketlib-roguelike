use bracket_lib::color::{BLACK, GREY, MAGENTA, RED, RGB, WHITE, YELLOW};
use bracket_lib::prelude::{BTerm, letter_to_option, Point, to_cp437, VirtualKeyCode};
use bracket_lib::prelude::VirtualKeyCode::N;
use bracket_lib::terminal::FontCharType;
use specs::prelude::*;

use crate::components::{CombatStats, InBackpack, Item, Name, Player, Position};
use crate::gamelog::GameLog;
use crate::map::{Map, MAPHEIGHT, MAPWIDTH};
use crate::State;

const GUIY: usize = MAPHEIGHT;
const GUIHEIGHT: usize = 6;
const GUIWIDTH: usize = MAPWIDTH-1;

pub fn dwaw_ui(ecs: &World, ctx: &mut BTerm) {
    ctx.draw_box(0, MAPHEIGHT, GUIWIDTH, GUIHEIGHT, RGB::named(WHITE), RGB::named(BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let log = ecs.fetch::<GameLog>();
    for(_p, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, GUIY, RGB::named(YELLOW), RGB::named(BLACK), &health);

        ctx.draw_bar_horizontal(28, GUIY, 51, stats.hp, stats.max_hp, RGB::named(RED), RGB::named(BLACK))
    }

    let mut y = GUIY + 1;
    for s in log.entries.iter().rev() {
        if y < GUIY + GUIHEIGHT {ctx.print(2, y, s);}
        y += 1;
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(MAGENTA));
    draw_tooltips(ecs, ctx);
}

pub fn draw_tooltips(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if !map.is_tile_in_bounds(mouse_pos.0, mouse_pos.1) {return;}
    let mut tooltip: Vec<String> = Vec::new();
    for (name, postion) in (&names, &positions).join() {
        if postion.x == mouse_pos.0 && postion.y == mouse_pos.1 && map.visible_tiles[postion.x as usize][postion.y as usize] {
            tooltip.push(name.name.to_string());
        }
    }

    ctx.print(GUIWIDTH-20, GUIY+1, format!("Coordinates: ({},{})", mouse_pos.0, mouse_pos.1));

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;

        if mouse_pos.0 > MAPWIDTH as i32/2 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) -1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, RGB::named(WHITE), RGB::named(GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(WHITE), RGB::named(GREY), &"->".to_string());
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) -1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(WHITE), RGB::named(GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(WHITE), RGB::named(GREY), &"<-".to_string());
        }
    }

}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult{ Cancel, NoResponse, Selected }

const INVENTORY_X: usize = MAPWIDTH / 2 - 20;

pub fn show_inventory(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(
        |item| item.0.owner == *player_entity
    );
    let count = inventory.count();

    let mut y = (25 - (count /2)) as i32;
    ctx.draw_box(INVENTORY_X, y-2, 31, (count + 3) as i32, RGB::named(WHITE), RGB::named(BLACK));
    ctx.print_color(INVENTORY_X + 3, y-2, RGB::named(YELLOW), RGB::named(BLACK), "Inventory");
    ctx.print_color(INVENTORY_X + 3, y+count as i32+1, RGB::named(YELLOW), RGB::named(BLACK), "ESCAPE to cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for(entity, _p, name) in (&entities, &backpack, &names).join().filter(
        |item| item.1.owner == *player_entity
    ) {
        ctx.set(INVENTORY_X + 2, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('('));
        ctx.set(INVENTORY_X + 3, y, RGB::named(WHITE), RGB::named(BLACK), 97+j as FontCharType);
        ctx.set(INVENTORY_X + 4, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(')'));

        ctx.print(INVENTORY_X + 6, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]))
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut BTerm) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(
        |item| item.0.owner == *player_entity
    );
    let count = inventory.count();

    let mut y = (25 - (count /2)) as i32;
    ctx.draw_box(INVENTORY_X, y-2, 31, (count + 3) as i32, RGB::named(WHITE), RGB::named(BLACK));
    ctx.print_color(INVENTORY_X + 3, y-2, RGB::named(YELLOW), RGB::named(BLACK), "Drop Which Item?");
    ctx.print_color(INVENTORY_X + 3, y+count as i32+1, RGB::named(YELLOW), RGB::named(BLACK), "ESCAPE to cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for(entity, _p, name) in (&entities, &backpack, &names).join().filter(
        |item| item.1.owner == *player_entity
    ) {
        ctx.set(INVENTORY_X + 2, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('('));
        ctx.set(INVENTORY_X + 3, y, RGB::named(WHITE), RGB::named(BLACK), 97+j as FontCharType);
        ctx.set(INVENTORY_X + 4, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(')'));

        ctx.print(INVENTORY_X + 6, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]))
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}
