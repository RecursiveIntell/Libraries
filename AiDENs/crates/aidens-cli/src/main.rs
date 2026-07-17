use aidens_cli::{learning_run_exit_code, run, Cli, Command, LearningCommand};
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let requires_verified_learning_outcome = matches!(
        &cli.command,
        Command::Learn {
            command: LearningCommand::Run { .. }
                | LearningCommand::Promote { .. }
                | LearningCommand::Revoke { .. }
                | LearningCommand::Compare { .. }
                | LearningCommand::Replay { .. }
                | LearningCommand::Stop { .. }
        }
    );
    let output = run(cli)?;
    println!("{output}");
    if requires_verified_learning_outcome {
        std::process::exit(learning_run_exit_code(&output));
    }
    Ok(())
}
