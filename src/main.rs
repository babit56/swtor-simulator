mod utils;
mod ecs_system;
mod parse;

use std::str::FromStr;

use ecs_system::{get_abilities, TreeChoices, TreeChoice, CombatStyle};
use utils::constants::*;
use bevy::prelude::*;

struct Gear {
    weapon_min: f64,
    weapon_max: f64,
    gear_crit: f64,
    gear_mastery: f64,
    gear_power: f64,
}

impl Gear {
    fn get_mastery(&self) -> f64 {
        let mastery = BASE_MASTERY + DATACRON_MASTERY + self.gear_mastery;
        mastery * 1.0 + CLASS_BUFF_MASTERY
    }

    fn get_power(&self) -> f64 {
        self.gear_power
    }

    fn get_bonus_dmg(&self) -> f64 {
        let bonus_dmg = self.get_mastery() * MASTERY_DMG_BONUS + self.get_power() * POWER_DMG_BONUS;
        bonus_dmg * 1.0 + CLASS_BUFF_BONUS_DMG
    }

    fn get_crit_surge(&self) -> f64 {
        BASE_CRIT + COMPANION_BUFF_CRIT_SURGE + 0.3 * (1.0 - (1.0f64 - 0.01/0.3).powf(self.gear_crit/LEVEL/CRIT_MAGIC))
    }

    fn tooltip_dmg(&self) -> (f64, f64) {
        let dmg_min = self.weapon_min * (1.0 + AMP) + COEFFICIENT * self.get_bonus_dmg() + SHP * BASE_LEVEL_DMG;
        let dmg_max = self.weapon_max * (1.0 + AMP) + COEFFICIENT * self.get_bonus_dmg() + SHP * BASE_LEVEL_DMG;
        let tooltip_min = dmg_min * (1.0 + PASSIVE_MODS);
        let tooltip_max = dmg_max * (1.0 + PASSIVE_MODS);
        (tooltip_min, tooltip_max)
    }

    fn dummy_dmg(&self) -> (f64, f64) {
        let (tooltip_min, tooltip_max) = self.tooltip_dmg();
        let armor = BOSS_ARMOR * (1.0 - ARMOR_DEBUFF);
        let armor_dmg_reduction = armor / (armor + ARMOR_MAGIC_1 * LEVEL + ARMOR_MAGIC_2);
        let dummy_min = tooltip_min * (1.0 - armor_dmg_reduction) / (1.0 - MELEE_DMG_BUFF);
        let dummy_max = tooltip_max * (1.0 - armor_dmg_reduction) / (1.0 - MELEE_DMG_BUFF);
        (dummy_min, dummy_max)
    }

    fn dummy_crit(&self) -> (f64, f64) {
        let (dummy_min, dummy_max) = self.dummy_dmg();
        let crit_min = dummy_min * (1.0 + self.get_crit_surge());
        let crit_max = dummy_max * (1.0 + self.get_crit_surge());
        (crit_min, crit_max)
    }
}

fn simple_dmg_calc(gear: Gear) {
    let (tooltip_min, tooltip_max) = gear.tooltip_dmg();
    println!("Tooltip min: {}", tooltip_min);
    println!("Tooltip max: {}", tooltip_max);
    println!("Tooltip: {} - {}", tooltip_min.round(), tooltip_max.round());

    let (dummy_min, dummy_max) = gear.dummy_dmg();
    println!("Dummy min: {}", dummy_min);
    println!("Dummy max: {}", dummy_max);

    let (crit_min, crit_max) = gear.dummy_crit();
    println!("Crit min: {}", crit_min);
    println!("Crit max: {}", crit_max);
}

fn main() {
    let dumb_saber = Gear {
        weapon_min: 46.0,
        weapon_max: 69.0,
        gear_crit: 5.0,
        gear_mastery: 10.0,
        gear_power: 6.0,
    };
    let _real_saber = Gear {
        weapon_min: 2441.0,
        weapon_max: 3661.0,
        gear_crit: 655.0,
        gear_mastery: 1223.0,
        gear_power: 940.0,
    };

    // simple_dmg_calc(dumb_saber);

    // App::new()
    //     .run();

    let choices = TreeChoices {
        combat_style: CombatStyle::from_str("darkness").unwrap(),
        choices: [
            TreeChoice::Right,
            TreeChoice::Left,
            TreeChoice::Right,
            TreeChoice::Right,
            TreeChoice::Middle,
            TreeChoice::Left,
            TreeChoice::Left,
            TreeChoice::Middle,
        ]
    };
    get_abilities(choices);
    // ecs_system::read_dis();
    // parse::test();
}
