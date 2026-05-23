use bracket_lib::{
    color::{
        BLACK, CHOCOLATE, CHOCOLATE2, CYAN, DARK_GRAY, FORESTGREEN, GREEN1, LIGHT_GRAY,
        LIGHT_SLATE, MEDIUM_AQUAMARINE, RGB, SADDLEBROWN, YELLOW,
    },
    prelude::{to_cp437, FontCharType},
};

use crate::map::{Map, TileType};

pub fn tile_glyph(x: usize, y: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let (glyph, mut fg, mut bg) = match map.depth {
        2 => get_forest_glyph(x, y, map),
        _ => get_tile_glyph_default(x, y, map),
    };

    if map.bloodstains.contains(&(x as i32, y as i32)) {
        bg = RGB::from_f32(0.75, 0., 0.)
    };

    if !map.visible_tiles[x][y] {
        fg = fg.lerp(RGB::named(BLACK), 0.5);
        bg = bg.lerp(RGB::named(BLACK), 0.7);
    }

    (glyph, fg, bg)
}

fn get_forest_glyph(x: usize, y: usize, map: &Map) -> (u16, RGB, RGB) {
    let glyph;
    let mut fg;
    let mut bg = RGB::from_f32(0., 0., 0.);
    let noise = map.noise[x][y];
    let noise_b = map.n_height[x][y];
    let w_noise = map.w_noise[x][y];

    match (map.tiles[x][y], noise + noise_b) {
        (TileType::Floor, _) => {
            glyph = to_cp437('"');
            let scaler = 0.5;
            fg = RGB::from_u8(
                0,
                (153. + (100.0 * noise * scaler)) as u8,
                0 + (20.0 * noise * scaler) as u8,
            );
            bg = RGB::from_u8(0, 80, 20);
        }
        (TileType::Wall, _) => {
            glyph = to_cp437('♣');
            fg = RGB::from_u8(
                0 + (20.0 * w_noise * 0.5) as u8,
                153 + (100.0 * w_noise * 0.5) as u8,
                0 + (20.0 * w_noise * 0.5) as u8,
            );
            bg = RGB::from_u8(0, 80, 20);
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
            fg = RGB::named(YELLOW);
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
    (glyph, fg, bg)
}

fn get_tile_glyph_default(x: usize, y: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let glyph;
    let mut fg;
    let mut bg = RGB::from_f32(0., 0., 0.);
    let noise = map.noise[x][y];
    let noise_b = map.n_height[x][y];
    let w_noise = map.w_noise[x][y];

    match (map.tiles[x][y], noise + noise_b) {
        (TileType::Floor, _) => {
            glyph = to_cp437('.');
            fg = RGB::from_u8(170, 131, 96);
            let scaler = 0.5;
            bg = RGB::from_u8(
                (170. + (100.0 * noise * scaler)) as u8,
                (131. + (100.0 * noise * scaler)) as u8,
                (96.) as u8,
            );
        }
        (TileType::Wall, _) => {
            //glyph = wall_glyph(map, x as i32, y as i32);
            glyph = to_cp437('.');
            bg = RGB::from_u8(
                127 + (127.0 * w_noise * 0.5) as u8,
                30 + (30.0 * w_noise * 0.5) as u8,
                20 + (20.0 * w_noise * 0.5) as u8,
            );
            fg = bg;
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
            fg = RGB::named(YELLOW);
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
