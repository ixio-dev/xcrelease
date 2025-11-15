use anyhow::Result;
use clap::{Parser};
mod commands;

/// A tool for releasing Xcode projects
#[derive(Parser)]
#[command(name = "xcrelease", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Show what would be done without actually doing it
    #[arg(long, global = true)]
    dry_run: bool,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Generate shell completion scripts
    Completion {
        /// Shell type (bash, zsh, fish, powershell, elvish)
        shell: clap_complete::Shell,
    },
    /// Show .deployment file template
    Template,
    /// Release the Xcode project.
    /// Fails when the current version is already released.
    Release {
        /// Increase major version number
        #[arg(long, conflicts_with_all(&["minor", "patch", "ver"]))]
        major: bool,

        /// Increase minor version number
        #[arg(long, conflicts_with_all(&["major", "patch", "ver"]))]
        minor: bool,

        /// Increase patch version number
        #[arg(long, conflicts_with_all(&["major", "minor", "ver"]))]
        patch: bool,

        /// Set explicit version
        #[arg(long, value_name = "VERSION", conflicts_with_all(&["major", "minor", "patch"]))]
        ver: Option<String>,
    },
}

fn main() -> Result<()> {
    // Parse command line arguments first to handle subcommands that don't need prerequisites
    let cli = Cli::parse();

    match cli.command {
        Commands::Completion { shell } => {
            commands::handle_completion(shell)
        }
        Commands::Template => {
            commands::handle_template()
        }
        Commands::Release { major, minor, patch, ver } => {
            commands::handle_release(cli.dry_run, major, minor, patch, ver)
        }
    }
}