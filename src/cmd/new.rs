use anyhow::{bail, Context, Result};
use chrono::Utc;
use petname::{Generator, Petnames};

use crate::config::Config;
use crate::git;
use crate::hooks;
use crate::state::WorktreeState;

/// Execute the `gj new` command
pub fn run(branch_suffix: Option<String>, random_suffix: bool) -> Result<()> {
    // Get the git repository root
    let git_root = git::get_repo_root().context("Must be run inside a git repository")?;

    // Load configuration (requires config file to exist)
    let config = Config::load_required()?;

    // Find the repository configuration (optional - works without registration)
    let repo_config = config.find_repo(&git_root).map(|(_, cfg)| cfg);

    // Get GitHub repository info from remote URL
    let github_repo = git::get_github_repo_info()?;

    // Get or prompt for branch name
    let input_name = if random_suffix {
        generate_random_name()
    } else {
        match branch_suffix {
            Some(name) => name,
            None => prompt_branch_name()?,
        }
    };

    // Generate branch name: {prefix}/{YYYYMMDD}_{input}
    let prefix = config.get_prefix(repo_config);
    let date = Utc::now().format("%Y%m%d");
    let branch = format!("{}/{}_{}", prefix, date, input_name);

    // Generate worktree path: {base_dir}/{owner}/{repo}/{branch}
    let base_dir = config.get_base_dir(repo_config);
    let worktree_path = base_dir
        .join(&github_repo.owner)
        .join(&github_repo.repo)
        .join(&branch);

    // Check if worktree path already exists
    if worktree_path.exists() {
        bail!(
            "Worktree already exists at {}. Use `gj cd {}` to switch to it.",
            worktree_path.display(),
            branch
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
    eprintln!(
        "Created worktree: {}",
        crate::state::display_path(&worktree_path)
    );
    eprintln!("Branch: {}", branch);
    println!("{}", worktree_path.display());

    Ok(())
}

/// Prompt the user for a branch name
fn prompt_branch_name() -> Result<String> {
    let random_name = generate_random_name();
    let help_message = format!("e.g., awesome-feature (empty = {})", random_name);

    let name = inquire::Text::new("Enter branch suffix:")
        .with_help_message(&help_message)
        .prompt()
        .context("Failed to get branch name input")?;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Ok(random_name);
    }

    // Sanitize the name (replace spaces with hyphens, etc.)
    let sanitized = sanitize_name(&name);
    Ok(sanitized)
}

/// Generate a random name using two English words (e.g., "charming-tomato")
fn generate_random_name() -> String {
    let petnames = Petnames::default();
    petnames
        .generate_one(2, "-")
        .unwrap_or_else(|| "random-branch".to_string())
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

    #[test]
    fn test_generate_random_name() {
        let name = generate_random_name();
        // Should contain exactly one hyphen (two words separated by hyphen)
        assert_eq!(
            name.matches('-').count(),
            1,
            "Expected format: word-word, got: {}",
            name
        );
        // Should not be empty
        assert!(!name.is_empty());
        // Should only contain lowercase letters and hyphens
        assert!(name.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
    }
}
