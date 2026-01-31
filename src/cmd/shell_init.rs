use anyhow::{bail, Result};

/// Execute the `gj shell-init` command
pub fn run(shell: &str) -> Result<()> {
    match shell {
        "zsh" => print_zsh_init(),
        "bash" => print_bash_init(),
        _ => bail!("Unsupported shell: {}. Supported shells: zsh, bash", shell),
    }
    Ok(())
}

fn print_zsh_init() {
    print!(
        r#"function gj() {{
  local output
  output=$(command gj "$@")
  local exit_code=$?

  if [[ $exit_code -eq 0 && -d "$output" ]]; then
    cd "$output"
  else
    echo "$output"
    return $exit_code
  fi
}}
"#
    );
}

fn print_bash_init() {
    print!(
        r#"function gj() {{
  local output
  output=$(command gj "$@")
  local exit_code=$?

  if [[ $exit_code -eq 0 && -d "$output" ]]; then
    cd "$output"
  else
    echo "$output"
    return $exit_code
  fi
}}
"#
    );
}
