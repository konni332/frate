mod cli;
mod execute;

use clap::Parser;
use crate::cli::CLI;
use anyhow::Result;

fn main() -> Result<()>{
    let cli = CLI::parse();
    execute::execute(cli)
}
