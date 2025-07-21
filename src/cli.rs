use clap::{Parser, Subcommand};

#[derive(Debug, Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    pub(crate) command: FrateCommand,
}

#[derive(Debug, Subcommand, Clone, PartialEq)]
pub enum FrateCommand {
    /// Uses the `activate` script to start a new shell with all installed tools in the `PATH`
    Shell,
    /// Installs packages listed in the `frate.lock` file.
    /// If no package name is specified, installs all packages.
    Install {
        /// Install a specific package by name.
        #[clap(long)]
        name: Option<String>,
    },
    /// Uninstalls packages and removes related directories and shims.
    /// If no package name is specified, uninstalls all packages.
    Uninstall {
        /// Uninstall a specific package by name.
        #[clap(short, long)]
        name: Option<String>,
    },
    /// Searches registries for a tool and lists available versions.
    Search {
        name: String,
    },
    /// Lists all tools defined in `frate.toml`.
    /// Use verbose mode for detailed info including lock status and installation.
    List {
        /// Enables verbose output.
        #[clap(short, long)]
        verbose: bool,
    },
    /// Runs the executable binary of a tool from `.frate/bin/<tool_name>/`.
    Run {
        /// Name of the tool to run.
        name: String,
        /// Arguments passed to the tool executable.
        args: Vec<String>,
    },
    /// Synchronizes the `frate.lock` file with the current `frate.toml`.
    Sync,
    /// Initializes a new `frate.toml` in the current directory.
    Init,
    /// Checks the health of the setup. (Currently unimplemented)
    Doctor,
    /// Cleans caches related to the tool. (Currently unimplemented)
    Clean,
    /// Adds a tool with a specific version to `frate.toml` and syncs the lock file.
    /// Note: The tool is not installed automatically.
    Add {
        /// Package name and version in the format `<name>@<version>` (version without leading 'v').
        name_at_version: String,
    },
    /// Outputs the paths to installed binaries and shims for a given tool, if found.
    Which {
        /// Name of the tool to query.
        name: String,
    },
}
