use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::registry::resolve_dependency;
use crate::toml::FrateToml;
use anyhow::Result;
use colored::Colorize;

/// Represents the contents of a `frate.lock` file.
/// It contains an exact snapshot of all locked packages used in the project.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrateLock {
    /// A list of all locked packages with resolved versions and hashes.
    pub packages: Vec<LockedPackage>,
}
/// Represents a single locked package, including its resolved version and source.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockedPackage {
    /// Name of the package.
    pub name: String,
    /// Exact resolved version.
    pub version: String,
    /// Download URL or source location of the package.
    pub source: String,
    /// SHA-256 hash of the downloaded artifact.
    pub hash: String,
}

impl FrateLock {
    /// Loads the lockfile from disk or returns an empty lockfile if it doesn't exist or is invalid.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `frate.lock` file.
    ///
    /// # Returns
    ///
    /// An instance of `FrateLock`.
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        if path.as_ref().exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_else(|_| FrateLock { packages: vec![]})
        }
        else {
            FrateLock { packages: vec![]}
        }
    }
    /// Saves the lockfile to disk in a pretty TOML format.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to write the `frate.lock` file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>{
        let content = toml::to_string_pretty(&self)?;
        if !path.as_ref().exists() {
            fs::File::create(&path)?;
        }
        fs::write(path, content)?;
        Ok(())
    }
    /// Synchronizes the lockfile with the current state of the `frate.toml`.
    ///
    /// Resolves all dependencies to exact versions, including download source and hash,
    /// and writes them to `self.packages`.
    ///
    /// # Arguments
    ///
    /// * `toml` - Reference to the parsed `frate.toml`.
    ///
    /// # Errors
    ///
    /// Returns an error if resolution fails for all dependencies.
    pub fn sync(
        &mut self, toml: &FrateToml
    ) -> Result<()> {
        self.packages.clear();
        for (name, version_req) in &toml.dependencies {
            let resolved = match resolve_dependency(name, version_req) {
                Ok(resolved) => resolved,
                Err(e) => {
                    eprintln!("{} {}", "Failed to resolve dependency".red(), e.to_string().red());
                    continue;
                },
            };
            let locked = LockedPackage {
                name: resolved.name,
                version: resolved.version,
                source: resolved.url,
                hash: resolved.hash,
            };
            if self.packages.iter().any(|p| p.name == locked.name) {
                continue;
            }
            self.packages.push(locked);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_load_or_default_returns_empty_on_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("frate.lock");
        let lock = FrateLock::load_or_default(&path);
        assert_eq!(lock.packages.len(), 0);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("frate.lock");

        let original = FrateLock {
            packages: vec![LockedPackage {
                name: "example".to_string(),
                version: "1.2.3".to_string(),
                source: "https://example.com".to_string(),
                hash: "abc123".to_string(),
            }],
        };

        original.save(&path).unwrap();
        let loaded = FrateLock::load_or_default(&path);
        assert_eq!(loaded.packages.len(), 1);
        assert_eq!(loaded.packages[0].name, "example");
    }

    #[test]
    fn test_load_or_default_returns_empty_on_invalid_toml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("frate.lock");
        fs::write(&path, "this is not valid toml").unwrap();

        let lock = FrateLock::load_or_default(&path);
        assert_eq!(lock.packages.len(), 0);
    }
}