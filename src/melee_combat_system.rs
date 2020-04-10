use super::{gamelog::GameLog, CombatStats, Name, SufferDamage, WantsToMelee, AtkBonus, DefBonus, Equipped};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AtkBonus>,
        ReadStorage<'a, DefBonus>,
        ReadStorage<'a, Equipped>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, mut wants_melee, names, combat_stats, mut inflict_damage, atk_bonuses, def_bonuses, equipped) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            
            if stats.hp > 0 {
                let mut offensive_bonus = 0;

                for(_item_entity, power_bonus, equipped_by) in (&entities, &atk_bonuses, &equipped).join(){

                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.amount
                    }
                }


                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;

                    for(item_entity, def_bonus, equipped_by) in (&entities, &def_bonuses, &equipped).join(){

                        if equipped_by.owner == wants_melee.target{
                            defensive_bonus += def_bonus.amount
                        }
                    }
                    let damage = i32::max(0, (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus)) ;

                    if damage == 0 {
                        log.entries.push(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ))
                    } else {
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
