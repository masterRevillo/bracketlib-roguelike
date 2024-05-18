use std::fs;
use std::fs::File;
use std::path::Path;
use bracket_lib::prelude::Point;
use specs::saveload::{SerializeComponents, SimpleMarkerAllocator, MarkedBuilder, SimpleMarker, DeserializeComponents};
use specs::error::NoError;
use specs::{Builder, Entity, Join, World, WorldExt};
use crate::components::{AreaOfEffect, Artefact, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus, Equippable, Equipped, Examinable, InBackpack, InflictsDamage, Item, MeleeAttackBonus, Monster, Name, ParticleLifetime, Player, Position, ProvidesHealing, Ranged, Renderable, SerializationHelper, SerializeMe, SufferDamage, Viewshed, WantsToDropItem, WantsToMelee, WantsToPickUpItem, WantsToUnequipItem, WantsToUseItem};


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
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    // do the serializing
    {
        let data = ( ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>() );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually! (
            ecs, serializer, data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
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
            MeleeAttackBonus,
            DefenseBonus,
            WantsToUnequipItem,
            ParticleLifetime
        );
    }
    ecs.delete_entity(savehelper).expect("Couldn't clean up helper")
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
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>()
        );

        deserialize_individually!(
            ecs, de, d,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
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
            MeleeAttackBonus,
            DefenseBonus,
            WantsToUnequipItem,
            ParticleLifetime
        );
    }
    let mut deleteme: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e,h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<super::map::Map>();
            *worldmap = h.map.clone();
            worldmap.tile_content = vec![vec![Vec::new(); super::map::MAPHEIGHT]; super::map::MAPWIDTH];
            deleteme = Some(e);
        }
        for (e,_p,pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<Point>();
            *ppos = Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap()).expect("Unable to delete helper");
}

pub fn delete_save() {
    if Path::new("./savegame.json").exists() {
        fs::remove_file("./savegame.json").expect("Unable to delete file");
    }
}
