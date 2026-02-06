use anyhow::{bail, Context, Result};

use crate::state::{self, WorktreeState};

/// Execute the `gj cd` command
pub fn run(target: Option<String>) -> Result<()> {
    match target.as_deref() {
        Some("@") => cd_to_origin(),
        Some(name) => cd_to_worktree(name),
        None => cd_interactive(),
    }
}

/// Navigate to the origin repository of the current worktree
fn cd_to_origin() -> Result<()> {
    let state = WorktreeState::load_current()?
        .ok_or_else(|| anyhow::anyhow!("Not in a gj-managed worktree"))?;

    println!("{}", state.origin_repo.display());
    Ok(())
}

/// Navigate to a worktree by name
fn cd_to_worktree(name: &str) -> Result<()> {
    let states = state::list_all_states()?;

    // Find worktree matching the name (check last path segment or last two segments)
    let matching: Vec<_> = states
        .iter()
        .filter(|s| {
            let path = &s.worktree_path;
            // Match against last segment
            if let Some(last) = path.file_name().and_then(|n| n.to_str()) {
                if last == name {
                    return true;
                }
            }
            // Match against last two segments (repo/branch)
            let display_name = get_display_name(path);
            display_name == name || display_name.ends_with(&format!("/{}", name))
        })
        .collect();

    match matching.len() {
        0 => bail!("No worktree found matching '{}'", name),
        1 => {
            let state = matching[0];
            if !state.worktree_path.exists() {
                bail!(
                    "Worktree no longer exists at {}",
                    state.worktree_path.display()
                );
            }
            println!("{}", state.worktree_path.display());
            Ok(())
        }
        _ => {
            eprintln!("Multiple worktrees match '{}'. Please be more specific:", name);
            for s in matching {
                eprintln!("  - {}", crate::state::display_path(&s.worktree_path));
            }
            bail!("Ambiguous worktree name");
        }
    }
}

/// Interactive selection of worktree
fn cd_interactive() -> Result<()> {
    let states = state::list_all_states()?;

    if states.is_empty() {
        bail!("No managed worktrees found. Create one with `gj new` or `gj pr`.");
    }

    // Filter to only existing worktrees
    let existing_states: Vec<_> = states
        .into_iter()
        .filter(|s| s.worktree_path.exists())
        .collect();

    if existing_states.is_empty() {
        bail!("No existing worktrees found.");
    }

    // Build selection options
    let options: Vec<String> = existing_states
        .iter()
        .map(|s| {
            let display_name = get_display_name(&s.worktree_path);
            format!("{} ({})", display_name, s.branch)
        })
        .collect();

    let selection = inquire::Select::new("Select worktree:", options)
        .prompt()
        .context("Failed to get selection")?;

    // Find the selected state
    let selected_index = existing_states
        .iter()
        .position(|s| {
            let display_name = get_display_name(&s.worktree_path);
            let option = format!("{} ({})", display_name, s.branch);
            option == selection
        })
        .unwrap();

    println!("{}", existing_states[selected_index].worktree_path.display());
    Ok(())
}

/// Get the display name from a worktree path (everything after "worktrees/")
/// Example: ~/.gj/worktrees/mirakui/my_repo/gj/20260205_hello -> mirakui/my_repo/gj/20260205_hello
fn get_display_name(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy();

    // Find "worktrees/" in the path and return everything after it
    if let Some(idx) = path_str.find("worktrees/") {
        return path_str[idx + "worktrees/".len()..].to_string();
    }

    // Fallback: return last 2 components if "worktrees/" not found
    let components: Vec<_> = path
        .components()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    components
        .iter()
        .filter_map(|c| c.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_display_name_with_worktrees() {
        let path = PathBuf::from("/Users/test/.gj/worktrees/mirakui/my_repo/gj/20260205_hello");
        assert_eq!(
            get_display_name(&path),
            "mirakui/my_repo/gj/20260205_hello"
        );
    }

    #[test]
    fn test_get_display_name_with_pr() {
        let path = PathBuf::from("/Users/test/.gj/worktrees/mirakui/my_repo/pr-123");
        assert_eq!(get_display_name(&path), "mirakui/my_repo/pr-123");
    }

    #[test]
    fn test_get_display_name_fallback() {
        // Without "worktrees/" in path, falls back to last 2 components
        let path = PathBuf::from("/a/b");
        assert_eq!(get_display_name(&path), "a/b");
    }
}
