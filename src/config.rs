use crate::error::{GitSwitchError, Result};
use crate::utils::{ensure_parent_dir_exists, read_file_content, write_file_content};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const CONFIG_FILE_NAME_TOML: &str = ".git-switch-config.toml";
const CONFIG_FILE_NAME_JSON: &str = ".git-switch-config.json"; // Legacy support

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub name: String,
    pub username: String,
    pub email: String,
    pub ssh_key_path: String,
    /// Optional SSH key paths for multiple keys per account
    #[serde(default)]
    pub additional_ssh_keys: Vec<String>,
    /// Account templates/presets
    #[serde(default)]
    pub provider: Option<String>, // github, gitlab, bitbucket, etc.
    /// Account groups/organizations
    #[serde(default)]
    pub groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub accounts: HashMap<String, Account>,
    /// Configuration version for migration purposes
    #[serde(default = "default_config_version")]
    pub version: String,
    /// Global settings
    #[serde(default)]
    pub settings: GlobalSettings,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GlobalSettings {
    /// Default provider for new accounts
    pub default_provider: Option<String>,
    /// Auto-detect account based on remote URL
    #[serde(default)]
    pub auto_detect_account: bool,
    /// Use colored output
    #[serde(default = "default_true")]
    pub colored_output: bool,
    /// Show progress indicators
    #[serde(default = "default_true")]
    pub show_progress: bool,
}

fn default_config_version() -> String {
    "2.0".to_string()
}

fn default_true() -> bool {
    true
}

pub fn get_config_file_path() -> Result<PathBuf> {
    if let Some(home_dir) = home::home_dir() {
        // Prefer TOML format
        let toml_path = home_dir.join(CONFIG_FILE_NAME_TOML);
        if toml_path.exists() {
            return Ok(toml_path);
        }

        // Check for legacy JSON format
        let json_path = home_dir.join(CONFIG_FILE_NAME_JSON);
        if json_path.exists() {
            return Ok(json_path);
        }

        // Default to TOML for new installations
        Ok(toml_path)
    } else {
        Err(GitSwitchError::HomeDirectoryNotFound)
    }
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_file_path()?;
    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = read_file_content(&config_path)?;

    // Try TOML first, then JSON for backwards compatibility
    let mut config = if config_path.extension().and_then(|s| s.to_str()) == Some("toml") {
        toml::from_str(&content).map_err(GitSwitchError::Toml)?
    } else {
        // JSON format (legacy)
        let json_config: Config = serde_json::from_str(&content).map_err(GitSwitchError::Json)?;

        // Migrate to TOML format
        migrate_to_toml(&json_config)?;
        json_config
    };

    // Migrate old config versions
    migrate_config(&mut config)?;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_file_path()?;

    // Always save in TOML format for new saves
    let toml_path = if config_path.extension().and_then(|s| s.to_str()) == Some("json") {
        config_path.with_extension("toml")
    } else {
        config_path
    };

    ensure_parent_dir_exists(&toml_path)?;
    let content = toml::to_string_pretty(config).map_err(GitSwitchError::TomlSer)?;
    write_file_content(&toml_path, &content)
}

/// Migrate JSON config to TOML format
fn migrate_to_toml(config: &Config) -> Result<()> {
    tracing::info!("Migrating configuration from JSON to TOML format");

    let home_dir = home::home_dir().ok_or(GitSwitchError::HomeDirectoryNotFound)?;
    let json_path = home_dir.join(CONFIG_FILE_NAME_JSON);
    let toml_path = home_dir.join(CONFIG_FILE_NAME_TOML);

    // Save as TOML
    ensure_parent_dir_exists(&toml_path)?;
    let content = toml::to_string_pretty(config).map_err(GitSwitchError::TomlSer)?;
    write_file_content(&toml_path, &content)?;

    // Backup old JSON config
    if json_path.exists() {
        let backup_path = json_path.with_extension("json.backup");
        std::fs::rename(&json_path, &backup_path)?;
        tracing::info!("Old JSON config backed up to: {}", backup_path.display());
    }

    tracing::info!("Migration to TOML format completed");
    Ok(())
}

/// Migrate old config versions to current version
fn migrate_config(config: &mut Config) -> Result<()> {
    let current_version = &config.version;

    if current_version.is_empty() || current_version == "1.0" {
        tracing::info!("Migrating config from version {} to 2.0", current_version);

        // Add default values for new fields
        for account in config.accounts.values_mut() {
            if account.additional_ssh_keys.is_empty() {
                account.additional_ssh_keys = Vec::new();
            }
            if account.groups.is_empty() {
                account.groups = Vec::new();
            }
            if account.provider.is_none() {
                // Try to detect provider from email domain
                if account.email.contains("@github.com") {
                    account.provider = Some("github".to_string());
                } else if account.email.contains("@gitlab.com") {
                    account.provider = Some("gitlab".to_string());
                }
            }
        }

        config.version = "2.0".to_string();
        tracing::info!("Config migration to version 2.0 completed");
    }

    Ok(())
}

impl Config {
    pub fn get_profiles_path(&self) -> std::path::PathBuf {
        let home_dir = home::home_dir().expect("Home directory should be available");
        home_dir.join("profiles.toml")
    }
}
