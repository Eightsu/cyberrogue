use rltk::{RGB, RandomNumberGenerator};
use specs::prelude::*;
use super::{CombatStats, Player, Renderable, Name,
            Position, Viewshed, Monster, BlocksTile};



// Spawn player
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {

    ecs.create_entity()
       .with(Position{x: player_x, y: player_y})
       .with(Renderable{
           glyph: rltk::to_cp437('@'),
           fg: RGB::named(rltk::CYAN),
           bg: RGB::named(rltk::BLACK)
       })
       .with(Player{})
       .with(CombatStats{
           max_hp: 20,
           hp: 20,
           defense: 2,
           power: 6
       })
       .with(Viewshed{
           visible_tiles: Vec::new(),
           range:8,
           dirty: true,
       })
       .with(Name{
           name: "Hero".to_string()
       })
        .build()
}

pub fn random_monster(ecs: &mut World, x: i32, y: i32) {

    let roll:i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1,2);
    }

    match roll {
        1 => {android(ecs,x, y)}
        _ => {robot(ecs, x, y)}
    }
}

fn android(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('A'), "ANDROID");
}

fn robot(ecs: &mut World, x: i32, y:i32) {
    monster(ecs, x, y, rltk::to_cp437('R'), "ROBOT");
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y:i32, glyph: u8, name: S){
    ecs.create_entity()
          .with(Position{x, y})
          .with(Renderable{
              glyph: glyph,
              fg: RGB::named(rltk::RED),
              bg: RGB::named(rltk::BLACK)
          })
          .with(Viewshed{
              visible_tiles: Vec::new(),
              range: 6,
              dirty: true
          })
          .with(Monster{})
          .with(CombatStats{
              max_hp: 10,
              hp: 10,
              defense: 1,
              power: 3
          })
          .with(BlocksTile{})
          .with(Name{
              name: name.to_string()
          })
        .build();
}
