#![allow(dead_code)]

use bevy::prelude::*;
use assoc::AssocExt;
use std::fs::File;
use std::io::BufReader;
use crate::parse::{FieldValue, NodeObjPair};

static DATA_PATH: &'static str = "data";

#[derive(Component)]
enum EntityType {
    Player,
    Boss,
    Add,
}

#[derive(Component, Debug, strum::Display, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Class {
    Assassin,
    Juggernaut,
    Marauder,
    Mercenary,
    Operative,
    Powertech,
    Sniper,
    Sorcerer,
}

enum PassiveType {
    Buff,
    // Passive,
    Debuff,
}

struct Passive {
    slot: PassiveType,
    hidden: bool,
    duration: f64,
}

#[derive(Component)]
struct Passives(Vec<Passive>);

impl Default for Passives {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Bundle)]
struct PlayerBundle {
    kind: EntityType,
    class: Class,
    passives: Passives,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            kind: EntityType::Player,
            class: Class::Juggernaut,
            passives: Default::default(),
        }
    }
}

#[derive(Bundle)]
struct DummyBundle {
    kind: EntityType,
    passives: Passives,
}

impl Default for DummyBundle {
    fn default() -> Self {
        Self {
            kind: EntityType::Boss,
            passives: Default::default(),
        }
    }
}

// Marker for ability entities
#[derive(Component)]
struct Ability;

#[derive(Component)]
enum AttackType {
    Instant,
    Cast(f64), // Change to duration
    Channel(f64), // Change to duration
}

// Change to duration
#[derive(Component)]
struct Cooldown(f64);

// Might need multiple, at least for % based and integer based resources
// Make into meaningful unit?
#[derive(Component)]
struct EnergyCost(f64);

// Make into meaningful unit?
#[derive(Component)]
struct MaxRange(f64);

#[derive(Component)]
enum TargetType {
    Any,
    Attackable,
}

#[derive(Component)]
enum CombatMode {
    Melee,
    Ranged,
}

#[derive(Component)]
struct LoSRequired(bool);

#[derive(Component)]
struct BreaksStealth(bool);

#[derive(Component)]
struct Effects(Vec<Effect>);

#[derive(Component)]
struct EffectZero(Effect);

// struct Ability {
//     name: String,

//     cooldown_time: f64,
//     channeling_time: f64,
//     casting_time: f64,

//     energy_cost: f64,
//     max_range: f64,
//     combat_mode: CombatMode,
//     los_required: bool,
//     breaks_stealth: bool,
//     effects: Vec<Effect>,
//     effect_zero: Effect,

// }

// heh
struct Effect;

// Effect needs:
// number: usize
// is_hidden: bool
// is_passive: bool
// stack_limit: usize
// stack_limit_is_by_caster: bool
// subeffect_epp_details: ..., sound stuff
// slot: PassiveType
// persist_after_death: bool
// subeffects: Vec<SubEffect>
// tags: Vec<Tag> or HashMap<Tag, bool>
// cast+channel+1.5s: Duration
// conEntitySpec: Effect, self?
// effAbilitySpec: Ability, owner ability
// effIsInstant: bool
// effDescriptionStringId: usize

// SubEffect needs:
// actions: Vec<EffAct>
// target_overrides: Vec<EffTargOver>
// triggers: Vec<EffTrigger>
// inits: Vec<EffInit>
// cons:
// cons_logic:

// EffAct needs:
// name: Type = { ModifyStat, SpellDamage, ModifyThreat, CallEffect }
// bool/string/int/float_params: Vec<(String/Enum, T)>
// function_tags: Vec<Tag>
// time_interval_params:
// float/int/id_list_params:
// tag_exclusions:
// list_params_mapping:

// EffInit needs same as EffAct except:
// name: Type = { SetTags, SetDescription, SetPassive, SetPersistsAfterDeath, SetType }

// EffTargOver needs same as EffAct except:
// name: Type = { TriggerTarget }

// EffTrigger needs same as EffAct except:
// name: Type = { OnDamageDealt, OnEnterCombat }

#[derive(Debug, strum::Display, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum CombatStyle {
    Darkness,
    Hatred,
    Deception,

    Immortal,
    Rage,
    Vengeance,

    Annihilation,
    Carnage,
    Fury,

    Arsenal,
    Bodyguard,
    InnovativeOrdinance,

    Concealment,
    Lethality,
    Medic,

    AdvancedPrototype,
    Pyrotech,
    ShieldTech,

    Engineering,
    Marksmanship,
    Virulence,

    Corruption,
    Lightning,
    Madness,
}

impl CombatStyle {
    fn get_class(&self) -> Class {
        match self {
            CombatStyle::Darkness | CombatStyle::Hatred | CombatStyle::Deception             => Class::Assassin,
            CombatStyle::Immortal | CombatStyle::Rage | CombatStyle::Vengeance               => Class::Juggernaut,
            CombatStyle::Annihilation | CombatStyle::Carnage | CombatStyle::Fury             => Class::Marauder,
            CombatStyle::Arsenal | CombatStyle::Bodyguard | CombatStyle::InnovativeOrdinance => Class::Mercenary,
            CombatStyle::Concealment | CombatStyle::Lethality | CombatStyle::Medic           => Class::Operative,
            CombatStyle::AdvancedPrototype | CombatStyle::Pyrotech | CombatStyle::ShieldTech => Class::Powertech,
            CombatStyle::Engineering | CombatStyle::Marksmanship | CombatStyle::Virulence    => Class::Sniper,
            CombatStyle::Corruption | CombatStyle::Lightning | CombatStyle::Madness          => Class::Sorcerer,
        }
    }

    fn get_dis_fqn(&self) -> String {
        let class = self.get_class();
        ["dis", &class.to_string(), &self.to_string()].map(|x| x.to_lowercase()).join(".")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TreeChoice {
    Left,
    Middle,
    Right,
}

pub struct TreeChoices {
    pub combat_style: CombatStyle,
    pub choices: [TreeChoice; 8],
}

impl TreeChoices {
    fn get_mods_list(&self) -> Vec<u64> {
        let dis_fqn = self.combat_style.get_dis_fqn();
        let dis_path = format!("{}/dis.json", DATA_PATH);
        let reader = BufReader::new(File::open(dis_path).expect("should find file"));
        let dis_classes: Vec<NodeObjPair> = serde_json::from_reader(reader).expect("json should be good");
        let dis_obj = dis_classes.iter()
                                 .filter(|pair| pair.node.fqn == dis_fqn)
                                 .map(|pair| pair.obj.clone())
                                 .next()
                                 .expect("dis node should exist");
        match (&dis_obj.0[8].value, &dis_obj.0[9].value) {
            (FieldValue::LookupList(lvl_modlist), FieldValue::LookupList(int_to_abl_id)) => {
                self.choices
                    .iter()
                    .map(|choice| *choice as usize)
                    .enumerate()
                    .map(|(i, choice_num)| {
                        match &lvl_modlist[i] {
                            (FieldValue::Int(_), FieldValue::List(v)) => v[choice_num].clone(),
                            _ => unimplemented!(),
                        }
                    })
                    // .inspect(|x| println!("{x:?}"))
                    .map(|int| int_to_abl_id.get(&int).expect("correct dis format"))
                    .map(|id| match id {
                        FieldValue::Id(num) => *num,
                        _ => unimplemented!(),
                    })
                    .collect()
            }
            _ => unimplemented!(),
        }
    }
}

pub fn read_dis() {
    let dis_path = format!("{}/dis.json", DATA_PATH);
    let reader = BufReader::new(File::open(dis_path).expect("should find file"));
    let classes: Vec<NodeObjPair> = serde_json::from_reader(reader).expect("json should be good");
    println!("{:?}", classes[0].obj);
}

pub fn get_abilities(choices: TreeChoices) {
    let mods_ids = choices.get_mods_list();
    let abl_path = format!("{}/abl.json", DATA_PATH);
    let reader = BufReader::new(File::open(abl_path).expect("should find file"));
    let dis_classes: Vec<NodeObjPair> = serde_json::from_reader(reader).expect("json should be good");
    let abilities: Vec<_> = mods_ids.iter()
                                    .map(|id| dis_classes.iter()
                                                         .filter(|pair| &id.to_string() == &pair.node.id)
                                                         .map(|pair| pair.node.fqn.clone())
                                                         .next()
                                                         .or_else(|| get_talent(*id))
                                                         .unwrap())
                                    .collect();
    println!("{:?}", abilities);
}

fn get_talent(id: u64) -> Option<String> {
    let tal_path = format!("{}/tal.json", DATA_PATH);
    let reader = BufReader::new(File::open(tal_path).expect("should find file"));
    let tal_classes: Vec<NodeObjPair> = serde_json::from_reader(reader).expect("json should be good");
    tal_classes.iter()
               .filter(|pair| &id.to_string() == &pair.node.id)
               .map(|pair| pair.node.fqn.clone())
               .next()
}


fn create_abilities(mut commands: Commands) {
    commands.spawn((
        Ability,
        Name::new("Rail shot"),
        AttackType::Instant,
        Cooldown(15.0),
        EnergyCost(15.0),
        MaxRange(10.0),
        TargetType::Attackable,
        CombatMode::Ranged,
        LoSRequired(true),
        BreaksStealth(true),
        Effects(vec![]),
        EffectZero(Effect {})
    ));
}

fn do_stuff(entity: Entity, world: &World) {
    let ability = world.get_entity(entity).expect("Ability should exist in this world");
    match ability.get::<AttackType>().expect("Ability should have attacktype") {
        AttackType::Instant => println!("Instant"),
        AttackType::Cast(t) => println!("Cast: {t}"),
        AttackType::Channel(t) => println!("Channel: {t}"),
    }

    if ability.contains::<Cooldown>() {
        println!("Has cooldown");
    }

    if let Some(Cooldown(cooldown)) = ability.get::<Cooldown>() {
        println!("Cooldown: {cooldown}");
    }
}
