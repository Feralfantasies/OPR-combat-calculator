//! Integration tests for Battle Brothers army loaded from YAML.
//!
//! These tests verify that Battle Brothers units loaded from YAML can be
//! used in combat simulations and behave correctly.

#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::panic,
    clippy::similar_names
)]

use opr_api::{
    CombatContext, SpecialRule, get_army, get_unit, load_battle_brothers_default, resolve_attack,
};

/// Load Battle Brothers, panicking if the YAML file cannot be loaded.
///
/// The goal of this module is to prove that loading works, and the YAML is
/// committed under `data/armies/`, so a failed load must fail loudly
/// rather than silently skipping the assertions.
fn load_battle_brothers_for_tests() -> opr_api::armies::Army {
    load_battle_brothers_default()
        .unwrap_or_else(|err| panic!("Battle Brothers YAML file failed to load: {err:?}"))
}

#[test]
fn battle_brothers_loads_successfully() {
    let army = load_battle_brothers_for_tests();

    assert_eq!(army.id, "battle-brothers");
    assert_eq!(army.name, "Battle Brothers");
    assert!(army.version.is_some());
    assert!(!army.units.is_empty());

    // Should have multiple units
    assert!(
        army.units.len() > 5,
        "Battle Brothers should have more than 5 units, found {}",
        army.units.len()
    );
}

#[test]
fn battle_brothers_has_master_destroyer() {
    let army = load_battle_brothers_for_tests();

    let master = army
        .units
        .iter()
        .find(|u| u.name == "Master Destroyer")
        .expect("Battle Brothers should have Master Destroyer unit");

    assert_eq!(master.quantity, 1);
    assert_eq!(master.quality, 3);
    assert_eq!(master.defense, 3);
    assert_eq!(master.tough, 6);
    assert_eq!(master.points, 145);

    // Should have weapons
    assert!(!master.weapons.is_empty());

    // Should have special rules
    assert!(!master.special_rules.is_empty());

    // Check for specific special rules
    assert!(master.has_rule(&SpecialRule::Fearless));
    assert!(master.has_rule(&SpecialRule::Hero));
}

#[test]
fn battle_brothers_get_army_works() {
    load_battle_brothers_for_tests();

    let army = get_army("battle-brothers");
    assert!(army.is_some(), "get_army should find Battle Brothers");

    let army = army.unwrap();
    assert_eq!(army.name, "Battle Brothers");
}

#[test]
fn battle_brothers_get_unit_works() {
    load_battle_brothers_for_tests();

    let unit = get_unit("battle-brothers", "Master Destroyer");
    assert!(unit.is_some(), "get_unit should find Master Destroyer");

    let unit = unit.unwrap();
    assert_eq!(unit.name, "Master Destroyer");
    assert_eq!(unit.quality, 3);
}

#[test]
fn battle_brothers_unit_can_attack() {
    let army = load_battle_brothers_for_tests();

    // Get Master Destroyer as attacker
    let attacker = army
        .units
        .iter()
        .find(|u| u.name == "Master Destroyer")
        .expect("Should have Master Destroyer");

    // Get a defender (use another Battle Brothers unit)
    let defender = army
        .units
        .iter()
        .find(|u| u.name == "Veteran Master Brother")
        .expect("Should have Veteran Master Brother");

    // Create a melee combat context
    let context = CombatContext::melee_charge();

    // Resolve the attack
    let result = resolve_attack(attacker, defender, &context);
    assert!(result.is_ok(), "Combat resolution should succeed");

    let result = result.unwrap();

    // Should have attacks
    assert!(!result.attacks.is_empty());

    // Should have some wounds (statistically likely)
    // We can't assert exact values since it's random, but we can check structure
    // Note: total_net_wounds and models_removed are unsigned, so >= 0 is always true
}

#[test]
fn battle_brothers_units_have_valid_stats() {
    let army = load_battle_brothers_for_tests();

    for unit in &army.units {
        // Quality should be between 2 and 6 (reasonable range)
        assert!(
            unit.quality >= 2 && unit.quality <= 6,
            "Unit {} has invalid quality: {}",
            unit.name,
            unit.quality
        );

        // Defense should be between 2 and 6 (reasonable range)
        assert!(
            unit.defense >= 2 && unit.defense <= 6,
            "Unit {} has invalid defense: {}",
            unit.name,
            unit.defense
        );

        // Tough should be at least 1
        assert!(
            unit.tough >= 1,
            "Unit {} has invalid tough: {}",
            unit.name,
            unit.tough
        );

        // Should have at least one weapon
        assert!(
            !unit.weapons.is_empty(),
            "Unit {} should have at least one weapon",
            unit.name
        );
    }
}

#[test]
fn battle_brothers_weapons_have_valid_stats() {
    let army = load_battle_brothers_for_tests();

    for unit in &army.units {
        for weapon in &unit.weapons {
            // Attacks should be at least 1
            assert!(
                weapon.attacks >= 1,
                "Weapon {} on unit {} has invalid attacks: {}",
                weapon.name,
                unit.name,
                weapon.attacks
            );

            // If ranged, range should be reasonable
            if let Some(range) = weapon.range {
                assert!(
                    (1..=48).contains(&range),
                    "Weapon {} on unit {} has invalid range: {}",
                    weapon.name,
                    unit.name,
                    range
                );
            }
        }
    }
}
