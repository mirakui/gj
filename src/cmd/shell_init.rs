use anyhow::{bail, Result};

const SHELL_FUNCTION: &str = r#"function gj() {
  local output
  output=$(command gj "$@")
  local exit_code=$?

  if [[ $exit_code -eq 0 && -d "$output" ]]; then
    cd "$output"
    echo "You are now in: ${output/#$HOME/~}"
  else
    echo "$output"
    return $exit_code
  fi
}
"#;

/// Execute the `gj shell-init` command
pub fn run(shell: &str) -> Result<()> {
    match shell {
        "zsh" => print!("{}", zsh_init_script()),
        "bash" => print!("{}", bash_init_script()),
        _ => bail!("Unsupported shell: {}. Supported shells: zsh, bash", shell),
    }
    Ok(())
}

/// Returns the shell initialization script for zsh
fn zsh_init_script() -> &'static str {
    SHELL_FUNCTION
}

/// Returns the shell initialization script for bash
fn bash_init_script() -> &'static str {
    SHELL_FUNCTION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zsh_init_script_contains_function_definition() {
        let script = zsh_init_script();
        assert!(script.contains("function gj()"));
    }

    #[test]
    fn test_zsh_init_script_contains_cd_logic() {
        let script = zsh_init_script();
        assert!(script.contains("cd \"$output\""));
    }

    #[test]
    fn test_bash_init_script_contains_function_definition() {
        let script = bash_init_script();
        assert!(script.contains("function gj()"));
    }

    #[test]
    fn test_bash_init_script_contains_exit_code_handling() {
        let script = bash_init_script();
        assert!(script.contains("exit_code"));
        assert!(script.contains("return $exit_code"));
    }

    #[test]
    fn test_zsh_and_bash_scripts_are_identical() {
        // Both shells use the same script
        assert_eq!(zsh_init_script(), bash_init_script());
    }

    #[test]
    fn test_run_with_unsupported_shell_returns_error() {
        let result = run("fish");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unsupported shell"));
        assert!(err.to_string().contains("fish"));
    }
}
