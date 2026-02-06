use anyhow::Result;
use clap::{Parser, Subcommand};

mod cmd;
mod config;
mod git;
mod hooks;
mod state;

#[derive(Parser)]
#[command(name = "gj")]
#[command(about = "A CLI tool for managing temporary git worktree environments")]
#[command(version)]
struct Cli {
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
        branch_suffix: Option<String>,
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

    match cli.command {
        Commands::Pr { number } => cmd::pr::run(number),
        Commands::New { branch_suffix } => cmd::new::run(branch_suffix),
        Commands::Checkout { remote_branch } => cmd::checkout::run(remote_branch),
        Commands::List => cmd::list::run(),
        Commands::Cd { target } => cmd::cd::run(target),
        Commands::Exit { force, merge } => cmd::exit::run(force, merge),
        Commands::ShellInit { shell } => cmd::shell_init::run(&shell),
        Commands::Init { force } => cmd::init::run(force),
    }
}
