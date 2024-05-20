use bracket_lib::random::RandomNumberGenerator;

#[derive(Clone, Copy, Debug)]
pub enum EntryType {
    None,
    Bisat,
    Ogur,
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
    TowerShield,
    MagicMappingScroll
}

pub struct RandomEntry {
    entry: EntryType,
    weight: i32
}

impl RandomEntry {
    pub fn new(entry: EntryType, weight: i32) -> Self {
        Self { entry, weight }
    }
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

    pub fn add(mut self, entry: EntryType, weight: i32) -> RandomTable {
        if weight > 0 {
            self.total_weight += weight;
            self.entries.push(RandomEntry{entry, weight});
        }
        self
    }

    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> EntryType {
        if self.total_weight == 0 { return EntryType::None;}
        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        let mut index: usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].entry
            }
            roll -= self.entries[index].weight;
            index += 1;
        }
        EntryType::None
    }
}
