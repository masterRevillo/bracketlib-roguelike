use bracket_lib::prelude::{embedded_resource, XpFile};
use bracket_embedding::prelude::*;

embedded_resource!(SMALL_DUNGEON, "../resources/desert2.xp");

pub struct RexAssets {
    pub menu: XpFile
}

impl RexAssets {
    pub fn new() -> RexAssets {
        link_resource!(SMALL_DUNGEON, "../resources/desert2.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/desert2.xp").unwrap()
        }
    }
}
