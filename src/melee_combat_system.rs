use bracket_lib::color::{BLACK, ORANGE, RGB};
use bracket_lib::prelude::to_cp437;
use specs::prelude::*;

use crate::components::{CombatStats, DefenseBonus, Equipped, HungerClock, HungerState, MeleeAttackBonus, Name, Position, SufferDamage, WantsToMelee};
use crate::gamelog::GameLog;
use crate::particle_system::ParticleBuilder;

pub struct MeleeCombatSystem {}

impl <'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, MeleeAttackBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut gamelog,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            melee_attack_bonus,
            defense_bonus,
            equipped,
            mut particle_builder,
            positions,
            hunger_clock
        ) = data;

        for (entity, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, attack_bonus, equipped_item) in (&entities, &melee_attack_bonus, &equipped).join() {
                    if equipped_item.owner == entity {
                        offensive_bonus += attack_bonus.attack
                    }
                }
                let hc = hunger_clock.get(entity);
                if let Some(hc) = hc {
                    if hc.state == HungerState::WellFed {
                        offensive_bonus += 1;
                    }
                }
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_item) in (&entities, &defense_bonus, &equipped).join() {
                        if equipped_item.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }
                    let pos = positions.get(wants_melee.target);
                    if let Some(pos) = pos {
                        particle_builder.request(
                            pos.x, pos.y, RGB::named(ORANGE), RGB::named(BLACK), to_cp437('!'), 150.0
                        );
                    }
                    let damage = i32::max(0, (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus));

                    if damage == 0 {
                        gamelog.entries.push(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        gamelog.entries.push(format!("{} hits {} for {} hp", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }
        wants_melee.clear();
    }
}
