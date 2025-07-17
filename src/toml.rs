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