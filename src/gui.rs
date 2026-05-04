use bracket_lib::color::{
    BLACK, BLUE, CYAN, GREEN, GREY, MAGENTA, ORANGE, RED, RGB, WHEAT, WHITE, YELLOW,
};
use bracket_lib::prelude::{
    letter_to_option, to_cp437, BTerm, DijkstraMap, DistanceAlg, Point, VirtualKeyCode,
};
use bracket_lib::terminal::FontCharType;
use specs::prelude::*;

use crate::camera::{get_screen_bounds, VIEWPORT_X, VIEWPORT_Y};
use crate::components::{
    Attribute, Attributes, Consumable, Equipped, Hidden, HungerClock, HungerState, InBackpack,
    Name, Player, Pools, Position, Viewshed,
};
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::rex_assets::RexAssets;
use crate::saveload_system::does_save_exist;
use crate::{RunState, State, DEBUGGING, SCREEN_X, SCREEN_Y};

const GUIHEIGHT: usize = 6;
const GUIY: usize = SCREEN_Y as usize - GUIHEIGHT - 1;
const GUIWIDTH: usize = 30 - 1;

pub fn dwaw_ui(ecs: &World, ctx: &mut BTerm) {
    ctx.draw_box(
        0,
        SCREEN_Y as usize - GUIHEIGHT - 1,
        GUIWIDTH,
        GUIHEIGHT,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );

    let box_gray: RGB = RGB::from_hex("#999999").expect("fail");
    let black = RGB::named(BLACK);

    let pools = ecs.read_storage::<Pools>();
    let players = ecs.read_storage::<Player>();
    let hunger = ecs.read_storage::<HungerClock>();
    let log = ecs.fetch::<GameLog>();
    let map = ecs.fetch::<Map>();
    for (_p, stats, hc) in (&players, &pools, &hunger).join() {
        let health = format!(
            " HP: {} / {} ",
            stats.hit_points.current, stats.hit_points.max
        );
        ctx.print_color(20, GUIY, RGB::named(YELLOW), RGB::named(BLACK), &health);

        ctx.draw_bar_horizontal(
            36,
            GUIY,
            51,
            stats.hit_points.current,
            stats.hit_points.max,
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

struct Tooltip {
    lines: Vec<String>,
}

impl Tooltip {
    fn new() -> Self {
        Tooltip { lines: Vec::new() }
    }

    fn add<S: ToString>(&mut self, line: S) {
        self.lines.push(line.to_string());
    }

    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            if s.len() > max {
                max = s.len();
            }
        }
        max as i32 + 2i32
    }

    fn height(&self) -> i32 {
        self.lines.len() as i32 + 2i32
    }

    fn render(&self, ctx: &mut BTerm, x: i32, y: i32) {
        let box_gray = RGB::from_hex("#999999").expect("fail");
        let light_gray = RGB::from_hex("#DDDDDD").expect("fail");
        let white = RGB::named(WHITE);
        let black = RGB::named(BLACK);
        ctx.draw_box(x, y, self.width() - 1, self.height() - 1, white, box_gray);
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_gray };
            ctx.print_color(x + 1, y + i as i32 + 1, col, black, &s);
        }
    }
}

pub fn draw_tooltips(ecs: &World, ctx: &mut BTerm) {
    let (min_x, _max_x, min_y, _max_y) = get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();
    let attrs = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();
    let dkm = ecs.try_fetch::<DijkstraMap>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    if !map.is_tile_in_bounds(mouse_map_pos.0, mouse_map_pos.1) {
        return;
    }
    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    for (entity, name, pos, _h) in (&entities, &names, &positions, !&hidden).join() {
        if pos.x == mouse_map_pos.0 && pos.y == mouse_map_pos.1 {
            let mut tip = Tooltip::new();
            tip.add(name.name.to_string());

            // Attributes on tooltip
            let att = attrs.get(entity);
            if let Some(att) = att {
                let mut s = "".to_string();
                if att.might.bonus < 0 {
                    s += "Weak. "
                };
                if att.might.bonus > 0 {
                    s += "Strong. "
                };
                if att.quickness.bonus < 0 {
                    s += "Clumsy. "
                };
                if att.quickness.bonus > 0 {
                    s += "Agile. "
                };
                if att.fitness.bonus < 0 {
                    s += "Unhealthy. "
                };
                if att.fitness.bonus > 0 {
                    s += "Healthy. "
                };
                if att.intelligence.bonus < 0 {
                    s += "Dumb. "
                };
                if att.intelligence.bonus > 0 {
                    s += "Smart. "
                };
                if s.is_empty() {
                    s = "Pretty Average".to_string();
                }
                tip.add(s);
            }

            // Pools on tooltip
            let stat = pools.get(entity);
            if let Some(stat) = stat {
                tip.add(format!("Level: {}", stat.level));
            }

            tip_boxes.push(tip);
        }
    }
    if tip_boxes.is_empty() {
        return;
    }
    let box_gray: RGB = RGB::from_hex("#999999").expect("fail");
    let white: RGB = RGB::named(WHITE);

    let arrow;
    let arrow_x;
    let arrow_y = mouse_pos.1;
    if mouse_pos.0 < VIEWPORT_X / 2 {
        arrow = to_cp437('←');
        arrow_x = mouse_pos.0 + 1;
    } else {
        arrow = to_cp437('→');
        arrow_x = mouse_pos.0 - 1;
    }
    ctx.set(arrow_x, arrow_y, white, box_gray, arrow);

    let mut total_height = 0;
    for tt in tip_boxes.iter() {
        total_height += tt.height();
    }

    let mut y = mouse_pos.1 - (total_height / 2);
    while y + (total_height / 2) > VIEWPORT_Y {
        y -= 1;
    }

    for tt in tip_boxes.iter() {
        let x = if mouse_pos.0 < VIEWPORT_X / 2 {
            mouse_pos.0 + 2
        } else {
            mouse_pos.0 - (1 + tt.width())
        };
        tt.render(ctx, x, y);
        y += tt.height();
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

const INVENTORY_X: usize = SCREEN_X as usize / 2 - 20;

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
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(&gs.ecs, ctx);
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
                let screen_x = point.x - min_x;
                let screen_y = point.y - min_y;
                if screen_x > 1
                    && screen_x < (max_x - min_x) - 1
                    && screen_y > 1
                    && screen_y < (max_y - min_y) - 1
                {
                    ctx.set_bg(screen_x, screen_y, RGB::named(BLUE));
                    available_cells.push(point);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;
    let mut valid_target = false;
    for i in available_cells.iter() {
        if i.x == mouse_map_pos.0 && i.y == mouse_map_pos.1 {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(CYAN));
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)),
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

    ctx.draw_box_double(
        (SCREEN_X / 2) - 16,
        18,
        31,
        10,
        RGB::named(WHEAT),
        RGB::named(BLACK),
    );
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
            Some(key) => {
                return match key {
                    VirtualKeyCode::Escape => MainMenuResult::Selected {
                        selected: MainMenuSelection::Quit,
                    },
                    VirtualKeyCode::Up => {
                        let mut new_selection;
                        match selection {
                            MainMenuSelection::NewGame => new_selection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => {
                                new_selection = MainMenuSelection::NewGame
                            }
                            MainMenuSelection::Quit => new_selection = MainMenuSelection::LoadGame,
                        }
                        if new_selection == MainMenuSelection::LoadGame && !save_exists {
                            new_selection = MainMenuSelection::NewGame;
                        }
                        MainMenuResult::NoSelection {
                            selected: new_selection,
                        }
                    }
                    VirtualKeyCode::Down => {
                        let mut new_selection;
                        match selection {
                            MainMenuSelection::NewGame => {
                                new_selection = MainMenuSelection::LoadGame
                            }
                            MainMenuSelection::LoadGame => new_selection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => new_selection = MainMenuSelection::NewGame,
                        }
                        if new_selection == MainMenuSelection::LoadGame && !save_exists {
                            new_selection = MainMenuSelection::Quit;
                        }
                        MainMenuResult::NoSelection {
                            selected: new_selection,
                        }
                    }
                    VirtualKeyCode::Return => MainMenuResult::Selected {
                        selected: selection,
                    },
                    _ => MainMenuResult::NoSelection {
                        selected: selection,
                    },
                }
            }
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

pub fn draw_hollow_box(
    console: &mut BTerm,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┌'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

pub fn draw_ui(ecs: &World, ctx: &mut BTerm) {
    let box_gray = RGB::from_hex("#999999").expect("fail");
    let black = RGB::named(BLACK);
    let white = RGB::named(WHITE);

    // boarders
    draw_hollow_box(ctx, 0, 0, SCREEN_X - 1, SCREEN_Y - 1, box_gray, black);
    draw_hollow_box(ctx, 0, 0, VIEWPORT_X - 1, VIEWPORT_Y - 1, box_gray, black);
    draw_hollow_box(ctx, 0, VIEWPORT_Y - 1, SCREEN_X - 1, 15, box_gray, black);
    draw_hollow_box(ctx, VIEWPORT_X - 1, 0, 30, 8, box_gray, black);
    ctx.set(0, VIEWPORT_Y - 1, box_gray, black, to_cp437('├'));
    ctx.set(VIEWPORT_X - 1, 8, box_gray, black, to_cp437('├'));
    ctx.set(VIEWPORT_X - 1, 0, box_gray, black, to_cp437('┬'));
    ctx.set(
        VIEWPORT_X - 1,
        VIEWPORT_Y - 1,
        box_gray,
        black,
        to_cp437('┴'),
    );
    ctx.set(SCREEN_X - 1, 8, box_gray, black, to_cp437('┤'));
    ctx.set(SCREEN_X - 1, VIEWPORT_Y - 1, box_gray, black, to_cp437('┤'));

    // Map name
    let map = ecs.fetch::<Map>();
    let name_len = map.name.len() + 2;
    let x_pos = (22 - (name_len / 2)) as i32;
    ctx.set(x_pos, 0, box_gray, black, to_cp437('┤'));
    ctx.set(x_pos + name_len as i32, 0, box_gray, black, to_cp437('├'));
    ctx.print_color(x_pos + 1, 0, white, black, &map.name);

    // Stats
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let health = format!(
        "Health: {}/{}",
        player_pools.hit_points.current, player_pools.hit_points.max
    );
    let mana = format!(
        "Mana:   {}/{}",
        player_pools.mana.current, player_pools.mana.max
    );
    ctx.print_color(VIEWPORT_X, 1, white, black, &health);
    ctx.print_color(VIEWPORT_X, 2, white, black, &mana);
    ctx.draw_bar_horizontal(
        VIEWPORT_X + 14,
        1,
        14,
        player_pools.hit_points.current,
        player_pools.hit_points.max,
        RGB::named(RED),
        RGB::named(BLACK),
    );
    ctx.draw_bar_horizontal(
        VIEWPORT_X + 14,
        2,
        14,
        player_pools.mana.current,
        player_pools.mana.max,
        RGB::named(BLUE),
        RGB::named(BLACK),
    );

    // Attributes
    let attributes = ecs.read_storage::<Attributes>();
    let attrs = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &attrs.might, 4, ctx);
    draw_attribute("Quickness:", &attrs.quickness, 5, ctx);
    draw_attribute("Fitness:", &attrs.fitness, 6, ctx);
    draw_attribute("Intelligence:", &attrs.intelligence, 7, ctx);

    // Equipment
    let mut y = 9;
    let equipped = ecs.read_storage::<Equipped>();
    let name = ecs.read_storage::<Name>();
    for (equipped_by, item_name) in (&equipped, &name).join() {
        if equipped_by.owner == *player_entity {
            ctx.print_color(VIEWPORT_X, y, white, black, &item_name.name);
            y += 1;
        }
    }

    y += 1;
    let green = RGB::named(GREEN);
    let yellow = RGB::named(YELLOW);
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut idx = 1;
    for (carried_by, _c, item_name) in (&backpack, &consumables, &name).join() {
        if carried_by.owner == *player_entity && idx < 10 {
            ctx.print_color(VIEWPORT_X, y, yellow, black, &format!("↑{}", idx));
            ctx.print_color(VIEWPORT_X + 3, y, green, black, &item_name.name);
            y += 1;
            idx += 1;
        }
    }

    // Status
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    let hunger_to_print = match hc.state {
        HungerState::WellFed => Some((green, black, "Well Fed")),
        HungerState::Normal => None,
        HungerState::Hungry => Some((RGB::named(ORANGE), black, "Hungry")),
        HungerState::Starving => Some((RGB::named(RED), black, "Starving")),
    };
    if let Some(printables) = hunger_to_print {
        ctx.print_color(VIEWPORT_X, 44, printables.0, printables.1, printables.2);
    }

    // Logs
    let log = ecs.fetch::<GameLog>();
    let mut y = VIEWPORT_Y;
    for s in log.entries.iter().rev() {
        if y < SCREEN_Y - 1 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    draw_tooltips(ecs, ctx);

    let (min_x, _max_x, min_y, _max_y) = get_screen_bounds(ecs, ctx);
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;

    if !map.is_tile_in_bounds(mouse_map_pos.0, mouse_map_pos.1) {
        return;
    }

    ctx.print(
        VIEWPORT_X - 20,
        VIEWPORT_Y + 1,
        format!("Window X/Y: ({},{})", mouse_pos.0, mouse_pos.1),
    );
    ctx.print(
        VIEWPORT_X - 20,
        VIEWPORT_Y + 2,
        format!("Map X/Y: ({},{})", mouse_map_pos.0, mouse_map_pos.1),
    );
    ctx.print(
        VIEWPORT_X - 20,
        VIEWPORT_Y + 3,
        format!(
            "Tile: {:?}; Noise: {}",
            map.tiles[mouse_map_pos.0 as usize][mouse_map_pos.1 as usize],
            map.noise[mouse_map_pos.0 as usize][mouse_map_pos.1 as usize],
        ),
    );
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut BTerm) {
    let black = RGB::named(BLACK);
    let attr_gray: RGB = RGB::from_hex("#CCCCCC").expect("fail");
    ctx.print_color(VIEWPORT_X, y, attr_gray, black, name);
    let color: RGB = if attribute.modifiers < 0 {
        RGB::from_f32(1., 0., 0.)
    } else if attribute.modifiers == 0 {
        RGB::named(WHITE)
    } else {
        RGB::from_f32(0., 1., 0.)
    };
    ctx.print_color(
        VIEWPORT_X + 17,
        y,
        color,
        black,
        &format!("{}", attribute.base + attribute.modifiers),
    );
    ctx.print_color(
        VIEWPORT_X + 23,
        y,
        color,
        black,
        &format!("{}", attribute.bonus),
    );
    if attribute.bonus > 0 {
        ctx.set(VIEWPORT_X + 22, y, color, black, to_cp437('+'))
    };
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
