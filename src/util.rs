use std::fs::File;
use std::io::{BufReader, Bytes, Cursor, Read};
use std::path::{Path, PathBuf};
use sha2::Digest;
use hex;
use crate::registry::RegistryTool;

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

