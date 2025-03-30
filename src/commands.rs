use crate::config::{load_accounts, save_account, Account};
use crate::git::update_git_remote;
use crate::ssh::{update_ssh_config, generate_ssh_key, add_ssh_key, display_public_key};
use crate::utils::run_command;
use std::io::{self, Write};

pub fn add_account(name: &str, username: &str, email: &str) {
    // Generate SSH key path based on account name
    let ssh_key_path = format!("~/.ssh/id_rsa_{}", name.replace(" ", "_").to_lowercase());

    // Create parent directory if it doesn't exist
    let expanded_key_path = shellexpand::tilde(&ssh_key_path).to_string();
    if let Some(parent) = std::path::Path::new(&expanded_key_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("Failed to create SSH directory");
        }
    }

    // Generate SSH key automatically
    generate_ssh_key(&ssh_key_path);

    // Create and save account
    let account = Account {
        name: name.to_string(),
        username: username.to_string(),
        email: email.to_string(),
        ssh_key: ssh_key_path.clone(),
    };

    save_account(&account);

    if let Err(e) = update_ssh_config(name, &ssh_key_path) {
        eprintln!("❌ Failed to update SSH config: {}", e);
    }

    // Display the public key for the user to copy
    println!("✅ Account '{}' added successfully!", name);
    println!("\n🔑 Here is your public SSH key to add to GitHub:");
    println!("--------------------------------------------------");
    display_public_key(&ssh_key_path);
    println!("--------------------------------------------------");
    println!("Copy this key and add it to your GitHub account at: https://github.com/settings/keys");
}

pub fn use_account(name_or_username: &str) {
    let accounts = load_accounts();

    // Try to find account by name first, then by username
    let account = accounts.iter()
        .find(|acc| acc.name == name_or_username || acc.username == name_or_username)
        .cloned();

    match account {
        Some(acc) => {
            // Set Git global config
            run_command("git", &["config", "--global", "user.name", &acc.username]);
            run_command("git", &["config", "--global", "user.email", &acc.email]);

            // Start ssh-agent if not already running
            println!("🔄 Ensuring SSH agent is running...");
            run_command("ssh-agent", &["-s"]);

            // Add SSH key to agent
            if add_ssh_key(&acc.ssh_key) {
                println!("✅ Switched to Git account: {} ({})", acc.name, acc.username);

                // Ask if user wants to update current repo's remote URL
                print!("Do you want to update remote URL for the current repository? (y/n): ");
                io::stdout().flush().unwrap();
                let mut response = String::new();
                io::stdin().read_line(&mut response).unwrap();

                if response.trim().to_lowercase() == "y" {
                    print!("Enter repository name (e.g., 'username/repo'): ");
                    io::stdout().flush().unwrap();
                    let mut repo = String::new();
                    io::stdin().read_line(&mut repo).unwrap();

                    update_git_remote(&acc.username, &repo.trim());
                }
            } else {
                eprintln!("❌ Failed to add SSH key to agent.");
            }
        },
        None => {
            println!("❌ Account with name or username '{}' not found.", name_or_username);

            // List available accounts to help the user
            if !accounts.is_empty() {
                println!("\nAvailable accounts:");
                println!("----------------------------------------");
                println!("Account Name | Git Username | Email");
                println!("----------------------------------------");
                for acc in &accounts {
                    println!("{} | {} | {}", acc.name, acc.username, acc.email);
                }
                println!("----------------------------------------");
            }
        }
    }
}

pub fn list_accounts() {
    crate::config::list_accounts();
}
