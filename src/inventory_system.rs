use super::{
    gamelog::GameLog, CombatStats, Consumeable, InBackpack, Name, Position, ProvidesHealing,
    InflictsDamage,SufferDamage,WantsToDropItem, WantsToPickupItem, WantsToUseItem,Map
};
use specs::prelude::*;

pub struct InventorySystem {}

impl<'a> System<'a> for InventorySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("unable to regist entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut dropping_item,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, dropped_item) in (&entities, &dropping_item).join() {
            let mut entity_position: Position = Position { x: 0, y: 0 };

            {
                // closure
                let dropping_position = positions.get(entity).unwrap();
                entity_position.x = dropping_position.x;
                entity_position.y = dropping_position.y;
            }
            positions
                .insert(
                    dropped_item.item,
                    Position {
                        x: entity_position.x,
                        y: entity_position.y,
                    },
                )
                .expect("unable to render position");

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    names.get(dropped_item.item).unwrap().name
                ));
            }
        }
        dropping_item.clear();
    }
}

pub struct UseConsumableSystem {}

impl<'a> System<'a> for UseConsumableSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumeable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut wants_to_use,
            names,
            consumables,
            healing,
            inflict_damage,
            mut suffer_damage,
            mut combat_stats,
        ) = data;

        for (entity, useitem, stats) in (&entities, &wants_to_use, &mut combat_stats).join() {

            let mut used_item = true;
            // check if the item can heal
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(heal) => {
                    used_item = false;
                    stats.hp = i32::min(stats.max_hp, stats.hp + heal.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You connect the {}, regenerating {} volts.",
                            names.get(useitem.item).unwrap().name,
                            heal.heal_amount
                        ));
                    }
                }
            }

            let item_damages = inflict_damage.get(useitem.item);

            match item_damages{
                None => {}
                Some(damage) =>{
                    let target_point = useitem.target.unwrap();
                    let idx = map.xy_idx(target_point.x,target_point.y);

                    used_item = false;

                    for enemy in map.tile_content[idx].iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *enemy, damage.damage);
                        if entity == *player_entity {
                            let enemy_name = names.get(*enemy).unwrap();
                            let item_name = names.get(useitem.item).unwrap();

                            gamelog.entries.push(format!("You charged your {}, and shot the {}, inflicting {} damage", item_name.name, enemy_name.name, damage.damage));
                        }

                        used_item = true;
                    }
                } 
            }

            // if consumeable, then delete.
            let consumeable = consumables.get(useitem.item);
            match consumeable {
                None => {}
                Some(_) => {
                    // stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    // if entity == *player_entity {
                    //     gamelog.entries.push(format!(
                    //         "You useitem the {}, healing {} hp.",
                    //         names.get(useitem.item).unwrap().name,
                    //         potion.heal_amount
                    //     ));
                    // }
                    entities.delete(useitem.item).expect("Delete failed");
                }
            }
        }

        wants_to_use.clear();
    }
}
