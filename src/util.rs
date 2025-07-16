use std::env::consts::EXE_EXTENSION;
use std::io::{Cursor};
use std::path::{Path, PathBuf};
use sha2::Digest;
use hex;
use crate::lock::{FrateLock, LockedPackage};

pub fn ensure_frate_dirs<P: AsRef<Path>>(root: P) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(root.as_ref());
    path.push(".frate");
    std::fs::create_dir_all(&path)?;
    let bin_path = path.join("bin");
    std::fs::create_dir_all(&bin_path)?;
    let shims_path = path.join("shims");
    std::fs::create_dir_all(&shims_path)?;
    Ok(path)
}

pub fn download_and_extract(url: &str, dest_dir: &str, expected_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let expected_hash = format_hash(expected_hash);
    println!("Downloading {}", url);
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to download {}: {}", url, response.status()).into());
    }
    let bytes = response.bytes()?;

    // Check hash
    let mut hasher = sha2::Sha256::new();
    hasher.update(&bytes);
    let actual_hash = hex::encode(hasher.finalize());

    if actual_hash != expected_hash {
        return Err(format!(
            "Hash mismatch for {}: expected {}, got {}",
            url,
            expected_hash,
            actual_hash
        ).into());
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
        return Err("Unsupported archive format".into());
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
    for package in &lock.package {
        if package.name == name {
            return true;
        }
    }
    false
}
pub fn get_locked(name: &str, lock: &FrateLock) -> Option<LockedPackage> {
    for package in &lock.package {
        if package.name == name {
            return Some(package.clone());
        }
    }
    None
}

pub fn is_installed(name: &str) -> bool {
    let path = std::env::current_dir().expect("Failed to get current directory")
        .join(".frate").join("bin").join(name).join(name).with_extension(EXE_EXTENSION);
    path.exists()
}

pub fn find_installed_paths(
    name: &str
) -> Result<(Option<PathBuf>, Option<PathBuf>), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let exe_path = cwd
        .join(".frate")
        .join("bin")
        .join(name)
        .join(name)
        .with_extension(EXE_EXTENSION);
    let exe_found = exe_path.exists();

    #[cfg(target_os = "windows")]
    let shim_path = cwd.join(".frate").join("shims").join(name).with_extension("bat");
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