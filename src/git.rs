use crate::error::{GitSwitchError, Result};
use crate::utils::run_command_with_full_output;

pub fn update_git_remote(remote_name: &str, remote_url: &str) -> Result<()> {
    let output =
        run_command_with_full_output("git", &["remote", "set-url", remote_name, remote_url], None)?;
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
    Err(GitSwitchError::GitRemoteUrlNotFound {
        remote_name: remote_name.to_string(),
    })
}

pub fn is_git_repository() -> Result<bool> {
    // The `?` operator will propagate errors from run_command_with_full_output,
    // such as GitSwitchError::CommandExecution if 'git' command is not found.
    let output =
        run_command_with_full_output("git", &["rev-parse", "--is-inside-work-tree"], None)?;

    if output.status.success() {
        // Command succeeded, stdout should be "true"
        Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
    } else {
        // Command executed but failed. Check if it's because it's not a git repository.
        let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
        // Typical message for not a git repository: "fatal: not a git repository..."
        if stderr.contains("not a git repository") || stderr.contains("fatal: not a git repository")
        {
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

// Alias for backward compatibility and intuitive naming
pub fn is_in_git_repository() -> Result<bool> {
    is_git_repository()
}

/// Set global Git configuration
pub fn set_global_config(username: &str, email: &str) -> Result<()> {
    run_command_with_full_output("git", &["config", "--global", "user.name", username], None)?;
    run_command_with_full_output("git", &["config", "--global", "user.email", email], None)?;
    Ok(())
}

/// Set local Git configuration for current repository
pub fn set_local_config(username: &str, email: &str) -> Result<()> {
    run_command_with_full_output("git", &["config", "--local", "user.name", username], None)?;
    run_command_with_full_output("git", &["config", "--local", "user.email", email], None)?;
    Ok(())
}

/// Get global Git configuration
pub fn get_global_config() -> Result<(String, String)> {
    let name_output =
        run_command_with_full_output("git", &["config", "--global", "user.name"], None)?;
    let email_output =
        run_command_with_full_output("git", &["config", "--global", "user.email"], None)?;

    if !name_output.status.success() || !email_output.status.success() {
        return Err(GitSwitchError::Other(
            "Failed to get global Git config".to_string(),
        ));
    }

    let name = String::from_utf8_lossy(&name_output.stdout)
        .trim()
        .to_string();
    let email = String::from_utf8_lossy(&email_output.stdout)
        .trim()
        .to_string();

    Ok((name, email))
}

/// Get local Git configuration for current repository
pub fn get_local_config() -> Result<(String, String)> {
    let name_output =
        run_command_with_full_output("git", &["config", "--local", "user.name"], None)?;
    let email_output =
        run_command_with_full_output("git", &["config", "--local", "user.email"], None)?;

    if !name_output.status.success() || !email_output.status.success() {
        return Err(GitSwitchError::Other(
            "Failed to get local Git config".to_string(),
        ));
    }

    let name = String::from_utf8_lossy(&name_output.stdout)
        .trim()
        .to_string();
    let email = String::from_utf8_lossy(&email_output.stdout)
        .trim()
        .to_string();

    Ok((name, email))
}

/// Get remote URL (alias for get_git_remote_url)
pub fn get_remote_url(remote_name: &str) -> Result<String> {
    get_git_remote_url(remote_name)
}

/// Set remote URL
pub fn set_remote_url(remote_name: &str, url: &str) -> Result<()> {
    update_git_remote(remote_name, url)
}

/// Set SSH command for Git
pub fn set_ssh_command(ssh_key_path: &str) -> Result<()> {
    let ssh_command = format!("ssh -i {}", ssh_key_path);
    run_command_with_full_output("git", &["config", "core.sshCommand", &ssh_command], None)?;
    Ok(())
}

/// Get current branch name
pub fn get_current_branch() -> Result<String> {
    let output = run_command_with_full_output("git", &["branch", "--show-current"], None)?;
    if !output.status.success() {
        return Err(GitSwitchError::GitCommandFailed {
            command: "git branch --show-current".to_string(),
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Set local git config for a specific key-value pair
pub fn set_local_config_key(key: &str, value: &str) -> Result<()> {
    let output = run_command_with_full_output("git", &["config", key, value], None)?;
    if !output.status.success() {
        return Err(GitSwitchError::GitCommandFailed {
            command: format!("git config {} {}", key, value),
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

/// Get local git config for a specific key
pub fn get_local_config_key(key: &str) -> Result<String> {
    let output = run_command_with_full_output("git", &["config", "--local", key], None)?;
    if !output.status.success() {
        return Err(GitSwitchError::GitCommandFailed {
            command: format!("git config --local {}", key),
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Set global git config for a specific key-value pair
#[allow(dead_code)]
pub fn set_global_config_key(key: &str, value: &str) -> Result<()> {
    let output = run_command_with_full_output("git", &["config", "--global", key, value], None)?;
    if !output.status.success() {
        return Err(GitSwitchError::GitCommandFailed {
            command: format!("git config --global {} {}", key, value),
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

/// Get global git config for a specific key
#[allow(dead_code)]
pub fn get_global_config_key(key: &str) -> Result<String> {
    let output = run_command_with_full_output("git", &["config", "--global", key], None)?;
    if !output.status.success() {
        return Err(GitSwitchError::GitCommandFailed {
            command: format!("git config --global {}", key),
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
