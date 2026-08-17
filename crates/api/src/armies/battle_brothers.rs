//! Battle Brothers army list loader.
//!
//! This module loads Battle Brothers unit data from the YAML file generated
//! by the data-collector crate. The YAML file is expected to be located at
//! `data/battle-brothers.yaml` relative to the workspace root.

use crate::armies::Army;
use crate::yaml_loader::{self, YamlError};
use std::path::{Path, PathBuf};

/// Default path to the Battle Brothers YAML file relative to workspace root.
const BATTLE_BROTHERS_YAML_PATH: &str = "data/battle-brothers.yaml";

/// Load the Battle Brothers army from the YAML file.
///
/// # Errors
///
/// Returns an error if:
/// - The YAML file cannot be found or read
/// - The YAML cannot be parsed
/// - The army data cannot be converted to the internal model
pub fn load_battle_brothers(workspace_root: &Path) -> Result<Army, YamlError> {
    let yaml_path = workspace_root.join(BATTLE_BROTHERS_YAML_PATH);

    let (version, yaml_army) = yaml_loader::load_army_from_yaml(&yaml_path)?;
    let units = yaml_loader::convert_army(&yaml_army)?;

    Ok(Army {
        id: "battle-brothers".to_string(),
        name: yaml_army.name,
        version: Some(version),
        units,
    })
}

/// Load the Battle Brothers army using the default workspace path.
///
/// This function attempts to find the workspace root by looking for the
/// `Cargo.toml` file in parent directories.
///
/// # Errors
///
/// Returns an error if:
/// - The workspace root cannot be determined
/// - The YAML file cannot be found or read
/// - The YAML cannot be parsed
/// - The army data cannot be converted to the internal model
pub fn load_battle_brothers_default() -> Result<Army, YamlError> {
    let workspace_root = find_workspace_root()?;
    load_battle_brothers(&workspace_root)
}

/// Find the workspace root by looking for Cargo.toml.
fn find_workspace_root() -> Result<PathBuf, YamlError> {
    let mut current_dir = std::env::current_dir().map_err(YamlError::Io)?;

    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if this is the workspace root (has [workspace] section)
            let contents = std::fs::read_to_string(&cargo_toml).map_err(YamlError::Io)?;
            if contents.contains("[workspace]") {
                return Ok(current_dir);
            }
        }

        if !current_dir.pop() {
            return Err(YamlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find workspace root (Cargo.toml with [workspace] section)",
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_battle_brothers() {
        // Use the workspace root directly
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        let result = load_battle_brothers(&workspace_root);

        match result {
            Ok(army) => {
                assert_eq!(army.id, "battle-brothers");
                assert_eq!(army.name, "Battle Brothers");
                assert!(army.version.is_some());
                assert!(!army.units.is_empty(), "Battle Brothers should have units");

                // Check that we have some expected units
                let has_master_destroyer = army
                    .units
                    .iter()
                    .map(|u| u.name.as_str())
                    .any(|name| name == "Master Destroyer");
                assert!(has_master_destroyer, "Should contain Master Destroyer unit");

                println!(
                    "Successfully loaded Battle Brothers with {} units",
                    army.units.len()
                );
            }
            Err(e) => {
                panic!("Failed to load Battle Brothers: {e:?}");
            }
        }
    }
}
