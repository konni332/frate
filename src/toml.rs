use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{bail, Result};
use crate::util::is_valid_version;

/// Represents the contents of a `frate.toml` file.
///
/// This includes project metadata and a map of tool dependencies with pinned versions.
#[derive(Deserialize, Serialize, Debug)]
pub struct FrateToml {
    /// Metadata about the project using `frate`.
    pub project: Project,
    /// A map of tool names to version strings (e.g., `"just" => "1.42.0"`).
    pub dependencies: HashMap<String, String>
}
/// Basic metadata for a `frate` project.
#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    /// The name of the project.
    pub name: String,
    /// The version of the project (semantic versioning).
    pub version: String,
}

impl FrateToml {
    /// Creates a new `FrateToml` with the given project name and version `0.1.0`.
    ///
    /// # Arguments
    /// * `name` - The name of the project to initialize.
    ///
    /// # Returns
    /// A `FrateToml` instance with empty dependencies.
    pub fn default(name: &str) -> FrateToml {
        FrateToml {
            project: Project {
                name: String::from(name),
                version: String::from("0.1.0"),
            },
            dependencies: HashMap::new()
        }
    }
    /// Saves the `FrateToml` to the given file path in pretty TOML format.
    ///
    /// # Errors
    /// Returns an error if the file can't be written or serialization fails.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_str = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }
    /// Loads a `FrateToml` from a file path.
    ///
    /// # Errors
    /// Returns an error if the file can't be read or deserialized.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<FrateToml> {
        let toml = std::fs::read_to_string(path)?;
        toml::from_str(&toml).map_err(|e| e.into())
    }
    /// Adds a new dependency to the `frate.toml` file.
    ///
    /// # Arguments
    /// * `name` - The name of the tool.
    /// * `version` - A semver-compatible version string (e.g., `"1.0.2"`).
    ///
    /// # Errors
    /// Returns an error if the version is invalid or the dependency already exists.
    pub fn add(&mut self, name: &str, version: &str) -> Result<()> {
        if !is_valid_version(version) {
            bail!("Invalid version: {}", version);
        }
        if self.dependencies.contains_key(name) {
            bail!("Dependency {} already exists", name);
        }
        self.dependencies.insert(name.to_string(), version.to_string());
        Ok(())
    }
    /// Removes a dependency from the list.
    ///
    /// If the dependency does not exist, nothing happens.
    pub fn remove(&mut self, name: &str) {
        self.dependencies.remove(name);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Helper: returns a FrateToml with one dep
    fn sample_with_dep() -> FrateToml {
        let mut frate = FrateToml::default("test");
        frate.add("tool", "1.2.3").unwrap();
        frate
    }

    #[test]
    fn test_default() {
        let frate = FrateToml::default("myproj");
        assert_eq!(frate.project.name, "myproj");
        assert_eq!(frate.project.version, "0.1.0");
        assert!(frate.dependencies.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("frate.toml");

        let frate = sample_with_dep();
        frate.save(&file_path).unwrap();

        let loaded = FrateToml::load(&file_path).unwrap();
        assert_eq!(loaded.project.name, "test");
        assert_eq!(loaded.project.version, "0.1.0");
        assert_eq!(loaded.dependencies.get("tool").unwrap(), "1.2.3");
    }

    #[test]
    fn test_add_valid() {
        let mut frate = FrateToml::default("x");
        frate.add("foo", "1.0.0").unwrap();
        assert_eq!(frate.dependencies.get("foo").unwrap(), "1.0.0");
    }

    #[test]
    fn test_add_invalid_version() {
        let mut frate = FrateToml::default("x");
        let result = frate.add("foo", "bad.version");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_duplicate() {
        let mut frate = FrateToml::default("x");
        frate.add("foo", "1.0.0").unwrap();
        let result = frate.add("foo", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_existing() {
        let mut frate = sample_with_dep();
        frate.remove("tool");
        assert!(!frate.dependencies.contains_key("tool"));
    }

    #[test]
    fn test_remove_non_existing() {
        let mut frate = FrateToml::default("x");
        frate.remove("nonexistent");
        // Should not panic or error
        assert!(frate.dependencies.is_empty());
    }
}
