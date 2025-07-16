use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::registry::resolve_dependency;
use crate::toml::FrateToml;
use crate::util::expand_version;

#[derive(Serialize, Deserialize, Debug)]
pub struct FrateLock {
    pub package: Vec<LockedPackage>,
}

#[derive(Serialize, Deserialize, Debug)]
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
            toml::from_str(&content).unwrap_or_else(|_| FrateLock {package: vec![]})
        }
        else {
            FrateLock {package: vec![]}
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()>{
        let content = toml::to_string_pretty(&self).unwrap();
        if !path.as_ref().exists() {
            fs::File::create(&path)?;
        }
        fs::write(path, content)
    }

    pub fn sync(
        &mut self, toml: &FrateToml
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (name, version_req) in &toml.dependencies {
            let resolved = resolve_dependency(name, version_req)?;
            let locked = LockedPackage {
                name: resolved.name,
                version: resolved.version,
                source: resolved.url,
                hash: resolved.hash,
            };
            self.package.push(locked);
        }
        Ok(())
    }
}

