use bracket_lib::color::{BLACK, GREY, RGB};
use bracket_lib::prelude::{BTerm, Point, to_cp437};
use bracket_lib::terminal::FontCharType;
use specs::{Join, World, WorldExt};

use crate::components::{Hidden, Position, Renderable};
use crate::Map;
use crate::map::TileType;

const SHOW_BOUNDARIES: bool = true;

pub fn render_camera(ecs: &World, ctx: &mut BTerm) {
    let map= ecs.fetch::<Map>();
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);

    let map_width = map.width-1;
    let map_height = map.height-1;

    // x and y are the coordinates on the screen
    // tx and ty are coordinates of the Tiles
    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                if map.revealed_tiles[tx as usize][ty as usize] {
                    let (glyph, fg, bg) = get_tile_glyph(tx as usize, ty as usize, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(x, y, RGB::named(GREY), RGB::named(BLACK), to_cp437('.'))
            }
            x += 1;
        }
        y += 1;
    }

    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();

    let mut data = (&positions, &renderables, !&hidden).join().collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render, _h) in data.iter() {
        if map.visible_tiles[pos.x as usize][pos.y as usize] {
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            if entity_screen_x > 0 && entity_screen_x < map_width && entity_screen_y > 0 && entity_screen_y < map_height {
                ctx.set(entity_screen_x, entity_screen_y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

pub fn render_debug_map(map: &Map, ctx: &mut BTerm) {
    let player_pos = Point::new(map.width / 2, map.height / 2);
    let (x_chars, y_chars) = ctx.get_char_size();

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    let map_width = map.width-1;
    let map_height = map.height-1;

    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                if map.revealed_tiles[tx as usize][ty as usize] {
                    let (glyph, fg, bg) = get_tile_glyph(tx as usize, ty as usize, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(x, y, RGB::named(GREY), RGB::named(BLACK), to_cp437('.'))
            }
            x += 1;
        }
        y += 1;
    }

}

pub fn get_screen_bounds(ecs: &World, ctx: &mut BTerm) -> (i32, i32, i32, i32){
    let player_pos = ecs.fetch::<Point>();
    let (x_chars, y_chars) = ctx.get_char_size();

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    (min_x, max_x, min_y, max_y)
}

fn get_tile_glyph(x: usize, y: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let glyph;
    let mut fg;
    let mut bg = RGB::from_f32(0.,0.,0.);

    match map.tiles[x][y] {
        TileType::Floor => {
            glyph = to_cp437('.');
            fg = RGB::from_u8(170, 131, 96);
            bg = RGB::from_u8(170, 131, 96);
        }
        TileType::Wall => {
            glyph = wall_glyph(map, x as i32, y as i32);
            fg = RGB::from_u8(127, 30, 20);
        }
        TileType::DownStairs => {
            glyph = to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
    }
    if map.bloodstains.contains(&(x as i32, y as i32)) {
        bg = RGB::from_f32(0.75, 0., 0.);
    }
    if !map.visible_tiles[x][y] {
        fg = fg.to_greyscale();
        bg = RGB::from_f32(0., 0., 0.);
    }
    (glyph, fg, bg)
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2i32 {
        return 35;
    }
    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8;
    }

    match mask {
        0 => 9,
        1 => 186,
        2 => 186,
        3 => 186,
        4 => 205,
        5 => 188,
        6 => 187,
        7 => 185,
        8 => 205,
        9 => 200,
        10 => 201,
        11 => 204,
        12 => 205,
        13 => 202,
        14 => 203,
        15 => 206,
        _ => 35,
    }
}
fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    map.tiles[x as usize][y as usize] == TileType::Wall
        && map.revealed_tiles[x as usize][y as usize]
}
