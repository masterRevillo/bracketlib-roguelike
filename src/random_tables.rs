use std::str::FromStr;
use bracket_lib::prelude::console;
use bracket_lib::random::RandomNumberGenerator;
use serde::Deserialize;
use strum_macros::EnumString;

#[derive(Clone, Copy, Debug, EnumString, Eq, PartialEq, Hash, Deserialize)]
pub enum EntityType {
    None,
    Rat,
    Bisat,
    Ogur,
    Spectre,
    TukkaWarrior,
    HealthPotion,
    MagicMissileScroll,
    FireballScroll,
    ConfusionScroll,
    ChickenLeg,
    Sandwich,
    GobletOfWine,
    Artefact,
    Dagger,
    Shield,
    Longsword,
    Battleaxe,
    TowerShield,
    MagicMappingScroll,
    BearTrap,
    Door
}

impl EntityType {

    pub(crate) fn get_entry_type_from_raw(t: &String) -> EntityType {
        EntityType::from_str(t.as_str()).unwrap_or_else(|_| {
            console::log(format!("Unrecognized EntityType [{t}]"));
            EntityType::None
        })
    }
    pub fn get_display_name(&self) -> String {
        String::from(match self {
            EntityType::None => "None",
            EntityType::Rat => "Rat",
            EntityType::Bisat => "Bisat",
            EntityType::Ogur => "Ogur",
            EntityType::Spectre => "Spectre",
            EntityType::TukkaWarrior => "Tukka Warrior",
            EntityType::HealthPotion => "Health Potion",
            EntityType::MagicMissileScroll => "Magic Missile Scroll",
            EntityType::FireballScroll => "Fireball Scroll",
            EntityType::ConfusionScroll => "Confusion Scroll",
            EntityType::ChickenLeg => "Chicken Leg",
            EntityType::Sandwich => "Sandwich",
            EntityType::GobletOfWine => "Goblet of Wine",
            EntityType::Artefact => "Artefact",
            EntityType::Dagger => "Dagger",
            EntityType::Shield => "Shield",
            EntityType::Longsword => "Longsword",
            EntityType::Battleaxe => "Battleaxe",
            EntityType::TowerShield => "Tower Shield",
            EntityType::MagicMappingScroll => "Magic Mapping Scroll",
            EntityType::BearTrap => "Bear Trap",
            EntityType::Door => "Door"
        })
    }
}

pub struct RandomEntry {
    entry: EntityType,
    weight: i32
}

#[derive(Default)]
pub struct RandomTable {
    entries: Vec<RandomEntry>,
    total_weight: i32
}

impl RandomTable {
    pub fn new() -> Self {
        Self { entries: Vec::new(), total_weight: 0 }
    }

    pub fn add(mut self, entry: EntityType, weight: i32) -> RandomTable {
        if weight > 0 {
            self.total_weight += weight;
            self.entries.push(RandomEntry{entry, weight});
        }
        self
    }

    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> EntityType {
        if self.total_weight == 0 { return EntityType::None;}
        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        let mut index: usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].entry
            }
            roll -= self.entries[index].weight;
            index += 1;
        }
        EntityType::None
    }
}
