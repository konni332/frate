use std::process::Command;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::Deserialize;
use verbosio::{set_verbosity, verbose};
use frate::installer::{install_package, install_packages, uninstall_package, uninstall_packages};
use frate::lock::FrateLock;
use frate::registry::fetch_registry;
use frate::{clean_cache, fetch_description, filter_versions, is_cached, remove_cached_archive};
use frate::shims::{run_shell_with_frate_path};
#[cfg(windows)]
use frate::shims::{write_windows_activate};
#[cfg(unix)]
use frate::shims::{write_unix_activate};
use frate::toml::FrateToml;
use frate::util::{ensure_frate_dirs, find_installed_paths, get_frate_toml, get_locked, is_installed, sort_versions};
use crate::cli::{FrateCommand, Cli};

/// Executes the given CLI command.
///
/// # Errors
/// Returns an error if command execution fails or required files are missing.
pub fn execute(cli: Cli) -> Result<()> {
    match &cli.command {
        FrateCommand::Search { .. } |
        FrateCommand::Shell |
        FrateCommand::Clean { .. } |
        FrateCommand::Init => {},
        _ => {
            let toml_path = get_frate_toml()?;
            if !toml_path.exists() {
                bail!("frate.toml not found. Run `frate init` to create one.")
            }
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
        FrateCommand::Which { name, verbose } => {
            if verbose {
                set_verbosity!();
            }
            execute_which(&name)
        }
        FrateCommand::Run { name, args} => {
            execute_run(&name, args)
        }
        FrateCommand::Add { name_at_version } => {
            execute_add(name_at_version)
        }
        FrateCommand::Search { name, versions, verbose } => {
            if verbose {
                set_verbosity!();
            }
            execute_search(name, versions)
        }
        FrateCommand::Clean { name } => {
            execute_clean(name)
        }
        FrateCommand::Registry { verbose } => {
            if verbose {
                set_verbosity!();
            }
            execute_registry()
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
        println!("{}", "No dependencies".yellow());
        return Ok(());
    }

    for (name, version) in &toml.dependencies {
        println!("{}: {}", name.bold(), version.bold());
        if let Some(lock) = &lock {
                let locked = get_locked(name, lock);
                match locked {
                    Some(locked) => {
                        print!("  {}", " locked".green());
                        verbose!(@lvl 1, " {} {}", "at:".green(), locked.version.green());
                        verbose!(@lvl 1, "  {} {}", " hash:".green(), locked.hash.green());
                        verbose!(@lvl 1, "  {} {}", "󰳏 source:".cyan(), locked.source.cyan());
                        match is_cached(format!("{}-{}", locked.name, locked.version ).as_str()) {
                            Ok(true) => {
                                println!("  {}", "󰃨 cached".green());
                            }
                            _ => {}
                        }

                    },
                    None => {
                        println!("  {}", " unlocked".yellow());
                    }
                }
                match is_installed(name) {
                    true => {
                        print!("  {}", " installed".green());
                    },
                    false => {
                        print!("  {}", " not installed".red());
                    },
                }
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
            install_package(&package, &cwd.join(".frate"))
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
    let (exe_path, shim_path) = find_installed_paths(name)?;
    if exe_path.is_none() && shim_path.is_none() {
        println!("{}", "No installed paths found".yellow());
        return Ok(());
    }
    if let Some(exe_path) = exe_path {
        println!("{}", "bin found".green());
        verbose!("  {}", exe_path.to_string_lossy().green());
    }
    if let Some(shim_path) = shim_path {
        println!("{}", "shim found".green());
        verbose!("  {}", shim_path.to_string_lossy().green());
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
    let (exe_path, _) = find_installed_paths(name)?;
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
        bail!("{}", String::from_utf8(output.stderr)?.red());
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
pub fn execute_search(name: String, versions: usize) -> Result<()> {
    let tool = fetch_registry(&name)?;
    let sorted = sort_versions(tool.releases);
    let filtered = filter_versions(sorted);
    if filtered.is_empty() {
        println!("{}", "No versions found for:".yellow());
        println!("  {}", std::env::consts::OS.yellow());
        println!("  {}", std::env::consts::ARCH.yellow());
        return Ok(());
    }
    println!("{}", name.bold());
    if let Some(desc) = fetch_description(tool.repo.as_str())? {
        println!("  {}", desc.dimmed());
    }
    let (latest_version, latest_info) = filtered.last().unwrap();
    println!("  {}", "latest:".bold());
    println!("      {}", latest_version.split('-').next().unwrap_or(&latest_version).bold().green());
    verbose!("          {}", latest_info.url.cyan());
    verbose!("          {}", latest_info.hash.cyan());

    if versions <= 1 {
        return Ok(());
    }
    println!("  {}", "other versions:".bold());
    for (version, info) in filtered[1..versions].iter() {
        println!("      {}", version.split('-').next().unwrap_or(&version).bold());
        verbose!("          {}", &info.url.cyan());
        verbose!("          {}", &info.hash.cyan());
    }
    Ok(())
}

pub fn execute_shell() -> Result<()> {
    run_shell_with_frate_path().with_context(|| "Failed to run shell")
}

pub fn execute_clean(name: Option<String>) -> Result<()> {
    if let Some(name) = name {
        remove_cached_archive(&name)?;
    }
    else {
        clean_cache()?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ToolInfo {
    name: String,
    repo: String,
}

#[derive(Debug, Deserialize)]
struct RegistryIndex {
    registered: Vec<ToolInfo>,
}

pub fn execute_registry() -> Result<()> {
    let url = "https://raw.githubusercontent.com/konni332/frate-registry/refs/heads/master/registry.json";
    let resp = reqwest::blocking::get(url)?;
    let registry: RegistryIndex = serde_json::from_reader(resp)?;
    for tool in &registry.registered {
        println!("{}", tool.name.bold());
        verbose!("  {}", tool.repo.cyan());
    }
    Ok(())
}