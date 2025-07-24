use tempfile::TempDir;
use frate::toml::FrateToml;


fn setup_tests() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let mut toml = FrateToml::default(&temp_dir.path().file_name().unwrap().to_string_lossy());
    toml.dependencies.insert("just".to_string(), "1.42.1".to_string());
    toml.save(temp_dir.path().join("frate.toml").to_str().unwrap()).unwrap();
    temp_dir
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use frate::installer::install_packages;
    use frate::lock::FrateLock;
    use frate::{get_binary};
    use frate::toml::FrateToml;
    use crate::setup_tests;

    #[test]
    fn test_sync_lock() {
        let dir = setup_tests();
        let toml_path = dir.path().join("frate.toml");

        // Load frate.toml
        let toml = FrateToml::load(toml_path.to_str().unwrap()).expect("frate.toml not found");
        assert_eq!(toml.dependencies.len(), 1);

        // Sync lockfile
        let mut lock = FrateLock::load_or_default(dir.path().join("frate.lock"));
        lock.sync(&toml).unwrap();
        assert_eq!(lock.packages.len(), 1);

        // Save and assert
        lock.save(dir.path().join("frate.lock")).unwrap();
        assert!(dir.path().join("frate.lock").exists());
    }

    #[cfg(not(ci_skip))]
    #[test]
    fn test_install_packages() {
        let dir = setup_tests();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let toml_path = dir.path().join("frate.toml");
        let toml = FrateToml::load(toml_path.to_str().unwrap()).expect("frate.toml not found");

        // Lockfile sync + install
        let mut lock = FrateLock::load_or_default(dir.path().join("frate.lock"));
        lock.sync(&toml).unwrap();
        lock.save(dir.path().join("frate.lock")).unwrap();
        install_packages(&lock, dir.path()).unwrap();

        // Check binary existence
        assert!(get_binary("just").expect("Binary not found").exists());
        std::env::set_current_dir(old_cwd).unwrap();
    }

    #[cfg(not(ci_skip))]
    #[test]
    fn test_shims() {
        let dir = setup_tests();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let toml_path = dir.path().join("frate.toml");
        let toml = FrateToml::load(toml_path.to_str().unwrap()).expect("frate.toml not found");

        // Lock + Install
        let mut lock = FrateLock::load_or_default(dir.path().join("frate.lock"));
        lock.sync(&toml).unwrap();
        lock.save(dir.path().join("frate.lock")).unwrap();
        install_packages(&lock, dir.path()).unwrap();

        #[cfg(target_os = "windows")]
        {
            // Check .bat shim
            let shim_path = dir.path().join(".frate").join("shims").join("just.bat");
            assert!(shim_path.exists(), "Shim file not found: {:?}", shim_path);

            // Run shim
            let output = std::process::Command::new("cmd")
                .args(&["/C", shim_path.to_str().unwrap(), "--version"])
                .output()
                .expect("failed to execute shim");

            assert!(output.status.success(), "Shim execution failed");
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Check shim file
            let shim_path = dir.path().join(".frate").join("shims").join("just");
            assert!(shim_path.exists(), "Shim file not found: {:?}", shim_path);

            // Ensure executable
            let metadata = std::fs::metadata(&shim_path).expect("failed to get metadata");
            let permissions = metadata.permissions();
            assert!(
                permissions.mode() & 0o111 != 0,
                "Shim is not marked executable"
            );

            // Run shim
            let output = std::process::Command::new(&shim_path)
                .arg("--version")
                .output()
                .expect("Failed to execute shim");

            assert!(output.status.success(), "Shim execution failed");
        }
        std::env::set_current_dir(old_cwd).unwrap();
    }

    #[test]
    fn test_load_or_default_fallback() {
        // No frate.lock should lead to empty FrateLock
        let dir = TempDir::new().unwrap();
        let lock = FrateLock::load_or_default(dir.path().join("frate.lock"));
        assert_eq!(lock.packages.len(), 0);
    }
}
