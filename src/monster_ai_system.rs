extern crate specs;
use super::{Map, Monster, Position, Viewshed};
use specs::prelude::*;
extern crate rltk;
use rltk::{console, Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
    );

    fn run(&mut self, data: Self::SystemData) {

        let (player_pos, viewshed, monster) = data;

        for (viewshed, _monster) in (&viewshed, &monster).join() {
            // console::log("Monster considers their own existence");
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(format!("Monster shouts insults!"));
            }
        }
    }
}
