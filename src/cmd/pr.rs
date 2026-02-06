use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::git;
use crate::hooks;
use crate::state::WorktreeState;

/// Execute the `gj pr` command
pub fn run(pr_number: u32) -> Result<()> {
    // Get the git repository root
    let git_root = git::get_repo_root().context("Must be run inside a git repository")?;

    // Load configuration (requires config file to exist)
    let config = Config::load_required()?;

    // Find the repository configuration (optional - works without registration)
    let repo_config = config.find_repo(&git_root).map(|(_, cfg)| cfg);

    // Get GitHub repository info from remote URL
    let github_repo = git::get_github_repo_info()?;

    // Get PR branch name using gh CLI
    let pr_branch = git::get_pr_branch(pr_number)?;

    // Generate worktree path: {base_dir}/{owner}/{repo}/pr-{number}
    let base_dir = config.get_base_dir(repo_config);
    let worktree_name = pr_worktree_name(pr_number);
    let worktree_path = base_dir
        .join(&github_repo.owner)
        .join(&github_repo.repo)
        .join(&worktree_name);

    // Check if worktree path already exists
    if worktree_path.exists() {
        bail!(
            "Worktree already exists at {}. Use `gj cd {}` to switch to it.",
            worktree_path.display(),
            worktree_name
        );
    }

    // Fetch the PR branch
    eprintln!("Fetching PR #{}...", pr_number);
    git::fetch_branch(&pr_branch)?;

    // Create the worktree with the PR branch name, tracking origin
    let git_ref = format!("origin/{}", pr_branch);
    git::worktree_add_with_branch(&worktree_path, &pr_branch, &git_ref)?;

    // Set upstream tracking
    git::set_upstream(&worktree_path, &pr_branch, &git_ref)?;

    // Save state
    let state = WorktreeState::new(worktree_path.clone(), git_root.clone(), pr_branch.clone());
    state.save()?;

    // Execute hooks
    let all_hooks = config.get_hooks(repo_config);
    if let Err(e) = hooks::execute_hooks(&all_hooks, &git_root, &worktree_path) {
        eprintln!("Warning: Hook failed: {}", e);
    }

    // Output the worktree path
    eprintln!("Created worktree: {}", crate::state::display_path(&worktree_path));
    eprintln!("Branch: {} (PR #{})", pr_branch, pr_number);
    println!("{}", worktree_path.display());

    Ok(())
}

/// Generate worktree name for a PR
fn pr_worktree_name(pr_number: u32) -> String {
    format!("pr-{}", pr_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_worktree_name_single_digit() {
        assert_eq!(pr_worktree_name(1), "pr-1");
        assert_eq!(pr_worktree_name(9), "pr-9");
    }

    #[test]
    fn test_pr_worktree_name_double_digit() {
        assert_eq!(pr_worktree_name(42), "pr-42");
        assert_eq!(pr_worktree_name(99), "pr-99");
    }

    #[test]
    fn test_pr_worktree_name_large_number() {
        assert_eq!(pr_worktree_name(12345), "pr-12345");
        assert_eq!(pr_worktree_name(999999), "pr-999999");
    }

    #[test]
    fn test_pr_worktree_name_zero() {
        // Edge case: PR #0 (unlikely but valid input)
        assert_eq!(pr_worktree_name(0), "pr-0");
    }
}
