use rltk::{Console, GameState, Point, Rltk};
use specs::prelude::*;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod inventory_system;
use inventory_system::{InventorySystem, ItemDropSystem, UseConsumableSystem};
mod components;

pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod gui;
mod rect;
mod spawner;
pub use rect::Rect;
mod gamelog;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        let mut map_index = MapIndexingSystem {};
        map_index.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);

        let mut pickup = InventorySystem {};
        pickup.run_now(&self.ecs);

        let mut volt_packs = UseConsumableSystem {};
        volt_packs.run_now(&self.ecs);

        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);

        self.ecs.maintain(); // MUST BE AT BOTTOM
    }
}

// RENDER LOOP
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut new_run_state;

        {
            let runstate = self.ecs.fetch::<RunState>();
            new_run_state = *runstate;
            // note - access data within mutable reference, instead of the ref itself.
        }

        match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_run_state = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToConsumeItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToConsumeItem {
                                    volt_pack: item_entity,
                                },
                            )
                            .expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowDropItem => {
                //
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to drop item");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_run_state;
        }

        //let map = self.ecs.fetch::<Vec<TileType>>();
        damage_system::delete_the_dead(&mut self.ecs);
        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        // https://specs.amethyst.rs/docs/tutorials/11_advanced_component.html?highlight=sort#sorting-entities-based-on-component-value
        let mut data = (&positions, &renderables).join().collect::<Vec<_>>();

        data.sort_by(|&a, &b | b.1.render_order.cmp(&a.1.render_order));
        // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/sorting.html

        for(pos, render) in data.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);

            }
        }

        gui::draw_ui(&self.ecs, ctx);
    }
}

fn main() {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50().with_title("Mainframe").build();
    context.with_post_scanlines(true);

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<HPotion>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToConsumeItem>();
    gs.ecs.register::<WantsToDropItem>();
    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    // Generate Monsters
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(player_entity);
    gs.ecs.insert(map); // resource
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welc0me to MainFrame".to_string()],
    });

    rltk::main_loop(context, gs);
}
