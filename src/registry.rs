use std::collections::HashMap;
use serde::Deserialize;
use crate::util::expand_version;

#[derive(Debug, Deserialize)]
pub struct RegistryTool {
    pub name: String,
    pub repo: String,
    pub releases: HashMap<String, ReleaseInfo>
}

#[derive(Debug, Deserialize)]
pub struct ReleaseInfo {
    pub url: String,
    pub hash: String,
}
#[derive(Debug)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub url: String,
    pub hash: String,
}

pub fn resolve_dependency(
    tool_name: &str,
    version: &str
) -> Result<ResolvedDependency, Box<dyn std::error::Error>> {
    let url = format!(
        "https://raw.githubusercontent.com/konni332/frate-registry/refs/heads/master/tools/{}.json",
        tool_name
    );
    let response = reqwest::blocking::get(&url)?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch {}: HTTP {}", url, response.status()).into());
    }
    let body = response.text()?;
    let tool: RegistryTool = serde_json::from_str(&body)?;
    
    let full_version = expand_version(version, );
    
    let release = tool.releases.get(&full_version)
        .or_else(|| {
            if full_version.contains("musl") {
                let gnu_version = full_version.replace("musl", "gnu");
                tool.releases.get(&gnu_version)
            }
            else if full_version.contains("gnu") {
                let musl_version = full_version.replace("gnu", "musl");
                tool.releases.get(&musl_version)
            }
            else {
                None
            }
        })
        .ok_or(format!("No release found for {} {}", tool.name, full_version))?;

    let resolved = ResolvedDependency {
        name: tool.name,
        version: full_version.to_string(),
        url: release.url.clone(),
        hash: release.hash.clone(),
    };
    Ok(resolved)
}

