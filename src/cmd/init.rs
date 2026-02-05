use anyhow::{bail, Result};
use std::fs;

use crate::config::Config;

/// Default configuration template with comments
const CONFIG_TEMPLATE: &str = r#"# gj configuration file
# See: https://github.com/user/gj for documentation

[default]
# Base directory for worktrees (default: ~/.gj/worktrees)
# base_dir = "~/.gj/worktrees"

# Default branch prefix (default: gj)
# prefix = "gj"

# Example: Default hooks applied to all repositories
# [[default.hooks.post_create]]
# type = "run"
# command = "echo 'Worktree created!'"

# Example: Repository-specific configuration
# [repos.my-app]
# path = "~/dev/my-app"
# prefix = "feature"
#
# [[repos.my-app.hooks.post_create]]
# type = "copy"
# from = ".env"
# required = true
#
# [[repos.my-app.hooks.post_create]]
# type = "run"
# command = "npm install"
"#;

/// Execute the `gj init` command
pub fn run(force: bool) -> Result<()> {
    let config_dir = Config::config_dir()?;
    let config_path = Config::config_path()?;

    // Check if config file already exists
    if config_path.exists() && !force {
        bail!(
            "Configuration file already exists at {}\n\n\
            Use `gj init --force` to overwrite.",
            config_path.display()
        );
    }

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    // Write the configuration template
    fs::write(&config_path, CONFIG_TEMPLATE)?;

    eprintln!("Created configuration file at {}", config_path.display());
    eprintln!("\nEdit this file to configure your repositories and hooks.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_template_is_valid_toml() {
        let result: Result<Config, _> = toml::from_str(CONFIG_TEMPLATE);
        assert!(result.is_ok(), "Template should be valid TOML: {:?}", result.err());
    }
}
