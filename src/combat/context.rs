/// The type of attack being made
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    /// Ranged attack (shooting) - one-way damage
    Ranged,
    /// Melee attack (charge) - defender can strike back
    MeleeCharge,
    /// Melee attack (return strikes) - defender fighting back
    MeleeReturn,
}

/// Contextual information about the combat that affects special rules.
/// This captures the "state" of the attack for rule resolution.
#[derive(Debug, Clone)]
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
    /// Whether the attacker is fatigued (only hit on unmodified 6)
    pub attacker_fatigued: bool,
}

impl CombatContext {
    /// Create a default ranged combat context
    pub fn ranged(distance: u8) -> Self {
        Self {
            attack_type: AttackType::Ranged,
            distance,
            is_charging: false,
            attacker_moved: false,
            defender_in_cover: false,
            attacker_fatigued: false,
        }
    }

    /// Create a default melee charge context
    pub fn melee_charge() -> Self {
        Self {
            attack_type: AttackType::MeleeCharge,
            distance: 0,
            is_charging: true,
            attacker_moved: false,
            defender_in_cover: false,
            attacker_fatigued: false,
        }
    }

    /// Create a melee return strikes context
    pub fn melee_return(fatigued: bool) -> Self {
        Self {
            attack_type: AttackType::MeleeReturn,
            distance: 0,
            is_charging: false,
            attacker_moved: false,
            defender_in_cover: false,
            attacker_fatigued: fatigued,
        }
    }

    /// Check if this is a melee attack
    pub fn is_melee(&self) -> bool {
        matches!(
            self.attack_type,
            AttackType::MeleeCharge | AttackType::MeleeReturn
        )
    }

    /// Check if this is a ranged attack
    pub fn is_ranged(&self) -> bool {
        self.attack_type == AttackType::Ranged
    }

    /// Check if distance is over 9" (relevant for Stealth, Relentless, Artillery)
    pub fn is_long_range(&self) -> bool {
        self.distance > 9
    }

    /// Set whether attacker moved (builder pattern)
    pub fn with_moved(mut self, moved: bool) -> Self {
        self.attacker_moved = moved;
        self
    }

    /// Set whether defender is in cover (builder pattern)
    pub fn with_cover(mut self, in_cover: bool) -> Self {
        self.defender_in_cover = in_cover;
        self
    }

    /// Set whether attacker is fatigued (builder pattern)
    pub fn with_fatigue(mut self, fatigued: bool) -> Self {
        self.attacker_fatigued = fatigued;
        self
    }
}