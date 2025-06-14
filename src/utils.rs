use crate::error::{GitSwitchError, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Expands a path that may start with '~' to an absolute path.
pub fn expand_path(path_str: &str) -> Result<PathBuf> {
    if path_str.starts_with('~') {
        if let Some(home_dir) = home::home_dir() {
            let mut path = home_dir;
            if path_str.len() > 1 {
                // Handles "~/" or "~something"
                // Skip "~" or "~/"
                let rest = &path_str[1..];
                if rest.starts_with('/') || rest.starts_with('\\') {
                    if rest.len() > 1 {
                        path.push(&rest[1..]);
                    }
                } else if !rest.is_empty() {
                    path.push(rest);
                }
            }
            Ok(path)
        } else {
            Err(GitSwitchError::HomeDirectoryNotFound)
        }
    } else {
        Ok(PathBuf::from(path_str))
    }
}

/// Ensures that the directory for the given path exists, creating it if necessary.
/// This function checks the parent directory of the provided path.
pub fn ensure_parent_dir_exists(path: &Path) -> Result<()> {
    if let Some(parent_dir) = path.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).map_err(|e| {
                GitSwitchError::Io(io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to create directory {}: {}",
                        parent_dir.display(),
                        e
                    ),
                ))
            })?;
        }
    }
    Ok(())
}

/// Reads the content of a file into a string.
pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| {
        GitSwitchError::Io(io::Error::new(
            e.kind(),
            format!("Failed to read file {}: {}", path.display(), e),
        ))
    })
}

/// Writes string content to a file.
pub fn write_file_content(path: &Path, content: &str) -> Result<()> {
    ensure_parent_dir_exists(path)?;
    fs::write(path, content).map_err(|e| {
        GitSwitchError::Io(io::Error::new(
            e.kind(),
            format!("Failed to write file {}: {}", path.display(), e),
        ))
    })
}

/// Runs a command and waits for it to complete, returning its status.
pub fn run_command(command_str: &str, args: &[&str], current_dir: Option<&Path>) -> Result<()> {
    let mut cmd = Command::new(command_str);
    cmd.args(args);
    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }

    let status = cmd.status().map_err(|e| {
        GitSwitchError::CommandExecution {
            command: command_str.to_string(),
            message: format!("Failed to spawn command: {}", e),
        }
    })?;

    if !status.success() {
        return Err(GitSwitchError::CommandExecution {
            command: command_str.to_string(),
            message: format!(
                "Command with args '{}' failed with status: {}",
                args.join(" "),
                status
            ),
        });
    }
    Ok(())
}

/// Runs a command and returns its output (stdout, stderr, status).
#[allow(dead_code)]
pub fn run_command_with_output(
    command_str: &str,
    args: &[&str],
    current_dir: Option<&Path>,
) -> Result<Output> {
    let mut cmd = Command::new(command_str);
    cmd.args(args);
    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| {
        GitSwitchError::CommandExecution {
            command: command_str.to_string(),
            message: format!("Failed to spawn command for output: {}", e),
        }
    })?;

    if !output.status.success() {
        return Err(GitSwitchError::CommandExecution {
            command: command_str.to_string(),
            message: format!(
                "Command with args '{}' failed with status: {}. Stderr: {}",
                args.join(" "),
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }
    Ok(output)
}

/// Runs a command and returns its output (stdout, stderr, status), including stderr even on success.
pub fn run_command_with_full_output(
    command_str: &str,
    args: &[&str],
    current_dir: Option<&Path>,
) -> Result<Output> {
    let mut cmd = Command::new(command_str);
    cmd.args(args);
    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }

    cmd.output().map_err(|e| {
        GitSwitchError::CommandExecution {
            command: command_str.to_string(),
            message: format!("Failed to spawn command for full output: {}", e),
        }
    })
}
