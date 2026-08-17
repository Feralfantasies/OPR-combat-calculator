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

    // Faction marker rules found in the committed catalogs. The combat
    // engine does not implement effects for these yet; they are carried so
    // rosters round-trip faithfully and can be queried by name.
    Bloodborn,             // Blood Brothers marker: +1 to wound rolls in melee
    Darkborn,              // Dark Brothers marker
    Highborn,              // High Elf Fleets marker
    Knightborn,            // Knight Brothers marker
    Primeborn,             // Prime Brothers marker
    Watchborn,             // Watch Brothers marker
    Wolfborn,              // Wolf Brothers marker
    Changebound,           // Change Disciples marker
    Havocbound,            // Havoc Brothers marker
    Lustbound,             // Lust Disciples marker
    Plaguebound,           // Plague Disciples marker
    Warbound,              // War Disciples marker
    ClanWarrior,           // Eternal Dynasty marker
    InquisitorialAgent,    // Human Inquisition operative marker
    Fanatic,               // +1 attack in melee (generic)
    Devout,                // +1 to Defense rolls when defending
    Primal,                // Feral marker: ignores some control
    Infected,              // Infected Colonies marker
    Sturdy,                // Dwarf Guilds: +1 Tough
    Evasive,               // -1 to hit rolls taken
    Agile,                 // +1" Advance/Rush
    Bounding,              // +2" Advance/Rush
    Scurry,                // +4" Advance
    Harassing,             // ranged harass profile
    Guerrilla,             // guerrilla repositioning after shooting
    Protected,             // first wound on the model is ignored
    Lacerate,              // wounds multiply on 6s to wound
    Shatter,               // model destroyed on Tough wounds
    Tear,                  // rending variant: extra wounds on 6s
    Crack,                 // blast variant: area on 6s
    Disintegrate,          // wounds ignore Tough
    BrutalFighter,         // +1 attack per model, -1 Tough (trade-off)
    MeleeSlayer,           // +1 attack vs models, not units
    MeleeEvasion,          // +1 defense vs melee
    RapidBlink,            // teleport-range Advance
    Teleport,              // short teleport action
    MachineFog,            // Machine Cults: -1 to hit rolls taken
    SelfRepair,            // 5+ to ignore 1 wound
    TargetingVisor,        // +1 to hit rolls
    GoodShot,              // +1 to the first hit roll
    BadShot,               // -1 to the first hit roll
    HoldTheLine,           // cannot rout
    HonorCode,             // hero code marker
    CounterAttack,         // return strike on failed charge
    RapidAmbush,           // Ambush + +1" Advance
    MobileArtillery,       // may shoot after moving (no Indirect)
    PiercingAssault,       // AP(+1) on charge
    PointBlankSurge,       // +1 attack below 9"
    TakedownShot,          // ranged Takedown variant
    AmbushingPiercingShot, // Ambush + AP(+1)
    CastingDebuff,         // +1 to friendly spell rolls
    CourageBuff,           // grant Fearless to a friendly unit once
    DevoutBoost,           // grant Devout to a friendly unit once
    Ferocious,             // +1 attack at 0 Tough
    FerociousBoost,        // grant Ferocious to a friendly unit once
    GroundedStealth,       // Stealth while stationary
    GuardedBuff,           // +1 to Defense rolls (unit buff)
    Guardian,              // nearby friendlies +1 Defense
    Hazardous,             // nearby enemies -1 Defense
    MartialProwess,        // +1 to hit and Defense
    QuickReadjustment,     // re-aim after a miss
    SwiftBuff,             // +1" to friendly movement
    UnpredictableShooter,  // Unpredictable, but only when shooting
    Mischievous,           // +1 to flee, -1 to be hit in close combat
    Scrapper,              // +1 to Defense while in melee
    Deathstrike(u8),       // X attacks at Q2+, AP(1), Deadly(2) per model
    HeavyImpact(u8),       // Impact(X) at double dice on charge
    CrossingAttack(u8),    // X dice, 6+ = hit, vs targets it crosses
    PrecisionFighterBuff,  // grant Precision Fighter to a friendly unit once
    PrimalBoostBuff,       // grant Primal to a friendly unit once
    SelfRepairBoostBuff,   // grant Self-Repair to a friendly unit once

    // Display rules present in committed catalogs that the model does not
    // give an explicit variant for. The raw display string is preserved so
    // consumers can match and present the rule by name.
    Unmodelled(String),
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
            Self::PiercingTag(x) => format!("Piercing Tag({x})"),
            Self::Ravage(x) => format!("Ravage({x})"),
            Self::Retaliate(x) => format!("Retaliate({x})"),
            Self::SelfDestruct(x) => format!("Self-Destruct({x})"),
            Self::SurpriseAttack(x) => format!("Surprise Attack({x})"),
            Self::Spawn(s) => format!("Spawn({s})"),
            Self::Unmodelled(s) => s.clone(),
            _ => format!("{self:?}"),
        }
    }
}
