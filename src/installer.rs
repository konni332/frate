use std::path::Path;
use crate::lock::FrateLock;
use crate::shims::create_shim;
use crate::util::{download_and_extract, ensure_frate_dirs};

#[cfg(windows)]
const EXEC_EXT: &str = "exe";
#[cfg(not(windows))]
const EXEC_EXT: &str = "";

pub fn install_packages<P: AsRef<Path>>(lock: &FrateLock, project_root: P) -> Result<(), Box<dyn std::error::Error>> {
    let frate_dir = ensure_frate_dirs(project_root)?;
    let bin_dir = frate_dir.join("bin");
    let shims_dir = frate_dir.join("shims");
    for package in &lock.package {
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
    }
    Ok(())
}