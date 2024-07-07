use bracket_lib::color::{BLACK, ORANGE, RGB};
use bracket_lib::prelude::to_cp437;
use specs::prelude::*;

use crate::components::{EntityMoved, EntryTrigger, Hidden, InflictsDamage, Name, Position, SingleActivation, SufferDamage};
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::particle_system::ParticleBuilder;

pub struct TriggerSystem{}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, SingleActivation>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entities_moved,
            positions,
            mut entry_triggers,
            mut hidden_things,
            names,
            entities,
            mut game_log,
            mut inflicts_damage,
            mut suffer_damage,
            mut particle_builder,
            single_adctivation
        ) = data;

        for (entity, _em, pos) in (&entities, &mut entities_moved, &positions).join() {
            for entity_id in map.tile_content[pos.x as usize][pos.y as usize].iter() {
                if entity != *entity_id {
                    match entry_triggers.get(*entity_id) {
                        None => {},
                        Some(_t) => {
                            let damage = inflicts_damage.get(*entity_id);
                            if let Some(damage) = damage {
                                particle_builder.request(pos.x, pos.y, RGB::named(ORANGE), RGB::named(BLACK), to_cp437('!'), 200.0);
                                SufferDamage::new_damage(&mut suffer_damage, entity, damage.damage)
                            }
                            let sa = single_adctivation.get(*entity_id);
                            if let Some(_sa) = sa {
                                inflicts_damage.remove(*entity_id);
                                entry_triggers.remove(*entity_id);
                            }
                            let name = names.get(*entity_id);
                            if let Some(name) = name {
                                game_log.entries.push(format!("{} triggers!", &name.name));
                            }
                            hidden_things.remove(*entity_id);
                        }
                    }
                }
            }
        }
        entities_moved.clear();
    }
}
