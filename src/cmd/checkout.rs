use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::git;
use crate::hooks;
use crate::state::WorktreeState;

/// Execute the `gj checkout` command
pub fn run(remote_branch: String, no_cd: bool) -> Result<()> {
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

    // Parse the branch name (remove origin/ prefix if present)
    let branch_name = remote_branch
        .strip_prefix("origin/")
        .unwrap_or(&remote_branch);

    // Fetch the branch from origin
    eprintln!("Fetching branch '{}'...", branch_name);
    git::fetch_branch(branch_name)?;

    // Generate worktree path: {base_dir}/{repo_name}/{branch-name}
    // Replace slashes in branch name with hyphens for path
    let safe_branch_name = branch_name.replace('/', "-");
    let base_dir = config.get_base_dir(repo_config);
    let worktree_path = base_dir.join(&repo_name).join(&safe_branch_name);

    // Check if worktree path already exists
    if worktree_path.exists() {
        bail!(
            "Worktree already exists at {}. Use `gj cd {}` to switch to it.",
            worktree_path.display(),
            safe_branch_name
        );
    }

    // Create the worktree at origin/{branch}
    let git_ref = format!("origin/{}", branch_name);
    git::worktree_add_at_ref(&worktree_path, &git_ref)?;

    // Save state
    let state = WorktreeState::new(
        worktree_path.clone(),
        git_root.clone(),
        branch_name.to_string(),
    );
    state.save()?;

    // Execute hooks
    let all_hooks = config.get_hooks(repo_config);
    if let Err(e) = hooks::execute_hooks(&all_hooks, &git_root, &worktree_path) {
        eprintln!("Warning: Hook failed: {}", e);
    }

    // Output the worktree path
    if no_cd {
        eprintln!("Worktree created at: {}", worktree_path.display());
        eprintln!("Branch: {}", branch_name);
    }
    println!("{}", worktree_path.display());

    Ok(())
}
