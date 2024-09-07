use std::collections::HashMap;

use serde::Deserialize;
use crate::components::EquipmentSlot;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub renderable: Option<Renderable>,
    pub consumable: Option<Consumable>,
    pub weapon: Option<Weapon>,
    pub wearable: Option<Wearable>,
    pub artefact: Option<Artefact>
}

#[derive(Deserialize, Debug)]
pub struct Renderable {
    pub glyph: String,
    pub fg: String,
    pub bg: String,
    pub order: i32
}

#[derive(Deserialize, Debug)]
pub struct Consumable {
    pub effects: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
pub struct Weapon {
    pub range: String,
    pub attribute: String,
    pub base_damage: String,
    pub hit_bonus: i32
}

#[derive(Deserialize, Debug)]
pub struct Wearable {
    pub slot: EquipmentSlot,
    pub armor_class: f32
}


#[derive(Deserialize, Debug)]
pub struct Artefact {
    pub effects: HashMap<String, String>
}
