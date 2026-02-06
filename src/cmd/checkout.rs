use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::git;
use crate::hooks;
use crate::state::WorktreeState;

/// Execute the `gj checkout` command
pub fn run(remote_branch: String) -> Result<()> {
    // Get the git repository root
    let git_root = git::get_repo_root().context("Must be run inside a git repository")?;

    // Load configuration (requires config file to exist)
    let config = Config::load_required()?;

    // Find the repository configuration (optional - works without registration)
    let repo_config = config.find_repo(&git_root).map(|(_, cfg)| cfg);

    // Get GitHub repository info from remote URL
    let github_repo = git::get_github_repo_info()?;

    // Parse the branch name (remove origin/ prefix if present)
    let branch_name = parse_branch_name(&remote_branch);

    // Fetch the branch from origin
    eprintln!("Fetching branch '{}'...", branch_name);
    git::fetch_branch(branch_name)?;

    // Generate worktree path: {base_dir}/{owner}/{repo}/{branch_name}
    let base_dir = config.get_base_dir(repo_config);
    let worktree_path = base_dir
        .join(&github_repo.owner)
        .join(&github_repo.repo)
        .join(branch_name);

    // Check if worktree path already exists
    if worktree_path.exists() {
        bail!(
            "Worktree already exists at {}. Use `gj cd {}` to switch to it.",
            worktree_path.display(),
            branch_name
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
    eprintln!("Created worktree: {}", worktree_path.display());
    eprintln!("Branch: {}", branch_name);
    println!("{}", worktree_path.display());

    Ok(())
}

/// Parse branch name, stripping `origin/` prefix if present
fn parse_branch_name(remote_branch: &str) -> &str {
    remote_branch.strip_prefix("origin/").unwrap_or(remote_branch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_branch_name_with_origin_prefix() {
        assert_eq!(parse_branch_name("origin/main"), "main");
        assert_eq!(parse_branch_name("origin/feature/foo"), "feature/foo");
    }

    #[test]
    fn test_parse_branch_name_without_prefix() {
        assert_eq!(parse_branch_name("main"), "main");
        assert_eq!(parse_branch_name("feature/bar"), "feature/bar");
    }

    #[test]
    fn test_parse_branch_name_empty() {
        assert_eq!(parse_branch_name(""), "");
    }

    #[test]
    fn test_parse_branch_name_only_origin_slash() {
        // "origin/" should become empty string
        assert_eq!(parse_branch_name("origin/"), "");
    }
}
