use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;

use bracket_lib::color::{
    BLACK, CYAN, CYAN3, GOLD, LIGHT_GRAY, MAGENTA, MAROON, OLIVE, ORANGE, PERU, PINK, RED, RGB,
    YELLOW,
};
use bracket_lib::prelude::VirtualKeyCode::M;
use bracket_lib::prelude::{console, to_cp437, FontCharType, CHOCOLATE3};
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};

use crate::components::{
    AreaOfEffect, Artefact, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus,
    EntryTrigger, EquipmentSlot, Equippable, Hidden, HungerClock, HungerState, InflictsDamage,
    Item, MagicMapper, MeleeAttackBonus, Monster, Name, Player, Position, ProvidesFood,
    ProvidesHealing, Ranged, Renderable, SerializeMe, SingleActivation, Viewshed,
};
use crate::map::{Map, TileType, MAPWIDTH};
use crate::random_tables::EntryType::*;
use crate::random_tables::{EntryType, RandomTable};
use crate::rect::Rect;
use crate::util::namegen::{generate_artefact_name, generate_ogur_name};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 3;

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(HungerClock {
            state: HungerState::WellFed,
            hunger_points: 20,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn room_table(level: i32) -> RandomTable {
    RandomTable::new()
        .add(Bisat, 10)
        .add(Ogur, 1 + level)
        .add(HealthPotion, 7)
        .add(FireballScroll, 2 + level)
        .add(ConfusionScroll, 2 + level)
        .add(MagicMissileScroll, 4)
        .add(ChickenLeg, 4)
        .add(Sandwich, 5)
        .add(GobletOfWine, 4)
        .add(Artefact, (level - 1))
        .add(Dagger, 3)
        .add(Shield, 3)
        .add(Longsword, level - 1)
        .add(TowerShield, level - 1)
        .add(MagicMappingScroll, 2)
        .add(BearTrap, 5)
}

fn ogur(ecs: &mut World, x: i32, y: i32) {
    monster(
        ecs,
        x,
        y,
        to_cp437('o'),
        generate_ogur_name(),
        RGB::named(OLIVE),
        CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        },
    )
}

fn bisat(ecs: &mut World, x: i32, y: i32) {
    monster(
        ecs,
        x,
        y,
        to_cp437('b'),
        generate_ogur_name(),
        RGB::named(PERU),
        CombatStats {
            max_hp: 12,
            hp: 12,
            defense: 1,
            power: 3,
        },
    )
}

fn monster<S: ToString>(
    ecs: &mut World,
    x: i32,
    y: i32,
    glyph: FontCharType,
    name: S,
    color: RGB,
    cstats: CombatStats,
) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: color,
            bg: RGB::named(BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(cstats)
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn spawn_entity(ecs: &mut World, spawn: &(&(i32, i32), &EntryType)) {
    let coords = spawn.0;
    match spawn.1 {
        None => {}
        Bisat => bisat(ecs, coords.0, coords.1),
        Ogur => ogur(ecs, coords.0, coords.1),
        HealthPotion => health_potion(ecs, coords.0, coords.1),
        FireballScroll => fireball_stroll(ecs, coords.0, coords.1),
        ConfusionScroll => confusion_stroll(ecs, coords.0, coords.1),
        MagicMissileScroll => magic_missile_stroll(ecs, coords.0, coords.1),
        Sandwich => food(
            ecs,
            to_cp437('='),
            "Sandwich",
            RGB::named(CHOCOLATE3),
            2,
            200,
            coords.0,
            coords.1,
        ),
        ChickenLeg => food(
            ecs,
            to_cp437('q'),
            "Chicken Leg",
            RGB::named(CHOCOLATE3),
            3,
            150,
            coords.0,
            coords.1,
        ),
        GobletOfWine => food(
            ecs,
            to_cp437('u'),
            "Goblet of Wine",
            RGB::named(MAROON),
            1,
            25,
            coords.0,
            coords.1,
        ),
        Artefact => artefact(ecs, coords.0, coords.1),
        Dagger => dagger(ecs, coords.0, coords.1),
        Shield => shield(ecs, coords.0, coords.1),
        Longsword => longsword(ecs, coords.0, coords.1),
        TowerShield => tower_shield(ecs, coords.0, coords.1),
        MagicMappingScroll => magic_mapping_stroll(ecs, coords.0, coords.1),
        BearTrap => bear_trap(ecs, coords.0, coords.1),
    }
}

pub fn spawn_room(ecs: &mut World, room: &Rect, map_level: i32) {
    let mut possible_targets: Vec<(i32, i32)> = Vec::new();
    {
        let map = ecs.fetch::<Map>();
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                if map.tiles[x as usize][y as usize] == TileType::Floor {
                    possible_targets.push((x, y))
                }
            }
        }
    }
    spawn_region(ecs, &possible_targets, map_level);

    // let spawn_table = room_table(map_level);
    // let mut spawn_points: HashMap<(i32, i32), EntryType> = HashMap::new();
    // {
    //     let mut rng = ecs.write_resource::<RandomNumberGenerator>();
    //     let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_level - 1) - 3;
    //     for _i in 0..num_spawns {
    //         let mut added = false;
    //         let mut tries = 0;
    //         while !added && tries < 20 {
    //             let x = room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1));
    //             let y = room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1));
    //             if !spawn_points.contains_key(&(x, y)) {
    //                 spawn_points.insert((x, y), spawn_table.roll(&mut rng));
    //                 added = true;
    //             } else {
    //                 tries += 1;
    //             }
    //         }
    //     }
    // }
    // for spawn in spawn_points.iter() {
    //     console::log(format!(
    //         "spawning stuff at {:?} in room with boundaries x{},x{},y{},y{}",
    //         spawn.0, room.x1, room.x2, room.y1, room.y2
    //     ));
    //     spawn_entity(ecs, &spawn);
    // }
}

pub fn spawn_region(ecs: &mut World, area: &[(i32, i32)], map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<(i32, i32), EntryType> = HashMap::new();
    let mut areas: Vec<(i32, i32)> = Vec::from(area);

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = i32::min(
            areas.len() as i32,
            rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3,
        );
        if num_spawns == 0 {
            return;
        }
        for _i in 0..num_spawns {
            let idx = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };
            spawn_points.insert(areas[idx], spawn_table.roll(&mut rng));
            areas.remove(idx);
        }
    }

    for spawn in spawn_points.iter() {
        spawn_entity(ecs, &spawn);
    }
}

fn artefact(ecs: &mut World, x: i32, y: i32) {
    let value: i32;
    {
        value = ecs.write_resource::<RandomNumberGenerator>().range(2, 40) * 500
    }
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('{'),
            fg: RGB::named(GOLD),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Artefact".to_string(),
        })
        .with(Item {})
        .with(Artefact {
            name: generate_artefact_name(),
            value,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('¡'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(ProvidesHealing { heal_amount: 8 })
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn food<T: ToString>(
    ecs: &mut World,
    glyph: FontCharType,
    name: T,
    color: RGB,
    heal_amount: i32,
    food_amount: i32,
    x: i32,
    y: i32,
) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: color,
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: name.to_string(),
        })
        .with(Item {})
        .with(ProvidesHealing { heal_amount })
        .with(ProvidesFood {
            points: food_amount,
        })
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_stroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('~'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_stroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('~'),
            fg: RGB::named(ORANGE),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(AreaOfEffect { radius: 3 })
        .with(InflictsDamage { damage: 20 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_stroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('~'),
            fg: RGB::named(PINK),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('-'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Dagger".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleeAttackBonus { attack: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('('),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('/'),
            fg: RGB::named(LIGHT_GRAY),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Longsword".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleeAttackBonus { attack: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('['),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Tower Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_mapping_stroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('~'),
            fg: RGB::named(CYAN3),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Mapping Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(MagicMapper {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('^'),
            fg: RGB::named(RED),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Bear Trap".to_string(),
        })
        .with(Hidden {})
        .with(EntryTrigger {})
        .with(InflictsDamage { damage: 6 })
        .with(SingleActivation {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
