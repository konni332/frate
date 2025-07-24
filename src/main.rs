mod cli;
mod execute;

use clap::Parser;
use crate::cli::Cli;
use anyhow::Result;

fn main() -> Result<()>{
    let cli = Cli::parse();
    execute::execute(cli)
}
