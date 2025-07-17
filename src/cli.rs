use clap::{Parser, Subcommand};

#[derive(Debug, Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    pub(crate) command: FrateCommand,
}

#[derive(Debug, Subcommand, Clone, PartialEq)]
pub enum FrateCommand {
    /// Installs packages from the `frate.lock`. Defaults to all
    Install {
        /// Install one specific package
        #[clap(long)]
        name: Option<String>,
    },
    /// Uninstall packages. Defaults to all. This will remove the tool directory in `.frate/tools/`
    /// and the shim in `.frate/shims/`
    Uninstall {
        /// Uninstall one specific package
        #[clap(long)]
        name: Option<String>,
    },
    /// Searches registries and outputs the available versions
    Search {
        name: String,
    },
    /// List all tools in `frate.toml`
    List {
        #[clap(short, long)]
        verbose: bool,
    },
    /// Run the actual binary file of a tool in `.frate/bin/<tool_name>/`
    Run {
        name: String,
        args: Vec<String>,
    },
    /// Syncs the `frate.lock` with the `frate.toml`
    Sync,
    /// Initializes
    Init,
    /// Check health (unimplemented!)
    Doctor,
    /// Cleans caches (unimplemented!)
    Clean,
    /// Adds a tool to the `frate.toml` and syncs the `frate.lock` file. The tool is not installed!
    Add {
        /// Name and version of the package: <name>@<version> (no leading 'v' for versions)
        name_at_version: String,
    },
    /// Output the binaries and shims path if they exist
    Which {
        name: String,
    },
}
