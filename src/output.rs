use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

/// JSON 出力用ヘルパー
pub fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

// --- 各コマンドの出力構造体 ---

#[derive(Serialize)]
pub struct WorktreeCreated {
    pub worktree_path: PathBuf,
    pub branch: String,
}

impl WorktreeCreated {
    pub fn print(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Text => {
                eprintln!("worktree: {}", display_path(&self.worktree_path));
                eprintln!("branch: {}", self.branch);
                println!("{}", self.worktree_path.display());
            }
            OutputFormat::Json => print_json(self)?,
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct WorktreeListEntry {
    pub display_name: String,
    pub branch: String,
    pub path: PathBuf,
    pub created_at: String,
    pub exists: bool,
}

pub fn print_worktree_list(entries: &[WorktreeListEntry], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            if entries.is_empty() {
                eprintln!("No managed worktrees found.");
            } else {
                for e in entries {
                    let marker = if e.exists { "" } else { " (not found)" };
                    println!(
                        "{:<30} {:<40} {}{}",
                        e.display_name, e.branch, e.created_at, marker
                    );
                }
            }
        }
        OutputFormat::Json => print_json(entries)?,
    }
    Ok(())
}

#[derive(Serialize)]
pub struct CdResult {
    pub path: PathBuf,
}

impl CdResult {
    pub fn print(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Text => println!("{}", self.path.display()),
            OutputFormat::Json => print_json(self)?,
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct ExitResult {
    pub path: PathBuf,
    pub removed_worktree: PathBuf,
    pub deleted_branch: String,
    pub merged: bool,
}

impl ExitResult {
    pub fn print(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Text => {
                eprintln!("removed: {}", display_path(&self.removed_worktree));
                eprintln!("branch: {}", self.deleted_branch);
                println!("{}", self.path.display());
            }
            OutputFormat::Json => print_json(self)?,
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct InitResult {
    pub config_path: PathBuf,
}

impl InitResult {
    pub fn print(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Text => {
                eprintln!(
                    "Created configuration file at {}",
                    self.config_path.display()
                );
                eprintln!("\nEdit this file to configure your repositories and hooks.");
            }
            OutputFormat::Json => print_json(self)?,
        }
        Ok(())
    }
}

fn display_path(path: &Path) -> String {
    crate::state::display_path(path)
}
