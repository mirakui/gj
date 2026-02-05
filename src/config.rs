use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub default: DefaultConfig,
    #[serde(default)]
    pub repos: HashMap<String, RepoConfig>,
}

/// Default settings applied to all repositories
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DefaultConfig {
    /// Base directory for worktrees (default: ~/.gj/worktrees)
    pub base_dir: Option<String>,
    /// Default branch prefix (default: gj)
    pub prefix: Option<String>,
    /// Default hooks
    #[serde(default)]
    pub hooks: HooksConfig,
}

/// Repository-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepoConfig {
    /// Path to the repository (required)
    pub path: String,
    /// Override base_dir for this repository
    pub base_dir: Option<String>,
    /// Override prefix for this repository
    pub prefix: Option<String>,
    /// Repository-specific hooks
    #[serde(default)]
    pub hooks: HooksConfig,
}

/// Hooks configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HooksConfig {
    /// Hooks executed after worktree creation
    #[serde(default)]
    pub post_create: Vec<Hook>,
}

/// Hook definition
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Hook {
    /// Copy a file from origin repo to worktree
    Copy {
        from: String,
        to: Option<String>,
        #[serde(default)]
        required: bool,
    },
    /// Run a shell command in the worktree
    Run { command: String },
}

impl Config {
    /// Load configuration from the default config file location
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

        Ok(config)
    }

    /// Load configuration, returning an error if the config file does not exist
    pub fn load_required() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            anyhow::bail!(
                "Configuration file not found at {}\n\n\
                Run `gj init` to create a configuration file.",
                config_path.display()
            );
        }

        Self::load()
    }

    /// Get the configuration directory path (~/.gj)
    pub fn config_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home_dir.join(".gj"))
    }

    /// Get the configuration file path (~/.gj/config.toml)
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Find repository configuration by matching the git root path
    pub fn find_repo(&self, git_root: &Path) -> Option<(&String, &RepoConfig)> {
        let git_root = git_root.canonicalize().ok()?;

        for (name, repo_config) in &self.repos {
            let expanded = shellexpand::tilde(&repo_config.path);
            if let Ok(repo_path) = PathBuf::from(expanded.as_ref()).canonicalize() {
                if repo_path == git_root {
                    return Some((name, repo_config));
                }
            }
        }
        None
    }

    /// Get the base directory for worktrees
    pub fn get_base_dir(&self, repo_config: Option<&RepoConfig>) -> PathBuf {
        let base_dir = repo_config
            .and_then(|r| r.base_dir.as_ref())
            .or(self.default.base_dir.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("~/.gj/worktrees");

        let expanded = shellexpand::tilde(base_dir);
        PathBuf::from(expanded.as_ref())
    }

    /// Get the branch prefix
    pub fn get_prefix<'a>(&'a self, repo_config: Option<&'a RepoConfig>) -> &'a str {
        repo_config
            .and_then(|r| r.prefix.as_ref())
            .or(self.default.prefix.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("gj")
    }

    /// Get all hooks (merged default + repo-specific)
    pub fn get_hooks<'a>(&'a self, repo_config: Option<&'a RepoConfig>) -> Vec<&'a Hook> {
        let mut hooks: Vec<&Hook> = self.default.hooks.post_create.iter().collect();

        if let Some(repo) = repo_config {
            hooks.extend(repo.hooks.post_create.iter());
        }

        hooks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.repos.is_empty());
        assert!(config.default.base_dir.is_none());
        assert!(config.default.prefix.is_none());
    }

    #[test]
    fn test_config_dir_location() {
        let dir = Config::config_dir().unwrap();
        let home = dirs::home_dir().unwrap();
        let expected = home.join(".gj");
        assert_eq!(dir, expected);
    }

    #[test]
    fn test_config_path_location() {
        let path = Config::config_path().unwrap();
        let home = dirs::home_dir().unwrap();
        let expected = home.join(".gj").join("config.toml");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_config_parse() {
        let toml_content = r#"
[default]
base_dir = "~/.gj/worktrees"
prefix = "gj"

[[default.hooks.post_create]]
type = "run"
command = "echo 'created'"

[repos.my-app]
path = "~/dev/my-app"
prefix = "feature"

[[repos.my-app.hooks.post_create]]
type = "copy"
from = ".env"
required = true

[[repos.my-app.hooks.post_create]]
type = "run"
command = "npm install"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();

        assert_eq!(config.default.base_dir, Some("~/.gj/worktrees".to_string()));
        assert_eq!(config.default.prefix, Some("gj".to_string()));
        assert_eq!(config.default.hooks.post_create.len(), 1);

        let repo = config.repos.get("my-app").unwrap();
        assert_eq!(repo.path, "~/dev/my-app");
        assert_eq!(repo.prefix, Some("feature".to_string()));
        assert_eq!(repo.hooks.post_create.len(), 2);

        // Check hooks
        match &repo.hooks.post_create[0] {
            Hook::Copy { from, to, required } => {
                assert_eq!(from, ".env");
                assert!(to.is_none());
                assert!(*required);
            }
            _ => panic!("Expected Copy hook"),
        }

        match &repo.hooks.post_create[1] {
            Hook::Run { command } => {
                assert_eq!(command, "npm install");
            }
            _ => panic!("Expected Run hook"),
        }
    }

    #[test]
    fn test_get_prefix() {
        let config: Config = toml::from_str(
            r#"
[default]
prefix = "default-prefix"

[repos.with-prefix]
path = "/path/with"
prefix = "custom"

[repos.without-prefix]
path = "/path/without"
"#,
        )
        .unwrap();

        let repo_with = config.repos.get("with-prefix").unwrap();
        let repo_without = config.repos.get("without-prefix").unwrap();

        assert_eq!(config.get_prefix(Some(repo_with)), "custom");
        assert_eq!(config.get_prefix(Some(repo_without)), "default-prefix");
        // When no repo config is provided, it falls back to default config's prefix
        assert_eq!(config.get_prefix(None), "default-prefix");

        // Test with no default prefix configured
        let config_no_default: Config = toml::from_str(
            r#"
[repos.test]
path = "/path/test"
"#,
        )
        .unwrap();
        assert_eq!(config_no_default.get_prefix(None), "gj");
    }
}
