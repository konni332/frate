use tempfile::TempDir;
use frate::*;
use frate::toml::FrateToml;

fn setup_tests() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let mut toml = FrateToml::default(&temp_dir.path().file_name().unwrap().to_string_lossy().to_string());
    toml.dependencies.insert("just".to_string(), "1.42.1".to_string());
    toml.save(temp_dir.path().join("frate.toml").to_str().unwrap()).unwrap();
    temp_dir
}


#[cfg(test)]
mod tests {
    use std::fs;
    use frate::installer::install_packages;
    use frate::lock::FrateLock;
    use frate::toml::FrateToml;
    use crate::setup_tests;
    #[test]
    fn test_setup() {
        let dir = setup_tests();
        assert!(dir.path().exists());
        let toml_path = dir.path().join("frate.toml");
        assert!(toml_path.exists());
        let toml = FrateToml::load(toml_path.to_str().unwrap()).unwrap();
    }

    #[test]
    fn test_sync_lock() {
        let dir = setup_tests();
        let toml_path = dir.path().join("frate.toml");
        let toml = FrateToml::load(toml_path.to_str().unwrap()).expect("frate.toml not found");
        let mut lock = FrateLock::load_or_default(dir.path().join("frate.lock"));
        lock.sync(&toml).unwrap();
        lock.save(dir.path().join("frate.lock")).unwrap();
        assert!(dir.path().join("frate.lock").exists());
    }
    #[test]
    fn test_install_packages() {
        let dir = setup_tests();
        let toml_path = dir.path().join("frate.toml");
        let toml = FrateToml::load(toml_path.to_str().unwrap()).expect("frate.toml not found");
        let mut lock = FrateLock::load_or_default(dir.path().join("frate.lock"));
        lock.sync(&toml).unwrap();
        lock.save(dir.path().join("frate.lock")).unwrap();
        assert!(dir.path().join("frate.lock").exists());
        install_packages(&lock, dir.path()).unwrap();
        #[cfg(target_os = "windows")]
        assert!(dir.path().join(".frate").join("bin").join("just").join("just.exe").exists());
        #[cfg(target_os = "linux")]
        assert!(dir.path().join(".frate").join("bin").join("just").join("just").exists())
    }

    #[test]
    fn test_shims() {
        let dir = setup_tests();
        let toml_path = dir.path().join("frate.toml");
        let toml = FrateToml::load(toml_path.to_str().unwrap()).expect("frate.toml not found");
        let mut lock = FrateLock::load_or_default(dir.path().join("frate.lock"));
        lock.sync(&toml).unwrap();
        lock.save(dir.path().join("frate.lock")).unwrap();
        assert!(dir.path().join("frate.lock").exists());
        install_packages(&lock, dir.path()).unwrap();

        #[cfg(target_os = "windows")]
        assert!(dir.path().join(".frate").join("shims").join("just.bat").exists());
        #[cfg(target_os = "linux")]
        assert!(dir.path().join(".frate").join("shims").join("just").exists())
    }
}