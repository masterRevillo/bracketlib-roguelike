use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;

use bracket_lib::color::RGB;
use bracket_lib::prelude::{console, to_cp437};
use bracket_lib::random::RandomNumberGenerator;
use regex::Regex;
use specs::{Builder, Entity, EntityBuilder, World, WorldExt};
use specs::saveload::{MarkedBuilder, SimpleMarker};

use crate::components::{AreaOfEffect, Artefact, Attribute, Attributes, BlocksTile, BlocksVisibility, Bystander, Confusion, Consumable, Door, EntryTrigger, EquipmentSlot, Equippable, Equipped, Hidden, InBackpack, InflictsDamage, MagicMapper, MeleeWeapon, Monster, Name, NaturalAttack, NaturalAttackDefense, Pool, Pools, Position, ProvidesFood, ProvidesHealing, Quips, Ranged, SerializeMe, SingleActivation, Skill, Skills, Vendor, Viewshed, WeaponAttribute, Wearable};
use crate::gamesystem::{attr_bonus, mana_at_level, npc_hp};
use crate::random_tables::RandomTable;
use crate::raws::rawmaster::SpawnType::AtPosition;
use crate::raws::Raws;
use crate::raws::spawn_table_structs::SpawnTableEntry;
use crate::util::namegen::{generate_artefact_name, generate_ogur_name};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity }
}

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
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
            if entries_used.contains(&item.id) {
                console::log(format!("WARNING - duplicate item type in raw file [{}]", item.id))
            }
            entries_used.insert(item.id.clone());
            self.item_index.insert(item.id.to_string(), i);
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if entries_used.contains(&mob.id) {
                console::log(format!("WARNING - duplicate mob type in raw file [{}]", mob.id))
            }
            self.mob_index.insert(mob.id.to_string(), i);
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            if entries_used.contains(&prop.id) {
                console::log(format!("WARNING - duplicate mob type in raw file [{}]", prop.id))
            }
            self.prop_index.insert(prop.id.to_string(), i);
        }
    }
}

fn spawn_position<'a>(pos: SpawnType, new_entity: EntityBuilder<'a>, tag: String, raws: &RawMaster) -> EntityBuilder<'a> {
    let mut eb = new_entity;
    match pos {
        AtPosition { x, y } => eb.with(Position { x, y }),
        SpawnType::Equipped { by } => {
            let slot = find_slot_for_equippable_item(&tag, raws);
            eb.with(Equipped{ owner: by, slot})
        }
        SpawnType::Carried { by } => eb.with(InBackpack{ owner: by })
    }
}

fn get_renderable_component(renderable: &super::item_structs::Renderable) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

pub fn spawn_named_entity(raws: &RawMaster, key: &String, pos: SpawnType, ecs: &mut World) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, key, pos, ecs);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, key, pos, ecs);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, key, pos, ecs);
    }
    None
}

pub fn spawn_named_item(raws: &RawMaster, key: &String, pos: SpawnType, ecs: &mut World) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        let item_template = &raws.raws.items[raws.item_index[key]];
        let artefact;
        {
            artefact = if let Some(_a) = &item_template.artefact {
                let mut rng = ecs.write_resource::<RandomNumberGenerator>();
                Some(Artefact {
                    value: rng.range(2, 40) * 500,
                    name: generate_artefact_name(rng.deref_mut()),
                })
            } else {
                None
            };
       }

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        eb = spawn_position(pos, eb, key.clone(), raws);
        if let Some(renderable) = &item_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }
        eb = eb.with(Name { name: item_template.name.clone() });

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
            let (n_dice, die_type, bonus) = parse_dice_string(&weapon.base_damage);
            let wpn = MeleeWeapon{
                attribute: match weapon.attribute.as_str() {
                    "Quickness" => WeaponAttribute::Quickness,
                    _ => WeaponAttribute::Might
                },
                damage_n_dice: n_dice,
                damage_die_type: die_type,
                damage_bonus: bonus,
                hit_bonus: weapon.hit_bonus
            };
            eb = eb.with(wpn);

        }
        if let Some(wearable) = &item_template.wearable {
            eb = eb.with(Equippable { slot: wearable.slot });
            eb = eb.with(Wearable { armor_class: wearable.armor_class})
            // eb = eb.with(MeleeWeapon { attack: shield.defense_bonus });
        }

        if let Some(artefact) = artefact {
            eb = eb.with(artefact);
        }
        return Some(eb.build());
    }
    None
}

pub fn spawn_named_mob(raws: &RawMaster, key: &String, pos: SpawnType, ecs: &mut World) -> Option<Entity> {
    if raws.mob_index.contains_key(key) {
        let mob_template = &raws.raws.mobs[raws.mob_index[key]];

        let name = match key.as_str() {
            "Ogur" | "Bisat" | "Spectre" | "TukkaWarrior" => Name {
                name: generate_ogur_name(
                    ecs.write_resource::<RandomNumberGenerator>().deref_mut()
                )
            },
            _ => Name { name: mob_template.name.clone() }
        };

        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        eb = spawn_position(pos, eb, key.clone(), raws);

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

        if let Some(na) = &mob_template.natural {
            let mut nature = NaturalAttackDefense {
                armor_class: na.armor_class,
                attacks: Vec::new()
            };
            if let Some(attacks) = &na.attacks {
                for a in attacks.iter() {
                    let (n, d, b) = parse_dice_string(&a.damage);
                    nature.attacks.push(
                        NaturalAttack {
                            name: a.name.clone(),
                            damage_n_dice: n,
                            damage_die_type: d,
                            damage_bonus: b,
                            hit_bonus: a.hit_bonus,
                        }
                    )
                }
            }
            eb = eb.with(nature);
        }

        let new_mob = eb.build();

        if let Some(equipped) = &mob_template.equipped {
            for id in equipped.iter() {
                spawn_named_entity(raws, id, SpawnType::Equipped {by: new_mob}, ecs);
            }
        }


        return Some(new_mob);
    }
    None
}

pub fn spawn_named_prop(raws: &RawMaster, key: &String, pos: SpawnType, ecs: &mut World) -> Option<Entity> {
    if raws.prop_index.contains_key(key) {
        let prop_template = &raws.raws.props[raws.prop_index[key]];
        let mut eb = ecs.create_entity().marked::<SimpleMarker<SerializeMe>>();

        eb = spawn_position(pos, eb, key.clone(), raws);

        if let Some(renderable) = &prop_template.renderable {
            eb = eb.with(get_renderable_component(renderable));
        }
        eb = eb.with(Name { name: prop_template.name.clone() });
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
        rt = rt.add(e.id.clone(), weight);
    }
    rt
}

pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("not a digit")
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("not a digit")
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("not a digit")
        }
    }
    (n_dice, die_type, die_bonus)


}

fn find_slot_for_equippable_item(tag: &String, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Tried to equip item that doesn't exist: {:?}", tag)
    }
    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if let Some(_wpn) = &item.weapon {
        return EquipmentSlot::Melee;
    } else if let Some(wearable) = &item.wearable {
        return wearable.slot;
    } else {
        panic!("Trying to equip {:?}, but it has not slot tag", tag);
    }
}
