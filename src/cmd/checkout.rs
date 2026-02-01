use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::git;
use crate::hooks;
use crate::state::WorktreeState;

/// Execute the `gj checkout` command
pub fn run(remote_branch: String, no_cd: bool) -> Result<()> {
    // Get the git repository root
    let git_root = git::get_repo_root().context("Must be run inside a git repository")?;

    // Load configuration (requires config file to exist)
    let config = Config::load_required()?;

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
    let branch_name = parse_branch_name(&remote_branch);

    // Fetch the branch from origin
    eprintln!("Fetching branch '{}'...", branch_name);
    git::fetch_branch(branch_name)?;

    // Generate worktree path: {base_dir}/{repo_name}/{branch-name}
    let safe_branch_name = sanitize_branch_for_path(branch_name);
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

/// Parse branch name, stripping `origin/` prefix if present
fn parse_branch_name(remote_branch: &str) -> &str {
    remote_branch.strip_prefix("origin/").unwrap_or(remote_branch)
}

/// Sanitize branch name for use in filesystem path
/// Replaces `/` with `-` to avoid nested directories
fn sanitize_branch_for_path(branch: &str) -> String {
    branch.replace('/', "-")
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

    #[test]
    fn test_sanitize_branch_for_path_with_slashes() {
        assert_eq!(sanitize_branch_for_path("feature/foo"), "feature-foo");
        assert_eq!(
            sanitize_branch_for_path("feature/nested/deep"),
            "feature-nested-deep"
        );
    }

    #[test]
    fn test_sanitize_branch_for_path_no_slashes() {
        assert_eq!(sanitize_branch_for_path("main"), "main");
        assert_eq!(sanitize_branch_for_path("develop"), "develop");
    }

    #[test]
    fn test_sanitize_branch_for_path_empty() {
        assert_eq!(sanitize_branch_for_path(""), "");
    }

    #[test]
    fn test_sanitize_branch_for_path_only_slashes() {
        assert_eq!(sanitize_branch_for_path("/"), "-");
        assert_eq!(sanitize_branch_for_path("//"), "--");
    }
}
