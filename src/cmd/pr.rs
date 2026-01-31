use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::git;
use crate::hooks;
use crate::state::WorktreeState;

/// Execute the `gj pr` command
pub fn run(pr_number: u32, no_cd: bool) -> Result<()> {
    // Get the git repository root
    let git_root = git::get_repo_root().context("Must be run inside a git repository")?;

    // Load configuration
    let config = Config::load()?;

    // Find the repository configuration (optional - works without registration)
    let (repo_name, repo_config) = match config.find_repo(&git_root) {
        Some((name, cfg)) => (name.clone(), Some(cfg)),
        None => {
            // Use directory name as repo_name when not registered
            let name = git_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repo")
                .to_string();
            (name, None)
        }
    };

    // Get PR branch name using gh CLI
    let pr_branch = git::get_pr_branch(pr_number)?;

    // Generate worktree path: {base_dir}/{repo_name}/pr-{number}
    let base_dir = config.get_base_dir(repo_config);
    let worktree_name = format!("pr-{}", pr_number);
    let worktree_path = base_dir.join(&repo_name).join(&worktree_name);

    // Check if worktree path already exists
    if worktree_path.exists() {
        bail!(
            "Worktree already exists at {}. Use `gj cd {}` to switch to it.",
            worktree_path.display(),
            worktree_name
        );
    }

    // Fetch the PR
    eprintln!("Fetching PR #{}...", pr_number);
    git::fetch_pr(pr_number)?;

    // Create the worktree at FETCH_HEAD
    git::worktree_add_at_ref(&worktree_path, "FETCH_HEAD")?;

    // Save state
    let state = WorktreeState::new(worktree_path.clone(), git_root.clone(), pr_branch.clone());
    state.save()?;

    // Execute hooks
    let all_hooks = config.get_hooks(repo_config);
    if let Err(e) = hooks::execute_hooks(&all_hooks, &git_root, &worktree_path) {
        eprintln!("Warning: Hook failed: {}", e);
    }

    // Output the worktree path
    if no_cd {
        eprintln!("Worktree created at: {}", worktree_path.display());
        eprintln!("Branch: {} (PR #{})", pr_branch, pr_number);
    }
    println!("{}", worktree_path.display());

    Ok(())
}
