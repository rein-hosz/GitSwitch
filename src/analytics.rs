use crate::config::Config;
use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use colored::*;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UsageStats {
    pub account_usage: HashMap<String, u32>,
    pub last_used: HashMap<String, String>, // ISO date string
    pub repository_count: HashMap<String, u32>,
}

/// Get analytics file path
fn get_analytics_file_path() -> Result<PathBuf> {
    let home_dir = home::home_dir()
        .ok_or_else(|| crate::error::GitSwitchError::HomeDirectoryNotFound)?;
    Ok(home_dir.join(".git-switch-analytics.toml"))
}

/// Load usage statistics
pub fn load_stats() -> Result<UsageStats> {
    let path = get_analytics_file_path()?;
    if !path.exists() {
        return Ok(UsageStats::default());
    }
    
    let content = fs::read_to_string(&path)?;
    let stats = toml::from_str(&content)
        .map_err(crate::error::GitSwitchError::Toml)?;
    Ok(stats)
}

/// Save usage statistics
pub fn save_stats(stats: &UsageStats) -> Result<()> {
    let path = get_analytics_file_path()?;
    let content = toml::to_string_pretty(stats)
        .map_err(crate::error::GitSwitchError::TomlSer)?;
    fs::write(&path, content)?;
    Ok(())
}

/// Record account usage
pub fn record_usage(account_name: &str) -> Result<()> {
    let mut stats = load_stats()?;
    
    // Increment usage count
    *stats.account_usage.entry(account_name.to_string()).or_insert(0) += 1;
    
    // Update last used timestamp
    let now = chrono::Utc::now().to_rfc3339();
    stats.last_used.insert(account_name.to_string(), now);
    
    save_stats(&stats)?;
    Ok(())
}

/// Record repository usage for an account
pub fn record_repository_usage(account_name: &str) -> Result<()> {
    let mut stats = load_stats()?;
    
    *stats.repository_count.entry(account_name.to_string()).or_insert(0) += 1;
    
    save_stats(&stats)?;
    Ok(())
}

/// Display usage analytics
pub fn show_analytics(config: &Config) -> Result<()> {
    let stats = load_stats()?;
    
    println!("{}", "Account Usage Analytics".bold().cyan());
    println!("{}", "─".repeat(35));
    
    if stats.account_usage.is_empty() {
        println!("{} No usage data available yet", "ℹ".blue());
        return Ok(());
    }
    
    // Sort accounts by usage count
    let mut usage_vec: Vec<(&String, &u32)> = stats.account_usage.iter().collect();
    usage_vec.sort_by(|a, b| b.1.cmp(a.1));
    
    println!("\n{}", "Most Used Accounts:".bold());
    for (account_name, count) in usage_vec.iter().take(5) {
        let account_exists = config.accounts.contains_key(*account_name);
        let status = if account_exists { "✓".green() } else { "✗".red() };
        
        let last_used = stats.last_used.get(*account_name)
            .map(|date| {
                // Parse and format the date
                chrono::DateTime::parse_from_rfc3339(date)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|_| "Unknown".to_string())
            })
            .unwrap_or_else(|| "Never".to_string());
        
        println!("  {} {} - {} uses (last: {})", 
            status, account_name.cyan(), count, last_used.dimmed());
    }
    
    // Show repository counts
    if !stats.repository_count.is_empty() {
        println!("\n{}", "Repository Usage:".bold());
        let mut repo_vec: Vec<(&String, &u32)> = stats.repository_count.iter().collect();
        repo_vec.sort_by(|a, b| b.1.cmp(a.1));
        
        for (account_name, count) in repo_vec.iter().take(5) {
            println!("  {} - {} repositories", account_name.cyan(), count);
        }
    }
    
    Ok(())
}

/// Clear analytics data
pub fn clear_analytics() -> Result<()> {
    let path = get_analytics_file_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
        println!("{} Analytics data cleared", "✓".green());
    } else {
        println!("{} No analytics data to clear", "ℹ".blue());
    }
    Ok(())
}
