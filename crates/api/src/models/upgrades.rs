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
    /// Replace all instances (book: "Replace X", "Replace all X",
    /// "Replace 3x X"). The target weapon entry is removed; the option's
    /// weapon (with its own quantity) is added.
    All,
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

        let option = group
            .options
            .iter()
            .find(|o| o.name == selection.option)
            .ok_or_else(|| {
                format!(
                    "unknown option '{}' in group '{}'",
                    selection.option, group.name
                )
            })?;

        unit.points = unit.points.saturating_add(option.cost);

        for rule in &option.add_rules {
            if !unit.special_rules.contains(rule) {
                unit.special_rules.push(rule.clone());
            }
        }

        if let Some(change) = &option.weapon_change {
            match change {
                WeaponChange::Replace(new_weapon) => {
                    let target = group.target_weapon.as_deref().ok_or_else(|| {
                        format!("replacement option '{}' has no target weapon", option.name)
                    })?;
                    let pos_weapon = unit
                        .weapons
                        .iter()
                        .position(|w| w.name == target)
                        .ok_or_else(|| format!("unit has no weapon '{target}' to replace"))?;

                    match group.replace_count {
                        ReplaceCount::One => {
                            // Decrement the target's quantity; remove if it hits 0.
                            let orig_qty = unit.weapons.get(pos_weapon).map_or(1, |w| w.quantity);
                            if orig_qty <= 1 {
                                unit.weapons.remove(pos_weapon);
                            } else if let Some(w) = unit.weapons.get_mut(pos_weapon) {
                                w.quantity = orig_qty.saturating_sub(1);
                            }
                            unit.weapons.push(new_weapon.clone());
                        }
                        ReplaceCount::All => {
                            unit.weapons.remove(pos_weapon);
                            unit.weapons.push(new_weapon.clone());
                        }
                    }
                }
                WeaponChange::Add(new_weapon) => {
                    unit.weapons.push(new_weapon.clone());
                }
            }
        }
    }

    Ok(unit)
}

/// Resolve a selection to the zero-based position of its group in
/// `unit.upgrade_groups`. An explicit `index` wins; otherwise the first
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
                },
                UpgradeSelection {
                    group: "Replace Shredder Cannon".to_string(),
                    option: "Spitter Cannon".to_string(),
                    index: None,
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
            }],
        );
        assert!(result.is_err());
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
            }],
        );
        let upgraded = upgraded.expect("second group resolves by index");
        assert!(upgraded.has_rule(&SpecialRule::Furious));
        assert!(!upgraded.has_rule(&SpecialRule::Reliable));
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
            }],
        );
        assert!(result.is_err());
    }
}
