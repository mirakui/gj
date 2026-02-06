# gj
A CLI tool for instantly creating and managing temporary git worktree environments.

## Setup

```sh
# Initialize shell integration (add to .zshrc or .bashrc)
eval "$(gj shell-init zsh)"

# Initialize configuration file in current repository
gj init
```

## Usage

### `gj new [BRANCH_SUFFIX]`

Create a new worktree for feature development.

```sh
gj new my-feature       # Create worktree with branch "my-feature"
gj new                  # Prompt for branch suffix interactively
gj new --random-suffix  # Generate a random branch suffix
```

### `gj pr <NUMBER>`

Create a worktree for reviewing a GitHub PR.

```sh
gj pr 42
```

### `gj checkout <REMOTE_BRANCH>` (alias: `gj co`)

Create a worktree from a remote branch.

```sh
gj co main
gj checkout feature/foo
```

### `gj list` (alias: `gj ls`)

List all managed worktrees.

```sh
gj ls
```

### `gj cd [TARGET]`

Change to a worktree directory. Use `@` to go to the origin repository.

```sh
gj cd my-feature
gj cd @              # Go to origin repository
gj cd                # Select interactively
```

### `gj exit [--force] [--merge]`

Clean up the current worktree and return to origin repository.

```sh
gj exit
gj exit --merge      # Merge branch into default branch before exiting
gj exit --force      # Force removal even with uncommitted changes
```

### `gj init`

Initialize gj configuration file in the current repository.

```sh
gj init
gj init --force      # Overwrite existing configuration
```

### `gj shell-init <SHELL>`

Output shell initialization script.

```sh
eval "$(gj shell-init zsh)"
eval "$(gj shell-init bash)"
```
