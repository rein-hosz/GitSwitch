use crate::config::Config;
use crate::error::Result;
use crate::git;
use colored::*;

/// Auto-detect account based on remote URL
pub fn detect_account_from_remote(config: &Config) -> Result<Option<String>> {
    if !git::is_in_git_repository()? {
        return Ok(None);
    }

    let remote_url = git::get_remote_url("origin").ok();
    if let Some(url) = remote_url {
        // Try to match accounts based on SSH key or provider
        for (name, account) in &config.accounts {
            if let Some(provider) = &account.provider {
                if url_matches_provider(&url, provider) {
                    return Ok(Some(name.clone()));
                }
            }
        }
    }

    Ok(None)
}

/// Check if URL matches a provider
fn url_matches_provider(url: &str, provider: &str) -> bool {
    match provider {
        "github" => url.contains("github.com"),
        "gitlab" => url.contains("gitlab.com"),
        "bitbucket" => url.contains("bitbucket.org"),
        _ => false,
    }
}

/// Suggest account based on current repository
pub fn suggest_account(config: &Config) -> Result<()> {
    if let Some(account_name) = detect_account_from_remote(config)? {
        println!("{} Detected account '{}' for this repository", 
            "💡".to_string(), account_name.cyan());
        println!("Use {} to apply this account", 
            format!("git-switch account {}", account_name).cyan());
    } else {
        println!("{} No account detected for this repository", "ℹ".blue());
        if !config.accounts.is_empty() {
            let account_names: Vec<String> = config.accounts.keys().cloned().collect();
            println!("Available accounts: {}", account_names.join(", "));
        }
    }
    Ok(())
}

/// Check for account mismatches
pub fn check_account_mismatch(config: &Config) -> Result<()> {
    if !git::is_in_git_repository()? {
        return Ok(());
    }

    let suggested = detect_account_from_remote(config)?;
    
    if let Ok((_, local_email)) = git::get_local_config() {
        let current_account = config.accounts.values()
            .find(|acc| acc.email == local_email)
            .map(|acc| acc.name.clone());

        if let (Some(suggested_name), Some(current_name)) = (suggested, current_account) {
            if suggested_name != current_name {
                println!("{} Account mismatch detected!", "⚠".yellow().bold());
                println!("  Current: {}", current_name.red());
                println!("  Suggested: {}", suggested_name.green());
                println!("  Use {} to switch", 
                    format!("git-switch account {}", suggested_name).cyan());
            }
        }
    }

    Ok(())
}

// Repository discovery and bulk operations are now handled by the repository.rs module

/// Detect account for a specific repository based on remote URL
pub fn detect_account_for_remote_url(config: &Config, remote_url: &str) -> Result<Option<String>> {
    // Parse the remote URL to extract the provider and repository info
    let remote_url = remote_url.to_lowercase();
    
    // GitHub patterns
    if remote_url.contains("github.com") {
        for (account_name, account) in &config.accounts {
            if let Some(ref provider) = account.provider {
                if provider.to_lowercase() == "github" {
                    return Ok(Some(account_name.clone()));
                }
            }
            // Also check if the username in the URL matches
            if let Some(github_user) = extract_github_username(&remote_url) {
                if account.username == github_user {
                    return Ok(Some(account_name.clone()));
                }
            }
        }
    }
    
    // GitLab patterns
    if remote_url.contains("gitlab.com") {
        for (account_name, account) in &config.accounts {
            if let Some(ref provider) = account.provider {
                if provider.to_lowercase() == "gitlab" {
                    return Ok(Some(account_name.clone()));
                }
            }
            if let Some(gitlab_user) = extract_gitlab_username(&remote_url) {
                if account.username == gitlab_user {
                    return Ok(Some(account_name.clone()));
                }
            }
        }
    }
    
    // Bitbucket patterns
    if remote_url.contains("bitbucket.org") {
        for (account_name, account) in &config.accounts {
            if let Some(ref provider) = account.provider {
                if provider.to_lowercase() == "bitbucket" {
                    return Ok(Some(account_name.clone()));
                }
            }
            if let Some(bitbucket_user) = extract_bitbucket_username(&remote_url) {
                if account.username == bitbucket_user {
                    return Ok(Some(account_name.clone()));
                }
            }
        }
    }
    
    Ok(None)
}

fn extract_github_username(url: &str) -> Option<String> {
    // Extract username from GitHub URLs like:
    // https://github.com/username/repo.git
    // git@github.com:username/repo.git
    if let Some(start) = url.find("github.com") {
        let after_github = &url[start + "github.com".len()..];
        if let Some(colon_pos) = after_github.find(':') {
            // SSH format: git@github.com:username/repo.git
            let path_part = &after_github[colon_pos + 1..];
            if let Some(slash_pos) = path_part.find('/') {
                return Some(path_part[..slash_pos].to_string());
            }
        } else if let Some(slash_pos) = after_github.find('/') {
            // HTTPS format: https://github.com/username/repo.git
            let path_part = &after_github[slash_pos + 1..];
            if let Some(next_slash) = path_part.find('/') {
                return Some(path_part[..next_slash].to_string());
            }
        }
    }
    None
}

fn extract_gitlab_username(url: &str) -> Option<String> {
    // Similar logic for GitLab
    if let Some(start) = url.find("gitlab.com") {
        let after_gitlab = &url[start + "gitlab.com".len()..];
        if let Some(colon_pos) = after_gitlab.find(':') {
            let path_part = &after_gitlab[colon_pos + 1..];
            if let Some(slash_pos) = path_part.find('/') {
                return Some(path_part[..slash_pos].to_string());
            }
        } else if let Some(slash_pos) = after_gitlab.find('/') {
            let path_part = &after_gitlab[slash_pos + 1..];
            if let Some(next_slash) = path_part.find('/') {
                return Some(path_part[..next_slash].to_string());
            }
        }
    }
    None
}

fn extract_bitbucket_username(url: &str) -> Option<String> {
    // Similar logic for Bitbucket
    if let Some(start) = url.find("bitbucket.org") {
        let after_bitbucket = &url[start + "bitbucket.org".len()..];
        if let Some(colon_pos) = after_bitbucket.find(':') {
            let path_part = &after_bitbucket[colon_pos + 1..];
            if let Some(slash_pos) = path_part.find('/') {
                return Some(path_part[..slash_pos].to_string());
            }
        } else if let Some(slash_pos) = after_bitbucket.find('/') {
            let path_part = &after_bitbucket[slash_pos + 1..];
            if let Some(next_slash) = path_part.find('/') {
                return Some(path_part[..next_slash].to_string());
            }
        }
    }
    None
}
