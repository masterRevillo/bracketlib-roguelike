use bracket_lib::color::RGB;
use bracket_lib::prelude::{FontCharType, Point};
use serde::{Deserialize, Serialize};
use specs::{Join, WriteStorage};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::ConvertSaveload;
use specs::saveload::Marker;
use specs_derive::*;

use crate::map::Map;

pub struct SerializeMe;

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool
}
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}


#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}


#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: Vec<i32>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToPickUpItem {
    pub collected_by: Entity,
    pub item: Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToDropItem {
    pub item: Entity
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Examinable {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Artefact {
    pub name: String,
    pub value: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SerializationHelper {
    pub map: Map
}

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum  EquipmentSlot { Melee, Shield }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MeleeAttackBonus {
    pub attack: i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct DefenseBonus {
    pub defense: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToUnequipItem {
    pub item: Entity
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
   pub lifetime_ms: f32
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum HungerState { WellFed, Normal, Hungry, Starving }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    pub hunger_points: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesFood {
    pub points: i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}


#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}


#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntityMoved {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Door {
    pub open: bool
}
