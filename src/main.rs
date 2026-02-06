use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;
mod config;
mod git;
mod hooks;
mod output;
mod state;

use output::OutputFormat;

#[derive(Parser)]
#[command(name = "gj")]
#[command(about = "A CLI tool for managing temporary git worktree environments")]
#[command(version)]
struct Cli {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, global = true)]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a worktree for reviewing a GitHub PR
    Pr {
        /// PR number
        number: u32,
    },

    /// Create a new worktree for feature development
    New {
        /// Branch suffix (prompted interactively if not provided)
        #[arg(conflicts_with = "random_suffix")]
        branch_suffix: Option<String>,
        /// Generate a random branch suffix automatically
        #[arg(long)]
        random_suffix: bool,
    },

    /// Create a worktree from a remote branch
    #[command(visible_alias = "co")]
    Checkout {
        /// Remote branch name (e.g., main, feature/foo, or origin/main)
        remote_branch: String,
    },

    /// List all managed worktrees
    #[command(visible_alias = "ls")]
    List,

    /// Change to a worktree directory
    Cd {
        /// Worktree name or '@' for origin repository
        target: Option<String>,
    },

    /// Clean up the current worktree and return to origin repository
    Exit {
        /// Force removal even with uncommitted changes
        #[arg(long, short)]
        force: bool,
        /// Merge the worktree branch into the default branch before exiting
        #[arg(long, short)]
        merge: bool,
    },

    /// Output shell initialization script
    #[command(name = "shell-init")]
    ShellInit {
        /// Shell type (zsh, bash)
        shell: String,
    },

    /// Initialize gj configuration file
    Init {
        /// Overwrite existing configuration file
        #[arg(long, short)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let output = cli.output;

    match cli.command {
        Commands::Pr { number } => cmd::pr::run(number, output),
        Commands::New { branch_suffix, random_suffix } => cmd::new::run(branch_suffix, random_suffix, output),
        Commands::Checkout { remote_branch } => cmd::checkout::run(remote_branch, output),
        Commands::List => cmd::list::run(output),
        Commands::Cd { target } => cmd::cd::run(target, output),
        Commands::Exit { force, merge } => cmd::exit::run(force, merge, output),
        Commands::ShellInit { shell } => cmd::shell_init::run(&shell),
        Commands::Init { force } => cmd::init::run(force, output),
    }
}
