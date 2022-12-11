use std::collections::BTreeSet;
use std::collections::BTreeMap;

pub struct Stats {

    poison_tolerance: PoisonTolerance,
    // Affected by eating spicy foods, walking on hot surfaces, getting set on
    // fire, etc.
    fire_tolerance: FireTolerance,
    skin_toughness: SkinToughness,
}

pub struct Character {
    model: Model,
    stats: Stats,
    status_effects: BTreeSet<StatusEffect>,
}
