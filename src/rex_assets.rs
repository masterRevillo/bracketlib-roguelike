use bracket_embedding::prelude::*;
use bracket_lib::prelude::{embedded_resource, XpFile};

embedded_resource!(SMALL_DUNGEON, "../resources/desert2.xp");
embedded_resource!(WFC_DEMO_IMG1, "../resources/wfc-demo1.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        link_resource!(SMALL_DUNGEON, "../resources/desert2.xp");
        link_resource!(WFC_DEMO_IMG1, "../resources/wfc-demo1.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/desert2.xp").unwrap(),
        }
    }
}
