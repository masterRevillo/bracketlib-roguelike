use std::fs;
use std::fs::File;
use std::path::Path;

use bracket_lib::prelude::Point;
use specs::error::NoError;
use specs::saveload::{
    DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};
use specs::{Builder, Entity, Join, World, WorldExt};

use crate::components::{
    AreaOfEffect, Artefact, Attributes, BlocksTile, BlocksVisibility, Bystander, Carnivore,
    Confusion, Consumable, DMSerializationHelper, Door, EntityMoved, EntryTrigger, Equippable,
    Equipped, Examinable, Fears, Herbivore, Hidden, HungerClock, InBackpack, InflictsDamage, Item,
    LootTable, MagicMapper, MeleeWeapon, Monster, Name, NaturalAttackDefense, OtherLevelPosition,
    ParticleLifetime, Player, Pools, Position, ProvidesFood, ProvidesHealing, Quips, Ranged,
    Renderable, SerializationHelper, SerializeMe, SingleActivation, Skills, Species, SufferDamage,
    Vendor, Viewshed, WantsToDropItem, WantsToMelee, WantsToPickUpItem, WantsToUnequipItem,
    WantsToUseItem, Wearable,
};
use crate::map::dungeon::MasterDungeonMap;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs: &mut World) {
    // create helper
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let dungeon_master = ecs.get_mut::<MasterDungeonMap>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let savehelper2 = ecs
        .create_entity()
        .with(DMSerializationHelper {
            map: dungeon_master,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    // do the serializing
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            Species,
            BlocksTile,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickUpItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Examinable,
            Artefact,
            Equippable,
            Equipped,
            MeleeWeapon,
            Wearable,
            WantsToUnequipItem,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            BlocksVisibility,
            Door,
            Bystander,
            Vendor,
            Quips,
            Attributes,
            Skills,
            Pools,
            NaturalAttackDefense,
            LootTable,
            Carnivore,
            Herbivore,
            Fears,
            OtherLevelPosition,
            DMSerializationHelper
        );
    }
    ecs.delete_entity(savehelper)
        .expect("Couldn't clean up helper");
    ecs.delete_entity(savehelper2)
        .expect("Couldn't clean up helper");
}

pub fn does_save_exist() -> bool {
    Path::new("./savegame.json").exists()
}

pub fn load_game(ecs: &mut World) {
    {
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }
    let data = fs::read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );

        deserialize_individually!(
            ecs,
            de,
            d,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            Species,
            BlocksTile,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickUpItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            DMSerializationHelper,
            Examinable,
            Artefact,
            Equippable,
            Equipped,
            MeleeWeapon,
            Wearable,
            WantsToUnequipItem,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            BlocksVisibility,
            Door,
            Bystander,
            Vendor,
            Quips,
            Attributes,
            Skills,
            Pools,
            NaturalAttackDefense,
            LootTable,
            Carnivore,
            Herbivore,
            Fears,
            OtherLevelPosition
        );
    }
    let mut deleteme: Option<Entity> = None;
    let mut deleteme2: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let helper2 = ecs.read_storage::<DMSerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<super::map::Map>();
            *worldmap = h.map.clone();
            worldmap.tile_content =
                vec![vec![Vec::new(); worldmap.height as usize]; worldmap.width as usize];
            deleteme = Some(e);
        }
        for (e, h) in (&entities, &helper2).join() {
            let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
            *dungeon_master = h.map.clone();
            deleteme2 = Some(e);
        }
        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<Point>();
            *ppos = Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap())
        .expect("Unable to delete helper");
    ecs.delete_entity(deleteme2.unwrap())
        .expect("Unable to delete helper");
}

pub fn delete_save() {
    if Path::new("./savegame.json").exists() {
        fs::remove_file("./savegame.json").expect("Unable to delete file");
    }
}
