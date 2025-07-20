use std::process::Command;
use anyhow::{bail, Context, Result};
use verbosio::{set_verbosity, verbose};
use frate::installer::{install_package, install_packages, uninstall_package, uninstall_packages};
use frate::lock::FrateLock;
use frate::registry::fetch_registry;
use frate::shims::{run_shell_with_frate_path, write_windows_activate};
#[cfg(unix)]
use frate::shims::{write_unix_activate};
use frate::toml::FrateToml;
use frate::util::{ensure_frate_dirs, find_installed_paths, get_frate_toml, get_locked, is_installed, sort_versions};
use crate::cli::{FrateCommand, CLI};

/// Executes the given CLI command.
///
/// # Errors
/// Returns an error if command execution fails or required files are missing.
pub fn execute(cli: CLI) -> Result<()> {
    if cli.command != FrateCommand::Init {
        let toml_path = get_frate_toml()?;
        if !toml_path.exists() {
            bail!("frate.toml not found. Run `frate init` to create one.")
        }
    }
    match cli.command {
        FrateCommand::List { verbose } => {
            if verbose {
                set_verbosity!()
            }
            execute_list()
        },
        FrateCommand::Shell => {
            set_verbosity!();
            execute_shell()
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
        FrateCommand::Add { name_at_version } => {
            execute_add(name_at_version)
        }
        FrateCommand::Search { name } => {
            execute_search(name)
        }
        _ => {
            Ok(())
        }
    }
}


/// Lists all dependencies from `frate.toml` and their status.
///
/// # Arguments
/// * `verbose` - If true, shows detailed information.
///
/// # Errors
/// Returns an error if reading or parsing the manifest or lock file fails.
pub fn execute_list() -> Result<()> {
    let toml_path = get_frate_toml()?;
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
                        println!("   locked");
                        verbose!(@lvl 1, "   locked at: {}", locked.version);
                        verbose!(@lvl 1, "   hash: {}", locked.hash);
                        verbose!(@lvl 1, "  󰳏 source: {}", locked.source);
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
/// Initializes a new `frate.toml` in the current directory.
///
/// # Errors
/// Returns an error if the current directory name cannot be determined or file operations fail.
pub fn execute_init() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let name = cwd.file_name().ok_or(anyhow::anyhow!("Could not get file name"))?
        .to_str().ok_or(anyhow::anyhow!("Invalid directory name"))?;
    let _ = ensure_frate_dirs(&cwd)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    let toml = FrateToml::default(name);
    toml.save(cwd.join("frate.toml")).map_err(|e| anyhow::anyhow!("{:?}", e))?;

    #[cfg(windows)]
    write_windows_activate()?;
    #[cfg(not(windows))]
    write_unix_activate()?;
    Ok(())
}
/// Synchronizes the `frate.lock` file with the current `frate.toml`.
///
/// # Errors
/// Returns an error if reading, parsing, syncing or saving fails.
pub fn execute_sync() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let toml_str = std::fs::read_to_string(cwd.join("frate.toml"))?;
    let toml: FrateToml = toml::from_str(&toml_str)?;
    let mut lock = FrateLock::load_or_default(cwd.join("frate.lock"));
    lock.sync(&toml)?;
    lock.save(cwd.join("frate.lock"))?;
    Ok(())
}
/// Installs a specific package or all packages if none specified.
///
/// # Arguments
/// * `name` - Optional package name to install.
///
/// # Errors
/// Returns an error if the package is not found or installation fails.
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
/// Uninstalls a specific package or all packages if none specified.
///
/// # Arguments
/// * `name` - Optional package name to uninstall.
///
/// # Errors
/// Returns an error if uninstallation fails.
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
/// Prints paths of installed executable and shim for the given package name.
///
/// # Arguments
/// * `name` - Name of the package.
///
/// # Errors
/// Returns an error if path lookup fails.
pub fn execute_which(name: &str) -> Result<()> {
    let (exe_path, shim_path) = find_installed_paths(name)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
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
/// Runs an installed executable with given arguments.
///
/// # Arguments
/// * `name` - Name of the executable.
/// * `args` - Arguments to pass to the executable.
///
/// # Errors
/// Returns an error if execution fails or the executable is not found.
pub fn execute_run(name: &str, args: Vec<String>) -> Result<()> {
    let (exe_path, _) = find_installed_paths(&name)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
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
/// Parses a string of the format "name@version" into a tuple.
///
/// # Arguments
/// * `name_at_version` - The string to parse.
///
/// # Errors
/// Returns an error if the format is invalid.
fn extract_name_at_version(name_at_version: String) -> Result<(String, String)> {
    let mut split = name_at_version.split('@');
    let name = split.next().ok_or(anyhow::anyhow!("Invalid name@version"))?;
    let version = split.next().ok_or(anyhow::anyhow!("Invalid name@version"))?;
    Ok((name.to_string(), version.to_string()))
}
/// Adds a new dependency to `frate.toml`.
///
/// # Arguments
/// * `name_at_version` - Dependency in the form "name@version".
///
/// # Errors
/// Returns an error if parsing, loading, or saving fails.
pub fn execute_add(name_at_version: String) -> Result<()> {
    let (name, version) = extract_name_at_version(name_at_version)?;
    let mut toml = FrateToml::load(std::env::current_dir()?.join("frate.toml"))
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    toml.add(&name, &version)?;
    toml.save(std::env::current_dir()?.join("frate.toml"))
        .map_err(|e| anyhow::anyhow!("{:?}", e))
}
/// Searches the registry for a tool and lists available versions.
///
/// # Arguments
/// * `name` - Name of the tool to search for.
///
/// # Errors
/// Returns an error if fetching or parsing registry data fails.
pub fn execute_search(name: String) -> Result<()> {
    let tool = fetch_registry(&name)?;
    let sorted = sort_versions(tool.releases);
    for (version, info) in sorted {
        println!("{name}@{version}");
        verbose!("  {}", &info.url);
        verbose!("  {}", &info.hash);
    }
    Ok(())
}

pub fn execute_shell() -> Result<()> {
    run_shell_with_frate_path().with_context(|| "Failed to run shell")
}