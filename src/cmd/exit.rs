use anyhow::{bail, Context, Result};

use crate::git;
use crate::state::WorktreeState;

/// Execute the `gj exit` command
pub fn run(force: bool, merge: bool) -> Result<()> {
    // Load state for current directory
    let state = WorktreeState::load_current()?.context(
        "Not in a gj-managed worktree. Use this command inside a worktree created by gj.",
    )?;

    // Check for uncommitted changes unless --force
    // For --merge, we always require clean state
    if merge && git::has_uncommitted_changes()? {
        bail!(
            "Worktree has uncommitted changes. Commit or stash them before using --merge."
        );
    } else if !force && !merge && git::has_uncommitted_changes()? {
        bail!(
            "Worktree has uncommitted changes. Use --force to discard them, or commit/stash first."
        );
    }

    // Get the origin repo path before we delete the worktree
    let origin_repo = state.origin_repo.clone();
    let branch = state.branch.clone();
    let worktree_path = state.worktree_path.clone();

    // Handle merge if requested
    let target_dir = if merge {
        // Get the default branch
        let default_branch = git::get_default_branch(&origin_repo)?;

        // Find the worktree that has the default branch checked out
        let merge_worktree = git::find_worktree_for_branch(&default_branch, &origin_repo)?
            .context(format!(
                "Default branch '{}' is not checked out in any worktree",
                default_branch
            ))?;

        // Merge the worktree branch in the target worktree
        if let Err(e) = git::merge_branch(&branch, &merge_worktree) {
            // Merge failed, abort and return error
            let _ = git::merge_abort(&merge_worktree);
            bail!(
                "Merge failed. Conflict detected. Aborting merge.\nError: {}",
                e
            );
        }

        eprintln!("Merged '{}' into '{}'", branch, default_branch);
        merge_worktree
    } else {
        origin_repo.clone()
    };

    // Remove the worktree (run from origin repo)
    git::worktree_remove(&worktree_path, force, &origin_repo)?;

    // Delete the branch (run from origin repo)
    // When merging, the branch is already merged so we can safely delete it
    git::branch_delete(&branch, force || merge, &origin_repo)?;

    // Delete the state file
    state.delete()?;

    // Output status message and target directory path
    eprintln!("Removed worktree: {}", worktree_path.display());
    eprintln!("Deleted branch: {}", branch);
    println!("{}", target_dir.display());

    Ok(())
}
