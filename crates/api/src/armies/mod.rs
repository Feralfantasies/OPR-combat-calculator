//! Faction army lists.

pub mod alien_hives;

use crate::models::unit::Unit;

/// A named army roster.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Army {
    /// Machine-readable key, e.g. `alien_hives`
    pub id: String,
    /// Display name, e.g. "Alien Hives"
    pub name: String,
    /// Base roster (default loadouts)
    pub units: Vec<Unit>,
}

/// All armies known to the calculator.
#[must_use]
pub fn all_armies() -> Vec<Army> {
    vec![Army {
        id: "alien_hives".to_string(),
        name: "Alien Hives".to_string(),
        units: alien_hives::alien_hives(),
    }]
}

/// Look up an army by its id. Returns None if unknown.
#[must_use]
pub fn get_army(id: &str) -> Option<Army> {
    all_armies().into_iter().find(|a| a.id == id)
}

/// Look up a unit within an army by unit name. Returns None if unknown.
#[must_use]
pub fn get_unit(army_id: &str, unit_name: &str) -> Option<Unit> {
    get_army(army_id)?
        .units
        .into_iter()
        .find(|u| u.name == unit_name)
}
