use std::path::PathBuf;
use anyhow::{anyhow, Result};
use directories::ProjectDirs;

pub fn get_global_config_dir() -> Result<PathBuf> {
    let (config_dir, _, _) = get_global_dirs()?;
    Ok(config_dir)
}

pub fn get_global_cache_dir() -> Result<PathBuf> {
    let (_, cache_dir, _) = get_global_dirs()?;
    Ok(cache_dir)
}

pub fn get_global_data_dir() -> Result<PathBuf> {
    let (_, _, data_dir) = get_global_dirs()?;
    Ok(data_dir)
}

pub fn get_global_dirs() -> Result<(PathBuf, PathBuf, PathBuf)> {
    let proj_dirs = ProjectDirs::from("org", "frate", "frate")
        .ok_or_else(|| anyhow!("Could not get project directories"))?;

    let config_dir = proj_dirs.config_dir().to_path_buf();
    let cache_dir = proj_dirs.cache_dir().to_path_buf();
    let data_dir = proj_dirs.data_dir().to_path_buf();

    Ok((config_dir, cache_dir, data_dir ))
}