use super::{
    gamelog::GameLog, AreaOfEffect, CombatStats, Consumeable, Disable, Equippable, Equipped,
    InBackpack, InflictsDamage, Map, Name, Position, ProvidesHealing, SufferDamage,
    WantsToDropItem, WantsToPickupItem, WantsToRemoveItem, WantsToUseItem,
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

            backpack.remove(dropped_item.item);

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

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
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
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Disable>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    #[allow(clippy::cognitive_complexity)]
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
            aoe,
            mut inflict_disable,
            mut combat_stats,
            equippable,
            mut equipped,
            mut backpack,
        ) = data;

        for (entity, useitem) in (&entities, &wants_to_use).join() {
            let mut used_item = true;
            let mut targets: Vec<Entity> = Vec::new();

            match useitem.target {
                None => targets.push(*player_entity),
                Some(target) => {
                    let a_effect = aoe.get(useitem.item);
                    match a_effect {
                        None => {
                            let idx = map.xy_idx(target.x, target.y);
                            for enemies in map.tile_content[idx].iter() {
                                targets.push(*enemies);
                            }
                        }
                        Some(a_effect) => {
                            let mut affected_tiles =
                                rltk::field_of_view(target, a_effect.radius, &*map);
                            affected_tiles.retain(|z| {
                                z.x > 0 && z.x < map.width - 1 && z.y > 0 && z.y < map.height - 1
                            });
                            // filter out everything except what exactly fufills the predicate

                            for tile_index in affected_tiles.iter() {
                                let idx = map.xy_idx(tile_index.x, tile_index.y);

                                for enemies in map.tile_content[idx].iter() {
                                    targets.push(*enemies)
                                }
                            }
                        }
                    }
                }
            }

            // check if equippable
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // if item already in slot
                    let mut unequip: Vec<Entity> = Vec::new();

                    // check slot
                    for (item_entity, already_equipped, name) in
                        (&entities, &equipped, &names).join()
                    {
                        if already_equipped.owner == target && already_equipped.slot == target_slot
                        {
                            unequip.push(item_entity);

                            if target == *player_entity {
                                gamelog
                                    .entries
                                    .push(format!("Unequipping {}...", name.name));
                            }
                        }
                    }
                    // remove item from slot, and place in backpack
                    for item in unequip.iter() {
                        equipped.remove(*item);
                        backpack
                            .insert(*item, InBackpack { owner: target })
                            .expect("unable to put item in backpack");
                    }

                    // finally equip the item
                    equipped
                        .insert(
                            useitem.item,
                            Equipped {
                                owner: target,
                                slot: target_slot,
                            },
                        )
                        .expect("unable to equip item");
                    // remove from backpack
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!(
                            "Equipping {}...",
                            names.get(useitem.item).unwrap().name
                        ))
                    }
                }
            }

            // check if the item can heal
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(heal) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);

                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + heal.heal_amount);
                        }
                    }
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
            match item_damages {
                None => {}
                Some(damage) => {
                    used_item = false;
                    for enemy in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *enemy, damage.damage);
                        if entity == *player_entity {
                            let enemy_name = names.get(*enemy).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!(
                                "You use {} on {}, inflicting {} hp.",
                                item_name.name, enemy_name.name, damage.damage
                            ));
                        }

                        used_item = true;
                    }
                }
            }

            let mut disable_affected = Vec::new();

            {
                let item_disables = inflict_disable.get(useitem.item);
                match item_disables {
                    None => {}
                    Some(disabling) => {
                        used_item = false;

                        for enemy in targets.iter() {
                            disable_affected.push((*enemy, disabling.turns));

                            if entity == *player_entity {
                                let enemy_name = names.get(*enemy).unwrap();
                                let item_name = names.get(useitem.item).unwrap();

                                gamelog.entries.push(format!(
                                    "You activated {}! disabling the {}",
                                    item_name.name, enemy_name.name
                                ))
                            }
                            used_item = true;
                        }
                    }
                }
            }
            for enemy in disable_affected.iter() {
                inflict_disable
                    .insert(enemy.0, Disable { turns: enemy.1 })
                    .expect("Unable to inflic disable");
            }
            // if consumeable, then delete.

            if used_item {
                let consumeable = consumables.get(useitem.item);
                match consumeable {
                    None => {}
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }

        wants_to_use.clear();
    }
}

pub struct ItemRemoveSystem {}

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_to_remove, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_to_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("can't insert");
        }
        wants_to_remove.clear()
    }
}
