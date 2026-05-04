use bracket_lib::color::{
    BLACK, CHOCOLATE2, CORNFLOWERBLUE, CYAN, DARK_GRAY, FORESTGREEN, GREY, LIGHT_GRAY, LIGHT_SLATE,
    MEDIUM_AQUAMARINE, NAVY_BLUE, RGB,
};
use bracket_lib::prelude::{console, to_cp437, BTerm, Point, CHOCOLATE, SADDLEBROWN};
use bracket_lib::terminal::{FontCharType, GREEN1};
use specs::{Join, World, WorldExt};

use crate::components::{Hidden, Position, Renderable};
use crate::map::tiletype::TileType;
use crate::{Map, DEBUGGING, SCREEN_X, SCREEN_Y};

const SHOW_BOUNDARIES: bool = false;

pub const VIEWPORT_X: i32 = SCREEN_X - 31;
pub const VIEWPORT_Y: i32 = SCREEN_Y - 15;

pub fn render_map(map: &Map, ctx: &mut BTerm) {
    let (min_x, max_x, min_y, max_y) = (0, map.width, 0, map.height);

    let map_width = map.width - 1;
    let map_height = map.height - 1;

    // x and y are the coordinates on the screen
    // tx and ty are coordinates of the Tiles
    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                if map.revealed_tiles[tx as usize][ty as usize] {
                    let (glyph, fg, bg) = get_tile_glyph(tx as usize, ty as usize, map);
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

pub fn render_camera(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.fetch::<Map>();
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(ecs, ctx);

    let map_width = map.width;
    let map_height = map.height;

    // x and y are the coordinates on the screen
    // tx and ty are coordinates of the Tiles
    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                if map.revealed_tiles[tx as usize][ty as usize] || DEBUGGING {
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

    let mut data = (&positions, &renderables, !&hidden)
        .join()
        .collect::<Vec<_>>();
    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render, _h) in data.iter() {
        if map.visible_tiles[pos.x as usize][pos.y as usize] {
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            if entity_screen_x > 0
                && entity_screen_x < map_width
                && entity_screen_y > 0
                && entity_screen_y < map_height
            // Had to put this here to stop entities rendering outside the map window
                && entity_screen_x < VIEWPORT_X
                && entity_screen_y < VIEWPORT_Y
            {
                ctx.set(
                    entity_screen_x,
                    entity_screen_y,
                    render.fg,
                    render.bg,
                    render.glyph,
                );
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

    let map_width = map.width - 1;
    let map_height = map.height - 1;

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

pub fn get_screen_bounds(ecs: &World, ctx: &mut BTerm) -> (i32, i32, i32, i32) {
    let player_pos = ecs.fetch::<Point>();
    // let (x_chars, y_chars) = ctx.get_char_size();
    // Use the screen dimensions to offset where the camera is looking. This is to preserve the
    // viewport with the addition of the GUI
    let (x_chars, y_chars) = (
        VIEWPORT_X,
        VIEWPORT_Y, // (SCREEN_X as f32 * 0.7) as i32,
                   // (SCREEN_Y as f32 * 0.7) as i32,
    );

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
    let mut bg = RGB::from_f32(0., 0., 0.);
    let noise = map.noise[x][y];

    match (map.tiles[x][y], noise) {
        (TileType::Floor, _) => {
            glyph = to_cp437('.');
            fg = RGB::from_u8(170, 131, 96);
            bg = RGB::from_u8(
                (170.0 * noise * 0.75) as u8,
                (131.0 * noise * 0.75) as u8,
                (96.0 * noise * 0.75) as u8,
            );
        }
        // (TileType::Floor, _) => {
        // glyph = to_cp437('~');
        // fg = RGB::from_u8(140, 101, 66);
        // bg = RGB::from_u8(170, 131, 96);
        // }
        (TileType::Wall, _) => {
            glyph = wall_glyph(map, x as i32, y as i32);
            fg = RGB::from_u8(
                127 + (127.0 * noise * 0.5) as u8,
                30 + (30.0 * noise * 0.5) as u8,
                20 + (20.0 * noise * 0.5) as u8,
            );
        }
        (TileType::DownStairs, _) => {
            glyph = to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        (TileType::Bridge, _) => {
            glyph = to_cp437('|');
            fg = RGB::named(CHOCOLATE);
            bg = RGB::named(SADDLEBROWN);
        }
        (TileType::Road, _) => {
            glyph = to_cp437('~');
            fg = RGB::named(GREY);
            bg = RGB::named(LIGHT_GRAY);
        }
        (TileType::Grass, _) => {
            glyph = to_cp437('"');
            fg = RGB::named(FORESTGREEN);
            bg = RGB::named(GREEN1);
        }
        (TileType::ShallowWater, _) => {
            glyph = to_cp437('≈');
            fg = RGB::named(CYAN);
            bg = RGB::named(MEDIUM_AQUAMARINE);
        }
        (TileType::DeepWater, _) => {
            glyph = to_cp437('≈');
            fg = RGB::from_f32(0.1 * noise, 0.1 * noise, 0.6 * noise);
            //100, 149, 237
            bg = RGB::from_f32(0.45 * noise, 0.55 * noise, 0.9 * noise);
        }
        (TileType::WoodFloor, _) => {
            glyph = to_cp437('.');
            fg = RGB::named(CHOCOLATE);
            bg = RGB::named(CHOCOLATE2);
        }
        (TileType::Gravel, _) => {
            glyph = to_cp437('\'');
            fg = RGB::named(LIGHT_SLATE);
            bg = RGB::named(DARK_GRAY)
        }
        (TileType::Moss, _) => {
            glyph = to_cp437('#');
            fg = RGB::from_u8(91, 128, 125);
            bg = RGB::from_u8(91, 168, 110);
        }
    }
    if map.bloodstains.contains(&(x as i32, y as i32)) {
        bg = RGB::from_f32(0.75, 0., 0.);
    }
    if !map.visible_tiles[x][y] {
        fg = fg.lerp(RGB::named(BLACK), 0.5);
        bg = bg.lerp(RGB::named(BLACK), 0.7);
        // bg = RGB::from_f32(0., 0., 0.);
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
