use crate::config::{self, Account};
use crate::error::{GitSwitchError, Result};
use crate::git;
use crate::ssh;
use crate::utils; // Assuming utils functions are used, otherwise remove
use std::fs;
use std::io::{self, Write};

pub fn add_account(name: &str, username: &str, email: &str) -> Result<()> {
    // Define SSH key path string (tilde path for storage)
    let ssh_key_path_str = format!("~/.ssh/id_rsa_{}", name.replace(" ", "_").to_lowercase());
    // Expand it for immediate use
    let expanded_key_path = utils::expand_path(&ssh_key_path_str)?;

    // Ensure parent directory for SSH key exists
    utils::ensure_parent_dir_exists(&expanded_key_path)?;

    // Generate SSH key
    ssh::generate_ssh_key(&expanded_key_path)?;

    let account = Account {
        name: name.to_string(),
        username: username.to_string(),
        email: email.to_string(),
        ssh_key_path: ssh_key_path_str.clone(), // Store the tilde version
    };

    // Save account to config file
    config::save_account(&account)?;

    // Update SSH config (~/.ssh/config)
    // Pass the tilde version of the path to update_ssh_config, it will expand it.
    ssh::update_ssh_config(name, &ssh_key_path_str)?;

    println!("✅ Account '{}' added successfully!", name);
    println!("
🔑 Here is your public SSH key to add to GitHub:");
    println!("--------------------------------------------------");
    ssh::display_public_key(&expanded_key_path)?;
    println!("--------------------------------------------------");
    println!(
        "Copy this key and add it to your GitHub account at: https://github.com/settings/keys"
    );
    Ok(())
}

pub fn use_account(name_or_username: &str) -> Result<()> {
    let account = config::find_account(name_or_username)?;

    // Set Git global config
    utils::run_command("git", &["config", "--global", "user.name", &account.username], None)?;
    utils::run_command("git", &["config", "--global", "user.email", &account.email], None)?;

    println!("🔄 Ensuring SSH agent is running and key is added...");
    
    // Add SSH key to agent. ssh::add_ssh_key now returns Result<bool>
    // where bool indicates if the key was newly added or if an issue like agent not running occurred.
    match ssh::add_ssh_key(&account.ssh_key_path) {
        Ok(true) => { // Key was successfully added or confirmed in agent
            println!(
                "✅ Switched to Git account: {} ({})",
                account.name, account.username
            );

            print!("Do you want to update remote URL for the current repository? (y/n): ");
            io::stdout().flush().map_err(GitSwitchError::Io)?;
            let mut response = String::new();
            io::stdin().read_line(&mut response).map_err(GitSwitchError::Io)?;

            if response.trim().to_lowercase() == "y" {
                print!("Enter repository name (e.g., 'owner/repo'): ");
                io::stdout().flush().map_err(GitSwitchError::Io)?;
                let mut repo_input = String::new();
                io::stdin().read_line(&mut repo_input).map_err(GitSwitchError::Io)?;
                
                // Pass account name and repo input to update_git_remote
                git::update_git_remote(&account.name, repo_input.trim())?;
            }
        }
        Ok(false) => { // Key was not added (e.g., agent issue, but not a hard error from ssh::add_ssh_key)
             eprintln!("⚠️ SSH key for '{}' could not be added to the agent. Please check agent status or add manually.", account.name);
             // Optionally, you might still want to proceed or return a specific error/message
        }
        Err(e) => { // An actual error occurred during ssh::add_ssh_key
            return Err(e);
        }
    }
    Ok(())
}

pub fn list_accounts() -> Result<()> {
    config::list_accounts()
}

pub fn remove_account(name: &str) -> Result<()> {
    let mut current_config = config::load_config()?;
    
    // Find the account details before removing from config, to get ssh_key_path
    let account_to_remove = match current_config.accounts.get(name) {
        Some(acc) => acc.clone(), // Clone to use after removal from map
        None => return Err(GitSwitchError::AccountNotFound { name: name.to_string() }),
    };

    // 1. Remove from config
    if current_config.accounts.remove(name).is_some() {
        config::save_config(&current_config)?;
        println!("✅ Account '{}' removed from configuration.", name);

        // 2. Optionally, remove the associated SSH key file
        print!("Do you want to delete the associated SSH key file '{}'? (y/n): ", account_to_remove.ssh_key_path);
        io::stdout().flush().map_err(GitSwitchError::Io)?;
        let mut response = String::new();
        io::stdin().read_line(&mut response).map_err(GitSwitchError::Io)?;

        if response.trim().to_lowercase() == "y" {
            match utils::expand_path(&account_to_remove.ssh_key_path) {
                Ok(expanded_path) => {
                    if expanded_path.exists() {
                        match fs::remove_file(&expanded_path) {
                            Ok(_) => println!("🔑 SSH key file '{}' deleted.", expanded_path.display()),
                            Err(e) => eprintln!("⚠️ Warning: Failed to delete SSH key file {}: {}", expanded_path.display(), e),
                        }
                        let pub_key_path = expanded_path.with_extension("pub");
                        if pub_key_path.exists() {
                            match fs::remove_file(&pub_key_path) {
                                Ok(_) => println!("🔑 SSH public key file '{}' deleted.", pub_key_path.display()),
                                Err(e) => eprintln!("⚠️ Warning: Failed to delete SSH public key file {}: {}", pub_key_path.display(), e),
                            }
                        }
                    } else {
                        println!("ℹ️ SSH key file for '{}' not found at '{}', skipping deletion.", name, expanded_path.display());
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Warning: Could not expand SSH key path for deletion: {}. Manual deletion might be required.", e);
                }
            }
        }

        // 3. Remove from SSH config
        match ssh::remove_ssh_config_entry(name) {
            Ok(_) => {} // Already prints messages
            Err(e) => eprintln!("⚠️ Warning: Failed to update SSH config while removing account '{}': {}", name, e),
        }
        Ok(())
    } else {
        // This case should ideally be caught by the initial check
        Err(GitSwitchError::AccountNotFound { name: name.to_string() })
    }
}
