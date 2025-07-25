use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::lock::{FrateLock, LockedPackage};
use anyhow::{bail, Result};
use regex::Regex;
use reqwest::blocking::Client;
use semver::Version;
use walkdir::WalkDir;
use crate::registry::ReleaseInfo;

#[cfg(target_os = "windows")]
pub const PATH_SEPARATOR: &str = "\\";

#[cfg(not(target_os = "windows"))]
pub const PATH_SEPARATOR: &str = "/";

/// Ensures the `.frate` directory structure exists under the given root path.
/// Creates `.frate/bin` and `.frate/shims` if they don't already exist.
///
/// Returns the full path to the `.frate` directory.
pub fn ensure_frate_dirs<P: AsRef<Path>>(root: P) -> Result<PathBuf> {
    let mut path = PathBuf::from(root.as_ref());
    path.push(".frate");
    std::fs::create_dir_all(&path)?;
    let bin_path = path.join("bin");
    std::fs::create_dir_all(&bin_path)?;
    let shims_path = path.join("shims");
    std::fs::create_dir_all(&shims_path)?;
    Ok(path)
}


/// Strips the `sha256:` prefix from a hash if present.
/// This is useful for formatting hashes uniformly.
pub fn format_hash(hash: &str) -> String {
    if let Some(hash) = hash.strip_prefix("sha256:") {
        hash.to_string()
    } else {
        hash.to_string()
    }
}
/// Returns the current target triple (e.g. `x86_64-unknown-linux-gnu`)
/// based on the host system's architecture and operating system.
pub fn current_target_triple() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    match (arch, os) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_string(),
        ("x86", "windows") => "i686-pc-windows-msvc".to_string(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_string(),
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu".to_string(),
        ("aarch64", "macos") => "aarch64-apple-darwin".to_string(),
        ("x86_64", "macos") => "x86_64-apple-darwin".to_string(),
        _ => format!("{}-unknown-{}", arch, os),
    }
}
/// Expands a version string to include the target triple.
/// For example, `1.2.3` becomes `1.2.3-x86_64-unknown-linux-gnu`.
pub fn expand_version(version: &str) -> String {
    let triple = current_target_triple();
    format!("{}-{}", version, triple)

}
/// Checks whether a package with the given name is listed in the `frate.lock`.
pub fn is_locked(name: &str, lock: &FrateLock) -> bool {
    for package in &lock.packages {
        if package.name == name {
            return true;
        }
    }
    false
}
/// Returns the locked package entry for the given name, if it exists.
pub fn get_locked(name: &str, lock: &FrateLock) -> Option<LockedPackage> {
    for package in &lock.packages {
        if package.name == name {
            return Some(package.clone());
        }
    }
    None
}
/// Checks whether a package is installed by verifying the binary path exists.
pub fn is_installed(name: &str) -> bool {
    let (exe_path, _) = find_installed_paths(name).unwrap_or((None, None));
    exe_path.is_some()
}
/// Finds the paths of both the installed binary and shim for a given package.
/// Returns a tuple of `Option<PathBuf>` for (binary, shim).
pub fn find_installed_paths(
    name: &str
) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    let cwd = std::env::current_dir()?;
    let exe_path = get_binary(name)?;
    let exe_path = match exe_path {
        Some(exe_path) => {
            exe_path
        },
        None => {
            return Ok((None, None));
        }
    };
    let exe_found = exe_path.exists();

    #[cfg(target_os = "windows")]
    let shim_path = cwd.join(".frate").join("shims")
        .join(name).with_extension("bat");
    #[cfg(not(target_os = "windows"))]
    let shim_path = cwd.join(".frate").join("shims").join(name);

    let shim_found = shim_path.exists();
    Ok((
        match exe_found {
            true => Some(exe_path),
            false => None,
        },
        match shim_found {
            true => Some(shim_path),
            false => None,
        }
    ))
}
/// Returns the full path to the `.frate` directory in the current working directory.
pub fn get_frate_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".frate"))
}
/// Returns the path to the `.frate/bin` directory.
pub fn get_frate_bin_dir() -> Result<PathBuf> {
    Ok(get_frate_dir()?.join("bin"))
}
/// Returns the path to the `.frate/shims` directory.
pub fn get_frate_shims_dir() -> Result<PathBuf> {
    Ok(get_frate_dir()?.join("shims"))
}
/// Returns the path to the `frate.lock` file in the current working directory.
pub fn get_frate_lock_file() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("frate.lock"))
}
/// Returns the path to the `frate.toml` file in the current working directory.
pub fn get_frate_toml() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("frate.toml"))
}
/// Sorts a map of version strings to `ReleaseInfo` entries in descending semver order.
/// Preserves any build or target-triple suffixes.
pub fn sort_versions(releases: HashMap<String, ReleaseInfo>) -> Vec<(String, ReleaseInfo)>{
    let mut versions: Vec<_> = releases.into_iter().collect();
    versions.sort_by(|(a, _), (b, _)| {
        let a = a.split('-').next().unwrap();
        let b = b.split('-').next().unwrap();
        Version::parse(a).unwrap().cmp(&Version::parse(b).unwrap())
    });
    versions
}
/// Validates whether a version string is a valid SemVer version.
/// Ignores build metadata and target suffixes.
pub fn is_valid_version(version: &str) -> bool {
    let version = version.split('-').next().unwrap();
    Version::parse(version).is_ok()
}
/// Searches for the binary file in the `.frate/bin/<name>` directory.
/// Picks the first executable that matches the tool name heuristically.
///
/// Returns an error if no suitable binary is found.
pub fn get_binary(name: &str) -> Result<Option<PathBuf>> {
    let path = get_frate_bin_dir()?.join(name);
    if !path.exists() {
        return Ok(None);
    }
    let entries = WalkDir::new(&path);
    let mut candidates = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type().is_file() && is_executable(path) {
            candidates.push(path.to_path_buf());
        }
    }

    if candidates.is_empty() {
        bail!("No executable found in '{}'", path.display());
    }

    let re = Regex::new(&format!(r"(?i)\b{}.*", regex::escape(name)))?;
    candidates.sort_by_key(|p| {
        let fname = p.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
        if re.is_match(&fname) {
            0
        } else {
            10
        }
    });
    Ok(Some(candidates.remove(0)))
}
/// Checks if a given path is an executable file on Unix.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
/// Checks if a given path has a Windows executable extension (.exe, .bat, .cmd).
#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        let ext = ext.to_ascii_lowercase();
        matches!(ext.as_str(), "exe" | "bat" | "cmd")
    } else {
        false
    }
}

/// Filters versions based on platform and architecture
pub fn filter_versions(versions: Vec<(String, ReleaseInfo)>) -> Vec<(String, ReleaseInfo)> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let mut filtered_versions = Vec::new();
    for version in versions {
        if version.0.contains(arch) && version.0.contains(os) {
            filtered_versions.push(version);
        }
    }
    filtered_versions
}

#[cfg(windows)]
pub fn is_power_shell() -> bool {
    std::env::var("PSModulePath").is_ok() ||
    std::env::var("PSVersionTable").is_ok() ||
    std::env::var("Pwsh").is_ok()
}

#[derive(serde::Deserialize)]
struct GitHubRepo {
    description: Option<String>,
}

pub fn fetch_description(url: &str) -> Result<Option<String>> {
    let api_url = convert_url_to_api_url(url)?;
    let client = Client::new();
    let resp = client
        .get(&api_url)
        .header("User-Agent", "frate")
        .send()?
        .error_for_status()?;

    let repo_data: GitHubRepo = resp.json()?;
    Ok(repo_data.description)
}

fn convert_url_to_api_url(url: &str) -> Result<String> {
    let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
    if parts.len() < 2 {
        bail!("Invalid URL: {}", url);
    }
    let owner = parts[parts.len() - 2];
    let name = parts[parts.len() - 1];

    let api_url = format!("https://api.github.com/repos/{}/{}", owner, name);
    Ok(api_url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_frate_dirs_creates_directories() {
        let dir = tempdir().unwrap();
        let path = ensure_frate_dirs(dir.path()).unwrap();

        assert!(path.exists());
        assert!(path.join("bin").exists());
        assert!(path.join("shims").exists());
    }

    #[test]
    fn test_format_hash_removes_prefix() {
        let input = "sha256:abcdef123456";
        let expected = "abcdef123456";
        assert_eq!(format_hash(input), expected);
    }

    #[test]
    fn test_format_hash_without_prefix() {
        let input = "abcdef123456";
        assert_eq!(format_hash(input), input);
    }

    #[test]
    fn test_expand_version_appends_triple() {
        let version = "1.2.3";
        let triple = current_target_triple();
        let expected = format!("1.2.3-{}", triple);
        assert_eq!(expand_version(version), expected);
    }

    #[test]
    fn test_is_valid_version_valid() {
        assert!(is_valid_version("1.2.3"));
        assert!(is_valid_version("1.2.3-alpha")); // suffix is ignored
    }

    #[test]
    fn test_is_valid_version_invalid() {
        assert!(!is_valid_version("1.2")); // incomplete semver
        assert!(!is_valid_version("not-a-version"));
    }

    use crate::ReleaseInfo;
    use std::collections::HashMap;

    #[test]
    fn test_sort_versions_orders_correctly() {
        let mut map = HashMap::new();
        map.insert("1.0.0".to_string(), ReleaseInfo::default());
        map.insert("1.2.0".to_string(), ReleaseInfo::default());
        map.insert("1.1.0".to_string(), ReleaseInfo::default());

        let sorted = sort_versions(map);
        let versions: Vec<_> = sorted.iter().map(|(v, _)| v.clone()).collect();

        assert_eq!(versions, vec!["1.0.0", "1.1.0", "1.2.0"]);
    }

    use crate::{FrateLock, LockedPackage};

    fn mock_lock() -> FrateLock {
        FrateLock {
            packages: vec![
                LockedPackage {
                    name: "tool-a".to_string(),
                    version: "1.0.0".to_string(),
                    source: "".to_string(),
                    hash: "".to_string(),
                },
                LockedPackage {
                    name: "tool-b".to_string(),
                    version: "2.0.0".to_string(),
                    source: "".to_string(),
                    hash: "".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_is_locked_true() {
        let lock = mock_lock();
        assert!(is_locked("tool-a", &lock));
    }

    #[test]
    fn test_is_locked_false() {
        let lock = mock_lock();
        assert!(!is_locked("tool-x", &lock));
    }

    #[test]
    fn test_get_locked_some() {
        let lock = mock_lock();
        let pkg = get_locked("tool-b", &lock);
        assert!(pkg.is_some());
        assert_eq!(pkg.unwrap().version, "2.0.0");
    }

    #[test]
    fn test_get_locked_none() {
        let lock = mock_lock();
        assert!(get_locked("unknown", &lock).is_none());
    }
}
