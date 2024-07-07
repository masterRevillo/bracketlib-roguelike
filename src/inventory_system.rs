use bracket_lib::color::{BLACK, GREEN, MAGENTA, ORANGE, RED, RGB};
use bracket_lib::prelude::{field_of_view, to_cp437};
use specs::prelude::*;

use crate::components::{AreaOfEffect, Artefact, CombatStats, Confusion, Consumable, Equippable, Equipped, HungerClock, InBackpack, InflictsDamage, MagicMapper, Name, Position, ProvidesFood, ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickUpItem, WantsToUnequipItem, WantsToUseItem};
use crate::gamelog::GameLog;
use crate::hunger_system::HungerSystem;
use crate::map::Map;
use crate::particle_system::ParticleBuilder;
use crate::RunState;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickUpItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by})
                .expect("Failed to insert item in backpack");
            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You picked up the {}", names.get(pickup.item).unwrap().name));
            }

        }
        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, Artefact>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
        ReadStorage<'a, MagicMapper>,
        WriteExpect<'a, RunState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut map,
            mut wants_use_item,
            names,
            healing,
            artefacts,
            mut combat_stats,
            consumables,
            inflicts_damage,
            mut suffer_damage,
            area_of_effect,
            mut confusions,
            equippable,
            mut equipped,
            mut inBackpack,
            mut particle_builder,
            positions,
            provides_food,
            mut hunger_clock,
            magic_mapper,
            mut runsatate
        ) = data;

        for (entity, use_item) in (&entities, &wants_use_item).join() {
            let mut targets: Vec<Entity> = Vec::new();
            match use_item.target {
                None => { targets.push(*player_entity); }
                Some(target) => {
                    let area_effect = area_of_effect.get(use_item.item);
                    match area_effect {
                        None => {
                            for mob in map.tile_content[target.x as usize][target.y as usize].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            let mut blast_tiles = field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| map.is_tile_in_bounds(p.x, p.y));
                            for tile in blast_tiles.iter() {
                                for mob in map.tile_content[tile.x as usize][tile.y as usize].iter() {
                                    targets.push(*mob);
                                }
                                particle_builder.request(
                                    tile.x, tile.y, RGB::named(ORANGE), RGB::named(BLACK), to_cp437('!'), 200.0
                                );
                            }
                        }
                    }
                }
            }

            let item_equippable = equippable.get(use_item.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            gamelog.entries.push(format!("You unequip {}", name.name));
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        inBackpack.insert(*item, InBackpack{ owner: target }).expect("Unable to insert item in backpack");
                    }

                    equipped.insert(use_item.item, Equipped { owner: target, slot: target_slot}).expect("Unable to equip item");
                    inBackpack.remove(use_item.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!("You equip {}", names.get(use_item.item).unwrap().name));
                    }
                }
            }
            let item_provides_food = provides_food.get(use_item.item);
            if let Some(item_provides_food) = item_provides_food {
                let target = targets[0];
                let hc = hunger_clock.get_mut(target);
                if let Some(hc) = hc {
                    let updated_state = HungerSystem::calculate_new_hunger_state(
                        hc.hunger_points, hc.state, item_provides_food.points
                    );
                    hc.hunger_points = updated_state.1;
                    hc.state = updated_state.0;
                    gamelog.entries.push(format!(
                        "You eat the {}. It fills you up.", names.get(use_item.item).unwrap().name
                    ))
                }
            }

            let item_heals = healing.get(use_item.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("You drink the {}, and it heals {}hp", names.get(use_item.item).unwrap().name, healer.heal_amount));
                            }
                            let pos = positions.get(*target);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.x, pos.y, RGB::named(GREEN), RGB::named(BLACK), to_cp437('❤'), 200.0
                                );
                            }
                        }
                    }
                }
            }
            let artefact = artefacts.get(use_item.item);
            match artefact {
                None => {}
                Some(art) => {
                    if entity == *player_entity {
                        gamelog.entries.push(format!("This artefact is named {}, and it is worth {} gold", art.name, art.value));
                    }
                }
            }

            let item_damages = inflicts_damage.get(use_item.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(use_item.item).unwrap();
                            gamelog.entries.push(
                                format!("You use {} on {}, inflicting {} damage",
                                        item_name.name,
                                        mob_name.name,
                                        damage.damage
                                )
                            );
                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.x, pos.y, RGB::named(RED), RGB::named(BLACK), to_cp437('!'), 200.0
                                );
                            }
                        }
                    }
                }
            }
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confusions.get(use_item.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(use_item.item).unwrap();
                                gamelog.entries.push(
                                    format!("You use {} on {}, confusing them",
                                            item_name.name,
                                            mob_name.name,
                                    )
                                );
                                let pos = positions.get(*mob);
                                if let Some(pos) = pos {
                                    particle_builder.request(
                                        pos.x, pos.y, RGB::named(MAGENTA), RGB::named(BLACK), to_cp437('?'), 200.0
                                    );
                                }
                            }
                        }
                    }
                }
            }
            let is_mapper = magic_mapper.get(use_item.item);
            if let Some(is_mapper) = is_mapper {
                gamelog.entries.push("You use the scroll, which reveals the map to you".to_string());
                *runsatate = RunState::MagicMapReveal{ row: 0 };
            };
            for mob in add_confusion.iter() {
                confusions.insert(mob.0, Confusion{ turns: mob.1})
                    .expect("Unable to insert status");
            }
            let consumable = consumables.get(use_item.item);
            match consumable {
                None => {}
                Some(_) => {
                    entities.delete(use_item.item).expect("Cannot delete consumable item");
                }
            }
        }
        wants_use_item.clear();
    }
}

pub struct ItemDropSystem{}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position{x: 0, y: 0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x: dropper_pos.x, y: dropper_pos.y})
                .expect("Unable to insert position while dropping item");

            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}", names.get(to_drop.item).unwrap().name));
            }
        }
        wants_drop.clear();
    }
}

pub struct ItemUnequippingSystem {}

impl<'a> System<'a> for ItemUnequippingSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToUnequipItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut want_to_unequip, mut equipped, mut backpack) = data;

        for (entity, to_unequip) in (&entities, &want_to_unequip).join() {
            equipped.remove(to_unequip.item);
            backpack.insert(to_unequip.item, InBackpack{ owner: entity})
                .expect("Unable to insert item into backpack");
        }

        want_to_unequip.clear();
    }
}
