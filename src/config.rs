use crate::error::{GitSwitchError, Result};
use crate::utils::{ensure_parent_dir_exists, read_file_content, write_file_content};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = ".git-switch-config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub name: String,
    pub username: String,
    pub email: String,
    pub ssh_key_path: String, // Changed from ssh_key to ssh_key_path
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub accounts: HashMap<String, Account>,
}

fn get_config_file_path() -> Result<PathBuf> {
    if let Some(home_dir) = home::home_dir() {
        Ok(home_dir.join(CONFIG_FILE_NAME))
    } else {
        Err(GitSwitchError::HomeDirectoryNotFound)
    }
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_file_path()?;
    if !config_path.exists() {
        return Ok(Config::default()); // Return a default empty config if file doesn't exist
    }
    let content = read_file_content(&config_path)?;
    serde_json::from_str(&content).map_err(GitSwitchError::Json)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_file_path()?;
    ensure_parent_dir_exists(&config_path)?;
    let content = serde_json::to_string_pretty(config).map_err(GitSwitchError::Json)?;
    write_file_content(&config_path, &content)
}

pub fn save_account(account: &Account) -> Result<()> {
    let mut config = load_config().unwrap_or_else(|_| Config::default()); // Load or create new
    if config.accounts.contains_key(&account.name) {
        return Err(GitSwitchError::AccountExists {
            name: account.name.clone(),
        });
    }
    config
        .accounts
        .insert(account.name.clone(), account.clone());
    save_config(&config)
}

pub fn find_account(name_or_username: &str) -> Result<Account> {
    let config = load_config()?;
    if let Some(account) = config.accounts.get(name_or_username) {
        return Ok(account.clone());
    }
    config
        .accounts
        .values()
        .find(|acc| acc.username == name_or_username)
        .cloned()
        .ok_or_else(|| GitSwitchError::AccountNotFound {
            name: name_or_username.to_string(),
        })
}

pub fn list_accounts() -> Result<()> {
    let config = load_config()?;
    if config.accounts.is_empty() {
        println!("No saved accounts.");
        return Ok(());
    }

    println!("🔹 Saved Git Accounts:");
    println!("---------------------------------------------------------------------");
    println!("{:<20} | {:<25} | {:<30}", "Account Name", "Git Username", "Email");
    println!("---------------------------------------------------------------------");
    for acc in config.accounts.values() {
        println!("{:<20} | {:<25} | {:<30}", acc.name, acc.username, acc.email);
    }
    println!("---------------------------------------------------------------------");
    Ok(())
}
