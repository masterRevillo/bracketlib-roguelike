use std::os::unix::raw::ino_t;
use bracket_lib::color::{BLACK, GREY, MAGENTA, RED, RGB, WHITE, YELLOW};
use bracket_lib::prelude::{BTerm, Point};
use specs::prelude::*;
use crate::components::{CombatStats, Name, Player, Position};
use crate::gamelog::GameLog;
use crate::map::{Map, MAPHEIGHT, MAPWIDTH};

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
