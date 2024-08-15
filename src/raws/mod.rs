use std::sync::Mutex;

use bracket_embedding::{embedded_resource, link_resource};
use bracket_embedding::prelude::EMBED;
use serde::Deserialize;

use rawmaster::*;

use crate::raws::item_structs::{Item};
use crate::raws::mob_structs::Mob;
use crate::raws::prop_structs::Prop;

mod item_structs;
pub mod rawmaster;
mod mob_structs;
mod prop_structs;

embedded_resource!(RAW_FILE, "../../raws/spawns.json");

lazy_static!{
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

pub fn load_raws() {
    link_resource!(RAW_FILE, "../../raws/spawns.json");

    let raw_data = EMBED
        .lock()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();
    let raw_string = std::str::from_utf8(&raw_data).expect("Cannot convert to utf8 string");
    let decoder: Raws = serde_json::from_str(&raw_string).expect("Failed to parse JSON");
    RAWS.lock().unwrap().load(decoder);
}

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>
}
