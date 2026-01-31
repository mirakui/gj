use anyhow::{bail, Context, Result};

use crate::git;
use crate::state::WorktreeState;

/// Execute the `gj exit` command
pub fn run(force: bool) -> Result<()> {
    // Load state for current directory
    let state = WorktreeState::load_current()?.context(
        "Not in a gj-managed worktree. Use this command inside a worktree created by gj.",
    )?;

    // Check for uncommitted changes unless --force
    if !force && git::has_uncommitted_changes()? {
        bail!(
            "Worktree has uncommitted changes. Use --force to discard them, or commit/stash first."
        );
    }

    // Get the origin repo path before we delete the worktree
    let origin_repo = state.origin_repo.clone();
    let branch = state.branch.clone();
    let worktree_path = state.worktree_path.clone();

    // Remove the worktree (run from origin repo)
    git::worktree_remove(&worktree_path, force, &origin_repo)?;

    // Delete the branch (run from origin repo)
    git::branch_delete(&branch, force, &origin_repo)?;

    // Delete the state file
    state.delete()?;

    // Output the origin repo path for the shell wrapper to cd into
    println!("{}", origin_repo.display());

    Ok(())
}
