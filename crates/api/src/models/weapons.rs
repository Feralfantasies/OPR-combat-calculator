use crate::models::rules::SpecialRule;

/// A weapon in OPR Grimdark Future.
/// Range is None for melee weapons, Some(x) for ranged weapons.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Weapon {
    pub name: String,
    /// How many models in the unit have this weapon
    pub quantity: u8,
    /// Attacks per model wielding this weapon
    pub attacks: u8,
    /// None = melee weapon, Some(x) = ranged weapon with x" range
    pub range: Option<u8>,
    /// Special rules attached to this weapon (e.g. AP, Rending, Blast)
    pub special_rules: Vec<SpecialRule>,
}

impl Weapon {
    /// Create a new melee weapon
    #[must_use]
    pub fn melee(name: &str, quantity: u8, attacks: u8) -> Self {
        Self {
            name: name.to_string(),
            quantity,
            attacks,
            range: None,
            special_rules: Vec::new(),
        }
    }

    /// Create a new ranged weapon
    #[must_use]
    pub fn ranged(name: &str, quantity: u8, attacks: u8, range: u8) -> Self {
        Self {
            name: name.to_string(),
            quantity,
            attacks,
            range: Some(range),
            special_rules: Vec::new(),
        }
    }

    /// Add a special rule to this weapon (builder pattern)
    #[must_use]
    pub fn with_rule(mut self, rule: SpecialRule) -> Self {
        self.special_rules.push(rule);
        self
    }

    /// Check if this weapon is a melee weapon
    #[must_use]
    pub const fn is_melee(&self) -> bool {
        self.range.is_none()
    }

    /// Check if this weapon is a ranged weapon
    #[must_use]
    pub const fn is_ranged(&self) -> bool {
        self.range.is_some()
    }

    /// Get the total number of attacks from this weapon
    /// (attacks per model * number of models with the weapon)
    #[must_use]
    pub const fn total_attacks(&self) -> u8 {
        self.attacks.saturating_mul(self.quantity)
    }

    /// Get the AP value from this weapon's special rules, if any
    #[must_use]
    pub fn get_ap(&self) -> Option<u8> {
        self.special_rules.iter().find_map(SpecialRule::ap_value)
    }
}
