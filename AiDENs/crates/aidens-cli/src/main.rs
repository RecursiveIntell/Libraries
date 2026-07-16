use aidens_cli::{learning_run_exit_code, run, Cli, Command, LearningCommand};
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let is_learning_run = matches!(
        &cli.command,
        Command::Learn {
            command: LearningCommand::Run { .. }
        }
    );
    let output = run(cli)?;
    println!("{output}");
    if is_learning_run {
        std::process::exit(learning_run_exit_code(&output));
    }
    Ok(())
}
