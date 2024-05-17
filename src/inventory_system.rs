use std::fmt::format;
use bracket_lib::prelude::field_of_view;
use specs::prelude::*;
use crate::components::{CombatStats, Consumable, InBackpack, Name, Position, ProvidesHealing, WantsToUseItem, WantsToDropItem, WantsToPickUpItem, Examinable, Artefact, InflictsDamage, SufferDamage, AreaOfEffect, Confusion, Equippable, Equipped, WantsToUnequipItem};
use crate::gamelog::GameLog;
use crate::map::Map;

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
        ReadExpect<'a, Map>,
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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            map,
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
            mut inBackpack
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
                            )
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
                                )
                            }
                        }
                    }
                }
            }
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
