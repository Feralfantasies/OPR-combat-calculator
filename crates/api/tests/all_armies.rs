//! Catalog sanity + upgrade end-to-end tests over the unified YAML registry.
//!
//! Loads every committed catalog under `data/armies/` through the public
//! API and asserts structural sanity on all of them, then exercises
//! `apply_upgrades` end-to-end against catalog data (rule + cost
//! application, pick-one enforcement, and position-based disambiguation
//! of repeated group names).

#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::panic
)]

use opr_api::{
    Army, SpecialRule, UpgradeSelection, all_armies, apply_upgrades, get_army, get_unit,
};
use std::sync::OnceLock;

/// Per-test-binary cache: the registry itself is `OnceLock`-cached in the
/// API, so at most one `Vec<Army>` clone is paid per test binary.
fn roster() -> &'static [Army] {
    static ROSTER: OnceLock<Vec<Army>> = OnceLock::new();
    ROSTER.get_or_init(all_armies)
}

/// Every committed catalog converts and carries a version key.
#[test]
fn all_committed_catalogs_convert_with_versions() {
    let roster = roster();
    assert!(
        (40..=60).contains(&roster.len()),
        "expected ~43 committed catalogs, found {}",
        roster.len()
    );
    for army in roster {
        assert!(!army.units.is_empty(), "{}: catalog has no units", army.id);
        assert!(
            army.version.as_ref().is_some_and(|v| v.starts_with('v')),
            "{}: missing version key (got {:?})",
            army.id,
            army.version
        );
    }
}

/// Structural sanity over every unit of every catalog.
#[test]
fn every_catalog_unit_has_sane_stats() {
    let roster = roster();
    for army in roster {
        for unit in &army.units {
            assert!(
                (2..=6).contains(&unit.quality),
                "{} / {}: quality out of range: {}",
                army.id,
                unit.name,
                unit.quality
            );
            assert!(
                (2..=6).contains(&unit.defense),
                "{} / {}: defense out of range: {}",
                army.id,
                unit.name,
                unit.defense
            );
            assert!(
                unit.tough >= 1,
                "{} / {}: tough below 1: {}",
                army.id,
                unit.name,
                unit.tough
            );
            assert!(
                !unit.weapons.is_empty(),
                "{} / {}: unit has no weapons",
                army.id,
                unit.name
            );
            for weapon in &unit.weapons {
                if let Some(range) = weapon.range {
                    assert!(
                        (1..=48).contains(&range),
                        "{} / {} / {}: range out of range: {range}",
                        army.id,
                        unit.name,
                        weapon.name
                    );
                }
                assert!(weapon.attacks >= 1);
            }
        }
    }
}

/// `get_army` / `get_unit` keep working for both YAML-backed factions.
#[test]
fn public_lookup_api_covers_both_factions() {
    let bb = get_army("battle-brothers").expect("battle-brothers loads");
    assert_eq!(bb.name, "Battle Brothers");
    assert_eq!(
        get_unit("battle-brothers", "Master Destroyer")
            .expect("Master Destroyer loads")
            .points,
        145
    );
    assert_eq!(get_army("alien-hives").map(|a| a.units.len()), Some(35));
}

/// End-to-end upgrade application on catalog data: a rule-only upgrade
/// adds its rules and cost; a weapon upgrade swaps out the target weapon.
#[test]
fn upgrade_selection_applies_rules_and_costs() {
    let md = get_unit("battle-brothers", "Master Destroyer").expect("Master Destroyer loads");
    let base_points = md.points;

    // 1) Rule-only upgrade: "Upgrade with one" -> Archivist (Caster(2), +40).
    let archivist = UpgradeSelection {
        group: "Upgrade with one".to_string(),
        option: "Archivist".to_string(),
        index: None,
    };
    let upgraded = apply_upgrades(&md, &[archivist]).expect("Archivist upgrade applies cleanly");
    assert_eq!(upgraded.points, base_points + 40);
    assert!(upgraded.has_rule(&SpecialRule::Caster(2)));

    // 2) Weapon upgrade: "Replace CCW" -> Energy Fist (A4, AP(4),
    //    +35): the CCW weapon is swapped out entirely, with no
    //    invented special rules.
    let energy_fist = UpgradeSelection {
        group: "Replace CCW".to_string(),
        option: "Energy Fist".to_string(),
        index: None,
    };
    let upgraded =
        apply_upgrades(&md, &[energy_fist]).expect("Energy Fist upgrade applies cleanly");
    assert_eq!(upgraded.points, base_points + 35);
    assert!(
        !upgraded.weapons.iter().any(|w| w.name == "CCW"),
        "CCW should have been replaced"
    );
    let fist = upgraded
        .weapons
        .iter()
        .find(|w| w.attacks == 4 && w.get_ap() == Some(4))
        .expect("Energy Fist weapon applied");
    assert_eq!(fist.name, "Energy Fist");
    assert_eq!(fist.special_rules, vec![SpecialRule::AP(4)]);

    // 3) Weapon upgrade with rules: "Replace CCW" -> Energy Sword
    //    (A4, AP(1), Rending, +20): the swapped-in weapon carries its
    //    own rule set.
    let energy_sword = UpgradeSelection {
        group: "Replace CCW".to_string(),
        option: "Energy Sword".to_string(),
        index: None,
    };
    let upgraded =
        apply_upgrades(&md, &[energy_sword]).expect("Energy Sword upgrade applies cleanly");
    assert_eq!(upgraded.points, base_points + 20);
    let sword = upgraded
        .weapons
        .iter()
        .find(|w| w.attacks == 4 && w.get_ap() == Some(1))
        .expect("Energy Sword weapon applied");
    assert!(sword.special_rules.contains(&SpecialRule::Rending));
}

/// Two options from one `PickOne` group are rejected.
#[test]
fn pick_one_group_rejects_double_selection() {
    let md = get_unit("battle-brothers", "Master Destroyer").expect("Master Destroyer loads");
    let result = apply_upgrades(
        &md,
        &[
            UpgradeSelection {
                group: "Upgrade with one".to_string(),
                option: "Archivist".to_string(),
                index: None,
            },
            UpgradeSelection {
                group: "Upgrade with one".to_string(),
                option: "Preacher".to_string(),
                index: None,
            },
        ],
    );
    assert!(result.is_err());
}

/// Repeated group names resolve by index: Watch Captain Brother has two
/// distinct "Upgrade with one" sections; the second one (index 7) holds
/// "Jetpack". Name-only lookup hits the first section and cannot find it.
#[test]
fn repeated_group_names_resolve_by_index() {
    let captain =
        get_unit("watch-brothers", "Watch Captain Brother").expect("Watch Captain Brother loads");
    let same_name = captain
        .upgrade_groups
        .iter()
        .filter(|g| g.name == "Upgrade with one")
        .count();
    assert_eq!(same_name, 2, "expected two 'Upgrade with one' groups");

    let jetpack_name_only = apply_upgrades(
        &captain,
        &[UpgradeSelection {
            group: "Upgrade with one".to_string(),
            option: "Jetpack".to_string(),
            index: None,
        }],
    );
    assert!(
        jetpack_name_only.is_err(),
        "name-only lookup must not silently reach the second group"
    );

    let jetpack_by_index = apply_upgrades(
        &captain,
        &[UpgradeSelection {
            group: "Upgrade with one".to_string(),
            option: "Jetpack".to_string(),
            index: Some(7),
        }],
    )
    .expect("index-based lookup selects the second group");
    assert!(jetpack_by_index.has_rule(&SpecialRule::Ambush));
    assert!(jetpack_by_index.has_rule(&SpecialRule::Flying));
}
