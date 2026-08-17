//! Integration tests for Battle Brothers army loaded from YAML.

#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::panic,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names
)]

use opr_api::{CombatContext, SpecialRule, Unit, Weapon, get_army, resolve_attack};

/// Load the Battle Brothers army from the unified YAML registry.
fn load_bb_army() -> opr_api::Army {
    get_army("battle-brothers").expect("Battle Brothers catalog should load")
}

/// Get a specific unit from Battle Brothers by name.
fn get_bb_unit(name: &str) -> Unit {
    let army = load_bb_army();
    army.units
        .into_iter()
        .find(|u| u.name == name)
        .unwrap_or_else(|| panic!("Unit '{name}' not found in Battle Brothers"))
}

#[test]
fn battle_brothers_loads_successfully() {
    let army = load_bb_army();
    assert_eq!(army.id, "battle-brothers");
    assert_eq!(army.name, "Battle Brothers");
    assert!(army.version.is_some());
    assert!(!army.units.is_empty());
}

#[test]
fn master_destroyer_has_correct_stats() {
    let unit = get_bb_unit("Master Destroyer");
    assert_eq!(unit.quantity, 1);
    assert_eq!(unit.quality, 3);
    assert_eq!(unit.defense, 3);
    assert_eq!(unit.points, 145);
    assert!(!unit.weapons.is_empty());
    assert!(!unit.special_rules.is_empty());
}

#[test]
fn master_destroyer_has_expected_special_rules() {
    let unit = get_bb_unit("Master Destroyer");
    assert!(unit.has_rule(&SpecialRule::Ambush));
    assert!(unit.has_rule(&SpecialRule::Fearless));
    assert!(unit.has_rule(&SpecialRule::Hero));
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
    let result = resolve_attack(&attacker, &defender, &context);
    assert!(result.is_ok(), "Master Destroyer should be able to attack");

    let combat_result = result.expect("attack should succeed");
    assert!(!combat_result.attacks.is_empty());
}

#[test]
fn battle_brothers_units_have_weapons() {
    let army = load_bb_army();

    // All units should have at least one weapon
    for unit in &army.units {
        assert!(
            !unit.weapons.is_empty(),
            "Unit '{}' should have at least one weapon",
            unit.name
        );
    }
}

#[test]
fn battle_brothers_units_have_upgrade_options() {
    let army = load_bb_army();

    // At least some units should have upgrade options
    let has_upgrades = army.units.iter().any(|u| !u.upgrade_groups.is_empty());

    assert!(
        has_upgrades,
        "At least some Battle Brothers units should have upgrade options"
    );
}

#[test]
fn battle_brothers_ranged_vs_melee() {
    let army = load_bb_army();

    // Find a unit with ranged weapons
    let has_ranged = army
        .units
        .iter()
        .any(|u| u.weapons.iter().any(|w| w.is_ranged()));

    // Find a unit with melee weapons
    let has_melee = army
        .units
        .iter()
        .any(|u| u.weapons.iter().any(|w| w.is_melee()));

    assert!(
        has_ranged,
        "Should have at least one unit with ranged weapons"
    );
    assert!(
        has_melee,
        "Should have at least one unit with melee weapons"
    );
}
