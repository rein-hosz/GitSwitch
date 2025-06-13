use crate::config::Config;
use crate::error::{GitSwitchError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use colored::*;

/// Represents a profile containing multiple accounts for different contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub description: Option<String>,
    pub accounts: Vec<String>, // Account names
    pub default_account: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Profile manager for handling profile operations
pub struct ProfileManager {
    config: Config,
    profiles: HashMap<String, Profile>,
}

impl ProfileManager {
    pub fn new(config: Config) -> Result<Self> {
        let profiles = Self::load_profiles(&config)?;
        Ok(Self { config, profiles })
    }

    fn load_profiles(config: &Config) -> Result<HashMap<String, Profile>> {
        let profiles_path = config.get_profiles_path();
        if !profiles_path.exists() {
            return Ok(HashMap::new());
        }

        let content = std::fs::read_to_string(&profiles_path)
            .map_err(|e| GitSwitchError::Io(e))?;
        
        let profiles: HashMap<String, Profile> = toml::from_str(&content)
            .map_err(|e| GitSwitchError::SerializationError(e.to_string()))?;

        Ok(profiles)
    }

    fn save_profiles(&self) -> Result<()> {
        let profiles_path = self.config.get_profiles_path();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = profiles_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| GitSwitchError::Io(e))?;
        }

        let content = toml::to_string_pretty(&self.profiles)
            .map_err(|e| GitSwitchError::SerializationError(e.to_string()))?;
        
        std::fs::write(&profiles_path, content)
            .map_err(|e| GitSwitchError::Io(e))?;

        Ok(())
    }

    /// Create a new profile
    pub fn create_profile(
        &mut self,
        name: String,
        description: Option<String>,
        accounts: Vec<String>,
        default_account: Option<String>,
    ) -> Result<()> {
        if self.profiles.contains_key(&name) {
            return Err(GitSwitchError::ProfileAlreadyExists { name });
        }

        // Validate that all accounts exist
        for account_name in &accounts {
            if !self.config.accounts.contains_key(account_name) {
                return Err(GitSwitchError::AccountNotFound { 
                    name: account_name.clone() 
                });
            }
        }

        // Validate default account if specified
        if let Some(ref default) = default_account {
            if !accounts.contains(default) {
                return Err(GitSwitchError::InvalidDefaultAccount {
                    profile: name.clone(),
                    account: default.clone(),
                });
            }
        }

        let profile = Profile {
            name: name.clone(),
            description,
            accounts,
            default_account,
            created_at: chrono::Utc::now(),
            last_used: None,
        };

        self.profiles.insert(name.clone(), profile);
        self.save_profiles()?;

        println!("{} Profile '{}' created successfully", "✓".green(), name);
        Ok(())
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, name: &str) -> Result<()> {
        if !self.profiles.contains_key(name) {
            return Err(GitSwitchError::ProfileNotFound { 
                name: name.to_string() 
            });
        }

        self.profiles.remove(name);
        self.save_profiles()?;

        println!("{} Profile '{}' deleted successfully", "✓".green(), name);
        Ok(())
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Result<()> {
        if self.profiles.is_empty() {
            println!("{} No profiles found", "ℹ".blue());
            println!("Create a profile with: {}", 
                    "git-switch profile create <name> --accounts <account1,account2>".cyan());
            return Ok(());
        }

        println!("{}", "Available Profiles:".bold().underline());
        println!();

        for (name, profile) in &self.profiles {
            println!("{} {}", "▶".green(), name.bold());
            
            if let Some(ref description) = profile.description {
                println!("  Description: {}", description.italic());
            }
            
            println!("  Accounts: {}", 
                    profile.accounts.join(", ").cyan());
            
            if let Some(ref default) = profile.default_account {
                println!("  Default: {}", default.yellow());
            }
            
            println!("  Created: {}", 
                    profile.created_at.format("%Y-%m-%d %H:%M UTC").to_string().dimmed());
            
            if let Some(last_used) = profile.last_used {
                println!("  Last used: {}", 
                        last_used.format("%Y-%m-%d %H:%M UTC").to_string().dimmed());
            }
            
            println!();
        }

        Ok(())
    }

    /// Switch to a profile
    pub fn switch_profile(&mut self, name: &str, account_override: Option<String>) -> Result<()> {
        // Determine which account to use
        let account_name = if let Some(override_account) = account_override {
            let profile = self.profiles.get(name)
                .ok_or_else(|| GitSwitchError::ProfileNotFound { 
                    name: name.to_string() 
                })?;
                
            if !profile.accounts.contains(&override_account) {
                return Err(GitSwitchError::AccountNotInProfile {
                    profile: name.to_string(),
                    account: override_account,
                });
            }
            override_account
        } else {
            let profile = self.profiles.get(name)
                .ok_or_else(|| GitSwitchError::ProfileNotFound { 
                    name: name.to_string() 
                })?;
                
            if let Some(ref default) = profile.default_account {
                default.clone()
            } else {
                // If no default, prompt user to choose
                return self.prompt_account_selection_by_name(name);
            }
        };

        // Update last used timestamp
        if let Some(profile) = self.profiles.get_mut(name) {
            profile.last_used = Some(chrono::Utc::now());
            self.save_profiles()?;
        }

        // Switch to the selected account
        crate::commands::handle_account_subcommand(&self.config, &account_name)?;

        println!("{} Switched to profile '{}' using account '{}'", 
                "✓".green(), name, account_name);

        Ok(())
    }

    fn prompt_account_selection_by_name(&self, profile_name: &str) -> Result<()> {
        use dialoguer::Select;

        let profile = self.profiles.get(profile_name)
            .ok_or_else(|| GitSwitchError::ProfileNotFound { 
                name: profile_name.to_string() 
            })?;

        println!("Profile '{}' has no default account. Please select one:", profile.name);
        
        let selection = Select::new()
            .with_prompt("Select account")
            .items(&profile.accounts)
            .interact()?;

        let selected_account = &profile.accounts[selection];
        crate::commands::handle_account_subcommand(&self.config, selected_account)?;

        println!("{} Switched to account '{}'", "✓".green(), selected_account);
        Ok(())
    }

    /// Update profile
    pub fn update_profile(
        &mut self,
        name: &str,
        description: Option<String>,
        add_accounts: Vec<String>,
        remove_accounts: Vec<String>,
        default_account: Option<String>,
    ) -> Result<()> {
        let profile = self.profiles.get_mut(name)
            .ok_or_else(|| GitSwitchError::ProfileNotFound { 
                name: name.to_string() 
            })?;

        // Update description if provided
        if let Some(desc) = description {
            profile.description = Some(desc);
        }

        // Add accounts
        for account in add_accounts {
            if !self.config.accounts.contains_key(&account) {
                return Err(GitSwitchError::AccountNotFound { name: account });
            }
            if !profile.accounts.contains(&account) {
                profile.accounts.push(account);
            }
        }

        // Remove accounts
        for account in remove_accounts {
            profile.accounts.retain(|a| a != &account);
            // Clear default if it was removed
            if profile.default_account.as_ref() == Some(&account) {
                profile.default_account = None;
            }
        }

        // Update default account
        if let Some(default) = default_account {
            if !profile.accounts.contains(&default) {
                return Err(GitSwitchError::InvalidDefaultAccount {
                    profile: name.to_string(),
                    account: default,
                });
            }
            profile.default_account = Some(default);
        }

        self.save_profiles()?;
        println!("{} Profile '{}' updated successfully", "✓".green(), name);

        Ok(())
    }

    /// Get profile usage statistics
    pub fn get_profile_stats(&self) -> Result<()> {
        if self.profiles.is_empty() {
            println!("{} No profiles found", "ℹ".blue());
            return Ok(());
        }

        println!("{}", "Profile Statistics:".bold().underline());
        println!();

        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| {
            b.last_used.unwrap_or(chrono::DateTime::from_timestamp(0, 0).unwrap())
                .cmp(&a.last_used.unwrap_or(chrono::DateTime::from_timestamp(0, 0).unwrap()))
        });

        for profile in profiles {
            println!("{} {} ({})", 
                    "▶".green(), 
                    profile.name.bold(),
                    format!("{} accounts", profile.accounts.len()).dimmed());
            
            if let Some(last_used) = profile.last_used {
                let days_ago = (chrono::Utc::now() - last_used).num_days();
                println!("  Last used: {} ({} days ago)", 
                        last_used.format("%Y-%m-%d").to_string().cyan(),
                        days_ago);
            } else {
                println!("  Last used: {}", "Never".dimmed());
            }
            
            println!();
        }

        Ok(())
    }
}


