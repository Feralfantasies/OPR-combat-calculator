//! Integration test preserving the original CLI demo behaviour:
//! resolve a ranged and a melee attack between two Alien Hives units.
//!
//! Test-only allowance: fixture lookup failures are expressed with `panic!`
//! for concise test code, which the workspace's production lint denies.
#![allow(clippy::panic)]

use opr_api::{CombatContext, alien_hives, resolve_attack};

fn find_unit(roster: &[opr_api::Unit], name: &str) -> opr_api::Unit {
    roster
        .iter()
        .find(|u| u.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("unit not found: {name}"))
}

#[test]
fn roster_has_35_units() {
    assert_eq!(alien_hives().len(), 35);
}

#[test]
fn ranged_attack_resolves() {
    let roster = alien_hives();
    let attacker = find_unit(&roster, "Shooter Grunts");
    let defender = find_unit(&roster, "Hive Warriors");

    let context = CombatContext::ranged(24);
    let result = resolve_attack(&attacker, &defender, &context);
    assert!(result.is_ok());
    let result = result.unwrap_or_default();
    // Shooter Grunts fire Bio-Spiners; expect exactly one weapon resolved.
    assert_eq!(result.attacks.len(), 1);
}

#[test]
fn melee_attack_resolves() {
    let roster = alien_hives();
    let attacker = find_unit(&roster, "Shooter Grunts");
    let defender = find_unit(&roster, "Hive Warriors");

    let context = CombatContext::melee_charge();
    let result = resolve_attack(&attacker, &defender, &context);
    assert!(result.is_ok());
}
