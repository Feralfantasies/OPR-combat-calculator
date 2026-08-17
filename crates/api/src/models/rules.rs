/// All special rules from OPR Grimdark Future v3.5.1
/// Rules with (X) parameters carry their value as an associated u8.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpecialRule {
    // Parameterized rules - the value inside is the (X)
    AP(u8),        // Target gets -X to Defense rolls
    Blast(u8),     // Each hit multiplied by X (capped at target models)
    Deadly(u8),    // Each wound multiplied by X
    Fear(u8),      // +X wounds for melee resolution
    Impact(u8),    // Roll X dice on charge, 2+ = hit
    Tough(u8),     // Model takes X wounds to die
    Caster(u8),    // X spell tokens per round
    Transport(u8), // Transport capacity

    // Simple flag rules (no parameters)
    Bane,         // Ignores Regen, reroll defense 6s
    Counter,      // Strikes first when charged
    Fast,         // +2" Advance, +4" Rush/Charge
    Fearless,     // 4+ to pass failed morale
    Flying,       // Move through units/terrain
    Furious,      // Unmodified 6 to hit in melee (charge) = +1 hit
    Immobile,     // Hold only
    Indirect,     // -1 hit after move, ignores LoS requirement
    Limited,      // Once per game
    Regeneration, // 5+ to ignore wounds
    Relentless,   // Unmodified 6 to hit vs >9" ranged = +1 hit
    Reliable,     // Attack at Quality 2+
    Rending,      // Ignores Regen, 6 to hit = AP(+4)
    Scout,
    Slow,
    Stealth, // -1 to enemy hit rolls from >9"
    Strider,
    Surge, // Unmodified 6 to hit = +1 hit
    Takedown,
    Thrust,      // +1 to hit and AP(+1) when charging
    Unstoppable, // Ignores Regen, ignores negative modifiers
    Aircraft,
    Ambush,
    Artillery,
    Hero,

    // ---- Alien Hives faction rules ----
    HiveBond,                 // +1 to morale test rolls (unit-wide)
    BreathAttack,             // Once/activation: 2+ = 1 hit Blast(3) AP(1) within 6"
    CasterGroup,              // One model becomes Caster(X), X = models with rule
    Destructive,              // Unmodified 6 to hit gains AP(+4)
    Fortified,                // Incoming hits count as AP(-1), min AP(0)
    FuriousBuff,              // Grant Furious to a friendly unit once
    HitAndRunFighter,         // Move 3" after being in melee once/round
    HiveBondBoost,            // Hive Bond gives +2 morale instead of +1
    IncreasedShootingRange,   // +6" range when shooting
    Infiltrate,               // Ambush, but deploy 3"+ from enemies
    NoRetreat,                // Failed Shaken/Rout morale counts as passed, then risk wounds
    PiercingGrowth,           // Markers grant AP(+1) per 2 markers
    PiercingTag(u8),          // Place X markers; friendly units spend for +AP
    Precise,                  // +1 to hit when attacking
    PrecisionDebuff,          // Enemy gets -1 to hit once
    PredatorFighter,          // +1 attack per unmodified 6 to hit in melee
    RapidCharge,              // +4" move on Charge actions
    Ravage(u8),               // Roll X dice in melee, 6+ = 1 wound
    RegenerativeStrength,     // Markers grant +X attacks in melee
    Resistance,               // 6+ ignore wounds (2+ vs spells)
    Retaliate(u8),            // Attacker takes X hits per wound in melee
    SelfDestruct(u8),         // On death/melee survival, enemy takes X hits
    Shred,                    // Unmodified 1 to block = 1 extra wound
    Spawn(String),            // Once/game: place new unit of X nearby
    SpellConduit,             // Friendly casters cast from this position, +1 roll
    StealthBuff,              // Grant Stealth to a friendly unit once
    Strafing,                 // Attack a unit moved through as if shooting
    SurpriseAttack(u8),       // Infiltrate + roll X dice on first activation
    TakedownStrike,           // Once/game: one attack at Q2+, AP(2), Deadly(3), Takedown
    Unpredictable,            // On attack: 1-3 AP(+1), 4-6 +1 to hit
    UnpredictableFighter,     // In melee: 1-3 AP(+1), 4-6 +1 to hit
    UnpredictableFighterMark, // Grant Unpredictable Fighter to friendlies
    Unique,                   // Named character (one per army)

    // Battle Brothers faction rules
    Battleborn,                 // +1 to hit rolls when attacking in melee
    Shielded,                   // +1 to defense rolls
    VersatileAttack,            // Can use best attack value in melee or shooting
    FuriousAura,                // Model + unit get Furious
    HiveBondBoostAura,          // Model + unit get Hive Bond Boost
    IncreasedShootingRangeAura, // Model + unit get +6" range
    RapidChargeAura,            // Model + unit get Rapid Charge
    RegenerationAura,           // Model + unit get Regeneration
}

impl SpecialRule {
    /// Returns the AP value if this is an AP rule
    #[must_use]
    pub const fn ap_value(&self) -> Option<u8> {
        match self {
            Self::AP(x) => Some(*x),
            _ => None,
        }
    }

    /// Returns the Blast multiplier if this is a Blast rule
    #[must_use]
    pub const fn blast_multiplier(&self) -> Option<u8> {
        match self {
            Self::Blast(x) => Some(*x),
            _ => None,
        }
    }

    /// Returns the Deadly multiplier if this is a Deadly rule
    #[must_use]
    pub const fn deadly_multiplier(&self) -> Option<u8> {
        match self {
            Self::Deadly(x) => Some(*x),
            _ => None,
        }
    }

    /// Returns the Impact dice count if this is an Impact rule
    #[must_use]
    pub const fn impact_dice(&self) -> Option<u8> {
        match self {
            Self::Impact(x) => Some(*x),
            _ => None,
        }
    }

    /// Returns the Tough value if this is a Tough rule
    #[must_use]
    pub const fn tough_value(&self) -> Option<u8> {
        match self {
            Self::Tough(x) => Some(*x),
            _ => None,
        }
    }

    /// Returns the Fear bonus if this is a Fear rule
    #[must_use]
    pub const fn fear_value(&self) -> Option<u8> {
        match self {
            Self::Fear(x) => Some(*x),
            _ => None,
        }
    }

    /// Check if this rule ignores regeneration
    #[must_use]
    pub const fn ignores_regeneration(&self) -> bool {
        matches!(self, Self::Bane | Self::Rending | Self::Unstoppable)
    }

    /// Check if this rule generates extra hits on an unmodified 6 to hit
    #[must_use]
    pub const fn extra_hit_on_six(&self) -> bool {
        matches!(self, Self::Furious | Self::Relentless | Self::Surge)
    }

    /// Check if this rule modifies hit rolls
    #[must_use]
    pub const fn modifies_hit_rolls(&self) -> bool {
        matches!(
            self,
            Self::Artillery | Self::Indirect | Self::Thrust | Self::Stealth
        )
    }

    /// Check if this rule modifies defense rolls
    #[must_use]
    pub const fn modifies_defense_rolls(&self) -> bool {
        matches!(self, Self::AP(_) | Self::Bane)
    }

    /// Check if this rule is only relevant in melee
    #[must_use]
    pub const fn is_melee_only(&self) -> bool {
        matches!(self, Self::Furious | Self::Thrust | Self::Impact(_))
    }

    /// Check if this rule is only relevant in ranged combat
    #[must_use]
    pub const fn is_ranged_only(&self) -> bool {
        matches!(self, Self::Relentless | Self::Artillery)
    }

    /// Get the display name of the rule
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::AP(x) => format!("AP({x})"),
            Self::Blast(x) => format!("Blast({x})"),
            Self::Deadly(x) => format!("Deadly({x})"),
            Self::Fear(x) => format!("Fear({x})"),
            Self::Impact(x) => format!("Impact({x})"),
            Self::Tough(x) => format!("Tough({x})"),
            Self::Caster(x) => format!("Caster({x})"),
            Self::Transport(x) => format!("Transport({x})"),
            _ => format!("{self:?}"),
        }
    }
}
