//! Alien Hives army list (Grimdark Future v3.5.3)
//!
//! Base unit profiles from the Alien Hives rulebook. Upgrades/exchanges
//! are not modelled here; these are the default loadouts.

use crate::models::rules::SpecialRule;
use crate::models::unit::Unit;
use crate::models::upgrades::{UpgradeGroup, UpgradeOption};
use crate::models::weapons::Weapon;

/// Build the full Alien Hives base roster.
#[must_use]
pub fn alien_hives() -> Vec<Unit> {
    vec![
        hive_lord(),
        prime_warrior(),
        snatcher_lord(),
        grunt_veteran(),
        assault_grunts(),
        shooter_grunts(),
        psycho_grunts(),
        winged_grunts(),
        support_grunts(),
        soul_snatchers(),
        hive_swarms(),
        hive_warriors(),
        ravenous_beasts(),
        venom_beasts(),
        hive_guardians(),
        shadow_leapers(),
        synapse_beasts(),
        spores(),
        massive_spores(),
        shadow_hunter(),
        mortar_beast(),
        synapse_tyrant(),
        flamer_beast(),
        invasion_carrier_spore(),
        carnivo_rex(),
        toxico_rex(),
        psycho_rex(),
        hive_burrower(),
        tyrant_great_beast(),
        spawning_great_beast(),
        devourer_great_beast(),
        artillery_great_beast(),
        invasion_artillery_spore(),
        hive_titan(),
        rapacious_beast(),
    ]
}

// ---------------------------------------------------------------------------
// Heroes
// ---------------------------------------------------------------------------

/// Hive Lord [1] - 360pts | Q3+ D2+ Tough(12)
#[allow(clippy::too_many_lines)] // declarative roster data, not logic
fn hive_lord() -> Unit {
    Unit::new("Hive Lord", 1, 3, 2)
        .with_points(360)
        .with_rule(SpecialRule::Fear(2))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::Hero)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(12))
        .with_weapon(Weapon::ranged("Shredder Cannon", 1, 4, 18).with_rule(SpecialRule::Rending))
        .with_weapon(Weapon::melee("Heavy Razor Claws", 2, 3).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Stomp", 1, 4).with_rule(SpecialRule::AP(1)))
        .with_upgrades(UpgradeGroup::pick_one(
            "Upgrade with one",
            vec![
                UpgradeOption::rule(
                    "Combat Bio-Engineer",
                    "Furious",
                    5,
                    vec![SpecialRule::Furious],
                ),
                UpgradeOption::rule(
                    "Bio-Tech Master",
                    "Increased Shooting Range",
                    5,
                    vec![SpecialRule::IncreasedShootingRange],
                ),
                UpgradeOption::rule(
                    "Brood Leader",
                    "Rapid Charge",
                    20,
                    vec![SpecialRule::RapidCharge],
                ),
                UpgradeOption::rule(
                    "Hive Protector",
                    "Regeneration",
                    45,
                    vec![SpecialRule::Regeneration],
                ),
                UpgradeOption::rule(
                    "Psychic Synapses",
                    "Caster(3)",
                    60,
                    vec![SpecialRule::Caster(3)],
                ),
            ],
        ))
        .with_upgrades(UpgradeGroup::replace_one(
            "Replace any Heavy Razor Claw",
            "Heavy Razor Claws",
            vec![
                UpgradeOption::replace(
                    "Smashing Club",
                    "A1, AP(2), Blast(3)",
                    0,
                    Weapon::melee("Smashing Club", 1, 1)
                        .with_rule(SpecialRule::AP(2))
                        .with_rule(SpecialRule::Blast(3)),
                ),
                UpgradeOption::replace(
                    "Piercing Spike",
                    "A1, AP(2), Deadly(3)",
                    5,
                    Weapon::melee("Piercing Spike", 1, 1)
                        .with_rule(SpecialRule::AP(2))
                        .with_rule(SpecialRule::Deadly(3)),
                ),
                UpgradeOption::replace(
                    "Slashing Blade",
                    "A3, AP(1), Rending",
                    5,
                    Weapon::melee("Slashing Blade", 1, 3)
                        .with_rule(SpecialRule::AP(1))
                        .with_rule(SpecialRule::Rending),
                ),
                UpgradeOption::replace(
                    "Razor Whip",
                    "A3, Bane, Precise",
                    5,
                    Weapon::melee("Razor Whip", 1, 3)
                        .with_rule(SpecialRule::Bane)
                        .with_rule(SpecialRule::Precise),
                ),
                UpgradeOption::replace(
                    "Serrated Blade",
                    "A3, AP(4)",
                    20,
                    Weapon::melee("Serrated Blade", 1, 3).with_rule(SpecialRule::AP(4)),
                ),
            ],
        ))
        .with_upgrades(UpgradeGroup::replace_all(
            "Replace Shredder Cannon",
            "Shredder Cannon",
            vec![
                UpgradeOption::replace(
                    "2x Heavy Razor Claws",
                    "A3, AP(1)",
                    5,
                    Weapon::melee("Heavy Razor Claws", 2, 3).with_rule(SpecialRule::AP(1)),
                ),
                UpgradeOption::replace(
                    "Spitter Cannon",
                    "24\", A2, Blast(3)",
                    5,
                    Weapon::ranged("Spitter Cannon", 1, 2, 24).with_rule(SpecialRule::Blast(3)),
                ),
                UpgradeOption::replace(
                    "Barb Cannon",
                    "36\", A1, AP(2), Blast(3)",
                    15,
                    Weapon::ranged("Barb Cannon", 1, 1, 36)
                        .with_rule(SpecialRule::AP(2))
                        .with_rule(SpecialRule::Blast(3)),
                ),
                UpgradeOption::replace(
                    "Heavy Smashing Club",
                    "A2, AP(2), Blast(3), Reliable",
                    20,
                    Weapon::melee("Heavy Smashing Club", 1, 2)
                        .with_rule(SpecialRule::AP(2))
                        .with_rule(SpecialRule::Blast(3))
                        .with_rule(SpecialRule::Reliable),
                ),
                UpgradeOption::replace(
                    "Heavy Piercing Spike",
                    "A2, AP(2), Deadly(3), Reliable",
                    30,
                    Weapon::melee("Heavy Piercing Spike", 1, 2)
                        .with_rule(SpecialRule::AP(2))
                        .with_rule(SpecialRule::Deadly(3))
                        .with_rule(SpecialRule::Reliable),
                ),
                UpgradeOption::replace(
                    "Heavy Ravager Cannon",
                    "18\", A6, AP(1), Destructive",
                    40,
                    Weapon::ranged("Heavy Ravager Cannon", 1, 6, 18)
                        .with_rule(SpecialRule::AP(1))
                        .with_rule(SpecialRule::Destructive),
                ),
                UpgradeOption::replace(
                    "Heavy Slashing Blade",
                    "A6, AP(1), Reliable, Rending",
                    40,
                    Weapon::melee("Heavy Slashing Blade", 1, 6)
                        .with_rule(SpecialRule::AP(1))
                        .with_rule(SpecialRule::Reliable)
                        .with_rule(SpecialRule::Rending),
                ),
                UpgradeOption::replace(
                    "Acid Cannon",
                    "36\", A1, AP(2), Deadly(6), Unstoppable",
                    50,
                    Weapon::ranged("Acid Cannon", 1, 1, 36)
                        .with_rule(SpecialRule::AP(2))
                        .with_rule(SpecialRule::Deadly(6))
                        .with_rule(SpecialRule::Unstoppable),
                ),
                UpgradeOption::replace(
                    "Heavy Serrated Blade",
                    "A6, AP(4), Reliable",
                    75,
                    Weapon::melee("Heavy Serrated Blade", 1, 6)
                        .with_rule(SpecialRule::AP(4))
                        .with_rule(SpecialRule::Reliable),
                ),
            ],
        ))
        .with_upgrades(UpgradeGroup::multiple(
            "Upgrade with",
            vec![UpgradeOption::rule(
                "Wings",
                "Ambush, Flying",
                90,
                vec![SpecialRule::Ambush, SpecialRule::Flying],
            )],
        ))
}

/// Prime Warrior [1] - 80pts | Q4+ D4+ Tough(6)
fn prime_warrior() -> Unit {
    Unit::new("Prime Warrior", 1, 4, 4)
        .with_points(80)
        .with_rule(SpecialRule::Hero)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(6))
        .with_weapon(Weapon::melee("Heavy Razor Claws", 2, 3).with_rule(SpecialRule::AP(1)))
}

/// Snatcher Lord [1] - 85pts | Q3+ D4+ Tough(3)
fn snatcher_lord() -> Unit {
    Unit::new("Snatcher Lord", 1, 3, 4)
        .with_points(85)
        .with_rule(SpecialRule::Fast)
        .with_rule(SpecialRule::Hero)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Strider)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(
            Weapon::melee("Heavy Claws", 1, 4)
                .with_rule(SpecialRule::AP(1))
                .with_rule(SpecialRule::Rending),
        )
}

/// Grunt Veteran [1] - 25pts | Q5+ D5+ Tough(3)
fn grunt_veteran() -> Unit {
    Unit::new("Grunt Veteran", 1, 5, 5)
        .with_points(25)
        .with_rule(SpecialRule::Hero)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Strider)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(Weapon::melee("Razor Claws", 1, 2))
}

// ---------------------------------------------------------------------------
// Troops
// ---------------------------------------------------------------------------

/// Assault Grunts [10] - 105pts | Q5+ D5+
fn assault_grunts() -> Unit {
    Unit::new("Assault Grunts", 10, 5, 5)
        .with_points(105)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Strider)
        .with_weapon(Weapon::melee("Razor Claws", 10, 2))
}

/// Shooter Grunts [10] - 115pts | Q5+ D5+
fn shooter_grunts() -> Unit {
    Unit::new("Shooter Grunts", 10, 5, 5)
        .with_points(115)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Strider)
        .with_weapon(Weapon::ranged("Bio-Spiners", 10, 2, 6).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Razor Claws", 10, 1))
}

/// Psycho-Grunts [10] - 125pts | Q5+ D5+
fn psycho_grunts() -> Unit {
    Unit::new("Psycho-Grunts", 10, 5, 5)
        .with_points(125)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Resistance)
        .with_rule(SpecialRule::Strider)
        .with_weapon(Weapon::melee("Rending Claws", 10, 1).with_rule(SpecialRule::Rending))
}

/// Winged Grunts [10] - 125pts | Q5+ D5+
fn winged_grunts() -> Unit {
    Unit::new("Winged Grunts", 10, 5, 5)
        .with_points(125)
        .with_rule(SpecialRule::Flying)
        .with_rule(SpecialRule::HiveBond)
        .with_weapon(Weapon::ranged("Bio-Spiners", 10, 2, 6).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Razor Claws", 10, 1))
}

/// Support Grunts [3] - 145pts | Q5+ D5+
fn support_grunts() -> Unit {
    Unit::new("Support Grunts", 3, 5, 5)
        .with_points(145)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Relentless)
        .with_rule(SpecialRule::Strider)
        .with_weapon(
            Weapon::ranged("Bio-Cannons", 3, 1, 24)
                .with_rule(SpecialRule::Blast(3))
                .with_rule(SpecialRule::Indirect)
                .with_rule(SpecialRule::Rending),
        )
        .with_weapon(Weapon::melee("Razor Claws", 3, 1))
}

// ---------------------------------------------------------------------------
// Elite / Specialists
// ---------------------------------------------------------------------------

/// Soul-Snatchers [5] - 160pts | Q3+ D4+
fn soul_snatchers() -> Unit {
    Unit::new("Soul-Snatchers", 5, 3, 4)
        .with_points(160)
        .with_rule(SpecialRule::Fast)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Strider)
        .with_weapon(
            Weapon::melee("Heavy Claws", 5, 2)
                .with_rule(SpecialRule::AP(1))
                .with_rule(SpecialRule::Rending),
        )
}

/// Hive Swarms [3] - 75pts | Q5+ D6+ Tough(3)
fn hive_swarms() -> Unit {
    Unit::new("Hive Swarms", 3, 5, 6)
        .with_points(75)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Strider)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(Weapon::melee("Swarm Attacks", 3, 3).with_rule(SpecialRule::Bane))
}

/// Hive Warriors [3] - 115pts | Q4+ D4+ Tough(3)
fn hive_warriors() -> Unit {
    Unit::new("Hive Warriors", 3, 4, 4)
        .with_points(115)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(Weapon::melee("Razor Claws", 6, 2))
}

/// Ravenous Beasts [3] - 155pts | Q4+ D4+ Tough(3)
fn ravenous_beasts() -> Unit {
    Unit::new("Ravenous Beasts", 3, 4, 4)
        .with_points(155)
        .with_rule(SpecialRule::Fast)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Strider)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(Weapon::melee("Razor Claws", 6, 2))
}

/// Venom Beasts [3] - 150pts | Q4+ D4+ Tough(3)
fn venom_beasts() -> Unit {
    Unit::new("Venom Beasts", 3, 4, 4)
        .with_points(150)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Regeneration)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(
            Weapon::ranged("Poison Spurts", 3, 1, 12)
                .with_rule(SpecialRule::Blast(3))
                .with_rule(SpecialRule::Bane),
        )
        .with_weapon(Weapon::melee("Toxin Claws", 3, 1).with_rule(SpecialRule::Bane))
}

/// Hive Guardians [3] - 155pts | Q3+ D3+ Tough(3)
fn hive_guardians() -> Unit {
    Unit::new("Hive Guardians", 3, 3, 3)
        .with_points(155)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Relentless)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(Weapon::melee("Razor Claws", 6, 2))
}

/// Shadow Leapers [3] - 230pts | Q3+ D4+ Tough(3)
fn shadow_leapers() -> Unit {
    Unit::new("Shadow Leapers", 3, 3, 4)
        .with_points(230)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Scout)
        .with_rule(SpecialRule::Stealth)
        .with_rule(SpecialRule::Strider)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(Weapon::melee("Razor Claws", 6, 2))
}

/// Synapse Beasts [3] - 200pts | Q4+ D4+ Tough(3)
fn synapse_beasts() -> Unit {
    Unit::new("Synapse Beasts", 3, 4, 4)
        .with_points(200)
        .with_rule(SpecialRule::CasterGroup)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Resistance)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(Weapon::ranged("Psy-Blasts", 3, 1, 18).with_rule(SpecialRule::Blast(3)))
        .with_weapon(Weapon::melee("Psy-Shocks", 3, 1))
}

// ---------------------------------------------------------------------------
// Spores
// ---------------------------------------------------------------------------

/// Spores [5] - 60pts | Q6+ D6+
fn spores() -> Unit {
    Unit::new("Spores", 5, 6, 6)
        .with_points(60)
        .with_rule(SpecialRule::NoRetreat)
        .with_rule(SpecialRule::SelfDestruct(2))
        .with_rule(SpecialRule::Slow)
        .with_weapon(Weapon::melee("Tendrils", 5, 1))
}

/// Massive Spores [3] - 115pts | Q6+ D6+ Tough(3)
fn massive_spores() -> Unit {
    Unit::new("Massive Spores", 3, 6, 6)
        .with_points(115)
        .with_rule(SpecialRule::NoRetreat)
        .with_rule(SpecialRule::SelfDestruct(6))
        .with_rule(SpecialRule::Slow)
        .with_rule(SpecialRule::Tough(3))
        .with_weapon(Weapon::melee("Tendrils", 3, 3))
}

// ---------------------------------------------------------------------------
// Monsters / Single-model
// ---------------------------------------------------------------------------

/// Shadow Hunter [1] - 180pts | Q3+ D4+ Tough(6)
fn shadow_hunter() -> Unit {
    Unit::new("Shadow Hunter", 1, 3, 4)
        .with_points(180)
        .with_rule(SpecialRule::Fear(1))
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Infiltrate)
        .with_rule(SpecialRule::Stealth)
        .with_rule(SpecialRule::Strider)
        .with_rule(SpecialRule::Tough(6))
        .with_weapon(Weapon::melee("Heavy Razor Claws", 2, 3).with_rule(SpecialRule::AP(1)))
}

/// Mortar Beast [1] - 155pts | Q4+ D3+ Tough(6)
fn mortar_beast() -> Unit {
    Unit::new("Mortar Beast", 1, 4, 3)
        .with_points(155)
        .with_rule(SpecialRule::Fear(1))
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(6))
        .with_weapon(
            Weapon::ranged("Spore Gun", 1, 2, 24)
                .with_rule(SpecialRule::Blast(3))
                .with_rule(SpecialRule::Indirect)
                .with_rule(SpecialRule::Shred),
        )
        .with_weapon(Weapon::melee("Heavy Razor Claws", 1, 3).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Stomp", 1, 2).with_rule(SpecialRule::AP(1)))
}

/// Synapse Tyrant [1] - 190pts | Q4+ D4+ Tough(6)
fn synapse_tyrant() -> Unit {
    Unit::new("Synapse Tyrant", 1, 4, 4)
        .with_points(190)
        .with_rule(SpecialRule::Caster(3))
        .with_rule(SpecialRule::Fear(1))
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Resistance)
        .with_rule(SpecialRule::Tough(6))
        .with_weapon(
            Weapon::ranged("Heavy Psy-Torrent", 1, 2, 9)
                .with_rule(SpecialRule::AP(1))
                .with_rule(SpecialRule::Blast(3))
                .with_rule(SpecialRule::Reliable),
        )
        .with_weapon(Weapon::melee("Psy-Shock", 1, 3))
}

/// Flamer Beast [1] - 175pts | Q4+ D3+ Tough(6)
fn flamer_beast() -> Unit {
    Unit::new("Flamer Beast", 1, 4, 3)
        .with_points(175)
        .with_rule(SpecialRule::Fear(1))
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(6))
        .with_weapon(
            Weapon::ranged("Spit Flames", 1, 2, 18)
                .with_rule(SpecialRule::AP(1))
                .with_rule(SpecialRule::Blast(3))
                .with_rule(SpecialRule::Reliable),
        )
        .with_weapon(Weapon::melee("Heavy Razor Claws", 1, 3).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Stomp", 1, 2).with_rule(SpecialRule::AP(1)))
}

/// Invasion Carrier Spore [1] - 135pts | Q4+ D3+ Tough(6)
fn invasion_carrier_spore() -> Unit {
    Unit::new("Invasion Carrier Spore", 1, 4, 3)
        .with_points(135)
        .with_rule(SpecialRule::Ambush)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Slow)
        .with_rule(SpecialRule::Tough(6))
        .with_rule(SpecialRule::Transport(11))
        .with_weapon(Weapon::melee("Razor Tendrils", 1, 6).with_rule(SpecialRule::AP(1)))
}

/// Carnivo-Rex [1] - 295pts | Q4+ D2+ Tough(12)
fn carnivo_rex() -> Unit {
    Unit::new("Carnivo-Rex", 1, 4, 2)
        .with_points(295)
        .with_rule(SpecialRule::Fear(2))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(12))
        .with_weapon(Weapon::melee("Heavy Razor Claws", 3, 3).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Stomp", 1, 4).with_rule(SpecialRule::AP(1)))
}

/// Toxico-Rex [1] - 360pts | Q4+ D2+ Tough(12)
fn toxico_rex() -> Unit {
    Unit::new("Toxico-Rex", 1, 4, 2)
        .with_points(360)
        .with_rule(SpecialRule::Fear(2))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Regeneration)
        .with_rule(SpecialRule::Tough(12))
        .with_weapon(
            Weapon::ranged("Acid Spurt", 1, 2, 12)
                .with_rule(SpecialRule::Blast(3))
                .with_rule(SpecialRule::Bane),
        )
        .with_weapon(Weapon::melee("Stomp", 1, 4).with_rule(SpecialRule::AP(1)))
        .with_weapon(
            Weapon::melee("Whip Limbs", 1, 8)
                .with_rule(SpecialRule::Bane)
                .with_rule(SpecialRule::Precise),
        )
}

/// Psycho-Rex [1] - 420pts | Q4+ D2+ Tough(12)
fn psycho_rex() -> Unit {
    Unit::new("Psycho-Rex", 1, 4, 2)
        .with_points(420)
        .with_rule(SpecialRule::Caster(3))
        .with_rule(SpecialRule::Fear(2))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Resistance)
        .with_rule(SpecialRule::Tough(12))
        .with_weapon(
            Weapon::ranged("Heavy Psy-Torrent", 1, 2, 9)
                .with_rule(SpecialRule::AP(1))
                .with_rule(SpecialRule::Blast(3))
                .with_rule(SpecialRule::Reliable),
        )
        .with_weapon(Weapon::melee("Heavy Razor Claws", 1, 6).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Stomp", 1, 4).with_rule(SpecialRule::AP(1)))
}

/// Hive Burrower [1] - 420pts | Q4+ D2+ Tough(15)
fn hive_burrower() -> Unit {
    Unit::new("Hive Burrower", 1, 4, 2)
        .with_points(420)
        .with_rule(SpecialRule::Ambush)
        .with_rule(SpecialRule::Fear(3))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(15))
        .with_weapon(Weapon::melee("Heavy Razor Claws", 3, 3).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Stomp", 1, 5).with_rule(SpecialRule::AP(2)))
}

/// Tyrant Great Beast [1] - 470pts | Q4+ D2+ Tough(15)
fn tyrant_great_beast() -> Unit {
    Unit::new("Tyrant Great Beast", 1, 4, 2)
        .with_points(470)
        .with_rule(SpecialRule::Fear(3))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(15))
        .with_weapon(Weapon::ranged("Bio-Pod", 1, 12, 24))
        .with_weapon(
            Weapon::ranged("Stinger Launcher", 1, 6, 18).with_rule(SpecialRule::Destructive),
        )
        .with_weapon(Weapon::melee("Stomp", 1, 5).with_rule(SpecialRule::AP(2)))
}

/// Spawning Great Beast [1] - 505pts | Q4+ D2+ Tough(15)
fn spawning_great_beast() -> Unit {
    Unit::new("Spawning Great Beast", 1, 4, 2)
        .with_points(505)
        .with_rule(SpecialRule::Fear(3))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(15))
        .with_weapon(
            Weapon::ranged("Stinger Launchers", 3, 6, 18).with_rule(SpecialRule::Destructive),
        )
        .with_weapon(Weapon::melee("Heavy Razor Claws", 1, 3).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Stomp", 1, 5).with_rule(SpecialRule::AP(2)))
}

/// Devourer Great Beast [1] - 445pts | Q4+ D2+ Tough(15)
fn devourer_great_beast() -> Unit {
    Unit::new("Devourer Great Beast", 1, 4, 2)
        .with_points(445)
        .with_rule(SpecialRule::Fear(3))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(15))
        .with_weapon(
            Weapon::ranged("Devouring Tongue", 1, 3, 12)
                .with_rule(SpecialRule::AP(2))
                .with_rule(SpecialRule::Deadly(3))
                .with_rule(SpecialRule::Takedown),
        )
        .with_weapon(Weapon::melee("Heavy Razor Claws", 3, 3).with_rule(SpecialRule::AP(1)))
        .with_weapon(Weapon::melee("Stomp", 1, 5).with_rule(SpecialRule::AP(2)))
}

/// Artillery Great Beast [1] - 550pts | Q4+ D2+ Tough(15)
fn artillery_great_beast() -> Unit {
    Unit::new("Artillery Great Beast", 1, 4, 2)
        .with_points(550)
        .with_rule(SpecialRule::Fear(3))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(15))
        .with_weapon(
            Weapon::ranged("Shredder Bio-Artillery", 1, 3, 36)
                .with_rule(SpecialRule::Blast(6))
                .with_rule(SpecialRule::Indirect)
                .with_rule(SpecialRule::Rending),
        )
        .with_weapon(Weapon::melee("Stomp", 1, 5).with_rule(SpecialRule::AP(2)))
}

/// Invasion Artillery Spore [1] - 195pts | Q4+ D3+ Tough(6)
fn invasion_artillery_spore() -> Unit {
    Unit::new("Invasion Artillery Spore", 1, 4, 3)
        .with_points(195)
        .with_rule(SpecialRule::Ambush)
        .with_rule(SpecialRule::Artillery)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Spawn("Spores [5]".to_string()))
        .with_rule(SpecialRule::Tough(6))
        .with_weapon(Weapon::melee("Razor Tendrils", 1, 6).with_rule(SpecialRule::AP(1)))
}

/// Hive Titan [1] - 685pts | Q3+ D2+ Tough(18)
fn hive_titan() -> Unit {
    Unit::new("Hive Titan", 1, 3, 2)
        .with_points(685)
        .with_rule(SpecialRule::Fear(3))
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Ravage(12))
        .with_rule(SpecialRule::Tough(18))
        .with_weapon(Weapon::melee("Stomp", 1, 6).with_rule(SpecialRule::AP(2)))
        .with_weapon(
            Weapon::melee("Titanic Heavy Claws", 1, 18)
                .with_rule(SpecialRule::AP(2))
                .with_rule(SpecialRule::Destructive),
        )
}

/// Rapacious Beast [1] - 215pts | Q4+ D2+ Tough(6)
fn rapacious_beast() -> Unit {
    Unit::new("Rapacious Beast", 1, 4, 2)
        .with_points(215)
        .with_rule(SpecialRule::Aircraft)
        .with_rule(SpecialRule::Fearless)
        .with_rule(SpecialRule::HiveBond)
        .with_rule(SpecialRule::Tough(6))
        .with_weapon(
            Weapon::ranged("Stinger Spitter", 1, 3, 18).with_rule(SpecialRule::Destructive),
        )
        .with_weapon(
            Weapon::ranged("Caustic Cannon", 1, 2, 12)
                .with_rule(SpecialRule::Blast(3))
                .with_rule(SpecialRule::Reliable),
        )
}
