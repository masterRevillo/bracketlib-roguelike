use bracket_lib::color::{BLACK, ORANGE, RGB};
use bracket_lib::prelude::to_cp437;
use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use crate::components::{Attributes, HungerClock, HungerState, Name, Pools, Position, Skill, Skills, SufferDamage, WantsToMelee};
use crate::gamelog::GameLog;
use crate::gamesystem::skill_bonus;
use crate::particle_system::ParticleBuilder;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, Pools>,
        WriteExpect<'a, RandomNumberGenerator>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut gamelog,
            mut wants_melee,
            names,
            attributes,
            skills,
            mut inflict_damage,
            mut particle_builder,
            positions,
            hunger_clock,
            pools,
            mut rng
        ) = data;

        for (entity, wants_melee, name, attacker_attrs, attacker_skills, attacker_pools) in (&entities, &wants_melee, &names, &attributes, &skills, &pools).join() {
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attrs = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();
            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0 {
                let target_name = names.get(wants_melee.target).unwrap();

                let natural_roll = rng.roll_dice(1, 20);
                let attribute_hit_bonus = attacker_attrs.might.bonus;
                let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_hit_bonus = 0; //
                let mut status_hit_bonus = 0;
                if let Some(hc) = hunger_clock.get(entity) {
                    if hc.state == HungerState::WellFed {
                        status_hit_bonus += 1;
                    }
                }
                let modified_hit_roll = natural_roll +
                    attribute_hit_bonus +
                    skill_hit_bonus +
                    weapon_hit_bonus +
                    status_hit_bonus;

                let base_armor_class = 10;
                let armor_quickness_bonus =  target_attrs.quickness.bonus;
                let armor_skill_bonus = skill_bonus(Skill::Defense, &*target_skills);
                let armor_item_bonus = 0; //
                let armor_class = base_armor_class +
                    armor_quickness_bonus +
                    armor_skill_bonus +
                    armor_item_bonus;

                // Target is hit if not crit fail AND either natural 20 or modified role is greater
                // than armor class
                if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
                    let base_damage = rng.roll_dice(1, 4);
                    let attr_damage_bonus = attacker_attrs.might.bonus;
                    let skill_damage_bonus = skill_hit_bonus; // save computation from above
                    let weapon_damage_bonus = 0;

                    let damage = i32::max(
                        0,
                        base_damage +
                            attr_damage_bonus +
                            skill_hit_bonus +
                            skill_damage_bonus +
                            weapon_damage_bonus
                    );
                    SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    gamelog.entries.push(
                        format!("{} hits {} for {} hp.", &name.name, &target_name.name, damage)
                    );
                } else if natural_roll == 1 {
                    gamelog.entries.push(
                        format!("{} tries to hit {}, but misses.", &name.name, &target_name.name)
                    );
                } else {
                    gamelog.entries.push(
                        format!("{} hits {}, but it does no damage.", &name.name, &target_name.name)
                    );
                }
                if let Some(pos) = positions.get(wants_melee.target) {
                    particle_builder.request(pos.x, pos.y, RGB::named(ORANGE), RGB::named(BLACK), to_cp437('!'), 200.0);
                }
            }
        }
        wants_melee.clear();
    }
}
