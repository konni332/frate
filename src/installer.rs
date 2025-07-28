use std::ffi::OsStr;
use std::io::{Cursor};
use std::path::{Path, PathBuf};
use crate::lock::{FrateLock, LockedPackage};
use crate::shims::create_shim;
use crate::util::{ensure_frate_dirs, get_frate_dir};
use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use sha2::Digest;
use verbosio::verbose;
use crate::{get_binary, is_cached};
use crate::global::cache::{cache_archive, get_cached_archive};

/// Installs all packages listed in the lockfile by downloading and extracting them
/// and creating executable shims in the `.frate/shims` directory.
///
/// # Arguments
///
/// * `lock` - Reference to the parsed `frate.lock` file containing resolved packages.
/// * `project_root` - Path to the root of the project where the `.frate` directory resides.
///
/// # Errors
///
/// Returns an error if any package fails to download, extract, or install properly.
pub fn install_packages<P: AsRef<Path>>(lock: &FrateLock, project_root: P) -> Result<()> {
    let frate_dir = ensure_frate_dirs(project_root)?;
    for package in &lock.packages {
        install_package(package, &frate_dir)?;
    }
    Ok(())
}
/// Installs a single package by downloading and extracting it into `.frate/bin/{name}`,
/// and creating a shim in `.frate/shims/{name}` pointing to the main binary.
///
/// # Arguments
///
/// * `package` - The locked package to install.
/// * `frate_dir` - Path to the `.frate` directory.
///
/// # Errors
///
/// Returns an error if the package cannot be downloaded, verified, extracted,
/// or if the shim cannot be created.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use frate::{install_package, LockedPackage};
///
/// let package = LockedPackage {
///     name: "example".to_string(),
///     version: "0.1.0".to_string(),
///     source: "https://example.com/example.zip".to_string(),
///     hash: "sha256:abc123...".to_string(),
/// };
/// let frate_dir = PathBuf::from(".frate");
/// install_package(&package, &frate_dir).unwrap();
/// ```
pub fn install_package(package: &LockedPackage, frate_dir: &Path) -> Result<()> {
    let bin_dir = frate_dir.join("bin");
    let shims_dir = frate_dir.join("shims");
    // install
    let url = &package.source;
    let dest_dir = bin_dir.join(&package.name);
    std::fs::create_dir_all(&dest_dir)?;
    if let Some(cached_path) = get_cached_archive(&package.source)? {
        extract_cached(cached_path, dest_dir, &package.hash)?;
    }
    else {
        download_and_extract(url, &dest_dir.to_string_lossy(), &package.hash)?;
    }
    // create shim
    let target_path = get_binary(&package.name)?
        .ok_or(anyhow!("Binary not found: {}", package.name))?;
    let shim_path = shims_dir.join(
        target_path
            .file_stem()
            .ok_or_else(|| anyhow!("Invalid file name: {}", target_path.display()))?
    );

    create_shim(target_path, shim_path)?;
    println!("   {} {}", "Installed".bold().green(), package.name);
    Ok(())
}
/// Uninstalls all installed packages by removing `.frate/bin` and `.frate/shims` directories
/// and recreating them empty.
///
/// # Errors
///
/// Returns an error if the directories cannot be removed or recreated.
pub fn uninstall_packages() -> Result<()> {
    println!("{} {}", "Uninstalling".bold().yellow(), "all packages");
    let frate_dir = get_frate_dir()?;

    std::fs::remove_dir_all(frate_dir.join("bin"))?;
    std::fs::remove_dir_all(frate_dir.join("shims"))?;

    std::fs::create_dir_all(frate_dir.join("bin"))?;
    std::fs::create_dir_all(frate_dir.join("shims"))?;
    println!("        {}", "Done".bold().green());
    Ok(())
}
/// Uninstalls a single package by removing its directory under `.frate/bin/{name}`
/// and deleting its corresponding shim in `.frate/shims/{name}`.
///
/// # Arguments
///
/// * `name` - Name of the package to uninstall.
///
/// # Errors
///
/// Returns an error if any part of the uninstallation fails.
///
/// # Example
///
/// ```no_run
/// use frate::uninstall_package;
///
/// uninstall_package("example").unwrap();
/// ```
pub fn uninstall_package(name: &str) -> Result<()> {
    println!("{} {}", "Uninstalling".bold().yellow(), name);
    let cwd = std::env::current_dir()?;
    let frate_dir = cwd.join(".frate");
    let bin_dir = frate_dir.join("bin");
    let shims_dir = frate_dir.join("shims");
    #[cfg(target_os = "windows")]
    {
        let bin_path = bin_dir.join(name);
        if bin_path.exists() {
            std::fs::remove_dir_all(bin_path)?;
        }
        let shim_path = shims_dir.join(format!("{name}.bat"));
        if shim_path.exists() {
            std::fs::remove_file(shim_path)?;
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let bin_path = bin_dir.join(name);
        if bin_path.exists() {
            std::fs::remove_dir_all(bin_path)?;
        }
        let shim_path = shims_dir.join(name);
        if shim_path.exists() {
            std::fs::remove_file(shim_path)?;
        }
    }
    println!("        {}", "Done".bold().green());
    Ok(())
}
/// Downloads an archive from a given URL, verifies its SHA-256 hash, and extracts it to the given directory.
/// Supports `.zip` and `.tar.gz` archives.
///
/// # Arguments
///
/// * `url` - The URL of the archive to download.
/// * `dest_dir` - Target directory for extraction.
/// * `expected_hash` - Expected SHA-256 hash (hex-encoded) to verify integrity.
///
/// # Errors
///
/// Returns an error if:
/// - the download fails,
/// - the hash doesn't match,
/// - the archive type is unsupported,
/// - or extraction fails.
pub fn download_and_extract(url: &str, dest_dir: &str, expected_hash: &str) -> Result<()> {
    let expected_hash = crate::util::format_hash(expected_hash);
    println!(" {} {}", "Downloading".bold().green(), url);
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        bail!(" {} {}: {}", "Failed to download".bold().red(), url, response.status());
    }
    let bytes = response.bytes()?;

    // Check hash
    let mut hasher = sha2::Sha256::new();
    hasher.update(&bytes);
    let actual_hash = hex::encode(hasher.finalize());

    if actual_hash != expected_hash {
        bail!(" {}\n  expected: {}\n  got: {}", "Hash mismatch:".bold().red(), expected_hash, actual_hash);
    }

    println!("  {} {} to {}", "Extracting".bold().green(), url, dest_dir);
    if url.ends_with(".zip") {
        let reader = Cursor::new(&bytes);
        let mut zip = zip::ZipArchive::new(reader)?;
        zip.extract(dest_dir)?;
    }
    else if url.ends_with(".tar.gz") {
        let tar = flate2::read::GzDecoder::new(Cursor::new(&bytes));
        let mut archive = tar::Archive::new(tar);
        archive.unpack(dest_dir)?;
    }
    else {
        bail!("Unsupported archive type: {}", url.split(crate::util::PATH_SEPARATOR).last().unwrap_or(url));
    }
    if !is_cached(url)? {
        println!("     {}", "Caching".bold().green());
        cache_archive(url, bytes.as_ref())?;
    }
    Ok(())
}

pub fn extract_cached<P: AsRef<Path>>(
    cached_path: P,
    dest_dir: P,
    expected_hash: &str
) -> Result<()> {
    let expected_hash = crate::util::format_hash(expected_hash);
    let archive_bytes = std::fs::read(&cached_path)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&archive_bytes);
    let actual_hash = hex::encode(hasher.finalize());
    if actual_hash != expected_hash {
        bail!(
            " {}\n  expected: {}\n  got: {}\n  for: {}",
            "Hash mismatch:".bold().red(),
            expected_hash,
            actual_hash,
            cached_path.as_ref().display()
        );
    }
    let cached_path_str = cached_path.as_ref().to_string_lossy();
    verbose!("  {} FROM CACHE {} to {}", "Extracting".bold().green(), cached_path.as_ref().display(), dest_dir.as_ref().display());
    if cached_path_str.ends_with(".zip") {
        let reader = Cursor::new(archive_bytes);
        let mut zip = zip::ZipArchive::new(reader)?;
        zip.extract(dest_dir)?;
    }
    else if cached_path_str.ends_with(".tar.gz") {
        let tar = flate2::read::GzDecoder::new(Cursor::new(archive_bytes));
        let mut archive = tar::Archive::new(tar);
        archive.unpack(dest_dir)?;
    }
    else {
        bail!("Unsupported archive type: {}",
            dest_dir
            .as_ref()
            .display()
            .to_string()
            .split(crate::util::PATH_SEPARATOR)
            .last()
            .unwrap_or(
                dest_dir.as_ref().display()
                .to_string().split('.').next_back()
                .unwrap_or(dest_dir.as_ref().display().to_string().as_str())
            )
        );
    }
    Ok(())
}

