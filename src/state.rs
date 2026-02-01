use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// State information for a managed worktree
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorktreeState {
    /// Absolute path to the worktree
    pub worktree_path: PathBuf,
    /// Absolute path to the origin repository
    pub origin_repo: PathBuf,
    /// Branch name used by this worktree
    pub branch: String,
    /// When the worktree was created
    pub created_at: DateTime<Utc>,
}

impl WorktreeState {
    /// Create a new worktree state
    pub fn new(worktree_path: PathBuf, origin_repo: PathBuf, branch: String) -> Self {
        WorktreeState {
            worktree_path,
            origin_repo,
            branch,
            created_at: Utc::now(),
        }
    }

    /// Save the state to a file
    pub fn save(&self) -> Result<()> {
        let state_file = state_file_path(&self.worktree_path)?;

        // Ensure parent directory exists
        if let Some(parent) = state_file.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create state directory: {}", parent.display())
            })?;
        }

        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize worktree state")?;

        fs::write(&state_file, content)
            .with_context(|| format!("Failed to write state file: {}", state_file.display()))?;

        Ok(())
    }

    /// Load state from a worktree path
    pub fn load(worktree_path: &Path) -> Result<Option<Self>> {
        let state_file = state_file_path(worktree_path)?;

        if !state_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&state_file)
            .with_context(|| format!("Failed to read state file: {}", state_file.display()))?;

        let state: WorktreeState = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse state file: {}", state_file.display()))?;

        Ok(Some(state))
    }

    /// Load state from the current directory
    pub fn load_current() -> Result<Option<Self>> {
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        Self::load(&current_dir)
    }

    /// Delete the state file
    pub fn delete(&self) -> Result<()> {
        let state_file = state_file_path(&self.worktree_path)?;

        if state_file.exists() {
            fs::remove_file(&state_file).with_context(|| {
                format!("Failed to delete state file: {}", state_file.display())
            })?;
        }

        Ok(())
    }
}

/// Get the state directory path (~/.gj/state/)
pub fn state_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home_dir.join(".gj").join("state"))
}

/// Compute a hash for a path to use as state file name
fn path_hash(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8]) // Use first 8 bytes (16 hex chars)
}

/// Get the state file path for a worktree
fn state_file_path(worktree_path: &Path) -> Result<PathBuf> {
    let state_dir = state_dir()?;
    let hash = path_hash(worktree_path);
    Ok(state_dir.join(format!("{}.json", hash)))
}

/// List all worktree states
pub fn list_all_states() -> Result<Vec<WorktreeState>> {
    let state_dir = state_dir()?;

    if !state_dir.exists() {
        return Ok(Vec::new());
    }

    let mut states = Vec::new();

    for entry in fs::read_dir(&state_dir)
        .with_context(|| format!("Failed to read state directory: {}", state_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<WorktreeState>(&content) {
                    states.push(state);
                }
            }
        }
    }

    // Sort by creation time, newest first
    states.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(states)
}

// Hex encoding helper (to avoid another dependency)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_hash() {
        let hash1 = path_hash(Path::new("/path/to/worktree"));
        let hash2 = path_hash(Path::new("/path/to/worktree"));
        let hash3 = path_hash(Path::new("/different/path"));

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn test_state_dir_location() {
        let dir = state_dir().unwrap();
        let home = dirs::home_dir().unwrap();
        let expected = home.join(".gj").join("state");
        assert_eq!(dir, expected);
    }

    #[test]
    fn test_worktree_state_new() {
        let state = WorktreeState::new(
            PathBuf::from("/worktree"),
            PathBuf::from("/origin"),
            "feature-branch".to_string(),
        );

        assert_eq!(state.worktree_path, PathBuf::from("/worktree"));
        assert_eq!(state.origin_repo, PathBuf::from("/origin"));
        assert_eq!(state.branch, "feature-branch");
    }

    #[test]
    fn test_state_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let worktree_path = temp_dir.path().join("worktree");

        // Override state_dir for test
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        let state = WorktreeState::new(
            worktree_path.clone(),
            PathBuf::from("/origin"),
            "test-branch".to_string(),
        );

        state.save().unwrap();

        let loaded = WorktreeState::load(&worktree_path).unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.worktree_path, state.worktree_path);
        assert_eq!(loaded.origin_repo, state.origin_repo);
        assert_eq!(loaded.branch, state.branch);
    }

    #[test]
    fn test_state_delete() {
        let temp_dir = TempDir::new().unwrap();
        let worktree_path = temp_dir.path().join("worktree");

        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        let state = WorktreeState::new(
            worktree_path.clone(),
            PathBuf::from("/origin"),
            "test-branch".to_string(),
        );

        state.save().unwrap();
        assert!(WorktreeState::load(&worktree_path).unwrap().is_some());

        state.delete().unwrap();
        assert!(WorktreeState::load(&worktree_path).unwrap().is_none());
    }
}
