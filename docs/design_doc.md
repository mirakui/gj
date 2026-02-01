# gj - Git Jump

A CLI tool for instantly creating and managing temporary git worktree environments.

## Overview

**gj** (git jump) is a command-line tool that simplifies working with git worktrees. It enables developers to quickly spin up isolated working environments for PR reviews or experimental feature development without interrupting their current work.

## Motivation

When working on a project, developers often need to:

1. **Review PRs** without disrupting their current uncommitted work
2. **Start new feature development** in a clean environment instantly

Traditional approaches like `git stash` or manual worktree management are cumbersome. **gj** provides a streamlined workflow for these common scenarios.

## Use Cases

### 1. PR Review

```bash
# Currently working on feature-x with uncommitted changes
$ gj pr 123
# Instantly opens PR #123 in a new worktree and cd into it
# Review, test, done
$ gj exit
# Worktree cleaned up, back to original repo
```

### 2. Quick Feature Development

```bash
$ gj new
# Prompts for feature name interactively
> Enter name: awesome-feature
# Creates worktree with branch wip/20250201_awesome-feature
$ gj exit
# Clean up when done
```

## Architecture

```
┌─────────────────────┐      ┌─────────────────────┐
│   zsh function      │ ───→ │     gj (Rust)       │
│   (cd handling)     │ ←─── │   (core logic)      │
└─────────────────────┘      └─────────────────────┘
         │                            │
         │                            ▼
         │                   ┌─────────────────────┐
         │                   │  ~/.gj/             │
         │                   │  ├── config.toml    │
         │                   │  └── state/         │
         │                   │      └── <hash>.json│
         │                   └─────────────────────┘
         │                            │
         ▼                            ▼
┌─────────────────────┐      ┌─────────────────────┐
│   cd to worktree    │      │   git worktree      │
│   or origin repo    │      │   operations        │
└─────────────────────┘      └─────────────────────┘
```

The Rust binary handles all git operations and outputs the target directory path to stdout. The zsh wrapper function captures this output and performs the `cd` operation.

## Commands

### `gj pr <number> [--no-cd]`

Creates a worktree for reviewing a GitHub PR.

- Fetches the PR branch name via `gh pr view <number> --json headRefName`
- Creates a worktree at the configured location
- Changes directory to the new worktree

| Option | Description |
|--------|-------------|
| `--no-cd` | Do not change directory. Outputs the worktree path to stdout instead. |

### `gj new [branch-name] [--no-cd]`

Creates a new worktree for feature development.

- If `branch-name` is provided, uses it directly
- If omitted, prompts interactively for a name
- Branch naming format: `<prefix>/<YYYYMMDD>_<input>`

| Option | Description |
|--------|-------------|
| `--no-cd` | Do not change directory. Outputs the worktree path to stdout instead. |

### `gj checkout <remote-branch> [--no-cd]`

Alias: `gj co`

Creates a worktree from an existing remote branch.

- Fetches the specified remote branch
- Creates a worktree tracking that branch
- Changes directory to the new worktree

| Option | Description |
|--------|-------------|
| `--no-cd` | Do not change directory. Outputs the worktree path to stdout instead. |

**Example:**

```bash
$ gj checkout origin/feature/some-branch
# or
$ gj co origin/feature/some-branch
```

### `gj list`

Displays all worktrees managed by gj.

```
my-app/pr-123
  Branch:  wip/20250201_fix
  Path:    ~/.gj/worktrees/my-app/pr-123
  Created: 2 hours ago

my-app/awesome-feature
  Branch:  wip/20250201_awesome-feature
  Path:    ~/.gj/worktrees/my-app/awesome-feature
  Created: 1 day ago
```

### `gj cd [name | @]`

Switches to an existing worktree or the origin repository.

- If `name` is provided, changes to that worktree directly
- If `@` is provided, changes to the origin repository (without deleting the worktree)
- If omitted, shows an interactive selector

**Example:**

```bash
# In a worktree, go back to origin repo without cleanup
$ gj cd @

# Switch to a specific worktree
$ gj cd pr-123

# Interactive selection
$ gj cd
```

### `gj exit [--force]`

Cleans up the current worktree and returns to the origin repository.

- Deletes the worktree directory
- Deletes the associated local branch
- Returns to the origin repository directory
- Fails if there are uncommitted changes (unless `--force` is specified)

**Note:** Remote branches are not deleted.

### `gj shell-init <shell>`

Outputs shell initialization script for the specified shell.

```bash
# Add to ~/.zshrc
eval "$(gj shell-init zsh)"
```

Currently supported: `zsh`

## Configuration

### File Location

`~/.gj/config.toml`

### Schema

```toml
# Default settings for all repositories
[default]
base_dir = "~/.gj/worktrees"  # Base directory for worktrees
prefix = "wip"                 # Default branch prefix

# Default hooks (applied to all repositories)
[[default.hooks.post_create]]
type = "run"
command = "echo 'worktree created'"

# Repository-specific settings
[repos.my-app]
path = "~/dev/my-app"                    # Path to the repository (required)
base_dir = "~/.gj/worktrees/my-app"      # Override base_dir (optional)
prefix = "feature"                        # Override prefix (optional)

# Repository-specific hooks (merged with default hooks)
[[repos.my-app.hooks.post_create]]
type = "copy"
from = ".env"
required = false  # Skip if file doesn't exist (default: false)

[[repos.my-app.hooks.post_create]]
type = "copy"
from = ".nvmrc"

[[repos.my-app.hooks.post_create]]
type = "run"
command = "npm install"

[repos.oss-project]
path = "~/dev/oss-project"
prefix = "contrib"
```

### Hooks

Hooks are executed after worktree creation. Default hooks and repository-specific hooks are **merged** (both are executed).

**Execution order:**
1. Default hooks (`default.hooks.post_create`)
2. Repository-specific hooks (`repos.<name>.hooks.post_create`)

#### Hook Types

**`copy`** - Copy a file from the origin repository to the worktree

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | string | yes | - | Must be `"copy"` |
| `from` | string | yes | - | Source path relative to origin repository |
| `to` | string | no | same as `from` | Destination path relative to worktree |
| `required` | bool | no | `false` | If `true`, fail when file doesn't exist. If `false`, skip silently. |

**`run`** - Execute a shell command in the worktree directory

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | string | yes | - | Must be `"run"` |
| `command` | string | yes | - | Shell command to execute |

#### Example

```toml
[[repos.my-app.hooks.post_create]]
type = "copy"
from = ".env.local"
to = ".env"
required = true  # Fail if .env.local doesn't exist

[[repos.my-app.hooks.post_create]]
type = "copy"
from = ".nvmrc"
# required = false (default) - skip if not found

[[repos.my-app.hooks.post_create]]
type = "run"
command = "npm install"

[[repos.my-app.hooks.post_create]]
type = "run"
command = "code ."
```

### Repository Identification

Repositories are identified by matching the current git repository root (via `git rev-parse --show-toplevel`) against the configured `repos.*.path` values.

**If the current repository is not registered in the configuration file, gj will exit with an error.**

## State Management

### File Location

`~/.gj/state/<worktree-path-hash>.json`

The hash is computed from the worktree's absolute path.

### Schema

```json
{
  "worktree_path": "/home/user/.gj/worktrees/my-app/pr-123",
  "origin_repo": "/home/user/dev/my-app",
  "branch": "wip/20250201_fix",
  "created_at": "2025-02-01T10:30:00Z"
}
```

This state file enables:
- `gj exit` to know which origin repository to return to
- `gj list` to display worktree information
- Tracking worktree lifecycle

## Shell Integration

### Setup

Add the following line to your `~/.zshrc`:

```zsh
eval "$(gj shell-init zsh)"
```

### `gj shell-init <shell>`

Outputs shell-specific initialization script to stdout.

Currently supported shells:
- `zsh`

**Example output for zsh:**

```zsh
function gj() {
  local output
  output=$(command gj "$@")
  local exit_code=$?
  
  if [[ $exit_code -eq 0 && -d "$output" ]]; then
    cd "$output"
  else
    echo "$output"
    return $exit_code
  fi
}
```

## Dependencies

### Runtime

- **git**: Core worktree operations
- **gh**: GitHub CLI for PR branch resolution

### Build

- **Rust**: Core implementation
- Recommended crates:
  - `clap` - Argument parsing
  - `serde` / `toml` - Configuration parsing
  - `serde_json` - State file handling
  - `dialoguer` or `inquire` - Interactive prompts
  - `chrono` - Date/time handling
  - `dirs` - XDG directory resolution
  - `sha2` or similar - Path hashing

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Not in a git repository | Error with message |
| Repository not registered | Error: "Repository not registered. Add it to ~/.gj/config.toml" |
| `gh` CLI not installed | Error when using `gj pr` |
| PR not found | Error with message from `gh` |
| Uncommitted changes on `gj exit` | Error unless `--force` is specified |
| Worktree already exists | Error with suggestion to use `gj cd` |

## Future Considerations

The following features are out of scope for the initial version but may be considered in the future:

- **GitLab / Bitbucket support**: Extend PR fetching beyond GitHub
- **Pre-exit hooks**: Run custom commands before worktree deletion (e.g., cleanup)
- **Auto-cleanup**: Remove stale worktrees after a configurable period
- **bash/fish support**: Extend `gj shell-init` for additional shells

## Glossary

| Term | Definition |
|------|------------|
| Origin repository | The main clone of a git repository where worktrees are created from |
| Worktree | A linked working tree created by `git worktree add` |
| State file | JSON file tracking metadata about a gj-managed worktree |