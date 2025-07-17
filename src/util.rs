use std::collections::HashMap;
use std::io::{Cursor};
use std::path::{Path, PathBuf};
use sha2::Digest;
use hex;
use crate::lock::{FrateLock, LockedPackage};
use anyhow::{bail, Result};
use regex::Regex;
use semver::Version;
use walkdir::WalkDir;
use crate::registry::ReleaseInfo;

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

pub fn download_and_extract(url: &str, dest_dir: &str, expected_hash: &str) -> Result<()> {
    let expected_hash = format_hash(expected_hash);
    println!("Downloading {}", url);
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        bail!("Failed to download {}: {}", url, response.status());
    }
    let bytes = response.bytes()?;

    // Check hash
    let mut hasher = sha2::Sha256::new();
    hasher.update(&bytes);
    let actual_hash = hex::encode(hasher.finalize());

    if actual_hash != expected_hash {
        bail!("Hash mismatch: expected {}, got {}", expected_hash, actual_hash);
    }

    println!("Extracting {} to {}", url, dest_dir);
    if url.ends_with(".zip") {
        let reader = Cursor::new(bytes);
        let mut zip = zip::ZipArchive::new(reader)?;
        zip.extract(dest_dir)?;
    }
    else if url.ends_with(".tar.gz") {
        let tar = flate2::read::GzDecoder::new(Cursor::new(bytes));
        let mut archive = tar::Archive::new(tar);
        archive.unpack(dest_dir)?;
    }
    else {
        bail!("Unsupported archive type");
    }
    Ok(())
}

fn format_hash(hash: &str) -> String {
    if hash.starts_with("sha256:") {
        hash[7..].to_string()
    }
    else {
        hash.to_string()
    }
}

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

pub fn expand_version(version: &str) -> String {
    let triple = current_target_triple();
    format!("{}-{}", version, triple)

}

pub fn is_locked(name: &str, lock: &FrateLock) -> bool {
    for package in &lock.packages {
        if package.name == name {
            return true;
        }
    }
    false
}
pub fn get_locked(name: &str, lock: &FrateLock) -> Option<LockedPackage> {
    for package in &lock.packages {
        if package.name == name {
            return Some(package.clone());
        }
    }
    None
}

pub fn is_installed(name: &str) -> bool {
    let (exe_path, _) = find_installed_paths(name).unwrap();
    exe_path.is_some()
}

pub fn find_installed_paths(
    name: &str
) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    let cwd = std::env::current_dir()?;
    let exe_path = get_binary(name)?;
    dbg!(&exe_path);
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

pub fn get_frate_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".frate"))
}
pub fn get_frate_bin_dir() -> Result<PathBuf> {
    Ok(get_frate_dir()?.join("bin"))
}
pub fn get_frate_shims_dir() -> Result<PathBuf> {
    Ok(get_frate_dir()?.join("shims"))
}
pub fn get_frate_lock_file() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("frate.lock"))
}
pub fn get_frate_toml() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("frate.toml"))
}

pub fn sort_versions(releases: HashMap<String, ReleaseInfo>) -> Vec<(String, ReleaseInfo)>{
    let mut versions: Vec<_> = releases.into_iter().collect();
    versions.sort_by(|(a, _), (b, _)| {
        let a = a.split('-').next().unwrap();
        let b = b.split('-').next().unwrap();
        Version::parse(a).unwrap().cmp(&Version::parse(b).unwrap())
    });
    versions
}

pub fn is_valid_version(version: &str) -> bool {
    let version = version.split('-').next().unwrap();
    Version::parse(version).is_ok()
}
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
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        let ext = ext.to_ascii_lowercase();
        matches!(ext.as_str(), "exe" | "bat" | "cmd")
    } else {
        false
    }
}
