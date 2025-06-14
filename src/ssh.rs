use crate::error::{GitSwitchError, Result};
use crate::utils::{
    ensure_parent_dir_exists, expand_path, read_file_content, run_command, run_command_with_output,
    write_file_content,
};
use colored::*;
use std::path::{Path, PathBuf};

fn get_ssh_dir_path() -> Result<PathBuf> {
    home::home_dir()
        .map(|home| home.join(".ssh"))
        .ok_or(GitSwitchError::HomeDirectoryNotFound)
}

fn get_ssh_config_file_path() -> Result<PathBuf> {
    get_ssh_dir_path().map(|ssh_dir| ssh_dir.join("config"))
}

pub fn generate_ssh_key(identity_file_path: &Path) -> Result<()> {
    if identity_file_path.exists() {
        return Ok(());
    }

    ensure_parent_dir_exists(identity_file_path)?;

    // Generate SSH key quietly
    run_command_with_output(
        "ssh-keygen",
        &[
            "-t",
            "rsa",
            "-b",
            "4096",
            "-f",
            identity_file_path
                .to_str()
                .ok_or_else(|| GitSwitchError::PathExpansion {
                    path: format!("{:?}", identity_file_path),
                })?,
            "-N",
            "",   // No passphrase
            "-q", // Quiet mode
        ],
        None, // No specific current_dir needed
    )
    .map_err(|e| GitSwitchError::SshKeyGeneration {
        message: format!(
            "Failed to generate SSH key at {}: {}",
            identity_file_path.display(),
            e
        ),
    })?;

    Ok(())
}

pub fn display_public_key_formatted(identity_file_path: &Path) -> Result<()> {
    let public_key_path = identity_file_path.with_extension("pub");
    if !public_key_path.exists() {
        return Err(GitSwitchError::SshKeyGeneration {
            message: format!(
                "Public key file not found at: {}",
                public_key_path.display()
            ),
        });
    }

    let content = read_file_content(&public_key_path)?;
    let key_content = content.trim();

    // Split the key into parts: type, key data, comment
    let parts: Vec<&str> = key_content.splitn(3, ' ').collect();

    if parts.len() >= 2 {
        // Show key type
        println!("{} {}", "Type:".dimmed(), parts[0].bold());

        // Show truncated key for readability
        let key_data = parts[1];
        let key_preview = if key_data.len() > 60 {
            format!("{}...{}", &key_data[..30], &key_data[key_data.len() - 30..])
        } else {
            key_data.to_string()
        };
        println!("{} {}", "Key:".dimmed(), key_preview);

        // Show comment (usually username@hostname)
        if parts.len() > 2 && !parts[2].is_empty() {
            println!("{} {}", "Comment:".dimmed(), parts[2].bright_black());
        }

        println!(
            "\n{} {}",
            "💾".yellow(),
            "Full key (select all to copy):".dimmed()
        );
        println!("{}", format!("┌{}┐", "─".repeat(78)).bright_black());

        // Print the full key wrapped nicely
        let chars: Vec<char> = key_content.chars().collect();
        for chunk in chars.chunks(76) {
            let line: String = chunk.iter().collect();
            println!("{}", format!("│ {} │", line).bright_black());
        }

        println!("{}", format!("└{}┘", "─".repeat(78)).bright_black());
    } else {
        // Fallback to simple display
        println!("{}", key_content);
    }

    Ok(())
}

pub fn update_ssh_config(account_name: &str, identity_file_path_str: &str) -> Result<()> {
    let identity_file_path = expand_path(identity_file_path_str)?; // Expand tilde
    let config_path = get_ssh_config_file_path()?;
    ensure_parent_dir_exists(&config_path)?;

    // Use a more specific host alias to avoid potential conflicts and ensure clarity
    let host_alias = format!(
        "github.com-{}",
        account_name.replace(" ", "_").to_lowercase()
    );
    let identity_file_display = identity_file_path.to_str().unwrap_or("INVALID_PATH");

    let config_entry = format!(
        "\n# {} GitHub Account (git-switch managed)\nHost {}\n  HostName github.com\n  User git\n  IdentityFile {}\n  IdentitiesOnly yes\n",
        account_name, host_alias, identity_file_display
    );

    let mut current_config = if config_path.exists() {
        read_file_content(&config_path)?
    } else {
        String::new()
    };

    // Prevent duplicate entries
    if current_config.contains(&format!("Host {}", host_alias)) {
        return Ok(());
    }

    current_config.push_str(&config_entry);
    write_file_content(&config_path, &current_config)?;

    Ok(())
}

pub fn add_ssh_key(key_path_str: &str) -> Result<bool> {
    let expanded_key_path = expand_path(key_path_str)?;

    if !expanded_key_path.exists() {
        return Err(GitSwitchError::SshKeyGeneration {
            // Reusing for "key not found"
            message: format!("SSH key not found: {}", expanded_key_path.display()),
        });
    }

    let key_path_arg = expanded_key_path
        .to_str()
        .ok_or_else(|| GitSwitchError::PathExpansion {
            path: format!("{:?}", expanded_key_path),
        })?;

    // Attempt to add the key. ssh-add will typically succeed if the key is valid
    // and the agent is running. It might print to stderr if already added.
    // We're interested if the command *fails* catastrophically.
    println!(
        "🔑 Adding SSH key to agent: {}",
        expanded_key_path.display()
    );
    match run_command("ssh-add", &[key_path_arg], None) {
        Ok(_) => Ok(true), // Assume success means it's added or already there and usable.
        Err(e) => {
            let error_msg = e.to_string();
            // Check if it's because the agent is not running
            if error_msg.contains("Could not open a connection to your authentication agent") {
                eprintln!(
                    "⚠️ ssh-agent not running or inaccessible. Please start it (e.g., `eval $(ssh-agent -s)`) and try again."
                );
                Ok(false) // Indicate key was not added due to agent issue
            } else if error_msg.contains("error in libcrypto")
                || error_msg.contains("invalid format")
            {
                // Handle invalid key format errors more gracefully in test environments
                eprintln!(
                    "⚠️ SSH key format issue detected. This may be expected in test environments."
                );
                Ok(false) // Indicate key was not added due to format issue
            } else {
                Err(GitSwitchError::SshCommand {
                    command: "ssh-add".to_string(),
                    message: format!("Failed to add key {}: {}", expanded_key_path.display(), e),
                })
            }
        }
    }
}

pub fn remove_ssh_config_entry(account_name: &str) -> Result<()> {
    let config_path = get_ssh_config_file_path()?;
    if !config_path.exists() {
        println!(
            "ℹ️ SSH config file not found at {}. Nothing to remove.",
            config_path.display()
        );
        return Ok(());
    }

    let original_content = read_file_content(&config_path)?;
    let mut new_content_lines = Vec::new();
    let mut in_matching_block = false;
    // Ensure the host_marker matches the one used in update_ssh_config
    let host_marker = format!(
        "Host github.com-{}",
        account_name.replace(" ", "_").to_lowercase()
    );
    let comment_marker = format!("# {} GitHub Account (git-switch managed)", account_name);

    for line in original_content.lines() {
        if line.trim() == comment_marker || line.trim().starts_with(&host_marker) {
            in_matching_block = true;
            // Skip this line and subsequent lines of the block
        } else if in_matching_block
            && (line.trim().starts_with("Host ") || line.trim().starts_with("# "))
        {
            // Reached the start of a new Host block or a new top-level comment, so the previous block ended
            in_matching_block = false;
            new_content_lines.push(line.to_string());
        } else if !in_matching_block {
            new_content_lines.push(line.to_string());
        }
        // If in_matching_block is true and it's not a new Host line, the line is part of the block to remove, so we do nothing.
    }

    // Edge case: if the block to remove was at the very end of the file
    // in_matching_block might still be true here. The logic should handle it.

    let new_content = new_content_lines.join("\n");

    if new_content.trim() == original_content.trim() {
        println!(
            "ℹ️ No SSH config entry found for account \'{}\' to remove.",
            account_name
        );
    } else {
        write_file_content(&config_path, &new_content)?;
        println!(
            "✅ SSH config entry for account \'{}\' removed.",
            account_name
        );
    }

    Ok(())
}
