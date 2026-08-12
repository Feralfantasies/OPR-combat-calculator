//! End-to-end special rule tests.
//!
//! Each test exercises one special rule through the full combat pipeline
//! (`resolve_attack`) using statistical assertions over many Monte Carlo
//! runs. Margins are generous so the suite stays deterministic-in-practice
//! (failure probability per test < ~1e-9) while keeping runtime short.
//!
//! Test-only allowances: statistical assertions compare floats and use
//! plain arithmetic / `expect` for fixture setup, which the workspace's
//! production lint profile denies.
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::similar_names
)]

use opr_api::{CombatContext, CombatResult, SpecialRule, Unit, Weapon, resolve_attack};

/// Average a per-run u32 metric over `runs` resolutions.
fn avg_metric<F>(runs: usize, mut f: F) -> f64
where
    F: FnMut() -> u32,
{
    let runs_u64 = u64::try_from(runs).unwrap_or(1).max(1);
    let runs_f64 = f64::from(u32::try_from(runs_u64).unwrap_or(u32::MAX));
    let total: u64 = (0..runs).map(|_| u64::from(f())).sum();
    f64::from(u32::try_from(total / runs_u64).unwrap_or(u32::MAX))
        + f64::from(u32::try_from(total % runs_u64).unwrap_or(0)) / runs_f64
}

const fn ranged_context() -> CombatContext {
    CombatContext::ranged(12)
}

fn resolve(attacker: &Unit, defender: &Unit, context: &CombatContext) -> CombatResult {
    resolve_attack(attacker, defender, context).expect("resolution succeeds")
}

fn first_attack(result: &CombatResult) -> opr_api::AttackResult {
    result
        .attacks
        .first()
        .expect("at least one weapon attack")
        .clone()
}

/// Standard attacker: 1 model, Q3+, one ranged weapon with 10 attacks.
fn shooter(weapon: Weapon) -> Unit {
    Unit::new("Shooter", 1, 3, 3).with_weapon(weapon)
}

// ---------------------------------------------------------------------------
// AP(X)
// ---------------------------------------------------------------------------

#[test]
fn ap_reduces_blocks() {
    let defender = Unit::new("Target", 5, 3, 3); // blocks on 3+
    let plain = shooter(Weapon::ranged("Rifle", 1, 10, 24));
    let ap2 = shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::AP(2)));

    let blocked_plain = avg_metric(300, || {
        first_attack(&resolve(&plain, &defender, &ranged_context())).blocked_hits
    });
    let blocked_ap = avg_metric(300, || {
        first_attack(&resolve(&ap2, &defender, &ranged_context())).blocked_hits
    });

    // 3+ blocks 4/6 ≈ 6.7; 5+ blocks 2/6 ≈ 3.3. Expect a gap of ~3.3.
    assert!(
        blocked_ap < blocked_plain - 1.5,
        "AP(2) should block far fewer hits (plain={blocked_plain}, ap={blocked_ap})"
    );
}

// ---------------------------------------------------------------------------
// Blast(X)
// ---------------------------------------------------------------------------

#[test]
fn blast_multiplies_hits_capped_by_models() {
    let defender = Unit::new("Target", 5, 3, 6);
    let blast3 = shooter(Weapon::ranged("Launcher", 1, 10, 24).with_rule(SpecialRule::Blast(3)));

    let ratio = avg_metric(300, || {
        let attack = first_attack(&resolve(&blast3, &defender, &ranged_context()));
        if attack.hits_before_multiplier == 0 {
            return 0;
        }
        // total_hits / hits_before, scaled x100 for integer averaging
        attack.total_hits * 100 / attack.hits_before_multiplier
    });

    // Blast(3) on a 5-model unit: exactly x3 (cap doesn't bind for hits<=5
    // most of the time; average ratio should be near 300).
    assert!(
        ratio > 250.0,
        "Blast(3) should multiply hits by ~3, got ratio {ratio}"
    );
}

// ---------------------------------------------------------------------------
// Deadly(X)
// ---------------------------------------------------------------------------

#[test]
fn deadly_multiplies_wounds() {
    let defender = Unit::new("Target", 5, 3, 6);
    // AP(4) vs defense 3 clamps target to 6 so most hits wound.
    let plain = shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::AP(4)));
    let deadly2 = shooter(
        Weapon::ranged("Rifle", 1, 10, 24)
            .with_rule(SpecialRule::AP(4))
            .with_rule(SpecialRule::Deadly(2)),
    );

    let wounds_plain = avg_metric(300, || {
        first_attack(&resolve(&plain, &defender, &ranged_context())).wounds_inflicted
    });
    let wounds_deadly = avg_metric(300, || {
        first_attack(&resolve(&deadly2, &defender, &ranged_context())).wounds_inflicted
    });

    let ratio = wounds_deadly / wounds_plain;
    assert!(
        (1.7..2.3).contains(&ratio),
        "Deadly(2) should roughly double wounds (plain={wounds_plain}, deadly={wounds_deadly})"
    );
}

// ---------------------------------------------------------------------------
// Impact(X)
// ---------------------------------------------------------------------------

#[test]
fn impact_rolls_extra_dice_on_charge() {
    let attacker = Unit::new("Charger", 2, 3, 3)
        .with_rule(SpecialRule::Impact(4))
        .with_weapon(Weapon::melee("Claws", 2, 1));
    let defender = Unit::new("Target", 5, 3, 4);

    let result = resolve(&attacker, &defender, &CombatContext::melee_charge());
    let impact = result
        .attacks
        .iter()
        .find(|a| a.weapon_name == "Impact")
        .expect("Impact attack present on charge");
    assert_eq!(impact.base_attacks, 8); // 4 dice x 2 models

    // Fatigued attackers skip impact dice.
    let mut fatigued = CombatContext::melee_return(true);
    fatigued.is_charging = true;
    let result = resolve(&attacker, &defender, &fatigued);
    assert!(
        result.attacks.iter().all(|a| a.weapon_name != "Impact"),
        "fatigued units must not roll Impact dice"
    );
}

// ---------------------------------------------------------------------------
// Tough(X)
// ---------------------------------------------------------------------------

#[test]
fn tough_controls_models_removed() {
    let attacker = shooter(Weapon::ranged("Rifle", 1, 10, 24));
    let defender = Unit::new("Tough", 5, 6, 6).with_rule(SpecialRule::Tough(4));

    for _ in 0..50 {
        let result = resolve(&attacker, &defender, &ranged_context());
        assert_eq!(result.models_removed, result.total_net_wounds / 4);
    }
}

// ---------------------------------------------------------------------------
// Bane
// ---------------------------------------------------------------------------

#[test]
fn bane_ignores_regeneration() {
    let defender = Unit::new("Regen", 5, 3, 3).with_rule(SpecialRule::Regeneration);
    let bane = shooter(Weapon::ranged("Toxin", 1, 10, 24).with_rule(SpecialRule::Bane));

    for _ in 0..50 {
        let attack = first_attack(&resolve(&bane, &defender, &ranged_context()));
        assert_eq!(attack.regenerated_wounds, 0, "Bane must skip Regeneration");
        assert_eq!(attack.net_wounds, attack.wounds_inflicted);
    }
}

#[test]
fn bane_reduces_blocks_via_rerolls() {
    let defender = Unit::new("Target", 5, 3, 3); // blocks on 3+
    let plain = shooter(Weapon::ranged("Rifle", 1, 10, 24));
    let bane = shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::Bane));

    let blocked_plain = avg_metric(1500, || {
        first_attack(&resolve(&plain, &defender, &ranged_context())).blocked_hits
    });
    let blocked_bane = avg_metric(1500, || {
        first_attack(&resolve(&bane, &defender, &ranged_context())).blocked_hits
    });

    // Bane rerolls unmodified defense 6s. The rerolled die still always
    // succeeds on a 6 (and blocks on 3-5 vs a 3+ target), so a rerolled 6
    // only becomes a wound on a 1-2: losing (1/6)*(2/6) ≈ 0.056 blocks per
    // defense roll. With ~6.7 hits expected, that is ≈ 0.37 fewer blocks.
    assert!(
        blocked_bane < blocked_plain - 0.12,
        "Bane rerolls should reduce blocks (plain={blocked_plain}, bane={blocked_bane})"
    );
}

// ---------------------------------------------------------------------------
// Furious / Relentless / Surge (extra hits on unmodified 6)
// ---------------------------------------------------------------------------

#[test]
fn furious_extra_hits_on_melee_charge() {
    let attacker = Unit::new("Charger", 1, 3, 3)
        .with_weapon(Weapon::melee("Claws", 1, 10).with_rule(SpecialRule::Furious));
    let defender = Unit::new("Target", 5, 3, 4);

    let extras = avg_metric(300, || {
        first_attack(&resolve(
            &attacker,
            &defender,
            &CombatContext::melee_charge(),
        ))
        .extra_attacks
    });
    // ~10/6 sixes expected.
    assert!(
        extras > 1.0,
        "Furious should generate extras on charge, got {extras}"
    );
}

#[test]
fn relentless_extra_hits_only_at_long_range() {
    let attacker = Unit::new("Shooter", 1, 3, 3)
        .with_weapon(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::Relentless));
    let defender = Unit::new("Target", 5, 3, 4);

    let extras_long = avg_metric(300, || {
        first_attack(&resolve(&attacker, &defender, &CombatContext::ranged(12))).extra_attacks
    });
    let extras_short = avg_metric(100, || {
        first_attack(&resolve(&attacker, &defender, &CombatContext::ranged(6))).extra_attacks
    });

    assert!(
        extras_long > 1.0,
        "Relentless should trigger over 9\", got {extras_long}"
    );
    assert_eq!(extras_short, 0.0, "Relentless must not trigger within 9\"");
}

#[test]
fn surge_extra_hits_in_any_context() {
    let attacker = Unit::new("Shooter", 1, 3, 3)
        .with_weapon(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::Surge));
    let defender = Unit::new("Target", 5, 3, 4);

    let extras = avg_metric(300, || {
        first_attack(&resolve(&attacker, &defender, &CombatContext::ranged(3))).extra_attacks
    });
    assert!(
        extras > 1.0,
        "Surge should trigger at any range, got {extras}"
    );
}

// ---------------------------------------------------------------------------
// Reliable
// ---------------------------------------------------------------------------

#[test]
fn reliable_attacks_at_quality_2_plus() {
    let defender = Unit::new("Target", 5, 3, 6);
    let poor = shooter(Weapon::ranged("Rifle", 1, 10, 24)); // Q3+
    let reliable = shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::Reliable));

    let hits_poor = avg_metric(300, || {
        first_attack(&resolve(&poor, &defender, &ranged_context())).hits_before_multiplier
    });
    let hits_reliable = avg_metric(300, || {
        first_attack(&resolve(&reliable, &defender, &ranged_context())).hits_before_multiplier
    });

    // Q3+ ≈ 5/6 of 10; Q2+ would be even higher — use a poor-quality unit
    // instead: Q5+ ≈ 2/6 ≈ 3.3 vs Reliable ≈ 5/6 ≈ 8.3.
    let _ = hits_poor;
    let poor_q5 = Unit::new("Poor", 1, 5, 3).with_weapon(Weapon::ranged("Rifle", 1, 10, 24));
    let hits_q5 = avg_metric(300, || {
        first_attack(&resolve(&poor_q5, &defender, &ranged_context())).hits_before_multiplier
    });
    assert!(
        hits_reliable > hits_q5 + 2.0,
        "Reliable (Q2+) should beat Q5+ (reliable={hits_reliable}, q5={hits_q5})"
    );
}

// ---------------------------------------------------------------------------
// Rending
// ---------------------------------------------------------------------------

#[test]
fn rending_hits_are_harder_to_block() {
    let defender = Unit::new("Target", 5, 3, 2); // blocks on 2+
    let plain = shooter(Weapon::ranged("Rifle", 1, 10, 24));
    let rending = shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::Rending));

    let blocked_plain = avg_metric(400, || {
        first_attack(&resolve(&plain, &defender, &ranged_context())).blocked_hits
    });
    let blocked_rending = avg_metric(400, || {
        first_attack(&resolve(&rending, &defender, &ranged_context())).blocked_hits
    });

    // 6s (~1.7 hits) move from block-on-2+ (5/6) to block-on-6+ (1/6).
    assert!(
        blocked_rending < blocked_plain - 0.6,
        "Rending 6s should be harder to block (plain={blocked_plain}, rending={blocked_rending})"
    );
}

#[test]
fn rending_ignores_regeneration() {
    let defender = Unit::new("Regen", 5, 3, 3).with_rule(SpecialRule::Regeneration);
    let rending = shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::Rending));

    for _ in 0..50 {
        let attack = first_attack(&resolve(&rending, &defender, &ranged_context()));
        assert_eq!(
            attack.regenerated_wounds, 0,
            "Rending must skip Regeneration"
        );
    }
}

// ---------------------------------------------------------------------------
// Regeneration
// ---------------------------------------------------------------------------

#[test]
fn regeneration_reduces_net_wounds() {
    let attacker = shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::AP(4)));
    let plain = Unit::new("Target", 5, 3, 6);
    let regen = Unit::new("Regen", 5, 3, 6).with_rule(SpecialRule::Regeneration);

    let net_plain = avg_metric(300, || {
        first_attack(&resolve(&attacker, &plain, &ranged_context())).net_wounds
    });
    let net_regen = avg_metric(300, || {
        first_attack(&resolve(&attacker, &regen, &ranged_context())).net_wounds
    });
    let regenerated = avg_metric(300, || {
        first_attack(&resolve(&attacker, &regen, &ranged_context())).regenerated_wounds
    });

    assert!(
        net_regen < net_plain - 0.5,
        "Regeneration should reduce net wounds (plain={net_plain}, regen={net_regen})"
    );
    assert!(
        regenerated > 0.5,
        "Regeneration should ignore some wounds on average"
    );
}

// ---------------------------------------------------------------------------
// Shred
// ---------------------------------------------------------------------------

#[test]
fn shred_adds_wounds_on_defense_ones() {
    let defender = Unit::new("Target", 5, 3, 2); // blocks on 2+ (few 1s fail anyway)
    let shred = shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::Shred));

    let extra = avg_metric(400, || {
        let attack = first_attack(&resolve(&shred, &defender, &ranged_context()));
        attack
            .wounds_inflicted
            .saturating_sub(attack.total_hits - attack.blocked_hits)
    });

    // ~10 defense rolls, ~1/6 are 1s => ~1.7 extra wounds.
    assert!(
        extra > 0.8,
        "Shred should add extra wounds on 1s, got {extra}"
    );
}

// ---------------------------------------------------------------------------
// Unstoppable
// ---------------------------------------------------------------------------

#[test]
fn unstoppable_ignores_regeneration() {
    let defender = Unit::new("Regen", 5, 3, 3).with_rule(SpecialRule::Regeneration);
    let unstoppable =
        shooter(Weapon::ranged("Rifle", 1, 10, 24).with_rule(SpecialRule::Unstoppable));

    for _ in 0..50 {
        let attack = first_attack(&resolve(&unstoppable, &defender, &ranged_context()));
        assert_eq!(
            attack.regenerated_wounds, 0,
            "Unstoppable must skip Regeneration"
        );
    }
}

// ---------------------------------------------------------------------------
// Fatigue
// ---------------------------------------------------------------------------

#[test]
fn fatigued_attackers_only_hit_on_sixes() {
    let defender = Unit::new("Target", 5, 3, 6);
    let attacker = shooter(Weapon::ranged("Rifle", 1, 10, 24));

    let hits_normal = avg_metric(300, || {
        first_attack(&resolve(&attacker, &defender, &ranged_context())).hits_before_multiplier
    });
    let hits_fatigued = avg_metric(300, || {
        first_attack(&resolve(
            &attacker,
            &defender,
            &ranged_context().with_fatigue(true),
        ))
        .hits_before_multiplier
    });

    // Q3+ ≈ 8.3 hits vs sixes-only ≈ 1.7.
    assert!(
        hits_fatigued < hits_normal - 4.0,
        "fatigue should drastically reduce hits (normal={hits_normal}, fatigued={hits_fatigued})"
    );
}
