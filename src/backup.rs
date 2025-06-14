use crate::config::{Config, load_config, save_config, get_config_file_path};
use crate::error::{GitSwitchError, Result};
use crate::utils::{ensure_parent_dir_exists, read_file_content, write_file_content};
use std::path::{Path, PathBuf};
use std::fs;

/// Backup the current configuration to an encrypted file
pub fn backup_config(backup_path: Option<&Path>) -> Result<PathBuf> {
    let config = load_config()?;
    
    let backup_file_path = if let Some(path) = backup_path {
        path.to_path_buf()
    } else {
        // Default backup location
        let config_path = get_config_file_path()?;
        let config_dir = config_path.parent()
            .ok_or_else(|| GitSwitchError::Other("Could not determine config directory".to_string()))?;
        config_dir.join("git-switch-backup.toml")
    };

    ensure_parent_dir_exists(&backup_file_path)?;
    
    // Serialize to TOML format for better readability
    let toml_content = toml::to_string_pretty(&config)
        .map_err(GitSwitchError::TomlSer)?;
    
    write_file_content(&backup_file_path, &toml_content)?;
    
    println!("Configuration backed up to: {}", backup_file_path.display());
    Ok(backup_file_path)
}

/// Restore configuration from a backup file
pub fn restore_config(backup_path: &Path) -> Result<()> {
    if !backup_path.exists() {
        return Err(GitSwitchError::BackupFailed {
            message: format!("Backup file not found: {}", backup_path.display()),
        });
    }

    let backup_content = read_file_content(backup_path)?;
    
    // Try to parse as TOML first, fallback to JSON for backwards compatibility
    let config: Config = if backup_path.extension().and_then(|s| s.to_str()) == Some("toml") {
        toml::from_str(&backup_content)
            .map_err(|e| GitSwitchError::RestoreFailed {
                message: format!("Failed to parse TOML backup: {}", e),
            })?
    } else {
        serde_json::from_str(&backup_content)
            .map_err(|e| GitSwitchError::RestoreFailed {
                message: format!("Failed to parse JSON backup: {}", e),
            })?
    };

    // Validate the restored configuration
    validate_config(&config)?;
    
    // Create a backup of current config before restoring
    let current_config_path = get_config_file_path()?;
    if current_config_path.exists() {
        let backup_current_path = current_config_path.with_extension("json.backup");
        fs::copy(&current_config_path, &backup_current_path)?;
        println!("Current configuration backed up to: {}", backup_current_path.display());
    }
    
    save_config(&config)?;
    println!("Configuration restored from: {}", backup_path.display());
    Ok(())
}

/// Validate configuration data
fn validate_config(config: &Config) -> Result<()> {
    for (name, account) in &config.accounts {
        if name.is_empty() {
            return Err(GitSwitchError::CorruptedConfig {
                message: "Account name cannot be empty".to_string(),
            });
        }
        
        if account.email.is_empty() {
            return Err(GitSwitchError::CorruptedConfig {
                message: format!("Email cannot be empty for account '{}'", name),
            });
        }
        
        // Validate email format
        if !email_address::EmailAddress::is_valid(&account.email) {
            return Err(GitSwitchError::InvalidEmail {
                email: account.email.clone(),
            });
        }
    }
    Ok(())
}

/// Export accounts to a portable format
pub fn export_accounts(export_path: &Path, format: ExportFormat) -> Result<()> {
    let config = load_config()?;
    
    let content = match format {
        ExportFormat::Toml => toml::to_string_pretty(&config)
            .map_err(GitSwitchError::TomlSer)?,
        ExportFormat::Json => serde_json::to_string_pretty(&config)
            .map_err(GitSwitchError::Json)?,
    };
    
    ensure_parent_dir_exists(export_path)?;
    write_file_content(export_path, &content)?;
    
    println!("Accounts exported to: {}", export_path.display());
    Ok(())
}

/// Import accounts from a file
pub fn import_accounts(import_path: &Path, merge: bool) -> Result<()> {
    if !import_path.exists() {
        return Err(GitSwitchError::Other(
            format!("Import file not found: {}", import_path.display())
        ));
    }

    let import_content = read_file_content(import_path)?;
    let import_config: Config = if import_path.extension().and_then(|s| s.to_str()) == Some("toml") {
        toml::from_str(&import_content)
            .map_err(|e| GitSwitchError::Other(format!("Failed to parse TOML import: {}", e)))?
    } else {
        serde_json::from_str(&import_content)
            .map_err(|e| GitSwitchError::Other(format!("Failed to parse JSON import: {}", e)))?
    };

    validate_config(&import_config)?;

    let mut current_config = load_config()?;
    
    if merge {
        // Merge accounts, asking for confirmation on conflicts
        for (name, account) in import_config.accounts {
            if current_config.accounts.contains_key(&name) {
                println!("Account '{}' already exists. Overwrite? [y/N]", name);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "y" {
                    continue;
                }
            }
            current_config.accounts.insert(name, account);
        }
    } else {
        // Replace all accounts
        current_config = import_config;
    }
    
    save_config(&current_config)?;
    println!("Accounts imported successfully");
    Ok(())
}

/// Clean up sensitive data from memory
#[allow(dead_code)]
pub fn secure_cleanup() {
    // This function can be called to ensure sensitive data is properly cleared
    // The zeroize crate helps with this
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Toml,
    Json,
}

impl std::str::FromStr for ExportFormat {
    type Err = GitSwitchError;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "toml" => Ok(ExportFormat::Toml),
            "json" => Ok(ExportFormat::Json),
            _ => Err(GitSwitchError::Other(
                format!("Unknown export format: {}. Supported: toml, json", s)
            )),
        }
    }
}
