use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::registry::resolve_dependency;
use crate::toml::FrateToml;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrateLock {
    pub packages: Vec<LockedPackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub hash: String,
}

impl FrateLock {
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        if path.as_ref().exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_else(|_| FrateLock { packages: vec![]})
        }
        else {
            FrateLock { packages: vec![]}
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>{
        let content = toml::to_string_pretty(&self)?;
        if !path.as_ref().exists() {
            fs::File::create(&path)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn sync(
        &mut self, toml: &FrateToml
    ) -> Result<()> {
        self.packages.clear();
        for (name, version_req) in &toml.dependencies {
            let resolved = match resolve_dependency(name, version_req) {
                Ok(resolved) => resolved,
                Err(e) => {
                    eprintln!("Failed to resolve dependency: {}", e.to_string());
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

