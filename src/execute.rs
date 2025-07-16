use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{bail, Result};
use frate::installer::{install_package, install_packages, uninstall_package, uninstall_packages};
use frate::lock::FrateLock;
use frate::toml::FrateToml;
use frate::util::{ensure_frate_dirs, find_installed_paths, get_locked, is_installed};
use crate::cli::{FrateCommand, CLI};

pub fn execute(cli: CLI) -> Result<()> {
    if cli.command != FrateCommand::Init {
        let toml_path = std::env::current_dir()?.join("frate.toml");
        if !toml_path.exists() {
            bail!("frate.toml not found. Run `frate init` to create one.")
        }
    }
    match cli.command {
        FrateCommand::List { verbose } => {
            execute_list(verbose)
        }
        FrateCommand::Init => {
            execute_init()
        },
        FrateCommand::Sync => {
            execute_sync()
        }
        FrateCommand::Install { name } => {
            execute_install(name)
        }
        FrateCommand::Uninstall { name } => {
            execute_uninstall(name)
        }
        FrateCommand::Which { name } => {
            let _ = execute_which(&name)?;
            Ok(())
        }
        FrateCommand::Run { name, args} => {
            execute_run(&name, args)
        }
        _ => {
            Ok(())
        }
    }
}



pub fn execute_list(verbose: bool) -> Result<()> {
    let toml_path = std::env::current_dir()?.join("frate.toml");
    let toml_str = std::fs::read_to_string(toml_path)?;
    let toml: FrateToml = toml::from_str(&toml_str)?;
    let lock_path = std::env::current_dir()?.join("frate.lock");
    let lock: Option<FrateLock> = if lock_path.exists() {
        let lock_str = std::fs::read_to_string(lock_path)?;
        Some(toml::from_str(&lock_str)?)
    }
    else {
        None
    };

    if toml.dependencies.is_empty() {
        println!("No dependencies");
        return Ok(());
    }

    for (name, version) in &toml.dependencies {
        println!("{}: {}", name, version);
        match &lock {
            Some(lock) => {
                let locked = get_locked(name, &lock);
                match locked {
                    Some(locked) => {
                        match verbose {
                            true => {
                                println!("   locked");
                            },
                            false => {
                                println!("   locked at: {}", locked.version);
                                println!("  # hash: {}", locked.hash);
                                println!("  source: {}", locked.source);
                            }
                        }
                    },
                    None => {
                        println!("   unlocked");
                    }
                }
                match is_installed(name) {
                    true => {
                        println!("   installed");
                    },
                    false => {
                        println!("   not installed");
                    },
                }

            }
            None => {}
        }
        println!();
    }
    Ok(())
}

pub fn execute_init() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let name = cwd.file_name().ok_or(anyhow::anyhow!("Could not get file name"))?
        .to_str().ok_or(anyhow::anyhow!("Invalid directory name"))?;
    let _ = ensure_frate_dirs(&cwd)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    let toml = FrateToml::default(name);
    toml.save(cwd.join("frate.toml")).map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}

pub fn execute_sync() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let toml_str = std::fs::read_to_string(cwd.join("frate.toml"))?;
    let toml: FrateToml = toml::from_str(&toml_str)?;
    let mut lock = FrateLock::load_or_default(cwd.join("frate.lock"));
    let _ = lock.sync(&toml);
    lock.save(cwd.join("frate.lock"))?;
    Ok(())
}

pub fn execute_install(name: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let lock = FrateLock::load_or_default(cwd.join("frate.lock"));
    match name {
        Some(name) => {
            let package = get_locked(&name, &lock)
                .ok_or(anyhow::anyhow!("Package not found: {}", name))?;
            install_package(&package, &cwd)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        }
        None => {
            install_packages(&lock, &cwd)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        }
    }
    Ok(())
}

pub fn execute_uninstall(name: Option<String>) -> Result<()> {
    match name {
        Some(name) => {
            uninstall_package(&name)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        }
        None => {
            uninstall_packages()
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        }
    }
    Ok(())
}

pub fn execute_which(name: &str) -> Result<()> {
    let (exe_path, shim_path) = find_installed_paths(name)?;
    if exe_path.is_none() && shim_path.is_none() {
        println!("No installed paths found");
        return Ok(());
    }
    match exe_path {
        Some(exe_path) => {
            println!("Found executable at: {}", exe_path.display());
        }
        None => {}
    }
    match shim_path {
        Some(shim_path) => {
            println!("Found shim at: {}", shim_path.display());
        }
        None => {}
    }
    Ok(())
}

pub fn execute_run(name: &str, args: Vec<String>) -> Result<()> {
    let (exe_path, _) = find_installed_paths(&name)?;
    let exe_path = match exe_path {
        Some(exe_path) => {
            exe_path
        }
        None => {
            return Ok(())
        }
    };
    let output = Command::new(exe_path)
        .args(args).output()?;
    if !output.status.success() {
        bail!("{}", String::from_utf8(output.stderr)?);
    }
    println!("{}", String::from_utf8(output.stdout)?);
    Ok(())
}