use anyhow::Result;
use clap::{CommandFactory, Parser};

pub fn handle_completion(shell: clap_complete::Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}

// Empty parser just for completion generation
#[derive(Parser)]
#[command(name = "xcrelease")]
struct Cli {
    #[command(subcommand)]
    command: Option<crate::Commands>,
}