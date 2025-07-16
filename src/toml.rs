use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};


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
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_str = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }
    pub fn load<P: AsRef<Path>>(path: P) -> Result<FrateToml, Box<dyn std::error::Error>> {
        let toml = std::fs::read_to_string(path)?;
        toml::from_str(&toml).map_err(|e| e.into())
    }
    pub fn add(&mut self, name: &str, version: &str) {
        self.dependencies.insert(name.to_string(), version.to_string());
    }
}