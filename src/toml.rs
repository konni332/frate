use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{bail, Result};
use crate::util::is_valid_version;

#[derive(Deserialize, Serialize, Debug)]
pub struct FrateToml {
    pub project: Project,
    pub dependencies: HashMap<String, String>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub name: String,
    pub version: String,
}

impl FrateToml {
    pub fn default(name: &str) -> FrateToml {
        FrateToml {
            project: Project {
                name: String::from(name),
                version: String::from("0.1.0"),
            },
            dependencies: HashMap::new()
        }
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_str = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }
    pub fn load<P: AsRef<Path>>(path: P) -> Result<FrateToml> {
        let toml = std::fs::read_to_string(path)?;
        toml::from_str(&toml).map_err(|e| e.into())
    }
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
    pub fn remove(&mut self, name: &str) {
        self.dependencies.remove(name);
    }
}