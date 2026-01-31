use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the root directory of the current git repository
pub fn get_repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        bail!("Not in a git repository");
    }

    let path = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();

    Ok(PathBuf::from(path))
}

/// Create a new worktree with a new branch
pub fn worktree_add_new_branch(path: &Path, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            branch,
            path.to_string_lossy().as_ref(),
        ])
        .output()
        .context("Failed to execute git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create worktree: {}", stderr.trim());
    }

    Ok(())
}

/// Create a worktree at a specific commit/ref
pub fn worktree_add_at_ref(path: &Path, git_ref: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "add", path.to_string_lossy().as_ref(), git_ref])
        .output()
        .context("Failed to execute git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create worktree: {}", stderr.trim());
    }

    Ok(())
}

/// Remove a worktree
pub fn worktree_remove(path: &Path, force: bool, repo_path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(&path_str);

    let output = Command::new("git")
        .args(&args)
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git worktree remove")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to remove worktree: {}", stderr.trim());
    }

    Ok(())
}

/// Delete a local branch
pub fn branch_delete(branch: &str, force: bool, repo_path: &Path) -> Result<()> {
    let flag = if force { "-D" } else { "-d" };

    let output = Command::new("git")
        .args(["branch", flag, branch])
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git branch delete")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if branch doesn't exist or can't be deleted
        eprintln!(
            "Warning: Could not delete branch {}: {}",
            branch,
            stderr.trim()
        );
    }

    Ok(())
}

/// Check if there are uncommitted changes
pub fn has_uncommitted_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to execute git status")?;

    if !output.status.success() {
        bail!("Failed to check git status");
    }

    Ok(!output.stdout.is_empty())
}

/// Fetch a PR from GitHub
pub fn fetch_pr(pr_number: u32) -> Result<()> {
    let refspec = format!("pull/{}/head", pr_number);

    let output = Command::new("git")
        .args(["fetch", "origin", &refspec])
        .output()
        .context("Failed to fetch PR")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to fetch PR #{}: {}", pr_number, stderr.trim());
    }

    Ok(())
}

/// Get PR branch name using gh CLI
pub fn get_pr_branch(pr_number: u32) -> Result<String> {
    // First check if gh is available
    if !is_gh_available() {
        bail!("gh CLI is not installed. Please install it from https://cli.github.com/");
    }

    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            &pr_number.to_string(),
            "--json",
            "headRefName",
            "-q",
            ".headRefName",
        ])
        .output()
        .context("Failed to execute gh pr view")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get PR #{} info: {}", pr_number, stderr.trim());
    }

    let branch = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in gh output")?
        .trim()
        .to_string();

    if branch.is_empty() {
        bail!("PR #{} not found or has no branch", pr_number);
    }

    Ok(branch)
}

/// Fetch a branch from origin
pub fn fetch_branch(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["fetch", "origin", branch])
        .output()
        .context("Failed to fetch branch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to fetch branch {}: {}", branch, stderr.trim());
    }

    Ok(())
}

/// Check if gh CLI is available
pub fn is_gh_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the current branch name
#[allow(dead_code)]
pub fn current_branch() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;

    if !output.status.success() {
        return Ok(None);
    }

    let branch = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git output")?
        .trim()
        .to_string();

    if branch == "HEAD" {
        // Detached HEAD state
        return Ok(None);
    }

    Ok(Some(branch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_gh_available() {
        // This just checks that the function doesn't panic
        let _ = is_gh_available();
    }
}
