use tempfile::TempDir;
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
    use frate::installer::install_packages;
    use frate::lock::FrateLock;
    use frate::toml::FrateToml;
    use crate::setup_tests;
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
        {
            let shim_path = dir.path().join(".frate").join("shims").join("just.bat");
            assert!(shim_path.exists());
            let content = std::fs::read_to_string(&shim_path).unwrap();
            println!("{}", &content);
            println!("{}", &shim_path.to_str().unwrap());
            let mut cmd = std::process::Command::new("cmd");
            cmd.args(&["/C", shim_path.to_str().unwrap(), "--version"]);
            println!("{:?}", cmd);
            let output = cmd.output().expect("failed to execute process");
            println!("{:?}", output);
            assert!(output.status.success());
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let shim_path = dir.path().join(".frate").join("shims").join("just");
            assert!(shim_path.exists());

            let metadata = std::fs::metadata(&shim_path).expect("failed to get metadata");
            let permissions = metadata.permissions();
            assert!(permissions.mode() & 0o111 != 0, "shim is not executable");

            let output = std::process::Command::new(&shim_path)
                .arg("--version")
                .output()
                .expect("Failed to execute shim");
            assert!(output.status.success());
        }
        assert!(dir.path().exists())
    }
}