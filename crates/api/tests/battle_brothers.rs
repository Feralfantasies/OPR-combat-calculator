//! Integration tests for the Battle Brothers army loaded from YAML.
//!
//! Catalog-wide sanity (versions, stat ranges, weapon presence, ranged
//! range bounds) lives in `tests/all_armies.rs`, which covers every
//! committed catalog. This file keeps faction-specific behaviour: exact
//! rule fidelity for a flagship unit and combat-resolution paths, and the
//! army is loaded once per test binary and cached in a `OnceLock`.

#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::panic
)]

use opr_api::{Army, CombatContext, SpecialRule, Unit, Weapon, get_army, get_unit, resolve_attack};
use std::sync::OnceLock;

/// Per-test-binary cache: the registry itself is `OnceLock`-cached in the
/// API, so at most one catalog load happens per test binary; this only
/// avoids re-returning clones from `get_army` on every call.
fn bb_army() -> &'static Army {
    static BB: OnceLock<Army> = OnceLock::new();
    BB.get_or_init(|| get_army("battle-brothers").expect("Battle Brothers catalog should load"))
}

/// Get a specific unit from Battle Brothers by name.
fn get_bb_unit(name: &str) -> &Unit {
    bb_army()
        .units
        .iter()
        .find(|u| u.name == name)
        .unwrap_or_else(|| panic!("Unit '{name}' not found in Battle Brothers"))
}

#[test]
fn battle_brothers_loads_successfully() {
    let army = bb_army();
    assert_eq!(army.id, "battle-brothers");
    assert_eq!(army.name, "Battle Brothers");
    assert!(army.version.is_some());
    assert!(
        army.units.len() > 5,
        "Battle Brothers should have more than 5 units, found {}",
        army.units.len()
    );
}

/// Exact stat + rule fidelity for the flagship unit.
#[test]
fn master_destroyer_has_correct_stats_and_rules() {
    let unit = get_bb_unit("Master Destroyer");
    assert_eq!(unit.quantity, 1);
    assert_eq!(unit.quality, 3);
    assert_eq!(unit.defense, 3);
    assert_eq!(unit.tough, 6);
    assert_eq!(unit.points, 145);
    assert!(!unit.weapons.is_empty());
    for rule in [
        SpecialRule::Ambush,
        SpecialRule::Battleborn,
        SpecialRule::Fearless,
        SpecialRule::Hero,
        SpecialRule::Shielded,
    ] {
        assert!(unit.has_rule(&rule), "missing rule: {rule:?}");
    }
}

#[test]
fn public_lookups_find_master_destroyer() {
    assert_eq!(
        get_army("battle-brothers").expect("army loads").name,
        "Battle Brothers"
    );
    let unit = get_unit("battle-brothers", "Master Destroyer").expect("unit loads");
    assert_eq!(unit.quality, 3);
}

#[test]
fn master_destroyer_can_attack() {
    let attacker = get_bb_unit("Master Destroyer");

    // Create a simple defender
    let defender = Unit::new("Test Defender", 1, 4, 4)
        .with_points(50)
        .with_weapon(Weapon::melee("Test Weapon", 1, 2));

    let context = CombatContext::melee_charge();

    // Should be able to resolve an attack
    let result = resolve_attack(attacker, &defender, &context);
    assert!(result.is_ok(), "Master Destroyer should be able to attack");

    let combat_result = result.expect("attack should succeed");
    assert!(!combat_result.attacks.is_empty());
}

#[test]
fn battle_brothers_units_cover_melee_and_ranged() {
    let army = bb_army();

    let has_ranged = army
        .units
        .iter()
        .any(|u| u.weapons.iter().any(Weapon::is_ranged));
    let has_melee = army
        .units
        .iter()
        .any(|u| u.weapons.iter().any(Weapon::is_melee));

    assert!(
        has_ranged,
        "Should have at least one unit with ranged weapons"
    );
    assert!(
        has_melee,
        "Should have at least one unit with melee weapons"
    );
}

#[test]
fn battle_brothers_units_have_upgrade_options() {
    let army = bb_army();

    // At least some units should have upgrade options
    let has_upgrades = army.units.iter().any(|u| !u.upgrade_groups.is_empty());

    assert!(
        has_upgrades,
        "At least some Battle Brothers units should have upgrade options"
    );
}
