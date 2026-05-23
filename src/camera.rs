use bracket_lib::color::{BLACK, GREY, RGB};
use bracket_lib::prelude::{to_cp437, BTerm, Point};
use specs::{Join, World, WorldExt};

use crate::components::{Hidden, Position, Renderable};
use crate::map::themes::tile_glyph;
use crate::{Map, DEBUGGING, SCREEN_X, SCREEN_Y};

const SHOW_BOUNDARIES: bool = false;

pub const VIEWPORT_X: i32 = SCREEN_X - 31;
pub const VIEWPORT_Y: i32 = SCREEN_Y - 15;

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
                    let (glyph, fg, bg) = tile_glyph(tx as usize, ty as usize, &*map);
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
                    let (glyph, fg, bg) = tile_glyph(tx as usize, ty as usize, &*map);
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
