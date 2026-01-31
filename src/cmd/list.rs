use anyhow::Result;
use chrono::Utc;

use crate::state;

/// Execute the `gj list` command
pub fn run() -> Result<()> {
    let states = state::list_all_states()?;

    if states.is_empty() {
        eprintln!("No managed worktrees found.");
        return Ok(());
    }

    let now = Utc::now();

    for state in states {
        // Get the last two path segments for display name
        let display_name = get_display_name(&state.worktree_path);

        // Calculate relative time
        let relative_time = format_relative_time(now, state.created_at);

        // Check if worktree still exists
        let exists_marker = if state.worktree_path.exists() {
            ""
        } else {
            " (not found)"
        };

        println!(
            "{:<30} {:<40} {}{}",
            display_name,
            state.branch,
            relative_time,
            exists_marker
        );
    }

    Ok(())
}

/// Get the display name from a worktree path (last 2 segments)
fn get_display_name(path: &std::path::Path) -> String {
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

/// Format a relative time string
fn format_relative_time(now: chrono::DateTime<Utc>, created: chrono::DateTime<Utc>) -> String {
    let duration = now.signed_duration_since(created);

    if duration.num_days() > 0 {
        let days = duration.num_days();
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        }
    } else if duration.num_hours() > 0 {
        let hours = duration.num_hours();
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else if duration.num_minutes() > 0 {
        let mins = duration.num_minutes();
        if mins == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", mins)
        }
    } else {
        "just now".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_display_name() {
        let path = PathBuf::from("/Users/test/.gj/my-repo/feature-branch");
        assert_eq!(get_display_name(&path), "my-repo/feature-branch");
    }

    #[test]
    fn test_format_relative_time() {
        let now = Utc::now();

        let just_now = now;
        assert_eq!(format_relative_time(now, just_now), "just now");

        let five_mins_ago = now - chrono::Duration::minutes(5);
        assert_eq!(format_relative_time(now, five_mins_ago), "5 minutes ago");

        let one_hour_ago = now - chrono::Duration::hours(1);
        assert_eq!(format_relative_time(now, one_hour_ago), "1 hour ago");

        let two_days_ago = now - chrono::Duration::days(2);
        assert_eq!(format_relative_time(now, two_days_ago), "2 days ago");
    }
}
