//! OPR Grimdark Future combat simulation core.
//!
//! Unofficial fan project — see the repository README for the disclaimer.
//!
//! # Modules
//! - [`models`]: [`models::Unit`], [`models::Weapon`], [`models::SpecialRule`]
//! - [`combat`]: the 8-phase combat resolution engine
//! - [`armies`]: faction army lists, all loaded from the committed YAML
//!   catalogs via the unified loader

pub mod armies;
pub mod combat;
pub mod models;
pub mod yaml_loader;

pub use armies::{Army, alien_hives, all_armies, cached_armies, get_army, get_unit, load_errors};
pub use combat::{AttackResult, AttackType, CombatContext, CombatResult, DieRoll, resolve_attack};
pub use models::{
    SpecialRule, Unit, UpgradeGroup, UpgradeOption, UpgradeSelection, Weapon, WeaponChange,
    apply_upgrades,
};

/// Maximum number of simulation iterations accepted by [`simulate`].
pub const MAX_ITERATIONS: u32 = 100_000;

/// Aggregated per-weapon statistics over many simulation runs.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WeaponAggregate {
    pub name: String,
    pub avg_hits: f64,
    pub avg_blocked: f64,
    pub avg_net_wounds: f64,
}

/// Aggregated result of running [`resolve_attack`] many times.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SimulationResult {
    pub iterations: u32,
    pub avg_net_wounds: f64,
    pub avg_models_removed: f64,
    pub min_net_wounds: u32,
    pub max_net_wounds: u32,
    pub weapons: Vec<WeaponAggregate>,
}

/// Run the sanctioned damage simulation: resolve the attack `iterations`
/// times and aggregate (mean/min/max). Iterations are clamped to
/// `1..=MAX_ITERATIONS`.
///
/// # Errors
/// Propagates any error from [`resolve_attack`].
pub fn simulate(
    attacker: &Unit,
    defender: &Unit,
    context: &CombatContext,
    iterations: u32,
) -> Result<SimulationResult, String> {
    let iterations = iterations.clamp(1, MAX_ITERATIONS);
    // f64 accumulators avoid integer division and as-conversions.
    let mut total_wounds = 0.0f64;
    let mut total_models = 0.0f64;
    let mut min_wounds = u32::MAX;
    let mut max_wounds = 0u32;
    // weapon name -> (hits, blocked, net_wounds) sums
    let mut weapons: std::collections::HashMap<String, (f64, f64, f64)> =
        std::collections::HashMap::new();

    for _ in 0..iterations {
        let result = resolve_attack(attacker, defender, context)?;
        total_wounds += f64::from(result.total_net_wounds);
        total_models += f64::from(result.models_removed);
        min_wounds = min_wounds.min(result.total_net_wounds);
        max_wounds = max_wounds.max(result.total_net_wounds);
        for attack in &result.attacks {
            let entry = weapons
                .entry(attack.weapon_name.clone())
                .or_insert((0.0, 0.0, 0.0));
            entry.0 += f64::from(attack.total_hits);
            entry.1 += f64::from(attack.blocked_hits);
            entry.2 += f64::from(attack.net_wounds);
        }
    }

    let n = f64::from(iterations);
    let mut weapon_aggs: Vec<WeaponAggregate> = weapons
        .into_iter()
        .map(|(name, (hits, blocked, net))| WeaponAggregate {
            name,
            avg_hits: hits / n,
            avg_blocked: blocked / n,
            avg_net_wounds: net / n,
        })
        .collect();
    weapon_aggs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(SimulationResult {
        iterations,
        avg_net_wounds: total_wounds / n,
        avg_models_removed: total_models / n,
        min_net_wounds: if min_wounds == u32::MAX {
            0
        } else {
            min_wounds
        },
        max_net_wounds: max_wounds,
        weapons: weapon_aggs,
    })
}
