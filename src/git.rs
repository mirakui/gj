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

/// Create a worktree at a specific ref with a named branch
pub fn worktree_add_with_branch(path: &Path, branch: &str, git_ref: &str) -> Result<()> {
    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            branch,
            path.to_string_lossy().as_ref(),
            git_ref,
        ])
        .output()
        .context("Failed to execute git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create worktree: {}", stderr.trim());
    }

    Ok(())
}

/// Set upstream tracking for a branch in a worktree
pub fn set_upstream(worktree_path: &Path, branch: &str, upstream: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["branch", "--set-upstream-to", upstream, branch])
        .current_dir(worktree_path)
        .output()
        .context("Failed to set upstream")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to set upstream: {}", stderr.trim());
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

/// Get the default branch name from origin
pub fn get_default_branch(repo_path: &Path) -> Result<String> {
    // Try to get from origin/HEAD
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .current_dir(repo_path)
        .output()
        .context("Failed to get default branch")?;

    if output.status.success() {
        let ref_name = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string();
        // refs/remotes/origin/main -> main
        if let Some(branch) = ref_name.strip_prefix("refs/remotes/origin/") {
            return Ok(branch.to_string());
        }
    }

    // Fallback: check if main exists
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/main"])
        .current_dir(repo_path)
        .output()
        .context("Failed to check main branch")?;

    if output.status.success() {
        return Ok("main".to_string());
    }

    // Fallback: check if master exists
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/master"])
        .current_dir(repo_path)
        .output()
        .context("Failed to check master branch")?;

    if output.status.success() {
        return Ok("master".to_string());
    }

    bail!("Could not determine default branch. Neither 'main' nor 'master' exists.");
}

/// Checkout a branch
#[allow(dead_code)]
pub fn checkout_branch(branch: &str, repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", branch])
        .current_dir(repo_path)
        .output()
        .context("Failed to checkout branch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to checkout branch {}: {}", branch, stderr.trim());
    }

    Ok(())
}

/// Merge a branch into the current branch
pub fn merge_branch(branch: &str, repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["merge", branch, "--no-edit"])
        .current_dir(repo_path)
        .output()
        .context("Failed to merge branch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to merge branch {}: {}", branch, stderr.trim());
    }

    Ok(())
}

/// Abort an in-progress merge
pub fn merge_abort(repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["merge", "--abort"])
        .current_dir(repo_path)
        .output()
        .context("Failed to abort merge")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to abort merge: {}", stderr.trim());
    }

    Ok(())
}

/// Find the worktree path that has a specific branch checked out
pub fn find_worktree_for_branch(branch: &str, repo_path: &Path) -> Result<Option<PathBuf>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .context("Failed to list worktrees")?;

    if !output.status.success() {
        bail!("Failed to list worktrees");
    }

    let output_str = String::from_utf8(output.stdout).context("Invalid UTF-8 in git output")?;

    let mut current_worktree: Option<PathBuf> = None;

    for line in output_str.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_worktree = Some(PathBuf::from(path));
        } else if let Some(branch_name) = line.strip_prefix("branch refs/heads/") {
            if branch_name == branch {
                return Ok(current_worktree);
            }
        }
    }

    Ok(None)
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
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to ensure tests that change cwd run serially
    static CWD_MUTEX: Mutex<()> = Mutex::new(());

    /// Helper to create a temporary git repository
    fn create_temp_git_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Initialize git repo
        let output = Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");
        assert!(
            output.status.success(),
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Configure git user for commits
        let output = Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to configure git email");
        assert!(output.status.success(), "git config email failed");

        let output = Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to configure git name");
        assert!(output.status.success(), "git config name failed");

        // Disable GPG signing for test commits
        let output = Command::new("git")
            .args(["config", "commit.gpgSign", "false"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to disable GPG signing");
        assert!(output.status.success(), "git config gpgSign failed");

        // Create initial commit
        fs::write(repo_path.join("README.md"), "# Test").expect("Failed to create file");

        let output = Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .expect("Failed to stage files");
        assert!(
            output.status.success(),
            "git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to create initial commit");
        assert!(
            output.status.success(),
            "git commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        temp_dir
    }

    /// Helper to run git status in a specific directory
    fn has_uncommitted_changes_in(repo_path: &Path) -> Result<bool> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_path)
            .output()
            .context("Failed to execute git status")?;

        if !output.status.success() {
            bail!("Failed to check git status");
        }

        Ok(!output.stdout.is_empty())
    }

    /// Helper to get current branch in a specific directory
    fn current_branch_in(repo_path: &Path) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(repo_path)
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
            return Ok(None);
        }

        Ok(Some(branch))
    }

    /// Helper to get repo root in a specific directory
    fn get_repo_root_in(dir: &Path) -> Result<PathBuf> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(dir)
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

    #[test]
    fn test_is_gh_available() {
        // This just checks that the function doesn't panic
        let _ = is_gh_available();
    }

    #[test]
    fn test_get_repo_root_in_git_repo() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        let root = get_repo_root_in(repo_path).expect("Should find repo root");

        // The paths should be equivalent (canonicalize to handle symlinks)
        assert_eq!(
            root.canonicalize().unwrap(),
            repo_path.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_get_repo_root_in_subdirectory() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        // Create a subdirectory
        let subdir = repo_path.join("src");
        fs::create_dir(&subdir).expect("Failed to create subdir");

        let root = get_repo_root_in(&subdir).expect("Should find repo root from subdir");

        assert_eq!(
            root.canonicalize().unwrap(),
            repo_path.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_has_uncommitted_changes_clean_repo() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        let has_changes = has_uncommitted_changes_in(repo_path).expect("Should check status");
        assert!(!has_changes, "Clean repo should have no uncommitted changes");
    }

    #[test]
    fn test_has_uncommitted_changes_with_unstaged() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        // Modify a file
        fs::write(repo_path.join("README.md"), "# Modified").expect("Failed to modify file");

        let has_changes = has_uncommitted_changes_in(repo_path).expect("Should check status");
        assert!(has_changes, "Repo with unstaged changes should be dirty");
    }

    #[test]
    fn test_has_uncommitted_changes_with_staged() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        // Create and stage a new file
        fs::write(repo_path.join("new.txt"), "new content").expect("Failed to create file");
        Command::new("git")
            .args(["add", "new.txt"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to stage file");

        let has_changes = has_uncommitted_changes_in(repo_path).expect("Should check status");
        assert!(has_changes, "Repo with staged changes should be dirty");
    }

    #[test]
    fn test_current_branch_on_main() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        let branch = current_branch_in(repo_path).expect("Should get branch");
        // Default branch could be 'main' or 'master' depending on git config
        assert!(
            branch == Some("main".to_string()) || branch == Some("master".to_string()),
            "Should be on main or master branch, got: {:?}",
            branch
        );
    }

    #[test]
    fn test_current_branch_on_feature_branch() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        // Create and checkout a new branch
        let output = Command::new("git")
            .args(["checkout", "-b", "feature/test"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to create branch");
        assert!(
            output.status.success(),
            "Failed to create branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let branch = current_branch_in(repo_path).expect("Should get branch");
        assert_eq!(branch, Some("feature/test".to_string()));
    }

    #[test]
    fn test_current_branch_detached_head() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        // Detach HEAD by checking out a commit
        Command::new("git")
            .args(["checkout", "--detach", "HEAD"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to detach HEAD");

        let branch = current_branch_in(repo_path).expect("Should get branch");
        assert_eq!(branch, None, "Detached HEAD should return None");
    }

    #[test]
    fn test_worktree_add_and_remove() {
        let _guard = CWD_MUTEX.lock().unwrap();
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();
        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        // Create a worktree
        let worktree_path = temp_dir.path().parent().unwrap().join("test-worktree");
        worktree_add_new_branch(&worktree_path, "test-branch").expect("Should create worktree");

        // Verify worktree exists
        assert!(worktree_path.exists(), "Worktree directory should exist");
        assert!(
            worktree_path.join(".git").exists(),
            "Worktree should have .git"
        );

        // Remove the worktree
        worktree_remove(&worktree_path, false, repo_path).expect("Should remove worktree");
        assert!(
            !worktree_path.exists(),
            "Worktree directory should be removed"
        );
    }

    #[test]
    fn test_branch_delete() {
        let temp_dir = create_temp_git_repo();
        let repo_path = temp_dir.path();

        // Create a branch
        let output = Command::new("git")
            .args(["branch", "to-delete"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to create branch");
        assert!(
            output.status.success(),
            "Failed to create branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Verify branch exists
        let output = Command::new("git")
            .args(["branch", "--list", "to-delete"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to list branches");
        assert!(
            !output.stdout.is_empty(),
            "Branch should exist before delete"
        );

        // Delete the branch
        branch_delete("to-delete", false, repo_path).expect("Should delete branch");

        // Verify branch is gone
        let output = Command::new("git")
            .args(["branch", "--list", "to-delete"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to list branches");
        assert!(output.stdout.is_empty(), "Branch should be deleted");
    }
}
