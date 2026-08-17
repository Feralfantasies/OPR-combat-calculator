//! Unified YAML army registry.
//!
//! Every faction (Alien Hives, Battle Brothers, and all other committed
//! catalogs) loads from the same source of truth: the army YAML files in
//! `data/armies/` at the workspace root. One loader serves every faction;
//! there are no per-army Rust modules to maintain.
//!
//! Resolution of the data directory (highest precedence first):
//! 1. `OPR_DATA_DIR` environment variable, if set (the directory holding
//!    the army YAML files, or a directory containing `armies/`)
//! 2. Walk up from the compile-time crate location
//!    (`CARGO_MANIFEST_DIR`) and its parent — the parent is the workspace
//!    root, so this works for `cargo test`/`cargo run` from any
//!    subdirectory
//! 3. Walk up from the process working directory as a last resort (for
//!    processes where the manifest anchors are unavailable)
//!
//! The roster is built once per process and cached; individual load errors
//! are recorded and surfaced via [`load_errors`] instead of being silently
//! dropped.

use crate::armies::Army;
use crate::models::unit::Unit;
use crate::yaml_loader::{self, YamlError};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static ROSTER: OnceLock<Vec<Army>> = OnceLock::new();
static LOAD_ERRORS: OnceLock<LoadErrors> = OnceLock::new();

/// Set at compile time to the two likely workspace anchors: this crate's
/// directory and its parent. Neither depends on the process working
/// directory.
const MANIFEST_DIRS: &[&str] = &[
    env!("CARGO_MANIFEST_DIR"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/.."),
];

/// Catalog name paired with the load error that kept it out of the roster.
type LoadErrors = Vec<(String, String)>;

fn io_err(message: &str) -> YamlError {
    YamlError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, message))
}

/// Locate the directory containing the army YAML catalogs.
///
/// # Errors
///
/// Returns a `YamlError::Io` when no candidate directory contains army
/// YAML files.
pub fn data_dir() -> Result<PathBuf, YamlError> {
    let override_dir = std::env::var("OPR_DATA_DIR").ok();
    resolve_data_dir(override_dir.as_deref())
}

/// Pure resolution logic, split out so the override behavior is testable
/// without mutating process environment (which is process-wide and would
/// race with parallel tests).
///
/// # Errors
///
/// See [`data_dir`].
fn resolve_data_dir(override_dir: Option<&str>) -> Result<PathBuf, YamlError> {
    if let Some(dir) = override_dir {
        let dir = PathBuf::from(dir);
        if has_yaml_files(&dir) {
            return Ok(dir);
        }
        let nested = dir.join("armies");
        if has_yaml_files(&nested) {
            return Ok(nested);
        }
        return Err(io_err("OPR_DATA_DIR contains no army YAML files"));
    }

    let cwd = std::env::current_dir().map_err(YamlError::Io)?;
    let mut anchors: Vec<&Path> = MANIFEST_DIRS.iter().map(Path::new).collect();
    anchors.push(cwd.as_path());
    if let Some(dir) = anchors.iter().find_map(|anchor| walk_up_for_armies(anchor)) {
        return Ok(dir);
    }
    Err(io_err("could not locate the army data directory"))
}

/// Locate the army catalog directory: `dir` itself, `dir/armies`, or the
/// committed layout `dir/data/armies`.
fn catalog_dir(dir: &Path) -> Option<PathBuf> {
    let candidates = [
        dir.to_path_buf(),
        dir.join("armies"),
        dir.join("data").join("armies"),
    ];
    candidates
        .iter()
        .find(|candidate| has_yaml_files(candidate))
        .cloned()
}

/// Walk up from `anchor` looking for a workspace root whose `data/armies`
/// holds at least one YAML file.
fn walk_up_for_armies(anchor: &Path) -> Option<PathBuf> {
    let mut current = anchor.to_path_buf();
    loop {
        if let Some(dir) = catalog_dir(&current) {
            return Some(dir);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn has_yaml_files(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|e| e.path().extension().is_some_and(|ext| ext == "yaml"))
    })
}

/// Load every army catalog from disk.
///
/// Each `*.yaml` file under the data directory becomes one [`Army`]; the
/// army id is the file name without extension (e.g. `battle-brothers`).
/// Returns `(armies, errors)` where `errors` lists every catalog that
/// failed to load or convert, keyed by file name — nothing is dropped
/// silently.
///
/// # Errors
///
/// Propagates [`YamlError::Io`] when the data directory cannot be found.
pub fn load_all_armies() -> Result<(Vec<Army>, LoadErrors), YamlError> {
    let dir = data_dir()?;
    let mut armies = BTreeMap::new();
    let mut errors = Vec::new();

    let entries = std::fs::read_dir(&dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "yaml") {
            continue;
        }
        let file_stem = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".yaml"))
            .unwrap_or_default();

        match load_army_catalog(&path) {
            Ok((version, yaml_army)) => match yaml_loader::convert_army(&yaml_army) {
                Ok(units) => {
                    armies.insert(
                        file_stem.to_string(),
                        Army {
                            id: file_stem.to_string(),
                            name: yaml_army.name,
                            version: Some(version),
                            units,
                        },
                    );
                }
                Err(e) => errors.push((file_stem.to_string(), e.to_string())),
            },
            Err(e) => errors.push((file_stem.to_string(), e.to_string())),
        }
    }

    Ok((armies.into_values().collect(), errors))
}

/// Load a single army catalog file.
///
/// # Errors
///
/// See [`yaml_loader::load_army_from_yaml`].
fn load_army_catalog(path: &Path) -> Result<(String, yaml_loader::YamlArmy), YamlError> {
    yaml_loader::load_army_from_yaml(path)
}

/// The cached army roster: all successfully loaded catalogs, sorted by id.
/// Build runs once per process; load errors are available via
/// [`load_errors`].
///
/// # Errors
///
/// Panics-free by design: if the data directory cannot be found the empty
/// roster is returned and the error is carried in [`load_errors`].
#[must_use]
pub fn cached_armies() -> Vec<Army> {
    let roster = ROSTER.get_or_init(|| match load_all_armies() {
        Ok((armies, errors)) => {
            record_load_errors(errors);
            armies
        }
        Err(e) => {
            record_load_errors(vec![("<data directory>".to_string(), e.to_string())]);
            Vec::new()
        }
    });
    roster.clone()
}

fn record_load_errors(errors: LoadErrors) {
    let _ = LOAD_ERRORS.set(errors);
}

/// Errors encountered while building the cached roster, as `(file name,
/// message)` pairs. Empty when every catalog loaded cleanly.
///
/// The first call builds the roster (see [`cached_armies`]), so calling
/// this also guarantees the roster is available.
#[must_use]
pub fn load_errors() -> LoadErrors {
    let _ = cached_armies();
    LOAD_ERRORS
        .get()
        .cloned()
        .unwrap_or_else(|| load_all_armies().unwrap_or_default().1)
}

/// All armies known to the calculator (cached, see [`cached_armies`]).
#[must_use]
pub fn all_armies() -> Vec<Army> {
    cached_armies()
}

/// Look up an army by its id (the YAML file stem, e.g.
/// `alien-hives`). Returns `None` if unknown.
#[must_use]
pub fn get_army(id: &str) -> Option<Army> {
    cached_armies().into_iter().find(|a| a.id == id)
}

/// Look up a unit within an army by unit name. Returns `None` if unknown.
#[must_use]
pub fn get_unit(army_id: &str, unit_name: &str) -> Option<Unit> {
    get_army(army_id)?
        .units
        .into_iter()
        .find(|u| u.name == unit_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Load the roster via the default (compile-time anchored) discovery so
    /// the test exercises the same path production uses.
    fn roster() -> (Vec<Army>, LoadErrors) {
        load_all_armies().unwrap_or_else(|e| panic!("data directory discovery failed: {e}"))
    }

    #[test]
    fn loads_all_committed_catalogs_without_error() {
        let (armies, errors) = roster();

        assert!(errors.is_empty(), "catalog loads must not fail: {errors:?}");
        assert!(
            (40..=60).contains(&armies.len()),
            "expected ~43 committed catalogs, found {}",
            armies.len()
        );
    }

    #[test]
    fn army_names_and_ids_match_catalogs() {
        let (armies, _) = roster();
        let ids: Vec<&str> = armies.iter().map(|a| a.id.as_str()).collect();
        assert!(
            ids.contains(&"alien-hives"),
            "missing Alien Hives id: {ids:?}"
        );
        assert!(
            ids.contains(&"battle-brothers"),
            "missing Battle Brothers id: {ids:?}"
        );

        let bb = armies
            .iter()
            .find(|a| a.id == "battle-brothers")
            .expect("battle-brothers present");
        assert_eq!(bb.name, "Battle Brothers");
        assert!(bb.version.is_some());
    }

    #[test]
    fn alien_hives_catalog_has_35_units() {
        let army = get_army("alien-hives").expect("alien-hives loads");
        assert_eq!(army.units.len(), 35);
    }

    #[test]
    fn get_unit_finds_master_destroyer_with_yaml_stats() {
        let unit = get_unit("battle-brothers", "Master Destroyer").expect("Master Destroyer loads");
        assert_eq!(unit.quality, 3);
        assert_eq!(unit.defense, 3);
        assert_eq!(unit.tough, 6);
        assert_eq!(unit.points, 145);
        assert!(!unit.weapons.is_empty());
        assert!(!unit.special_rules.is_empty());
    }

    #[test]
    fn explicit_directory_override_is_honored() {
        // An explicit directory (what OPR_DATA_DIR would carry) must be
        // used as-is instead of re-discovered. Exercised through the pure
        // resolver so parallel tests never touch the process environment.
        let discovered = resolve_data_dir(None).expect("discovery works");
        let overridden = resolve_data_dir(Some(discovered.to_str().expect("path is utf-8")))
            .expect("override honored");
        assert_eq!(discovered, overridden);
    }
}
