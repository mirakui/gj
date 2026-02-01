use anyhow::{bail, Context, Result};
use chrono::Utc;

use crate::config::Config;
use crate::git;
use crate::hooks;
use crate::state::WorktreeState;

/// Execute the `gj new` command
pub fn run(branch_name: Option<String>, no_cd: bool) -> Result<()> {
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

    // Get or prompt for branch name
    let input_name = match branch_name {
        Some(name) => name,
        None => prompt_branch_name()?,
    };

    // Generate branch name: {prefix}/{YYYYMMDD}_{input}
    let prefix = config.get_prefix(repo_config);
    let date = Utc::now().format("%Y%m%d");
    let branch = format!("{}/{}_{}", prefix, date, input_name);

    // Generate worktree path: {base_dir}/{repo_name}/{input}
    let base_dir = config.get_base_dir(repo_config);
    let worktree_path = base_dir.join(&repo_name).join(&input_name);

    // Check if worktree path already exists
    if worktree_path.exists() {
        bail!(
            "Worktree already exists at {}. Use `gj cd {}` to switch to it.",
            worktree_path.display(),
            input_name
        );
    }

    // Create the worktree
    git::worktree_add_new_branch(&worktree_path, &branch)?;

    // Save state
    let state = WorktreeState::new(worktree_path.clone(), git_root.clone(), branch.clone());
    state.save()?;

    // Execute hooks
    let all_hooks = config.get_hooks(repo_config);
    if let Err(e) = hooks::execute_hooks(&all_hooks, &git_root, &worktree_path) {
        eprintln!("Warning: Hook failed: {}", e);
    }

    // Output the worktree path
    if no_cd {
        eprintln!("Worktree created at: {}", worktree_path.display());
        eprintln!("Branch: {}", branch);
    }
    println!("{}", worktree_path.display());

    Ok(())
}

/// Prompt the user for a branch name
fn prompt_branch_name() -> Result<String> {
    let name = inquire::Text::new("Enter branch name:")
        .with_help_message("e.g., awesome-feature")
        .prompt()
        .context("Failed to get branch name input")?;

    let name = name.trim().to_string();
    if name.is_empty() {
        bail!("Branch name cannot be empty");
    }

    // Sanitize the name (replace spaces with hyphens, etc.)
    let sanitized = sanitize_name(&name);
    Ok(sanitized)
}

/// Sanitize a branch name input
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("feature"), "feature");
        assert_eq!(sanitize_name("my-feature"), "my-feature");
        assert_eq!(sanitize_name("my_feature"), "my_feature");
        assert_eq!(sanitize_name("my feature"), "my-feature");
        assert_eq!(sanitize_name("my/feature"), "my-feature");
        assert_eq!(sanitize_name("feature123"), "feature123");
    }
}
