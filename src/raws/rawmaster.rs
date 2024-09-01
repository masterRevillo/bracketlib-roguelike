use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;

use bracket_lib::color::RGB;
use bracket_lib::prelude::{console, to_cp437};
use bracket_lib::random::RandomNumberGenerator;
use specs::{Builder, Entity, EntityBuilder, World, WorldExt};

use crate::components::{AreaOfEffect, Artefact, Attribute, Attributes, BlocksTile, BlocksVisibility, Bystander, Confusion, Consumable, Door, EntryTrigger, EquipmentSlot, Equippable, Hidden, InflictsDamage, MagicMapper, MeleeAttackBonus, Monster, Name, Pool, Pools, Position, ProvidesFood, ProvidesHealing, Quips, Ranged, SingleActivation, Skill, Skills, Vendor, Viewshed};
use crate::gamesystem::{attr_bonus, mana_at_level, npc_hp};
use crate::random_tables::{EntityType, RandomTable};
use crate::random_tables::EntityType::{Bisat, Ogur, Spectre, TukkaWarrior};
use crate::raws::rawmaster::SpawnType::AtPosition;
use crate::raws::Raws;
use crate::raws::spawn_table_structs::SpawnTableEntry;
use crate::util::namegen::{generate_artefact_name, generate_ogur_name};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 }
}

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<EntityType, usize>,
    mob_index: HashMap<EntityType, usize>,
    prop_index: HashMap<EntityType, usize>,
}

impl RawMaster {
    pub fn empty() -> Self {
        Self {
            raws: Raws { items: Vec::new(), mobs: Vec::new(), props: Vec::new(), spawn_table: Vec::new() },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
        }
    }


    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        let mut entries_used: HashSet<String> = HashSet::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            if entries_used.contains(&item.r#type) {
                console::log(format!("WARNING - duplicate item type in raw file [{}]", item.r#type))
            }
            entries_used.insert(item.r#type.clone());
            self.item_index.insert(EntityType::get_entry_type_from_raw(&item.r#type), i);
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if entries_used.contains(&mob.r#type) {
                console::log(format!("WARNING - duplicate mob type in raw file [{}]", mob.r#type))
            }
            self.mob_index.insert(EntityType::get_entry_type_from_raw(&mob.r#type), i);
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            if entries_used.contains(&prop.r#type) {
                console::log(format!("WARNING - duplicate mob type in raw file [{}]", prop.r#type))
            }
            self.prop_index.insert(EntityType::get_entry_type_from_raw(&prop.r#type), i);
        }
    }
}

fn spawn_position(pos: SpawnType, new_entity: EntityBuilder) -> EntityBuilder {
    let mut eb = new_entity;
    match pos {
        AtPosition { x, y } => {
            eb = eb.with(Position { x, y })
        }
    }
    eb
}

fn get_renderable_component(renderable: &super::item_structs::Renderable) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

pub fn spawn_named_entity(raws: &RawMaster, key: &EntityType, pos: SpawnType, ecs: &mut World) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, key, pos, ecs);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, key, pos, ecs);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, ecs.create_entity(), key, pos);
    }
    None
}

pub fn spawn_named_item(raws: &RawMaster, key: &EntityType, pos: SpawnType, ecs: &mut World) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];
        let artefact;
        {
            let mut rng = ecs.write_resource::<RandomNumberGenerator>();

            artefact = if let Some(_a) = &item_template.artefact {
                Some(Artefact {
                    value: rng.range(2, 40) * 500,
                    name: generate_artefact_name(rng.deref_mut()),
                })
            } else {
                None
            };
        }

        let mut eb = ecs.create_entity();

        eb = spawn_position(pos, eb);
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }
        eb = eb.with(Name { name: key.get_display_name() });

        eb = eb.with(crate::components::Item {});

        if let Some(consumable) = &item_template.consumable {
            eb = eb.with(Consumable {});
            for effect in consumable.effects.iter() {
                let effect_name = effect.0.as_str();
                match effect_name {
                    "provides_healing" => {
                        eb = eb.with(ProvidesHealing { heal_amount: effect.1.parse::<i32>().unwrap() })
                    }
                    "ranged" => {
                        eb = eb.with(Ranged { range: effect.1.parse::<i32>().unwrap() })
                    }
                    "damage" => {
                        eb = eb.with(InflictsDamage { damage: effect.1.parse::<i32>().unwrap() })
                    }
                    "area_of_effect" => {
                        eb = eb.with(AreaOfEffect { radius: effect.1.parse::<i32>().unwrap() })
                    }
                    "confusion" => {
                        eb = eb.with(Confusion { turns: effect.1.parse::<i32>().unwrap() })
                    }
                    "magic_mapping" => {
                        eb = eb.with(MagicMapper {})
                    }
                    "food" => {
                        eb = eb.with(ProvidesFood { points: effect.1.parse::<i32>().unwrap() })
                    }
                    _ => console::log(format!("Warning: consumable effect {} not implemented!", effect_name))
                }
            }
        }

        if let Some(weapon) = &item_template.weapon {
            eb = eb.with(Equippable { slot: EquipmentSlot::Melee });
            eb = eb.with(MeleeAttackBonus { attack: weapon.power_bonus });
        }
        if let Some(shield) = &item_template.shield {
            eb = eb.with(Equippable { slot: EquipmentSlot::Shield });
            eb = eb.with(MeleeAttackBonus { attack: shield.defense_bonus });
        }

        if let Some(artefact) = artefact {
            eb = eb.with(artefact);
        }
        return Some(eb.build());
    }
    None
}

pub fn spawn_named_mob(raws: &RawMaster, key: &EntityType, pos: SpawnType, ecs: &mut World) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        let name = match key {
            Ogur | Bisat | Spectre | TukkaWarrior => Name {
                name: generate_ogur_name(
                    ecs.write_resource::<RandomNumberGenerator>().deref_mut()
                )
            },
            _ => Name { name: key.get_display_name() }
        };

        let mut eb = ecs.create_entity();

        eb = spawn_position(pos, eb);

        if let Some(renderable) = &mob_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }

        eb = eb.with(name);

        match mob_template.ai.as_ref() {
            "melee" => eb = eb.with(Monster {}),
            "bystander" => eb = eb.with(Bystander {}),
            "vendor" => eb = eb.with(Vendor {}),
            _ => {}
        }
        let might = mob_template.attributes.might.unwrap_or(11);
        let fitness = mob_template.attributes.fitness.unwrap_or(11);
        let quickness = mob_template.attributes.quickness.unwrap_or(11);
        let intelligence = mob_template.attributes.intelligence.unwrap_or(11);
        let attrs = Attributes {
            might: Attribute { base: might, modifiers: 0, bonus: attr_bonus(might) },
            fitness: Attribute { base: fitness, modifiers: 0, bonus: attr_bonus(fitness) },
            quickness: Attribute { base: quickness, modifiers: 0, bonus: attr_bonus(quickness) },
            intelligence: Attribute { base: intelligence, modifiers: 0, bonus: attr_bonus(intelligence) },
        };
        eb = eb.with(attrs);

        let mob_level = mob_template.level.unwrap_or(1);
        let mob_hp = npc_hp(fitness, mob_level);
        let mob_mana = mana_at_level(intelligence, mob_level);

        let pools = Pools {
            hit_points: Pool { current: mob_hp, max: mob_hp},
            mana: Pool { current: mob_mana, max: mob_mana},
            xp: 0,
            level: mob_level,
        };
        eb = eb.with(pools);

        let mut skill_map: HashMap<Skill, i32>  = HashMap::new();
        let template_skill_map = mob_template.skills.clone().unwrap_or(HashMap::new());
        skill_map.insert(
            Skill::Melee,
            *template_skill_map.get("Melee").unwrap_or(&1)
        );
        skill_map.insert(
            Skill::Defense,
            *template_skill_map.get("Defense").unwrap_or(&1)
        );
        skill_map.insert(
            Skill::Magic,
            *template_skill_map.get("Magic").unwrap_or(&1)
        );
        eb = eb.with(Skills{ skills: skill_map });

        if let Some(quips) = &mob_template.quips {
            eb = eb.with(
                Quips { available: quips.clone() }
            );
        }

        if mob_template.blocks_tile {
            eb = eb.with(BlocksTile {});
        }
        eb = eb.with(Viewshed { visible_tiles: Vec::new(), range: mob_template.vision_range, dirty: true });

        return Some(eb.build());
    }
    None
}

pub fn spawn_named_prop(raws: &RawMaster, new_entity: EntityBuilder, key: &EntityType, pos: SpawnType) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];
        let mut eb = new_entity;

        eb = spawn_position(pos, eb);

        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }
        eb = eb.with(Name { name: key.get_display_name() });
        if let Some(hidden) = prop_template.hidden {
            if hidden {
                eb = eb.with(Hidden {});
            }
        }
        if let Some(blocks_tile) = prop_template.blocks_tile {
            if blocks_tile {
                eb = eb.with(BlocksTile {});
            }
        }
        if let Some(blocks_visibility) = prop_template.blocks_visibility {
            if blocks_visibility {
                eb = eb.with(BlocksVisibility {});
            }
        }
        if let Some(door) = prop_template.door_open {
            if door {
                eb = eb.with(Door { open: door });
            }
        }
        if let Some(trigger) = &prop_template.entry_trigger {
            eb = eb.with(EntryTrigger {});
            for effect in trigger.effects.iter() {
                match effect.0.as_str() {
                    "damage" => {
                        eb = eb.with(InflictsDamage { damage: effect.1.parse::<i32>().unwrap() })
                    }
                    "single_activation" => {
                        eb = eb.with(SingleActivation {})
                    }
                    _ => {}
                }
            }
        }
        return Some(eb.build());
    }
    None
}

pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> RandomTable {
    let available_options: Vec<&SpawnTableEntry> = raws.raws.spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = RandomTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt = rt.add(e.r#type, weight);
    }
    rt
}
