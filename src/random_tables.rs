use bracket_lib::random::RandomNumberGenerator;

// #[derive(Clone, Copy, Debug, EnumString, Eq, PartialEq, Hash, Deserialize)]
// pub enum EntityType {
//     None,
//     Rat,
//     Bisat,
//     Ogur,
//     Spectre,
//     TukkaWarrior,
//     HealthPotion,
//     MagicMissileScroll,
//     FireballScroll,
//     ConfusionScroll,
//     ChickenLeg,
//     Sandwich,
//     GobletOfWine,
//     Artefact,
//     Dagger,
//     Shield,
//     Longsword,
//     Battleaxe,
//     TowerShield,
//     MagicMappingScroll,
//     BearTrap,
//     Door,
//     Bartender,
//     ShadySalesman,
//     Patron,
//     Table,
//     Chair,
//     Keg,
//     Priest,
//     Parishioner,
//     Candle,
//     Blacksmith,
//     Anvil,
//     WaterTrough,
//     WeaponRack,
//     ArmorStand,
//     Clothier,
//     Cabinet,
//     Loom,
//     HideRack,
//     Alchemist,
//     ChemistrySet,
//     DeadThing,
//     Mom,
//     Bed,
//     Peasant,
//     DockWorker,
//     Pirate,
//     Fisher,
//     Drunk,
//     RustyLongsword,
//     DriedSausage,
//     Beer,
//     StainedTunic,
//     TornTrousers,
//     OldBoots
// }
//
// impl EntityType {
//
//     pub(crate) fn get_entry_type_from_raw(t: &String) -> EntityType {
//         EntityType::from_str(t.as_str()).unwrap_or_else(|_| {
//             console::log(format!("Unrecognized EntityType [{t}]"));
//             EntityType::None
//         })
//     }
//     pub fn get_display_name(&self) -> String {
//         String::from(match self {
//             EntityType::None => "None",
//             EntityType::Rat => "Rat",
//             EntityType::Bisat => "Bisat",
//             EntityType::Ogur => "Ogur",
//             EntityType::Spectre => "Spectre",
//             EntityType::TukkaWarrior => "Tukka Warrior",
//             EntityType::HealthPotion => "Health Potion",
//             EntityType::MagicMissileScroll => "Magic Missile Scroll",
//             EntityType::FireballScroll => "Fireball Scroll",
//             EntityType::ConfusionScroll => "Confusion Scroll",
//             EntityType::ChickenLeg => "Chicken Leg",
//             EntityType::Sandwich => "Sandwich",
//             EntityType::GobletOfWine => "Goblet of Wine",
//             EntityType::Artefact => "Artefact",
//             EntityType::Dagger => "Dagger",
//             EntityType::Shield => "Shield",
//             EntityType::Longsword => "Longsword",
//             EntityType::Battleaxe => "Battleaxe",
//             EntityType::TowerShield => "Tower Shield",
//             EntityType::MagicMappingScroll => "Magic Mapping Scroll",
//             EntityType::BearTrap => "Bear Trap",
//             EntityType::Door => "Door",
//             EntityType::Bartender => "Bartender",
//             EntityType::ShadySalesman => "Shady Salesman",
//             EntityType::Patron => "Patron",
//             EntityType::Table => "Table",
//             EntityType::Chair => "Chair",
//             EntityType::Keg => "Keg",
//             EntityType::Priest => "Priest",
//             EntityType::Parishioner => "Parishioner",
//             EntityType::Candle => "Candle",
//             EntityType::Blacksmith => "Blacksmith",
//             EntityType::Anvil => "Anvil",
//             EntityType::WaterTrough =>"Water Trough",
//             EntityType::WeaponRack => "Weapon Rack",
//             EntityType::ArmorStand => "Armor Stand",
//             EntityType::Clothier => "Clothier",
//             EntityType::Cabinet => "Cabinet",
//             EntityType::Loom => "Loom",
//             EntityType::HideRack => "Hide Rack",
//             EntityType::Alchemist => "Alchemist",
//             EntityType::ChemistrySet => "Chemistry Set",
//             EntityType::DeadThing => "Dead Thing",
//             EntityType::Mom => "Mom",
//             EntityType::Bed => "Bed",
//             EntityType::Peasant => "Peasant",
//             EntityType::DockWorker => "Dock Worker",
//             EntityType::Pirate => "Pirate",
//             EntityType::Fisher => "Fisher",
//             EntityType::Drunk => "Drunk",
//             EntityType::RustyLongsword => "Rusty Longsword",
//             EntityType::DriedSausage => "Dried Sausage",
//             EntityType::Beer => "Beer",
//             EntityType::StainedTunic => "Stained Tunic",
//             EntityType::TornTrousers => "Torn Trousers",
//             EntityType::OldBoots => "Old Boots"
//         })
//     }
// }

pub struct RandomEntry {
    name: String,
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

    pub fn add(mut self, entry: String, weight: i32) -> RandomTable {
        if weight > 0 {
            self.total_weight += weight;
            self.entries.push(RandomEntry{ name: entry, weight});
        }
        self
    }

    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> String {
        if self.total_weight == 0 { return "None".to_string();}
        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        let mut index: usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].name.clone()
            }
            roll -= self.entries[index].weight;
            index += 1;
        }
        "None".to_string()
    }
}
