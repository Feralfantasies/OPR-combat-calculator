//! Faction army lists.
//!
//! Every faction loads from the committed YAML catalogs in
//! `data/armies/` via the unified loader in [`yaml`]. There are no
//! hardcoded rosters: the catalogs are the single source of truth.

mod yaml;

use crate::models::unit::Unit;
pub use yaml::{all_armies, data_dir, get_army, get_unit, load_all_armies, load_errors};

/// A named army roster.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Army {
    /// Machine-readable key, the YAML file stem, e.g. `alien-hives`
    pub id: String,
    /// Display name, e.g. "Alien Hives"
    pub name: String,
    /// Version from the YAML file, e.g. `v3.5.3`
    pub version: Option<String>,
    /// Base roster (default loadouts)
    pub units: Vec<Unit>,
}

/// The Alien Hives roster (compatibility helper).
///
/// Backed by the unified YAML loader, so the committed
/// `data/armies/alien-hives.yaml` catalog is the source of truth. Returns
/// an empty `Vec` if the catalog fails to load — callers that need the
/// failure surfaced should inspect [`load_errors`].
#[must_use]
pub fn alien_hives() -> Vec<Unit> {
    get_army("alien-hives")
        .map(|army| army.units)
        .unwrap_or_default()
}
