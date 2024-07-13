use std::cmp::min;

use specs::prelude::*;

use crate::components::{HungerClock, HungerState, SufferDamage};
use crate::gamelog::GameLog;
use crate::RunState;

pub struct HungerSystem {}

impl HungerSystem {

    pub fn calculate_new_hunger_state(current_points: i32, current_state: HungerState, food_points: i32) -> (HungerState, i32) {
        let combined_points = current_points + food_points;
        return match current_state {
            HungerState::WellFed => (HungerState::WellFed,  min(combined_points, 20)),
            HungerState::Normal => {
                if combined_points > 200 {
                    (HungerState::WellFed, combined_points - 200)
                } else {
                    (HungerState::Normal, combined_points)
                }
            }
            HungerState::Hungry => {
                if combined_points > 200 {
                    (HungerState::Normal, combined_points - 200)
                } else {
                    (HungerState::Hungry, combined_points)
                }
            }
            HungerState::Starving=> {
                if combined_points > 200 {
                    (HungerState::Hungry, combined_points - 200)
                } else {
                    (HungerState::Starving, combined_points)
                }
            }
        }
    }
}

impl <'a> System<'a> for HungerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut hunger_clock,
            player_entity,
            runstate,
            mut inflict_damage,
            mut gamelog
        ) = data;

        for (entity, clock) in (&entities, &mut hunger_clock).join() {
            let proceed = match *runstate {
                RunState::PlayerTurn if entity == *player_entity => true,
                RunState::MonsterTurn if entity != *player_entity => true,
                _ => false
            };

            if proceed {
                clock.hunger_points -= 1;
                if clock.hunger_points < 1 {
                    match clock.state {
                        HungerState::WellFed => {
                            clock.state = HungerState::Normal;
                            clock.hunger_points = 200;
                            if entity == *player_entity {
                                gamelog.entries.push("You are no longer well fed".to_string());
                            }
                        }
                        HungerState::Normal => {
                            clock.state = HungerState::Hungry;
                            clock.hunger_points = 200;
                            if entity == *player_entity {
                                gamelog.entries.push("You are getting hungry".to_string());
                            }
                        }
                        HungerState::Hungry => {
                            clock.state = HungerState::Starving;
                            clock.hunger_points = 200;
                            if entity == *player_entity {
                                gamelog.entries.push("You are starving".to_string());
                            }
                        }
                        HungerState::Starving => {
                            clock.hunger_points = 200;
                            if entity == *player_entity {
                                gamelog.entries.push("You are dying of starvation. You take 1 hp of damage".to_string());
                            }
                            SufferDamage::new_damage(&mut inflict_damage, entity, 1);
                        }
                    }
                }
            }
        }
    }

}
