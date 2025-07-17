use std::path::{Path, PathBuf};
use crate::lock::{FrateLock, LockedPackage};
use crate::shims::create_shim;
use crate::util::{download_and_extract, ensure_frate_dirs, get_frate_dir};
use anyhow::Result;

#[cfg(windows)]
const EXEC_EXT: &str = "exe";
#[cfg(not(windows))]
const EXEC_EXT: &str = "";

pub fn install_packages<P: AsRef<Path>>(lock: &FrateLock, project_root: P) -> Result<()> {
    let frate_dir = ensure_frate_dirs(project_root)?;
    for package in &lock.package {
        install_package(package, &frate_dir)?;
    }
    Ok(())
}

pub fn install_package(package: &LockedPackage, frate_dir: &Path) -> Result<()> {
    let bin_dir = frate_dir.join("bin");
    let shims_dir = frate_dir.join("shims");
    // install
    let url = &package.source;
    let dest_dir = bin_dir.join(&package.name);
    std::fs::create_dir_all(&dest_dir)?;
    download_and_extract(url, &dest_dir.to_string_lossy().to_string(), &package.hash)?;
    // create shim
    let shim_path = shims_dir.join(&package.name);
    let target_path = dest_dir
        .join(&package.name)
        .with_extension(EXEC_EXT);
    create_shim(target_path, shim_path)?;
    Ok(())
}

pub fn uninstall_packages() -> Result<()> {
    println!("Uninstalling all packages");
    let frate_dir = get_frate_dir()?;

    std::fs::remove_dir_all(&frate_dir.join("bin"))?;
    std::fs::remove_dir_all(&frate_dir.join("shims"))?;

    std::fs::create_dir_all(&frate_dir.join("bin"))?;
    std::fs::create_dir_all(&frate_dir.join("shims"))?;
    println!("Done");
    Ok(())
}

pub fn uninstall_package(name: &str) -> Result<()> {
    println!("Uninstalling {}", name);
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
    println!("Done");
    Ok(())
}