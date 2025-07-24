use std::collections::HashMap;
use serde::Deserialize;
use crate::util::expand_version;
use anyhow::{bail, Result};

/// A tool as defined in the frate registry.
///
/// Each tool corresponds to a GitHub repository and a set of releases.
#[derive(Debug, Deserialize)]
pub struct RegistryTool {
    /// The name of the tool.
    pub name: String,
    /// The GitHub repository of the tool, e.g. "user/repo".
    pub repo: String,
    /// A map of version identifiers to their release information.
    pub releases: HashMap<String, ReleaseInfo>
}

/// Metadata for a specific release of a tool.
///
/// This includes the download URL and a hash for integrity checking.
#[derive(Debug, Deserialize)]
pub struct ReleaseInfo {
    /// The URL to download the binary archive.
    pub url: String,
    /// The SHA-256 hash of the archive to verify integrity.
    pub hash: String,
}

impl Default for ReleaseInfo {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            hash: "".to_string(),
        }
    }   
}

/// A fully resolved dependency, ready to be downloaded and installed.
///
/// This includes the expanded version (e.g., including `x86_64-unknown-linux-gnu`)
/// as well as the verified source URL and hash.
#[derive(Debug)]
pub struct ResolvedDependency {
    /// Name of the tool (as registered).
    pub name: String,
    /// The full resolved version string.
    pub version: String,
    /// Download URL for the binary.
    pub url: String,
    /// SHA-256 hash of the binary archive.
    pub hash: String,
}
/// Resolves a tool version by looking it up in the registry.
///
/// If the requested version (e.g., `1.2.3-x86_64-unknown-linux-musl`) is not found,
/// the function attempts to fall back to a GNU/MUSL alternative if available.
///
/// # Arguments
///
/// * `tool_name` – The name of the tool to resolve (e.g., `"ripgrep"`).
/// * `version` – The version string to resolve. Can be a short version like `"1.2.3"` or a fully qualified triple like `"1.2.3-x86_64-unknown-linux-musl"`.
///
/// # Errors
///
/// Returns an error if the tool or the requested version cannot be found or fetched.
///
/// # Example
///
/// ```no_run
/// use frate::resolve_dependency;
///
/// let dep = resolve_dependency("ripgrep", "14.0.0").unwrap();
/// assert!(dep.url.ends_with(".tar.gz") || dep.url.ends_with(".zip"));
/// ```
pub fn resolve_dependency(
    tool_name: &str,
    version: &str
) -> Result<ResolvedDependency> {
    let tool = fetch_registry(tool_name)?;
    
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
        .ok_or(anyhow::anyhow!("Tool version not found in registry"))?;

    let resolved = ResolvedDependency {
        name: tool.name,
        version: full_version.to_string(),
        url: release.url.clone(),
        hash: release.hash.clone(),
    };
    Ok(resolved)
}

/// Fetches a tool's metadata from the frate registry.
///
/// This loads a JSON file hosted in the GitHub frate registry under:
/// `https://github.com/konni332/frate-registry/tools/<tool>.json`
///
/// # Arguments
///
/// * `tool_name` – The name of the tool to fetch (e.g., `"ripgrep"`).
///
/// # Returns
///
/// A parsed [`RegistryTool`] structure containing all available releases.
///
/// # Errors
///
/// Returns an error if the registry cannot be fetched or parsed.
pub fn fetch_registry(tool_name: &str) -> Result<RegistryTool> {
    let url = format!(
        "https://raw.githubusercontent.com/konni332/frate-registry/refs/heads/master/tools/{}.json",
        tool_name
    );
    let response = reqwest::blocking::get(&url)?;

    if !response.status().is_success() {
        bail!("Failed to fetch {} from registry", tool_name);
    }
    let body = response.text()?;
    let tool: RegistryTool = serde_json::from_str(&body)?;
    Ok(tool)
}

