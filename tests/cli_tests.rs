use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_execute_init_creates_frate_toml() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let mut cmd = Command::cargo_bin("frate").unwrap();
    cmd.current_dir(dir_path)
        .arg("init")
        .assert()
        .success();

    let toml_path = dir_path.join("frate.toml");
    assert!(toml_path.exists());
    let content = fs::read_to_string(toml_path).unwrap();
    assert!(content.contains("[dependencies]"));
}

#[test]
fn test_execute_add_and_list() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Init
    Command::cargo_bin("frate").unwrap()
        .current_dir(&dir_path)
        .arg("init")
        .assert()
        .success();

    // Add dep
    Command::cargo_bin("frate").unwrap()
        .current_dir(&dir_path)
        .args(&["add", "hello@1.0.0"])
        .assert()
        .success();

    // List
    let output = Command::cargo_bin("frate").unwrap()
        .current_dir(&dir_path)
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    assert!(output_str.contains("hello: 1.0.0"));
}
#[cfg(test)]
mod cli_integration_tests {
    use assert_cmd::Command;
    use tempfile::tempdir;
    use frate::FrateToml;
    // Voraussetzung: Du hast im Cargo.toml dev-deps: assert_cmd, predicates, tempfile

    #[test]
    fn test_execute_sync() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();
        let toml = FrateToml::default("tests");
        toml.save(dir_path.join("frate.toml")).unwrap();

        // Sync (setzt frate.lock)
        Command::cargo_bin("frate").unwrap()
            .current_dir(dir_path)
            .arg("sync")
            .assert()
            .success();

        // Prüfen, dass frate.lock jetzt existiert
        assert!(dir_path.join("frate.lock").exists());
    }

    #[test]
    fn test_execute_add_and_list() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // Init (wenn dein CLI sowas hat, sonst erstelle frate.toml)
        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .arg("init")
            .assert()
            .success();

        // Add dependency
        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .args(&["add", "hello@1.0.0"])
            .assert()
            .success();

        // List dependencies (oder wie deine CLI es nennt)
        let output = Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .arg("list")
            .assert()
            .success()
            .get_output()
            .stdout.clone();

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("hello: 1.0.0"));
    }

    #[test]
    fn test_execute_install_and_uninstall_and_run_and_which() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();
        let toml = FrateToml::default("tests");
        toml.save(dir_path.join("frate.toml")).unwrap();
        // Init und Add für Setup
        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .arg("init")
            .assert()
            .success();

        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .args(&["add", "just@1.42.1"])
            .assert()
            .success();

        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .arg("sync")
            .assert()
            .success();

        // Install specific package
        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .args(&["install", "--name","just"])
            .assert()
            .success();

        // Uninstall specific package
        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .args(&["uninstall", "--name", "just"])
            .assert()
            .success();

        // Install all packages
        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .arg("install")
            .assert()
            .success();

        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .args(&["run", "just", "--", "--version"])
            .assert()
            .success();

        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .arg("which")
            .arg("just")
            .assert()
            .success();

        // Uninstall all packages
        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .arg("uninstall")
            .assert()
            .success();


    }

    #[test]
    fn test_execute_search() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .args(&["search", "ripgrep"])
            .assert()
            .success();
    }

    #[test]
    fn test_execute_shell() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        Command::cargo_bin("frate").unwrap()
            .current_dir(&dir_path)
            .arg("shell")
            .assert()
            .success();
    }
}
