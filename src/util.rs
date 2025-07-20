use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::lock::{FrateLock, LockedPackage};
use anyhow::{bail, Result};
use regex::Regex;
use semver::Version;
use walkdir::WalkDir;
use crate::registry::ReleaseInfo;
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
    if hash.starts_with("sha256:") {
        hash[7..].to_string()
    }
    else {
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
pub fn get_binary(name: &str) -> Result<PathBuf> {
    let path = get_frate_bin_dir()?.join(name);
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
    Ok(candidates.remove(0))
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

#[cfg(windows)]
pub fn is_power_shell() -> bool {
    std::env::var("PSModulePath").is_ok() ||
    std::env::var("PSVersionTable").is_ok() ||
    std::env::var("Pwsh").is_ok()
}