use super::{
    rand_table::RandomTable, AreaOfEffect, BlocksTile, CombatStats, Consumeable, Disable,
    InflictsDamage, Item, Monster, Name, Player, Position, ProvidesHealing, Ranged, Rect,
    Renderable, SerializeMe, Viewshed, MAPWIDTH,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

// .marked::<SimpleMarker<SerializeMe>>() ADD TO ANYTHING YOU WANT SERIALIZED!

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    ecs.create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Hero".to_string(),
        })
        .with(CombatStats {
            max_hp: 100,
            hp: 100,
            defense: 2,
            power: 10,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

const MAX_MONSTERS: i32 = 6;
// const MAX_ITEMS: i32 = 2;

#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32 ) {
    let mut spawn_table = room_table();
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0; //
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        let x = (spawn.0 % MAPWIDTH) as i32;
        let y = (spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            "Android" => android(ecs, x, y),
            "Robot" => robot(ecs, x, y),
            "volt_pack" => volt_pack(ecs, x, y),
            "buster" => buster(ecs, x, y),
            "shockwave" => shockwave(ecs, x, y),
            "overload" => overload(ecs, x, y),
            _ => {}

        }
    }
}

fn android(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('A'), "Android");
}
fn robot(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('R'), "Robot");
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// HEAL
fn volt_pack(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('±'),
            fg: RGB::named(rltk::GHOSTWHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Volt Pack(HP)".to_string(),
        })
        .with(Item {})
        .with(Consumeable {})
        .with(ProvidesHealing { heal_amount: 10 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// RANGED ATTACK
fn buster(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('Θ'),
            fg: RGB::named(rltk::GREENYELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Buster Chip".to_string(),
        })
        .with(Item {})
        .with(Consumeable {})
        .with(Ranged { range: 8 })
        .with(InflictsDamage { damage: 12 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// RANGED ATTACK WITH AOE
fn shockwave(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('≡'),
            fg: RGB::named(rltk::YELLOW2),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Shockwave Chip".to_string(),
        })
        .with(Item {})
        .with(Consumeable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 5 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

// TEMPORARILY INCAPACITATE ENEMY
fn overload(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¿'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Overload Chip".to_string(),
        })
        .with(Item {})
        .with(Consumeable {})
        .with(Ranged { range: 3 })
        .with(Disable { turns: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn room_table() -> RandomTable {
    RandomTable::new()
        .add("Android", 6)
        .add("Robot", 9)
        .add("volt_pack", 4)
        .add("buster", 7)
        .add("shockwave", 2)
        .add("overload", 5)
}
