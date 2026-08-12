use crate::models::rules::SpecialRule;
use crate::models::upgrades::UpgradeGroup;
use crate::models::weapons::Weapon;

/// A unit in OPR Grimdark Future.
/// A unit is a group of one or more models that acts together.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Unit {
    pub name: String,
    /// Number of models in the unit
    pub quantity: u8,
    /// Points cost of the unit
    pub points: u16,
    /// Optional leader/hero name attached to the unit
    pub leader: Option<String>,
    /// Quality value - roll this or higher to hit / pass morale (e.g. 3 means 3+)
    pub quality: u8,
    /// Defense value - roll this or higher to block hits (e.g. 4 means 4+)
    pub defense: u8,
    /// Wounds needed to kill a single model (defaults to 1, overridden by Tough)
    pub tough: u8,
    /// All weapons carried by the unit
    pub weapons: Vec<Weapon>,
    /// Unit-level special rules (e.g. Stealth, Regeneration, Fearless)
    pub special_rules: Vec<SpecialRule>,
    /// Loadout customization options from the army book
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upgrade_groups: Vec<UpgradeGroup>,
}

impl Unit {
    /// Create a new unit with base stats
    #[must_use]
    pub fn new(name: &str, quantity: u8, quality: u8, defense: u8) -> Self {
        Self {
            name: name.to_string(),
            quantity,
            points: 0,
            leader: None,
            quality,
            defense,
            tough: 1,
            weapons: Vec::new(),
            special_rules: Vec::new(),
            upgrade_groups: Vec::new(),
        }
    }

    /// Add an upgrade group to the unit (builder pattern)
    #[must_use]
    pub fn with_upgrades(mut self, group: UpgradeGroup) -> Self {
        self.upgrade_groups.push(group);
        self
    }

    /// Add a weapon to the unit (builder pattern)
    #[must_use]
    pub fn with_weapon(mut self, weapon: Weapon) -> Self {
        self.weapons.push(weapon);
        self
    }

    /// Add a special rule to the unit (builder pattern)
    #[must_use]
    pub fn with_rule(mut self, rule: SpecialRule) -> Self {
        self.special_rules.push(rule);
        self
    }

    /// Set the points cost (builder pattern)
    #[must_use]
    pub const fn with_points(mut self, points: u16) -> Self {
        self.points = points;
        self
    }

    /// Set the leader (builder pattern)
    #[must_use]
    pub fn with_leader(mut self, leader: &str) -> Self {
        self.leader = Some(leader.to_string());
        self
    }

    /// Check if the unit has a specific special rule
    #[must_use]
    pub fn has_rule(&self, rule: &SpecialRule) -> bool {
        self.special_rules.contains(rule)
    }

    /// Get the effective tough value for models in this unit.
    /// Checks for Tough(X) in special rules, otherwise returns base tough.
    #[must_use]
    pub fn effective_tough(&self) -> u8 {
        self.special_rules
            .iter()
            .find_map(SpecialRule::tough_value)
            .unwrap_or(self.tough)
    }

    /// Check if the unit has regeneration
    #[must_use]
    pub fn has_regeneration(&self) -> bool {
        self.has_rule(&SpecialRule::Regeneration)
    }

    /// Get all melee weapons
    #[must_use]
    pub fn melee_weapons(&self) -> Vec<&Weapon> {
        self.weapons.iter().filter(|w| w.is_melee()).collect()
    }

    /// Get all ranged weapons
    #[must_use]
    pub fn ranged_weapons(&self) -> Vec<&Weapon> {
        self.weapons.iter().filter(|w| w.is_ranged()).collect()
    }

    /// Get total melee attacks from all melee weapons
    #[must_use]
    pub fn total_melee_attacks(&self) -> u8 {
        self.melee_weapons()
            .iter()
            .fold(0u8, |acc, w| acc.saturating_add(w.total_attacks()))
    }

    /// Get total ranged attacks from all ranged weapons
    #[must_use]
    pub fn total_ranged_attacks(&self) -> u8 {
        self.ranged_weapons()
            .iter()
            .fold(0u8, |acc, w| acc.saturating_add(w.total_attacks()))
    }
}
