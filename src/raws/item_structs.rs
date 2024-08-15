use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub r#type: String,
    pub renderable: Option<Renderable>,
    pub consumable: Option<Consumable>,
    pub weapon: Option<Weapon>,
    pub shield: Option<Shield>,
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
    pub power_bonus: i32
}

#[derive(Deserialize, Debug)]
pub struct Shield {
    pub defense_bonus: i32
}


#[derive(Deserialize, Debug)]
pub struct Artefact {
    pub effects: HashMap<String, String>
}
