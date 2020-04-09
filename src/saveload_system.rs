use super::components::*;
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{
    DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker, SimpleMarkerAllocator,
};
use std::fs::File;
use std::fs;

// https://doc.rust-lang.org/1.7.0/book/macros.html *Metaprogramming?*
macro_rules! serialize_individually {
  ($ecs:expr, $ser:expr, $data:expr, $( $type:ty ), *) => {

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

pub fn save_game(ecs: &mut World) {
    let map_copy = ecs.get_mut::<super::map::Map>().unwrap().clone();

    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: map_copy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    { // avoid borrow-checker issues.
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
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumeable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Disable,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper
        );
    }

    ecs.delete_entity(save_helper);
}
