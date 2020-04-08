use super::{
    gamelog::GameLog, CombatStats, HPotion, InBackpack, Name, Position, ProvidesHealing,
    WantsToDropItem, WantsToPickupItem, WantsToUseItem,
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
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, HPotion>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_to_use,
            names,
            consumables,
            healing,
            mut combat_stats,
        ) = data;

        for (entity, useitem, stats) in (&entities, &wants_to_use, &mut combat_stats).join() {
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

            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You drink the {}, healing {} hp.",
                            names.get(useitem.item).unwrap().name,
                            healer.heal_amount
                        ));
                    }
                }
            }
        }

        wants_to_use.clear();
    }
}
