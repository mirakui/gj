use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::Hook;

/// Execute hooks after worktree creation
pub fn execute_hooks(hooks: &[&Hook], origin_repo: &Path, worktree_path: &Path) -> Result<()> {
    for hook in hooks {
        match hook {
            Hook::Copy { from, to, required } => {
                execute_copy_hook(from, to.as_deref(), *required, origin_repo, worktree_path)?;
            }
            Hook::Run { command } => {
                execute_run_hook(command, worktree_path)?;
            }
        }
    }
    Ok(())
}

/// Execute a copy hook
fn execute_copy_hook(
    from: &str,
    to: Option<&str>,
    required: bool,
    origin_repo: &Path,
    worktree_path: &Path,
) -> Result<()> {
    let source = origin_repo.join(from);
    let dest_name = to.unwrap_or(from);
    let dest = worktree_path.join(dest_name);

    if !source.exists() {
        if required {
            bail!(
                "Required file not found: {} (from origin repo)",
                source.display()
            );
        } else {
            // Skip silently
            return Ok(());
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    fs::copy(&source, &dest)
        .with_context(|| format!("Failed to copy {} to {}", source.display(), dest.display()))?;

    eprintln!("Copied: {} -> {}", from, dest_name);

    Ok(())
}

/// Execute a run hook
fn execute_run_hook(command: &str, worktree_path: &Path) -> Result<()> {
    eprintln!("Running: {}", command);

    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(worktree_path)
        .status()
        .with_context(|| format!("Failed to execute command: {}", command))?;

    if !status.success() {
        bail!("Hook command failed: {}", command);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_copy_hook_success() {
        let origin = TempDir::new().unwrap();
        let worktree = TempDir::new().unwrap();

        // Create source file
        let source_file = origin.path().join(".env");
        fs::write(&source_file, "TEST=value").unwrap();

        execute_copy_hook(".env", None, false, origin.path(), worktree.path()).unwrap();

        let dest_file = worktree.path().join(".env");
        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(&dest_file).unwrap(), "TEST=value");
    }

    #[test]
    fn test_copy_hook_with_rename() {
        let origin = TempDir::new().unwrap();
        let worktree = TempDir::new().unwrap();

        let source_file = origin.path().join(".env.local");
        fs::write(&source_file, "TEST=value").unwrap();

        execute_copy_hook(
            ".env.local",
            Some(".env"),
            false,
            origin.path(),
            worktree.path(),
        )
        .unwrap();

        let dest_file = worktree.path().join(".env");
        assert!(dest_file.exists());
    }

    #[test]
    fn test_copy_hook_missing_optional() {
        let origin = TempDir::new().unwrap();
        let worktree = TempDir::new().unwrap();

        // Should not fail for optional missing file
        execute_copy_hook(".nonexistent", None, false, origin.path(), worktree.path()).unwrap();
    }

    #[test]
    fn test_copy_hook_missing_required() {
        let origin = TempDir::new().unwrap();
        let worktree = TempDir::new().unwrap();

        // Should fail for required missing file
        let result = execute_copy_hook(".nonexistent", None, true, origin.path(), worktree.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_run_hook_success() {
        let worktree = TempDir::new().unwrap();

        execute_run_hook("true", worktree.path()).unwrap();
    }

    #[test]
    fn test_run_hook_failure() {
        let worktree = TempDir::new().unwrap();

        let result = execute_run_hook("false", worktree.path());
        assert!(result.is_err());
    }
}
