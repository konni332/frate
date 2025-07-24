use std::path::PathBuf;
use crate::global::utils::get_global_cache_dir;
use anyhow::{anyhow, bail, Context, Result};
use walkdir::WalkDir;

pub fn get_cached_archive(url: &str) -> Result<Option<PathBuf>> {
    let cache_dir = get_global_cache_dir()?;
    let file_name = url.split('/').next_back().ok_or(anyhow!("Could not determine archive name"))?;
    let archive_path = cache_dir.join(file_name);
    if archive_path.exists() {
        Ok(Some(archive_path))
    }
    else {
        Ok(None)
    }
}

pub fn cache_archive(url: &str, bytes: &[u8]) -> Result<()>{
    let cache_dir = get_global_cache_dir()?;
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Could not create cache dir {:?}", cache_dir))?;
        println!("Cache directory created: {}", cache_dir.display());
    }
    let file_name = url.split('/').next_back().ok_or(anyhow!("Could not determine archive name"))?;
    let path = cache_dir.join(file_name);
    std::fs::File::create(&path)
        .with_context(|| format!("Could not create cache file {:?}", path))?;
    std::fs::write(&path, bytes)?;
    Ok(())
}

pub fn clean_cache() -> Result<()> {
    let cache_dir = get_global_cache_dir()?;
    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir)?;
    }
    std::fs::create_dir_all(&cache_dir)?;
    Ok(())
}

pub fn remove_cached_archive(name: &str) -> Result<()> {
    let cache_dir = get_global_cache_dir()?;
    if !cache_dir.exists() {
        bail!("Cache directory does not exist");
    }
    let dir = WalkDir::new(&cache_dir);
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if path.to_string_lossy().contains(name) {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub fn is_cached(full_name: &str) -> Result<bool> {
    let cache_dir = get_global_cache_dir()?;
    if !cache_dir.exists() {
        return Ok(false);
    }
    let dir = WalkDir::new(&cache_dir);
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if path.to_string_lossy().contains(full_name) {
            return Ok(true);
        }
    }
    Ok(false)
}