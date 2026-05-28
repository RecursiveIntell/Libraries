use aidens_cli::{run, Cli};
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("{}", run(cli)?);
    Ok(())
}
