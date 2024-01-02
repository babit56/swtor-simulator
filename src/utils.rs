pub mod constants {
    pub static LEVEL: f64 = 80.0;
    pub static BASE_LEVEL_DMG: f64 = 19335.0;

    // Mastery
    pub static BASE_MASTERY: f64 = 1150.0;
    pub static DATACRON_MASTERY: f64 = 17.0; // 224 with all datacrons
    pub static MASTERY_DMG_BONUS: f64 = 0.2;

    pub static POWER_DMG_BONUS: f64 = 0.23;
    // static POWER_HEAL_BONUS: f64 = 0.17; // From dulfy crit chance guide, need to double-check


    pub static BASE_CRIT: f64 = 0.5;
    pub static CRIT_MAGIC: f64 = 2.41;

    // Class buffs
    // Works for m/r and f/t and heals. Duplicate for heals or make common variable?
    pub static CLASS_BUFF_BONUS_DMG: f64 = 0.05; // Sith Warrior
    //pub static CLASS_BUFF_CRIT_CHANCE: f64 = 0.05; // Agent
    pub static CLASS_BUFF_MASTERY: f64 = 0.05; // Sith Inquisitor
    // 5% endurance buff from bh, maybe unnecessary

    // Companion buffs
    pub static COMPANION_BUFF_CRIT_SURGE: f64 = 0.01;
    // Same for acc, healing recieved, hp, crit chance

    pub static BOSS_ARMOR: f64 = 17227.0;
    pub static ARMOR_MAGIC_1: f64 = 389.99;
    pub static ARMOR_MAGIC_2: f64 = 800.0;

    // For thrash
    pub static COEFFICIENT: f64 = 0.72;
    pub static SHP: f64 = 0.072; // StandardHealthPercent, should be min/max
    pub static AMP: f64 = -0.52; // AmountModifierPercent
    pub static PASSIVE_MODS: f64 = -0.05; // Dark Charge (-0.1) + Trashing Blades (0.05)

    // From dummy armor reduction
    pub static MELEE_DMG_BUFF: f64 = 0.05;
    pub static ARMOR_DEBUFF: f64 = 0.2;
}
