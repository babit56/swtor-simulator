#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Component)]
enum EntityType {
    Player,
    Boss,
    Add,
}

#[derive(Component)]
enum Class {
    Jugg,
    Sin,
    Sorc,
    Mara,
    Oper,
    Sniper,
    Merc,
    PT,
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
            class: Class::Jugg,
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

// heh
struct Effect;

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

// enum CombatMode {
//     Melee,
//     Ranged,
// }

// struct Effect;

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
