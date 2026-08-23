/// The type of attack being made
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AttackType {
    /// Ranged attack (shooting) - one-way damage
    Ranged,
    /// Melee attack (charge) - defender can strike back
    MeleeCharge,
    /// Melee attack (return strikes) - defender fighting back
    MeleeReturn,
}

/// The Versatile Attack effect a unit picked when it activated (v3.5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VersatileMode {
    /// AP(+1) when shooting or charging enemies over 9".
    Ap1,
    /// +1 to hit rolls when shooting or charging enemies over 9".
    HitBonus,
}

impl VersatileMode {
    /// Parse the `versatile_mode` field of [`CombatContext`]
    /// (0 = AP(+1), 1 = hit bonus).
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Ap1),
            1 => Some(Self::HitBonus),
            _ => None,
        }
    }
}

/// Serde default for [`CombatContext::versatile_mode`]: `2`, the
/// no-effect pick. The implicit `u8::default()` of `0` would grant
/// AP(+1) to a deserialized context that never made an activation choice.
const fn default_versatile_mode() -> u8 {
    2
}

/// Contextual information about the combat that affects special rules.
/// This captures the "state" of the attack for rule resolution.
///
/// The boolean fields are independent combat-state flags (charging, moved,
/// cover, fatigue) rather than a tangled set of parameters, so the
/// excessive-bools lint does not apply.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CombatContext {
    /// Type of attack (ranged vs melee)
    pub attack_type: AttackType,
    /// Distance between attacker and defender in inches
    pub distance: u8,
    /// Whether the attacker is charging (relevant for Furious, Thrust, Impact)
    pub is_charging: bool,
    /// Whether the attacker moved before shooting (relevant for Indirect)
    pub attacker_moved: bool,
    /// Whether the defender is in cover (+1 Defense vs shooting)
    pub defender_in_cover: bool,
    /// Whether the attacker is fatigued (hitting on unmodified 6s only)
    pub attacker_fatigued: bool,
    /// Versatile Attack effect picked at activation (0 = AP(+1),
    /// 1 = +1 to hit, 2 = none; see [`VersatileMode`]).
    #[serde(default = "default_versatile_mode")]
    pub versatile_mode: u8,
}

impl CombatContext {
    /// Create a default ranged combat context
    #[must_use]
    pub const fn ranged(distance: u8) -> Self {
        Self {
            attack_type: AttackType::Ranged,
            distance,
            is_charging: false,
            attacker_moved: false,
            defender_in_cover: false,
            attacker_fatigued: false,
            versatile_mode: 2,
        }
    }

    /// Create a default melee charge context
    #[must_use]
    pub const fn melee_charge() -> Self {
        Self {
            attack_type: AttackType::MeleeCharge,
            distance: 0,
            is_charging: true,
            attacker_moved: false,
            defender_in_cover: false,
            attacker_fatigued: false,
            versatile_mode: 2,
        }
    }

    /// Create a melee return strikes context
    #[must_use]
    pub const fn melee_return(fatigued: bool) -> Self {
        Self {
            attack_type: AttackType::MeleeReturn,
            distance: 0,
            is_charging: false,
            attacker_moved: false,
            defender_in_cover: false,
            attacker_fatigued: fatigued,
            versatile_mode: 2,
        }
    }

    /// Check if this is a melee attack
    #[must_use]
    pub const fn is_melee(&self) -> bool {
        matches!(
            self.attack_type,
            AttackType::MeleeCharge | AttackType::MeleeReturn
        )
    }

    /// Check if this is a ranged attack
    #[must_use]
    pub fn is_ranged(&self) -> bool {
        self.attack_type == AttackType::Ranged
    }

    /// Check if distance is over 9" (relevant for Stealth, Relentless, Artillery)
    #[must_use]
    pub const fn is_long_range(&self) -> bool {
        self.distance > 9
    }

    /// Set whether attacker moved (builder pattern)
    #[must_use]
    pub const fn with_moved(mut self, moved: bool) -> Self {
        self.attacker_moved = moved;
        self
    }

    /// Set whether defender is in cover (builder pattern)
    #[must_use]
    pub const fn with_cover(mut self, in_cover: bool) -> Self {
        self.defender_in_cover = in_cover;
        self
    }

    /// Set the distance between attacker and defender (builder pattern)
    #[must_use]
    pub const fn with_distance(mut self, distance: u8) -> Self {
        self.distance = distance;
        self
    }

    /// Set whether attacker is fatigued (builder pattern)
    #[must_use]
    pub const fn with_fatigue(mut self, fatigued: bool) -> Self {
        self.attacker_fatigued = fatigued;
        self
    }

    /// Set the Versatile Attack effect picked at activation
    /// (0 = AP(+1), 1 = +1 to hit, 2 = none).
    #[must_use]
    pub const fn with_versatile_mode(mut self, mode: u8) -> Self {
        self.versatile_mode = mode;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An omitted `versatile_mode` must deserialize to the no-effect value
    /// (`2`). The implicit `u8::default()` of `0` would select AP(+1) for a
    /// context that never made an activation pick.
    #[test]
    fn omitted_versatile_mode_defaults_to_no_effect() {
        let json = r#"{"attack_type":"Ranged","distance":12,"is_charging":false,"attacker_moved":false,"defender_in_cover":false,"attacker_fatigued":false}"#;
        let context: CombatContext = serde_json::from_str(json).expect("context deserializes");
        assert_eq!(context.versatile_mode, 2);
        assert_eq!(VersatileMode::from_u8(context.versatile_mode), None);

        // An explicit pick round-trips unchanged.
        let with_pick = serde_json::to_string(&context.with_versatile_mode(0)).expect("serializes");
        let round: CombatContext = serde_json::from_str(&with_pick).expect("round-trips");
        assert_eq!(round.versatile_mode, 0);
    }
}
