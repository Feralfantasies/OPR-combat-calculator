//! Unit upgrade/loadout customization.
//!
//! Models the upgrade rules from the army books, e.g. Alien Hives:
//! "Upgrade with one ..." (pick-one rule), "Replace Shredder Cannon ..."
//! (weapon swap with costs), "Upgrade with ..." (optional add-ons).

use crate::models::rules::SpecialRule;
use crate::models::unit::Unit;
use crate::models::weapons::Weapon;

/// How an option changes the unit's weapons when applied.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WeaponChange {
    /// Remove (part of) the group's target weapon, add this one.
    Replace(Weapon),
    /// Add this weapon without removing anything.
    Add(Weapon),
}

/// A single selectable upgrade option.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpgradeOption {
    /// Display name, e.g. "Spitter Cannon" or "Combat Bio-Engineer"
    pub name: String,
    /// Human-readable summary of what the option grants
    pub description: String,
    /// Extra points cost on top of the base unit cost
    pub cost: u16,
    /// Weapon change, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon_change: Option<WeaponChange>,
    /// Special rules added to the unit when this option is taken
    pub add_rules: Vec<SpecialRule>,
}

impl UpgradeOption {
    /// Create a rule-only option (no weapon change).
    #[must_use]
    pub fn rule(name: &str, description: &str, cost: u16, rules: Vec<SpecialRule>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            cost,
            weapon_change: None,
            add_rules: rules,
        }
    }

    /// Create a weapon-replacement option.
    #[must_use]
    pub fn replace(name: &str, description: &str, cost: u16, weapon: Weapon) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            cost,
            weapon_change: Some(WeaponChange::Replace(weapon)),
            add_rules: Vec::new(),
        }
    }

    /// Create a weapon-addition option.
    #[must_use]
    pub fn add(name: &str, description: &str, cost: u16, weapon: Weapon) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            cost,
            weapon_change: Some(WeaponChange::Add(weapon)),
            add_rules: Vec::new(),
        }
    }
}

/// How selections within a group work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SelectionMode {
    /// Choose at most one option from the group (book: "Upgrade with one",
    /// "Replace X", "Replace all X").
    PickOne,
    /// Each option may be taken independently (book: "Upgrade with").
    Multiple,
}

/// How many instances of the target weapon a replacement removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReplaceCount {
    /// Replace one instance (book: "Replace any X"). The target weapon's
    /// quantity is decremented; the option's weapon is added.
    One,
    /// Replace all instances (book: "Replace X", "Replace all X"). The
    /// target weapon entry is removed; the option's weapon (with its own
    /// quantity) is added.
    All,
    /// Replace at most `max` instances (book: "Replace up to two X",
    /// "Replace 2x X"). If the unit carries no more than `max`, the entry
    /// is removed; otherwise the quantity is decremented by `max`. The
    /// option's weapon is added either way.
    UpTo { max: u8 },
}

/// A group of upgrade options with a shared constraint.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpgradeGroup {
    /// Display name, e.g. "Replace Shredder Cannon"
    pub name: String,
    /// Selection constraint
    pub mode: SelectionMode,
    /// Name of the weapon that replacement options swap out (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_weapon: Option<String>,
    /// Weapon the group's options attach to (e.g. a rifle attachment
    /// category). Selecting an option requires the unit to carry this
    /// weapon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_weapon: Option<String>,
    /// How many instances of the target weapon are replaced
    #[serde(default = "default_replace_count")]
    pub replace_count: ReplaceCount,
    /// Available options
    pub options: Vec<UpgradeOption>,
}

const fn default_replace_count() -> ReplaceCount {
    ReplaceCount::All
}

impl UpgradeGroup {
    /// Create a pick-one group of rule upgrades.
    #[must_use]
    pub fn pick_one(name: &str, options: Vec<UpgradeOption>) -> Self {
        Self {
            name: name.to_string(),
            mode: SelectionMode::PickOne,
            target_weapon: None,
            required_weapon: None,
            replace_count: ReplaceCount::All,
            options,
        }
    }

    /// Create a pick-one weapon replacement group that replaces one instance
    /// of the target weapon (book: "Replace any X").
    #[must_use]
    pub fn replace_one(name: &str, target_weapon: &str, options: Vec<UpgradeOption>) -> Self {
        Self {
            name: name.to_string(),
            mode: SelectionMode::PickOne,
            target_weapon: Some(target_weapon.to_string()),
            required_weapon: None,
            replace_count: ReplaceCount::One,
            options,
        }
    }

    /// Create a pick-one weapon replacement group that replaces all
    /// instances of the target weapon (book: "Replace X", "Replace all X").
    #[must_use]
    pub fn replace_all(name: &str, target_weapon: &str, options: Vec<UpgradeOption>) -> Self {
        Self {
            name: name.to_string(),
            mode: SelectionMode::PickOne,
            target_weapon: Some(target_weapon.to_string()),
            required_weapon: None,
            replace_count: ReplaceCount::All,
            options,
        }
    }

    /// Create a group where every option may be taken.
    #[must_use]
    pub fn multiple(name: &str, options: Vec<UpgradeOption>) -> Self {
        Self {
            name: name.to_string(),
            mode: SelectionMode::Multiple,
            target_weapon: None,
            required_weapon: None,
            replace_count: ReplaceCount::All,
            options,
        }
    }
}

/// A selection made by the user: one option from one group.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpgradeSelection {
    /// Group display name, e.g. "Replace Shredder Cannon". Multiple groups
    /// may share a name (army books repeat categories); disambiguate with
    /// `index`.
    pub group: String,
    /// Option display name within the group.
    pub option: String,
    /// Zero-based position of the group in the unit's `upgrade_groups`
    /// list. Takes precedence over name matching, so repeated group names
    /// resolve deterministically.
    #[serde(default)]
    pub index: Option<usize>,
    /// Zero-based position of the option within the resolved group's
    /// `options` list. Needed when one group carries the same option name
    /// more than once (e.g. two `Shishi Turret` variants with different
    /// rules and costs). Absent = first matching name.
    #[serde(default)]
    pub option_index: Option<usize>,
}

/// Apply a set of upgrade selections to a base unit, producing a new unit.
///
/// # Errors
/// Returns an error string if a group/option name is unknown, if more than
/// one option is selected from a `PickOne` group, or if a replacement
/// targets a weapon the unit does not have.
pub fn apply_upgrades(base: &Unit, selections: &[UpgradeSelection]) -> Result<Unit, String> {
    // Resolve each selection to a concrete group position up front so
    // repeated category names are addressed uniquely.
    let resolved: Vec<(usize, &UpgradeSelection)> = selections
        .iter()
        .map(|selection| {
            let pos = resolve_group(base, selection)?;
            Ok((pos, selection))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut unit = base.clone();

    for &(pos, selection) in &resolved {
        // The position was validated when building `resolved`, but this is
        // non-test code, so re-check instead of panicking.
        let Some(group) = unit.upgrade_groups.get(pos) else {
            return Err(format!("upgrade group index {pos} out of range"));
        };

        // Enforce pick-one per resolved group (identity, not display name).
        if group.mode == SelectionMode::PickOne {
            let count = resolved.iter().filter(|(p, _)| *p == pos).count();
            if count > 1 {
                return Err(format!("group '{}' allows only one selection", group.name));
            }
        }

        // Option name lookup honors an explicit position so groups that
        // carry the same option name twice stay individually selectable.
        let option = match selection.option_index {
            Some(pos) => group.options.get(pos).ok_or_else(|| {
                format!(
                    "upgrade option index {pos} out of range in group '{}'",
                    group.name
                )
            })?,
            None => group
                .options
                .iter()
                .find(|o| o.name == selection.option)
                .ok_or_else(|| {
                    format!(
                        "unknown option '{}' in group '{}'",
                        selection.option, group.name
                    )
                })?,
        };

        // Attachment groups require the parent weapon to be carried (the
        // book: "take one <weapon> attachment").
        if let Some(required) = &group.required_weapon {
            let has_parent = unit.weapons.iter().any(|w| w.name == required.as_str());
            if !has_parent {
                return Err(format!(
                    "option '{}' in group '{}' requires the {} weapon, which this unit does not have",
                    option.name, group.name, required
                ));
            }
        }

        unit.points = unit.points.saturating_add(option.cost);

        for rule in &option.add_rules {
            if !unit.special_rules.contains(rule) {
                unit.special_rules.push(rule.clone());
            }
        }

        if let Some(change) = &option.weapon_change {
            match change {
                WeaponChange::Replace(new_weapon) => {
                    // Clone everything borrowed from `unit` before the mutable
                    // calls below: while a reference into
                    // `unit.upgrade_groups` stays alive, NLL refuses `&mut unit`.
                    let target_name = group.target_weapon.clone();
                    let replace_count = group.replace_count;
                    let new_weapon = new_weapon.clone();
                    apply_replace_change(&mut unit, target_name, replace_count, new_weapon)?;
                }
                WeaponChange::Add(new_weapon) => {
                    unit.weapons.push(new_weapon.clone());
                }
            }
        }
    }

    Ok(unit)
}

/// Apply one replacement option's change: remove the `count`-specified
/// instances of every carried target part (a compound target, `"A and B"`,
/// drops each part independently), then add `new_weapon`.
fn apply_replace_change(
    unit: &mut Unit,
    target_name: Option<String>,
    count: ReplaceCount,
    new_weapon: Weapon,
) -> Result<(), String> {
    let Some(target) = target_name else {
        return Err("replacement option has no target weapon".to_string());
    };
    let removed_parts = remove_target_instances(unit, &target, count);
    if removed_parts == 0 {
        return Err(format!("unit has no weapon matching '{target}' to replace"));
    }
    unit.weapons.push(new_weapon);
    Ok(())
}

/// Remove this group's target instances from the unit for one replacement
/// option: every part of a compound target ("Replace A and B") that the
/// unit carries is dropped per `count`. Returns how many parts were removed
/// so the caller can reject selections with nothing to replace.
fn remove_target_instances(unit: &mut Unit, target: &str, count: ReplaceCount) -> usize {
    let mut removed = 0usize;
    for part in split_target_parts(target) {
        if let Some(pos_weapon) = unit
            .weapons
            .iter()
            .position(|w| weapon_name_matches_part(&w.name, &part))
        {
            drop_instances(unit, pos_weapon, count);
            removed = removed.saturating_add(1);
        }
    }
    removed
}

/// Remove the `count`-specified instances of the weapon at `pos_weapon`:
/// one, at most N (a carried quantity within the bound removes the entry
/// entirely), or all.
fn drop_instances(unit: &mut Unit, pos_weapon: usize, count: ReplaceCount) {
    match count {
        ReplaceCount::One => drop_up_to_instances(unit, pos_weapon, 1),
        ReplaceCount::UpTo { max } => drop_up_to_instances(unit, pos_weapon, max),
        ReplaceCount::All => {
            unit.weapons.remove(pos_weapon);
        }
    }
}

/// Remove up to `instances` copies of the weapon at `pos_weapon`, deleting
/// the whole entry when none remain.
fn drop_up_to_instances(unit: &mut Unit, pos_weapon: usize, instances: u8) {
    let Some(entry) = unit.weapons.get(pos_weapon) else {
        return;
    };
    let kept = entry.quantity.saturating_sub(instances);
    if kept == 0 {
        unit.weapons.remove(pos_weapon);
    } else if let Some(weapon) = unit.weapons.get_mut(pos_weapon) {
        weapon.quantity = kept;
    }
}

/// Resolve a selection to the zero-based position of its group in
/// group whose name matches is used (legacy behavior when names were
/// unique).
fn resolve_group(unit: &Unit, selection: &UpgradeSelection) -> Result<usize, String> {
    match selection.index {
        Some(idx) if idx < unit.upgrade_groups.len() => Ok(idx),
        Some(idx) => Err(format!("upgrade group index {idx} out of range")),
        None => unit
            .upgrade_groups
            .iter()
            .position(|g| g.name == selection.group)
            .ok_or_else(|| format!("unknown upgrade group: {}", selection.group)),
    }
}

/// Split a (possibly compound) replacement target into its weapon parts:
/// `"Combat Shields and CCWs"` -> `["Combat Shields", "CCWs"]`. Leading
/// quantifier tokens the loader normalization leaves behind (`Sgt.`,
/// `any/one/all`, `Nx`, `up to <n>`) are stripped per part.
fn split_target_parts(target: &str) -> Vec<String> {
    target
        .split(" and ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(strip_part_quantifiers)
        .collect()
}

/// Strip leading quantifier tokens from a single replacement target part,
/// consuming as much of the count qualifier as the grammar allows.
fn strip_part_quantifiers(part: &str) -> String {
    let words: Vec<&str> = part.split_whitespace().collect();
    let mut start = 0usize;
    while let Some(&word) = words.get(start) {
        let lower = word.to_ascii_lowercase();
        if matches!(lower.as_str(), "sgt." | "any" | "one" | "all") || is_count_token(word) {
            start = start.saturating_add(1);
        } else if lower == "up" && is_up_to_prefix(&words, start) {
            // `up to two/three X`: consume the whole 3-word qualifier.
            start = start.saturating_add(3);
        } else {
            break;
        }
    }
    part.split_whitespace()
        .skip(start)
        .collect::<Vec<_>>()
        .join(" ")
}

/// True when the words from `start` begin with `up to <one|two|three>`.
fn is_up_to_prefix(words: &[&str], start: usize) -> bool {
    let Some(next) = words.get(start.saturating_add(1)) else {
        return false;
    };
    let Some(count) = words.get(start.saturating_add(2)) else {
        return false;
    };
    next.eq_ignore_ascii_case("to")
        && matches!(count.to_ascii_lowercase().as_str(), "one" | "two" | "three")
}

/// Count-style tokens in replacement targets (`2x`, `3x`, ...).
fn is_count_token(word: &str) -> bool {
    let Some((head, tail)) = word.split_once('x') else {
        return false;
    };
    !head.is_empty() && head.chars().all(|c| c.is_ascii_digit()) && tail.is_empty()
}

/// Lenient weapon-name match: case-insensitive with single trailing-`s`
/// plural tolerance on either side (e.g. `Combat Shield` vs
/// `Combat Shields`, `Storm Rifle` vs `Storm Rifles`).
fn weapon_name_matches_part(weapon: &str, part: &str) -> bool {
    let w = weapon.to_ascii_lowercase();
    let t = strip_part_quantifiers(part).to_ascii_lowercase();
    plural_tolerant_match(&w, &t)
}

/// Equal, or equal after adding/removing one trailing `s` to either side.
fn plural_tolerant_match(a: &str, b: &str) -> bool {
    if a == b || a == b.trim_end_matches('s') || b == a.trim_end_matches('s') {
        return true;
    }
    // Re-add the `s` for the `trim_end_matches` comparison direction:
    let a_s = format!("{a}s");
    let b_s = format!("{b}s");
    a == b_s || b == a_s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_unit() -> Unit {
        Unit::new("Hive Lord", 1, 3, 2)
            .with_points(360)
            .with_weapon(
                Weapon::ranged("Shredder Cannon", 1, 4, 18).with_rule(SpecialRule::Rending),
            )
    }

    fn test_group() -> UpgradeGroup {
        UpgradeGroup::replace_all(
            "Replace Shredder Cannon",
            "Shredder Cannon",
            vec![UpgradeOption::replace(
                "Spitter Cannon",
                "24\", A2, Blast(3)",
                5,
                Weapon::ranged("Spitter Cannon", 1, 2, 24).with_rule(SpecialRule::Blast(3)),
            )],
        )
    }

    #[test]
    fn replace_all_removes_target_and_adds_cost() {
        let mut unit = test_unit();
        unit.upgrade_groups = vec![test_group()];

        let upgraded = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Replace Shredder Cannon".to_string(),
                option: "Spitter Cannon".to_string(),
                index: None,
                option_index: None,
            }],
        );
        let upgraded = upgraded.expect("upgrade should apply");
        assert_eq!(upgraded.points, 365);
        assert_eq!(upgraded.weapons.len(), 1);
        let weapon = upgraded.weapons.first().expect("weapon present");
        assert_eq!(weapon.name, "Spitter Cannon");
    }

    #[test]
    fn replace_one_decrements_quantity() {
        let mut unit = Unit::new("Hive Lord", 1, 3, 2)
            .with_points(360)
            .with_weapon(Weapon::melee("Heavy Razor Claws", 2, 3).with_rule(SpecialRule::AP(1)));
        unit.upgrade_groups = vec![UpgradeGroup::replace_one(
            "Replace any Heavy Razor Claw",
            "Heavy Razor Claws",
            vec![UpgradeOption::replace(
                "Smashing Club",
                "A1, AP(2), Blast(3)",
                0,
                Weapon::melee("Smashing Club", 1, 1)
                    .with_rule(SpecialRule::AP(2))
                    .with_rule(SpecialRule::Blast(3)),
            )],
        )];

        let upgraded = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Replace any Heavy Razor Claw".to_string(),
                option: "Smashing Club".to_string(),
                index: None,
                option_index: None,
            }],
        );
        let upgraded = upgraded.expect("upgrade should apply");
        assert_eq!(upgraded.weapons.len(), 2);
        let claws = upgraded
            .weapons
            .iter()
            .find(|w| w.name == "Heavy Razor Claws")
            .expect("claws remain");
        assert_eq!(claws.quantity, 1);
        assert!(upgraded.weapons.iter().any(|w| w.name == "Smashing Club"));
    }

    #[test]
    fn replace_target_parsing_is_case_and_plural_tolerant() {
        // Quantifier stripping: the loader leaves `Sgt.`/`up to two`/count
        // tokens in some targets; they must not block resolution.
        assert_eq!(
            strip_part_quantifiers("Sgt. Heavy Pistols"),
            "Heavy Pistols"
        );
        assert_eq!(strip_part_quantifiers("any CCW"), "CCW");
        assert_eq!(
            strip_part_quantifiers("up to three Heavy Rifles"),
            "Heavy Rifles"
        );
        assert_eq!(
            strip_part_quantifiers("3x Heavy Razor Claws"),
            "Heavy Razor Claws"
        );
        assert_eq!(strip_part_quantifiers("all CCWs"), "CCWs");

        // Compound targets split on ` and ` per part.
        assert_eq!(
            split_target_parts("Combat Shields and CCWs"),
            vec!["Combat Shields", "CCWs"]
        );

        // Name matching tolerates case plus a single trailing `s` either way.
        assert!(weapon_name_matches_part("Heavy Pistols", "Heavy Pistol"));
        assert!(weapon_name_matches_part("Heavy Pistol", "Heavy Pistols"));
        assert!(weapon_name_matches_part("CCWs", "ccw"));
        assert!(!weapon_name_matches_part(
            "Storm Rifle",
            "Storm Rifles and More"
        ));
    }

    #[test]
    fn replace_one_compound_target_removes_each_resolvable_part() {
        // `Replace Combat Shield and CCW` (PickOne): each part is removed
        // independently when both are carried; the option adds its weapon.
        let mut unit = Unit::new("Master", 1, 3, 3).with_points(100);
        for w in [
            Weapon::melee("Combat Shield", 1, 1),
            Weapon::melee("CCW", 1, 2),
        ] {
            unit = unit.with_weapon(w);
        }
        let group = UpgradeGroup::replace_one(
            "Replace Combat Shield and CCW",
            "Combat Shield and CCW",
            vec![UpgradeOption::replace(
                "Master Grave Heavy Pistol, CCW",
                "12\", A4, AP(1), A4",
                10,
                Weapon::melee("Energy Flail", 1, 3).with_rule(SpecialRule::AP(1)),
            )],
        );
        unit.upgrade_groups = vec![group];

        let upgraded = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Replace Combat Shield and CCW".to_string(),
                option: "Master Grave Heavy Pistol, CCW".to_string(),
                index: None,
                option_index: None,
            }],
        )
        .expect("compound replacement applies");

        assert!(upgraded.weapons.iter().any(|w| w.name == "Energy Flail"));
        let shield_remaining = upgraded
            .weapons
            .iter()
            .filter(|w| w.name == "Combat Shield")
            .count();
        let ccw_remaining = upgraded.weapons.iter().filter(|w| w.name == "CCW").count();
        assert_eq!(shield_remaining, 0, "first carried Combat Shield removed");
        assert_eq!(ccw_remaining, 0, "first carried CCW removed");

        // A part the unit no longer carries is skipped, not fatal:
        // a second selection on the same single-model unit still works as
        // long as at least one resolvable target part remains.
        let mut solo = Unit::new("Solo", 1, 3, 3)
            .with_points(50)
            .with_weapon(Weapon::melee("CCW", 1, 2));
        solo.upgrade_groups.push(UpgradeGroup::replace_one(
            "Replace Combat Shield and CCW",
            "Combat Shield and CCW",
            vec![UpgradeOption::replace(
                "Replacement Claw",
                "A3",
                5,
                Weapon::melee("Replacement Claw", 1, 3),
            )],
        ));
        let upgraded = apply_upgrades(
            &solo,
            &[UpgradeSelection {
                group: "Replace Combat Shield and CCW".to_string(),
                option: "Replacement Claw".to_string(),
                index: None,
                option_index: None,
            }],
        )
        .expect("missing part skipped, carried part replaced");
        assert!(!upgraded.weapons.iter().any(|w| w.name == "CCW"));
        assert!(
            upgraded
                .weapons
                .iter()
                .any(|w| w.name == "Replacement Claw")
        );

        // No resolvable part at all is still an error.
        let mut bare = Unit::new("Bare", 1, 3, 3).with_points(50);
        bare.upgrade_groups.push(UpgradeGroup::replace_one(
            "Replace Combat Shield and CCW",
            "Combat Shield and CCW",
            vec![UpgradeOption::replace(
                "Replacement Claw",
                "A3",
                5,
                Weapon::melee("Replacement Claw", 1, 3),
            )],
        ));
        let err = apply_upgrades(
            &bare,
            &[UpgradeSelection {
                group: "Replace Combat Shield and CCW".to_string(),
                option: "Replacement Claw".to_string(),
                index: None,
                option_index: None,
            }],
        )
        .expect_err("nothing to replace is an error");
        assert!(err.contains("no weapon matching"), "error was: {err}");
    }

    #[test]
    fn pick_one_rejects_two_selections() {
        let mut unit = test_unit();
        unit.upgrade_groups = vec![test_group()];

        let result = apply_upgrades(
            &unit,
            &[
                UpgradeSelection {
                    group: "Replace Shredder Cannon".to_string(),
                    option: "Spitter Cannon".to_string(),
                    index: None,
                    option_index: None,
                },
                UpgradeSelection {
                    group: "Replace Shredder Cannon".to_string(),
                    option: "Spitter Cannon".to_string(),
                    index: None,
                    option_index: None,
                },
            ],
        );
        assert!(result.is_err());
    }

    #[test]
    fn unknown_group_is_an_error() {
        let unit = test_unit();
        let result = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Nope".to_string(),
                option: "X".to_string(),
                index: None,
                option_index: None,
            }],
        );
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_option_names_resolve_by_option_index() {
        // A category carrying the same option name twice (the committed
        // `Replace one Long Rifles` group has two `Shishi Turret` variants
        // with different rules/costs): only the position selects the
        // second one; name-only lookup deterministically hits the first.
        let mut unit = test_unit();
        unit.upgrade_groups = vec![UpgradeGroup::replace_all(
            "Replace one Long Rifles",
            "Long Rifle",
            vec![
                UpgradeOption::rule("Shishi Turret", "Guns", 10, vec![SpecialRule::Caster(2)]),
                UpgradeOption::rule(
                    "Shishi Turret",
                    "Missiles",
                    20,
                    vec![SpecialRule::Caster(3)],
                ),
            ],
        )];

        // Name-only: always the first variant (index 0).
        let first = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Replace one Long Rifles".to_string(),
                option: "Shishi Turret".to_string(),
                index: None,
                option_index: None,
            }],
        )
        .expect("first duplicate resolves by name");
        assert!(first.has_rule(&SpecialRule::Caster(2)));
        assert!(!first.has_rule(&SpecialRule::Caster(3)));

        // option_index = 1 selects the second variant.
        let second = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Replace one Long Rifles".to_string(),
                option: "Shishi Turret".to_string(),
                index: None,
                option_index: Some(1),
            }],
        )
        .expect("second duplicate resolves by option_index");
        assert!(second.has_rule(&SpecialRule::Caster(3)));
        assert_eq!(second.points, test_unit().points + 20);

        // Out-of-range option index is an error.
        let bad = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Replace one Long Rifles".to_string(),
                option: "Shishi Turret".to_string(),
                index: None,
                option_index: Some(2),
            }],
        );
        assert!(bad.is_err());
    }

    #[test]
    fn attachment_groups_require_parent_weapon() {
        // `Take one Heavy Rifle attachment` requires the Heavy Rifle.
        let make_attach = || UpgradeGroup {
            name: "Take one Heavy Rifle attachment".to_string(),
            mode: SelectionMode::Multiple,
            target_weapon: None,
            required_weapon: Some("Heavy Rifle".to_string()),
            replace_count: ReplaceCount::All,
            options: vec![UpgradeOption::rule(
                "Targeting Array",
                "Unstoppable Mark",
                25,
                vec![SpecialRule::Unstoppable],
            )],
        };
        let selection = || UpgradeSelection {
            group: "Take one Heavy Rifle attachment".to_string(),
            option: "Targeting Array".to_string(),
            index: None,
            option_index: None,
        };

        // Without the parent weapon: rejected with a descriptive error.
        let mut no_rifle = test_unit();
        no_rifle.upgrade_groups = vec![make_attach()];
        assert!(
            apply_upgrades(&no_rifle, &[selection()]).is_err(),
            "attachment without parent must fail"
        );

        // With the parent weapon: applies cleanly.
        let mut with_rifle = test_unit().with_weapon(Weapon::ranged("Heavy Rifle", 1, 1, 24));
        with_rifle.upgrade_groups = vec![make_attach()];
        let upgraded = apply_upgrades(&with_rifle, &[selection()]).expect("attachment applies");
        assert!(upgraded.has_rule(&SpecialRule::Unstoppable));
    }

    #[test]
    fn duplicate_group_names_resolve_by_index() {
        // Two groups share the display name "Upgrade with one" (as the
        // committed catalogs do); only the index disambiguates.
        let mut unit = test_unit();
        unit.upgrade_groups = vec![
            UpgradeGroup::pick_one(
                "Upgrade with one",
                vec![UpgradeOption::rule(
                    "Bio-Tech Master",
                    "Reliable",
                    5,
                    vec![SpecialRule::Reliable],
                )],
            ),
            UpgradeGroup::pick_one(
                "Upgrade with one",
                vec![UpgradeOption::rule(
                    "Combat Bio-Engineer",
                    "Furious",
                    5,
                    vec![SpecialRule::Furious],
                )],
            ),
        ];

        // Name lookup (index None) resolves the first group.
        let upgraded = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Upgrade with one".to_string(),
                option: "Bio-Tech Master".to_string(),
                index: None,
                option_index: None,
            }],
        );
        let upgraded = upgraded.expect("first group resolves by name");
        assert!(upgraded.has_rule(&SpecialRule::Reliable));

        // Index 1 picks the second group despite the identical name.
        let upgraded = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Upgrade with one".to_string(),
                option: "Combat Bio-Engineer".to_string(),
                index: Some(1),
                option_index: None,
            }],
        );
        let upgraded = upgraded.expect("second group resolves by index");
        assert!(upgraded.has_rule(&SpecialRule::Furious));
        assert!(!upgraded.has_rule(&SpecialRule::Reliable));
    }

    #[test]
    fn up_to_bound_caps_removal_at_max() {
        // A unit carrying three CCWs under a "Replace up to two CCW" group
        // keeps one; the replacement weapon is still added.
        let mut unit = test_unit();
        unit.weapons = vec![Weapon::ranged("CCW", 3, 2, 18)];
        let mut group = UpgradeGroup::replace_all(
            "Replace up to two CCW",
            "CCW",
            vec![UpgradeOption::replace(
                "Flamer Pistol",
                "12\", A6, AP(4)",
                5,
                Weapon::ranged("Flamer Pistol", 2, 6, 12).with_rule(SpecialRule::AP(4)),
            )],
        );
        group.replace_count = ReplaceCount::UpTo { max: 2 };
        unit.upgrade_groups = vec![group];

        let upgraded = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Replace up to two CCW".to_string(),
                option: "Flamer Pistol".to_string(),
                index: None,
                option_index: None,
            }],
        )
        .expect("up-to bound applies");
        assert_eq!(
            upgraded.weapons.len(),
            2,
            "both weapons remain: {:?}",
            upgraded.weapons
        );
        let ccw = upgraded
            .weapons
            .iter()
            .find(|weapon| weapon.name == "CCW")
            .expect("one CCW pair survives");
        assert_eq!(ccw.quantity, 1);
    }

    #[test]
    fn up_to_bound_removes_all_when_carried_le_max() {
        // Carrying exactly `max` (or fewer) removes the entry entirely.
        for carried in [1u8, 3u8] {
            let mut unit = test_unit();
            unit.weapons = vec![Weapon::ranged("CCW", carried.min(2), 2, 18)];
            let mut group = UpgradeGroup::replace_all(
                "Replace up to two CCW",
                "CCW",
                vec![UpgradeOption::replace(
                    "Flamer Pistol",
                    "12\", A6, AP(4)",
                    5,
                    Weapon::ranged("Flamer Pistol", 2, 6, 12).with_rule(SpecialRule::AP(4)),
                )],
            );
            group.replace_count = ReplaceCount::UpTo { max: 2 };
            unit.upgrade_groups = vec![group];

            let upgraded = apply_upgrades(
                &unit,
                &[UpgradeSelection {
                    group: "Replace up to two CCW".to_string(),
                    option: "Flamer Pistol".to_string(),
                    index: None,
                    option_index: None,
                }],
            )
            .expect("up-to bound applies");
            assert!(
                !upgraded.weapons.iter().any(|w| w.name == "CCW"),
                "all {carried} carried CCWs must be removed, got: {:?}",
                upgraded.weapons
            );
        }
    }

    #[test]
    fn out_of_range_index_is_an_error() {
        let unit = test_unit();
        let result = apply_upgrades(
            &unit,
            &[UpgradeSelection {
                group: "Replace Shredder Cannon".to_string(),
                option: "Spitter Cannon".to_string(),
                index: Some(99),
                option_index: None,
            }],
        );
        assert!(result.is_err());
    }
}
