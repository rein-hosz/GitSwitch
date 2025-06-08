use crate::error::{GitSwitchError, Result};
use crate::utils::run_command_with_full_output;

pub fn update_git_remote(remote_name: &str, remote_url: &str) -> Result<()> {
    let output = run_command_with_full_output("git", &["remote", "set-url", remote_name, remote_url], None)?;
    if !output.status.success() {
        return Err(GitSwitchError::GitCommandFailed {
            command: format!("git remote set-url {} {}", remote_name, remote_url),
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

pub fn get_git_remote_url(remote_name: &str) -> Result<String> {
    let output = run_command_with_full_output("git", &["remote", "-v"], None)?;
    if !output.status.success() {
        return Err(GitSwitchError::GitCommandFailed {
            command: "git remote -v".to_string(),
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with(remote_name) && line.contains("(fetch)") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return Ok(parts[1].to_string());
            }
        }
    }
    Err(GitSwitchError::GitRemoteUrlNotFound { remote_name: remote_name.to_string() })
}

pub fn is_git_repository() -> Result<bool> {
    // The `?` operator will propagate errors from run_command_with_full_output,
    // such as GitSwitchError::CommandExecution if 'git' command is not found.
    let output = run_command_with_full_output("git", &["rev-parse", "--is-inside-work-tree"], None)?;

    if output.status.success() {
        // Command succeeded, stdout should be "true"
        Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
    } else {
        // Command executed but failed. Check if it's because it's not a git repository.
        let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
        // Typical message for not a git repository: "fatal: not a git repository..."
        if stderr.contains("not a git repository") || stderr.contains("fatal: not a git repository") {
            Ok(false) // It's confirmed not a git repository by the command's error output.
        } else {
            // Another type of failure from "git rev-parse --is-inside-work-tree".
            Err(GitSwitchError::GitCommandFailed {
                command: "git rev-parse --is-inside-work-tree".to_string(),
                status: output.status,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }
}

