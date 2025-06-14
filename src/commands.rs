use crate::analytics;
use crate::config::{self, Account, Config};
use crate::error::{GitSwitchError, Result};
use crate::git;
use crate::ssh;
use crate::utils;
use crate::validation;
use colored::*;
use dialoguer::{Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Detect provider from email domain
fn detect_provider_from_email(email: &str) -> Option<String> {
    if email.contains("@github.com") || email.contains("@users.noreply.github.com") {
        Some("github".to_string())
    } else if email.contains("@gitlab.com") {
        Some("gitlab".to_string())
    } else if email.contains("@bitbucket.org") {
        Some("bitbucket".to_string())
    } else {
        None
    }
}

/// Add account with enhanced validation and progress indicators
pub fn add_account(
    config: &mut Config,
    name: &str,
    username: &str,
    email: &str,
    ssh_key_path_opt: Option<PathBuf>,
    provider: Option<String>,
) -> Result<()> {
    // Validate inputs
    validation::validate_account_name(name)?;
    validation::validate_username(username)?;
    validation::validate_email(email)?;

    if config.accounts.contains_key(name) {
        return Err(GitSwitchError::AccountExists {
            name: name.to_string(),
        });
    }

    let ssh_key_path_str = if let Some(custom_path) = ssh_key_path_opt.as_ref() {
        custom_path
            .to_str()
            .ok_or_else(|| GitSwitchError::InvalidPath(custom_path.clone()))?
            .to_string()
    } else {
        format!("~/.ssh/id_rsa_{}", name.replace(" ", "_").to_lowercase())
    };

    let expanded_key_path = utils::expand_path(&ssh_key_path_str)?;
    utils::ensure_parent_dir_exists(&expanded_key_path)?;

    // Clean progress indicator for key generation
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    if ssh_key_path_opt.is_none() && !expanded_key_path.exists() {
        pb.set_message("🔐 Generating SSH key pair...");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        ssh::generate_ssh_key(&expanded_key_path)?;
        pb.finish_and_clear();
    } else if ssh_key_path_opt.is_some() && !expanded_key_path.exists() {
        return Err(GitSwitchError::SshKeyGeneration {
            message: format!(
                "Specified SSH key path does not exist: {}",
                expanded_key_path.display()
            ),
        });
    } else if expanded_key_path.exists() {
        // Validate existing SSH key
        validation::validate_ssh_key(&expanded_key_path)?;
    }

    let account = Account {
        name: name.to_string(),
        username: username.to_string(),
        email: email.to_string(),
        ssh_key_path: ssh_key_path_str.clone(),
        additional_ssh_keys: Vec::new(),
        provider: provider.or_else(|| detect_provider_from_email(email)),
        groups: Vec::new(),
    };

    config.accounts.insert(name.to_string(), account);
    config::save_config(config)?;

    // Update SSH config silently
    ssh::update_ssh_config(name, &ssh_key_path_str)?;

    // Beautiful success message
    println!("\n{}", "🎉 Account Created Successfully!".bold().green());
    println!("{}", "─".repeat(40).bright_black());

    println!("📧 {} {}", "Account:".bold(), name.cyan().bold());
    println!("👤 {} {}", "Username:".bold(), username.bright_white());
    println!("✉️  {} {}", "Email:".bold(), email.bright_white());

    if let Some(provider) = &config.accounts[name].provider {
        let provider_emoji = match provider.as_str() {
            "github" => "🐙",
            "gitlab" => "🦊",
            "bitbucket" => "🪣",
            _ => "🔗",
        };
        println!(
            "{} {} {}",
            provider_emoji,
            "Provider:".bold(),
            provider.bright_cyan()
        );
    }

    if ssh_key_path_opt.is_none() {
        println!("🔑 {} Generated and configured", "SSH Key:".bold());

        // Display formatted public key
        println!("\n{}", "📋 Your Public Key".bold().yellow());
        println!("{}", "─".repeat(40).bright_black());
        if let Ok(()) = ssh::display_public_key_formatted(&expanded_key_path) {
            // Provider-specific instructions
            if let Some(provider) = &config.accounts[name].provider {
                match provider.as_str() {
                    "github" => {
                        println!(
                            "\n{} {} Copy the key above and add it to GitHub:",
                            "🚀".bold(),
                            "Next Steps:".bold().bright_yellow()
                        );
                        println!(
                            "   {}",
                            "https://github.com/settings/keys".bright_blue().underline()
                        );
                    }
                    "gitlab" => {
                        println!(
                            "\n{} {} Copy the key above and add it to GitLab:",
                            "🚀".bold(),
                            "Next Steps:".bold().bright_yellow()
                        );
                        println!(
                            "   {}",
                            "https://gitlab.com/-/profile/keys"
                                .bright_blue()
                                .underline()
                        );
                    }
                    "bitbucket" => {
                        println!(
                            "\n{} {} Copy the key above and add it to Bitbucket:",
                            "🚀".bold(),
                            "Next Steps:".bold().bright_yellow()
                        );
                        println!(
                            "   {}",
                            "https://bitbucket.org/account/settings/ssh-keys/"
                                .bright_blue()
                                .underline()
                        );
                    }
                    _ => {
                        println!(
                            "\n{} {} Copy the key above and add it to your Git provider",
                            "🚀".bold(),
                            "Next Steps:".bold().bright_yellow()
                        );
                    }
                }
            }
        }
    } else {
        println!("🔑 {} Using existing key", "SSH Key:".bold());
    }

    println!(
        "\n{} {} to start using this account",
        "💡".bold(),
        format!("Run 'git-switch use {}'", name)
            .bright_green()
            .bold()
    );

    Ok(())
}

/// Interactive account creation
pub fn add_account_interactive(config: &mut Config, suggested_name: &str) -> Result<()> {
    println!("{}", "Interactive Account Setup".bold().cyan());
    println!("Let's create a new Git account configuration.\n");

    let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Account name")
        .default(suggested_name.to_string())
        .interact_text()?;

    validation::validate_account_name(&name)?;

    if config.accounts.contains_key(&name) {
        return Err(GitSwitchError::AccountExists { name });
    }

    let username: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Username")
        .interact_text()?;

    let email: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Email address")
        .validate_with(|input: &String| -> Result<(), &str> {
            if validation::validate_email(input).is_ok() {
                Ok(())
            } else {
                Err("Please enter a valid email address")
            }
        })
        .interact_text()?;

    let providers = vec!["github", "gitlab", "bitbucket", "other"];
    let provider_selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select Git provider")
        .default(0)
        .items(&providers)
        .interact()?;

    let provider = if provider_selection == 3 {
        None
    } else {
        Some(providers[provider_selection].to_string())
    };

    let generate_key = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Generate new SSH key?")
        .default(true)
        .interact()?;

    let ssh_key_path = if !generate_key {
        let path: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("SSH key path")
            .interact_text()?;
        Some(PathBuf::from(path))
    } else {
        None
    };

    add_account(config, &name, &username, &email, ssh_key_path, provider)
}

/// List accounts with optional detailed view
pub fn list_accounts(config: &Config, detailed: bool) -> Result<()> {
    if config.accounts.is_empty() {
        println!(
            "\n{} {}",
            "📭".yellow(),
            "No Git accounts configured yet".bold()
        );
        println!("{}", "──────────────────────────────────".bright_black());
        println!("Get started by adding your first account:");
        println!(
            "{} {}",
            "💡".bold(),
            "git-switch add <name> <username> <email>".bright_cyan()
        );
        println!(
            "{} {}",
            "📖".bold(),
            "git-switch add --help".bright_white().dimmed()
        );
        return Ok(());
    }

    let account_count = config.accounts.len();
    let plural = if account_count == 1 {
        "Account"
    } else {
        "Accounts"
    };

    println!(
        "\n{} {} {} {}",
        "📚".bold(),
        account_count.to_string().bright_yellow().bold(),
        plural.bold(),
        "Configured".bold()
    );
    println!("{}", "═".repeat(50).bright_black());

    if detailed {
        for (i, (name, account)) in config.accounts.iter().enumerate() {
            if i > 0 {
                println!(); // Add spacing between accounts
            }

            // Get provider emoji and info
            let (provider_emoji, provider_name) = match account.provider.as_deref() {
                Some("github") => ("🐙", "GitHub"),
                Some("gitlab") => ("🦊", "GitLab"),
                Some("bitbucket") => ("🪣", "Bitbucket"),
                Some(other) => ("�", other),
                None => ("❓", "Unknown"),
            };

            // Check if SSH key exists
            let ssh_key_status =
                if let Ok(expanded_path) = utils::expand_path(&account.ssh_key_path) {
                    if expanded_path.exists() {
                        ("✅", "Found".green())
                    } else {
                        ("❌", "Missing".red())
                    }
                } else {
                    ("⚠️", "Invalid Path".yellow())
                };

            println!(
                "╭─ {} {} {}",
                "📋".bold(),
                name.bright_cyan().bold(),
                format!("({})", provider_name).bright_black()
            );
            println!("│");
            println!(
                "├─ {} {} {}",
                "👤".bold(),
                "Username:".bold(),
                account.username.bright_white()
            );
            println!(
                "├─ {} {} {}",
                "✉️".bold(),
                "Email:".bold(),
                account.email.bright_white()
            );
            println!(
                "├─ {} {} {}",
                provider_emoji.bold(),
                "Provider:".bold(),
                provider_name.bright_cyan()
            );
            println!(
                "├─ {} {} {} {}",
                "🔑".bold(),
                "SSH Key:".bold(),
                ssh_key_status.1,
                ssh_key_status.0
            );
            println!("│   {}", account.ssh_key_path.bright_black());

            if !account.groups.is_empty() {
                println!(
                    "├─ {} {} {}",
                    "👥".bold(),
                    "Groups:".bold(),
                    account.groups.join(", ").bright_white()
                );
            }
            if !account.additional_ssh_keys.is_empty() {
                println!(
                    "├─ {} {} {}",
                    "🔐".bold(),
                    "Additional Keys:".bold(),
                    account.additional_ssh_keys.len().to_string().bright_white()
                );
            }
            println!(
                "╰─ {} {}",
                "🚀".bold(),
                format!("git-switch use '{}'", name).bright_green()
            );
        }
    } else {
        // Compact view with better formatting
        for (name, account) in &config.accounts {
            let (provider_emoji, provider_name) = match account.provider.as_deref() {
                Some("github") => ("🐙", "GitHub"),
                Some("gitlab") => ("🦊", "GitLab"),
                Some("bitbucket") => ("🪣", "Bitbucket"),
                Some(other) => ("🔗", other),
                None => ("❓", "Unknown"),
            };

            // Check SSH key status
            let key_status = if let Ok(expanded_path) = utils::expand_path(&account.ssh_key_path) {
                if expanded_path.exists() { "✅" } else { "❌" }
            } else {
                "⚠️"
            };

            println!(
                "  {} {} {} {} {} {} {}",
                provider_emoji,
                name.bright_cyan().bold(),
                "•".bright_black(),
                account.username.bright_white(),
                "•".bright_black(),
                provider_name.dimmed(),
                key_status
            );
        }
    }

    println!("\n{}", "─".repeat(50).bright_black());
    println!(
        "{} {} {}",
        "💡".bold(),
        "Quick commands:".bold().bright_yellow(),
        "git-switch use <name> | git-switch add <name>"
            .bright_white()
            .dimmed()
    );
    Ok(())
}

/// Find account by name or username/email
fn find_account<'a>(config: &'a Config, name_or_username: &str) -> Option<&'a Account> {
    config.accounts.get(name_or_username).or_else(|| {
        config
            .accounts
            .values()
            .find(|acc| acc.username == name_or_username || acc.email == name_or_username)
    })
}

/// Use account globally with enhanced feedback
pub fn use_account_globally(config: &Config, name: &str) -> Result<()> {
    let account = find_account(config, name).ok_or_else(|| GitSwitchError::AccountNotFound {
        name: name.to_string(),
    })?;

    println!(
        "{} Switching to account '{}'",
        "🔄".to_string(),
        account.name.cyan()
    );

    git::set_global_config(&account.username, &account.email)?;

    let expanded_key_path = utils::expand_path(&account.ssh_key_path)?;
    if expanded_key_path.exists() {
        ssh::add_ssh_key(&account.ssh_key_path)?;
        println!("{} SSH key loaded", "🔑".to_string());
    }

    // Record usage analytics
    if let Err(e) = analytics::record_usage(&account.name) {
        tracing::warn!("Failed to record usage analytics: {}", e);
    }

    println!("{} Global Git config updated", "✓".green().bold());
    Ok(())
}

/// Remove account with confirmation
pub fn remove_account(config: &mut Config, name: &str, no_prompt: bool) -> Result<()> {
    if !config.accounts.contains_key(name) {
        return Err(GitSwitchError::AccountNotFound {
            name: name.to_string(),
        });
    }

    if !no_prompt {
        let confirm = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(&format!("Remove account '{}'?", name.red()))
            .default(false)
            .interact()?;

        if !confirm {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    let account = config.accounts.remove(name).unwrap();

    // Remove SSH config entry
    ssh::remove_ssh_config_entry(name)?;

    config::save_config(config)?;

    println!(
        "{} Account '{}' removed successfully",
        "✓".green().bold(),
        name
    );

    // Ask if user wants to remove SSH key file
    if !no_prompt {
        let remove_key = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Remove SSH key file as well?")
            .default(false)
            .interact()?;

        if remove_key {
            let expanded_key_path = utils::expand_path(&account.ssh_key_path)?;
            if expanded_key_path.exists() {
                fs::remove_file(&expanded_key_path)?;
                println!("{} SSH key file removed", "🗑️".to_string());
            }
        }
    }

    Ok(())
}

/// Handle account subcommand (apply to current repo)
pub fn handle_account_subcommand(config: &Config, name: &str) -> Result<()> {
    let account = find_account(config, name).ok_or_else(|| GitSwitchError::AccountNotFound {
        name: name.to_string(),
    })?;

    // Check if we're in a git repository
    if !git::is_in_git_repository()? {
        return Err(GitSwitchError::NotInGitRepository);
    }

    println!(
        "{} Applying account '{}' to current repository",
        "🔧".to_string(),
        account.name.cyan()
    );

    git::set_local_config(&account.username, &account.email)?;

    let expanded_key_path = utils::expand_path(&account.ssh_key_path)?;
    if expanded_key_path.exists() {
        git::set_ssh_command(&account.ssh_key_path)?;
        println!(
            "{} SSH configuration updated for this repository",
            "🔑".to_string()
        );
    }

    // Record repository usage analytics
    if let Err(e) = analytics::record_repository_usage(&account.name) {
        tracing::warn!("Failed to record repository usage analytics: {}", e);
    }

    println!(
        "{} Repository configured for account '{}'",
        "✓".green().bold(),
        account.name.cyan()
    );
    Ok(())
}

/// Handle remote subcommand (convert between HTTPS and SSH)
pub fn handle_remote_subcommand(https: bool, ssh: bool) -> Result<()> {
    if !git::is_in_git_repository()? {
        return Err(GitSwitchError::NotInGitRepository);
    }

    let current_url = git::get_remote_url("origin")?;
    println!("Current remote URL: {}", current_url.cyan());

    let new_url = if https {
        convert_to_https(&current_url)?
    } else if ssh {
        convert_to_ssh(&current_url)?
    } else {
        return Err(GitSwitchError::Other(
            "Specify either --https or --ssh".to_string(),
        ));
    };

    git::set_remote_url("origin", &new_url)?;
    println!(
        "{} Remote URL updated to: {}",
        "✓".green().bold(),
        new_url.cyan()
    );
    Ok(())
}

/// Convert remote URL to HTTPS format
fn convert_to_https(url: &str) -> Result<String> {
    if url.starts_with("https://") {
        return Ok(url.to_string());
    }

    if url.starts_with("git@") {
        let parts: Vec<&str> = url.splitn(2, ':').collect();
        if parts.len() == 2 {
            let host = parts[0].trim_start_matches("git@");
            let path = parts[1].trim_end_matches(".git");
            return Ok(format!("https://{}/{}.git", host, path));
        }
    }

    Err(GitSwitchError::Other(format!(
        "Cannot convert URL to HTTPS: {}",
        url
    )))
}

/// Convert remote URL to SSH format
fn convert_to_ssh(url: &str) -> Result<String> {
    if url.starts_with("git@") {
        return Ok(url.to_string());
    }

    if url.starts_with("https://") {
        let url_without_protocol = url.trim_start_matches("https://");
        let parts: Vec<&str> = url_without_protocol.splitn(2, '/').collect();
        if parts.len() == 2 {
            let host = parts[0];
            let path = parts[1].trim_end_matches(".git");
            return Ok(format!("git@{}:{}.git", host, path));
        }
    }

    Err(GitSwitchError::Other(format!(
        "Cannot convert URL to SSH: {}",
        url
    )))
}

/// Handle whoami subcommand
pub fn handle_whoami_subcommand(config: &Config) -> Result<()> {
    println!("{}", "Current Git Identity".bold().cyan());
    println!("{}", "─".repeat(25));

    // Show global config
    if let Ok((global_name, global_email)) = git::get_global_config() {
        println!("\n{} Global Configuration:", "🌍".to_string());
        println!("  Name: {}", global_name);
        println!("  Email: {}", global_email);

        // Try to find matching account
        if let Some(account) = config
            .accounts
            .values()
            .find(|acc| acc.email == global_email)
        {
            println!(
                "  Account: {} {}",
                account.name.green(),
                "(matched)".dimmed()
            );
        } else {
            println!(
                "  Account: {} {}",
                "None".yellow(),
                "(no match found)".dimmed()
            );
        }
    }

    // Show local config if in a repository
    if git::is_in_git_repository()? {
        if let Ok((local_name, local_email)) = git::get_local_config() {
            println!("\n{} Repository Configuration:", "📁".to_string());
            println!("  Name: {}", local_name);
            println!("  Email: {}", local_email);

            if let Some(account) = config
                .accounts
                .values()
                .find(|acc| acc.email == local_email)
            {
                println!(
                    "  Account: {} {}",
                    account.name.green(),
                    "(matched)".dimmed()
                );
            } else {
                println!(
                    "  Account: {} {}",
                    "None".yellow(),
                    "(no match found)".dimmed()
                );
            }
        }

        // Show remote URL
        if let Ok(remote_url) = git::get_remote_url("origin") {
            println!("\n{} Remote URL:", "🔗".to_string());
            println!("  {}", remote_url);
        }
    } else {
        println!("\n{} Not in a Git repository", "ℹ".blue());
    }

    Ok(())
}

/// Handle auth test subcommand
pub fn handle_auth_test_subcommand(config: &Config) -> Result<()> {
    println!("{}", "Testing SSH Authentication".bold().cyan());
    println!("{}", "─".repeat(30));

    for (name, account) in &config.accounts {
        print!("Testing account '{}' ... ", name.cyan());
        io::stdout().flush()?;

        let expanded_key_path = utils::expand_path(&account.ssh_key_path)?;
        if !expanded_key_path.exists() {
            println!("{} (key not found)", "✗".red());
            continue;
        }

        // Test SSH connection based on provider
        let test_result = match account.provider.as_deref() {
            Some("github") => test_ssh_connection("git@github.com"),
            Some("gitlab") => test_ssh_connection("git@gitlab.com"),
            Some("bitbucket") => test_ssh_connection("git@bitbucket.org"),
            _ => test_ssh_connection("git@github.com"), // Default to GitHub
        };

        match test_result {
            Ok(_) => println!("{}", "✓".green()),
            Err(_) => println!("{}", "✗".red()),
        }
    }

    Ok(())
}

fn test_ssh_connection(host: &str) -> Result<()> {
    let output = std::process::Command::new("ssh")
        .args(&[
            "-T",
            "-o",
            "ConnectTimeout=5",
            "-o",
            "StrictHostKeyChecking=no",
            host,
        ])
        .output()?;

    // For Git hosting services, successful authentication often returns with exit code 1
    // but includes specific messages in stderr
    if output.status.success()
        || String::from_utf8_lossy(&output.stderr).contains("successfully authenticated")
    {
        Ok(())
    } else {
        Err(GitSwitchError::SshCommand {
            command: format!("ssh -T {}", host),
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

// Profile management functions

// Profile functionality is now handled by the profiles.rs module
// These functions have been moved to ProfileManager implementation
