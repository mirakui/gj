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
        /// Do not change directory, just print the path
        #[arg(long)]
        no_cd: bool,
    },

    /// Create a new worktree for feature development
    New {
        /// Branch name (prompted interactively if not provided)
        branch_name: Option<String>,
        /// Do not change directory, just print the path
        #[arg(long)]
        no_cd: bool,
    },

    /// Create a worktree from a remote branch
    #[command(visible_alias = "co")]
    Checkout {
        /// Remote branch name (e.g., main, feature/foo, or origin/main)
        remote_branch: String,
        /// Do not change directory, just print the path
        #[arg(long)]
        no_cd: bool,
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
        Commands::Pr { number, no_cd } => cmd::pr::run(number, no_cd),
        Commands::New { branch_name, no_cd } => cmd::new::run(branch_name, no_cd),
        Commands::Checkout { remote_branch, no_cd } => cmd::checkout::run(remote_branch, no_cd),
        Commands::List => cmd::list::run(),
        Commands::Cd { target } => cmd::cd::run(target),
        Commands::Exit { force } => cmd::exit::run(force),
        Commands::ShellInit { shell } => cmd::shell_init::run(&shell),
        Commands::Init { force } => cmd::init::run(force),
    }
}
